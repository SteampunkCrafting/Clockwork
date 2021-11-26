use self::inner_state::InnerState;
use asset_storage::asset_storage::AssetStorageKey;
use graphics::{
    state::GraphicsState,
    vulkano_layer::{OldVulkanoLayer, VulkanoLayer},
};
use kernel::{abstract_runtime::EngineState, prelude::Itertools, util::init_state::InitState};
use legion_ecs::{
    prelude::{component, IntoQuery},
    state::LegionState,
};
use physics::{prelude::RigidBodyHandle, state::PhysicsState};
use scene_utils::{
    components::{AmbientLight, Camera},
    prelude::{PhongMaterialStorage, TexturedMeshStorage},
};
use state_requirements::StateRequirements;
use std::{collections::HashMap, sync::Arc};
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer, TypedBufferAccess},
    command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage},
    descriptor_set::PersistentDescriptorSet,
    image::view::ImageView,
    pipeline::viewport::Viewport,
};

pub struct SkyboxDrawer<I>(InitState<(), InnerState<I>>)
where
    I: AssetStorageKey;

pub struct DrawMarker;

impl<S, I> VulkanoLayer<S> for SkyboxDrawer<I>
where
    S: StateRequirements<I>,
    I: AssetStorageKey,
{
    fn initialization(&mut self, _: &EngineState<S>, graphics_state: &GraphicsState) {
        self.0.initialize(|()| graphics_state.into())
    }

    fn window_resize(&mut self, _: &EngineState<S>, _: &GraphicsState) {}

    fn draw(
        &mut self,
        engine_state: &EngineState<S>,
        graphics_state
        @
        GraphicsState {
            target_image_size: [width, height],
            subpass,
            device,
            queue,
        }: &GraphicsState,
    ) -> vulkano::command_buffer::SecondaryAutoCommandBuffer {
        engine_state
            .start_access()
            /* ---- GETTING CAMERA ENTITY ---- */
            .get(|LegionState { world, .. }| {
                <(&Camera, &RigidBodyHandle)>::query()
                    .filter(component::<DrawMarker>())
                    .iter(world)
                    .map(|(c, b)| (c.clone(), b.clone()))
                    .next()
            })
            .then_get(|cam_entity, PhysicsState { bodies, .. }| {
                cam_entity.map(|(cam, body_handle)| (cam, bodies.get(body_handle).unwrap().clone()))
            })
            .finish()
            .map(|(camera, camera_body)| {
                (
                    Into::<[[f32; 4]; 4]>::into(camera),
                    Into::<[[f32; 4]; 4]>::into(camera_body.position().inverse().to_matrix()),
                )
            })
            /* ---- GETTING DRAWABLE ENTITIES ---- */
            .zip(
                engine_state
                    .start_access()
                    .get(|LegionState { world, .. }| {
                        <(&I, &RigidBodyHandle)>::query()
                            .filter(component::<DrawMarker>())
                            .iter(world)
                            .map(|(e, b)| (e.clone(), b.clone()))
                            .collect_vec()
                            .into_iter()
                    })
                    .then_get(|it, PhysicsState { bodies, .. }| {
                        it.map(|(e, b)| (e, bodies.get(b).unwrap().clone()))
                            .map(|(e, b)| (e, b.position().to_matrix().into()))
                            .collect_vec()
                            .into_iter()
                    })
                    .map(|instances| {
                        let mut instanced_data: HashMap<I, Vec<[[f32; 4]; 4]>> = Default::default();
                        instances.for_each(|(i, b)| instanced_data.entry(i).or_default().push(b));
                        instanced_data
                    })
                    .map(Option::from)
                    .finish(),
            )
            /* ---- SETTING UP WORLD UNIFORMS ---- */
            .map(|((camera_projection, camera_view), instanced_data)| {
                let InnerState {
                    pipeline,
                    vertex_uniform_pool,
                    fragment_uniform_world_pool,
                    ..
                } = self.0.get_init();

                let vertex_uniform_set = {
                    let mut set = PersistentDescriptorSet::start(
                        pipeline
                            .layout()
                            .descriptor_set_layouts()
                            .get(0)
                            .unwrap()
                            .clone(),
                    );
                    set.add_buffer(Arc::new(
                        vertex_uniform_pool
                            .next(inner_state::make_vertex_uniforms(
                                camera_projection,
                                camera_view,
                            ))
                            .unwrap(),
                    ))
                    .unwrap();
                    Arc::new(set.build().unwrap())
                };

                let fragment_world_uniform_set = {
                    let mut set = PersistentDescriptorSet::start(
                        pipeline
                            .layout()
                            .descriptor_set_layouts()
                            .get(1)
                            .unwrap()
                            .clone(),
                    );
                    set.add_buffer(Arc::new(
                        fragment_uniform_world_pool
                            .next(inner_state::make_world_fragment_uniforms(
                                engine_state
                                    .start_access()
                                    .get(|LegionState { world, .. }| {
                                        <&AmbientLight>::query()
                                            .filter(component::<DrawMarker>())
                                            .iter(world)
                                            .cloned()
                                            .next()
                                            .unwrap_or_default()
                                    })
                                    .finish(),
                            ))
                            .unwrap(),
                    ))
                    .unwrap();
                    Arc::new(set.build().unwrap())
                };

                (
                    vertex_uniform_set,
                    fragment_world_uniform_set,
                    instanced_data,
                )
            })
            /* ---- SETTING UP MESH UNIFORMS AND DRAWING ---- */
            .map(
                |(vertex_uniform_set, fragment_world_uniform_set, instanced_data)| {
                    let InnerState {
                        buffered_meshes,
                        pipeline,
                        fragment_uniform_mesh_pool,
                        texture_sampler,
                        default_texture,
                        ..
                    } = self.0.get_init_mut();

                    let mut cmd = AutoCommandBufferBuilder::secondary_graphics(
                        device.clone(),
                        queue.family(),
                        CommandBufferUsage::OneTimeSubmit,
                        subpass.clone(),
                    )
                    .unwrap();

                    cmd.bind_pipeline_graphics(pipeline.clone()).set_viewport(
                        0,
                        [Viewport {
                            origin: [0.0, *height as f32],
                            dimensions: [*width as f32, -(*height as f32)],
                            depth_range: 0.0..1.0,
                        }],
                    );

                    instanced_data.into_iter().for_each(|(mesh_id, instances)| {
                        let mesh = buffered_meshes.entry(mesh_id.clone()).or_insert_with(|| {
                            engine_state
                                .start_access()
                                .get(|materials: &PhongMaterialStorage<I>| {
                                    materials.get(mesh_id.clone()).clone()
                                })
                                .then_get_zip(|meshes: &TexturedMeshStorage<I>| {
                                    meshes.get(mesh_id.clone()).clone()
                                })
                                .map(|(material, mesh)| {
                                    (graphics_state, &*mesh.lock(), &*material.lock()).into()
                                })
                                .finish()
                        });

                        let fragment_mesh_uniform_set = {
                            let mut set = PersistentDescriptorSet::start(
                                pipeline
                                    .layout()
                                    .descriptor_set_layouts()
                                    .get(2)
                                    .unwrap()
                                    .clone(),
                            );
                            set.add_buffer(Arc::new(
                                fragment_uniform_mesh_pool
                                    .next(inner_state::make_mesh_fragment_uniforms(
                                        engine_state
                                            .start_access()
                                            .get(|materials: &PhongMaterialStorage<I>| {
                                                materials.get(mesh_id.clone()).lock().clone()
                                            })
                                            .finish(),
                                    ))
                                    .unwrap(),
                            ))
                            .unwrap();
                            Arc::new(set.build().unwrap())
                        };

                        let fragment_mesh_texture_set = {
                            let mut set = PersistentDescriptorSet::start(
                                pipeline
                                    .layout()
                                    .descriptor_set_layouts()
                                    .get(3)
                                    .unwrap()
                                    .clone(),
                            );
                            set.add_sampled_image(
                                ImageView::new(
                                    mesh.texture
                                        .as_ref()
                                        .map(Clone::clone)
                                        .unwrap_or_else(|| default_texture.clone()),
                                )
                                .unwrap(),
                                texture_sampler.clone(),
                            )
                            .unwrap();
                            Arc::new(set.build().unwrap())
                        };

                        cmd.bind_descriptor_sets(
                            vulkano::pipeline::PipelineBindPoint::Graphics,
                            pipeline.layout().clone(),
                            0,
                            (
                                vertex_uniform_set.clone(),
                                fragment_world_uniform_set.clone(),
                                fragment_mesh_uniform_set.clone(),
                                fragment_mesh_texture_set.clone(),
                            ),
                        )
                        .bind_vertex_buffers(
                            0,
                            (
                                mesh.vertices.clone(),
                                CpuAccessibleBuffer::from_iter(
                                    graphics_state.device.clone(),
                                    BufferUsage::all(),
                                    false,
                                    instances.iter().cloned(),
                                )
                                .unwrap(),
                            ),
                        )
                        .bind_index_buffer(mesh.indices.clone())
                        .draw_indexed(mesh.indices.len() as u32, instances.len() as u32, 0, 0, 0)
                        .unwrap();
                    });
                    cmd
                },
            )
            .map(|cmd| cmd.build().unwrap())
            .unwrap()
    }

    fn termination(&mut self, _: &EngineState<S>, _: &GraphicsState) {}
}

mod buffered_mesh;
mod inner_state;
mod state_requirements;

impl<I> Default for SkyboxDrawer<I>
where
    I: AssetStorageKey,
{
    fn default() -> Self {
        Self(Default::default())
    }
}
