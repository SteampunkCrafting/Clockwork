use self::inner_state::{generate_pipeline, InnerState};
use asset_storage::asset_storage::AssetStorageKey;
use graphics::{state::GraphicsState, vulkano_layer::VulkanoLayer};
use kernel::{abstract_runtime::EngineState, prelude::Itertools, util::init_state::InitState};
use legion_ecs::{
    prelude::{component, IntoQuery},
    state::LegionState,
};
use physics::{
    prelude::{Isometry, RigidBody, RigidBodyHandle},
    state::PhysicsState,
};
use scene_utils::{
    components::{AmbientLight, Camera, DirectionalLight, PointLight, SpotLight},
    prelude::{PhongMaterialStorage, TexturedMeshStorage},
};
use state_requirements::StateRequirements;
use std::{collections::HashMap, sync::Arc};
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer, TypedBufferAccess},
    command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage},
    descriptor_set::PersistentDescriptorSet,
    image::view::ImageView,
};

pub struct StaticMeshDrawer<I>(InitState<(), InnerState<I>>)
where
    I: AssetStorageKey;

pub struct DrawMarker;

impl<S, I> VulkanoLayer<S> for StaticMeshDrawer<I>
where
    S: StateRequirements<I>,
    I: AssetStorageKey,
{
    fn initialization(&mut self, _: &EngineState<S>, graphics_state: &GraphicsState) {
        self.0.initialize(|()| graphics_state.into());
    }

    fn window_resize(&mut self, _: &EngineState<S>, graphics_state: &GraphicsState) {
        self.0.get_init_mut().pipeline = generate_pipeline(graphics_state)
    }

    fn draw(
        &mut self,
        engine_state: &EngineState<S>,
        graphics_state: &GraphicsState,
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
            .finish()
            .map(|(camera, camera_body)| {
                (
                    camera,
                    engine_state
                        .start_access()
                        .get(|PhysicsState { bodies, .. }| {
                            bodies.get(camera_body).cloned().unwrap()
                        })
                        .finish(),
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
                    .then_get(|eb, PhysicsState { bodies, .. }| {
                        let mut instances: HashMap<I, Vec<[[f32; 4]; 4]>> = Default::default();
                        eb.map(|(e, b)| {
                            (
                                e,
                                bodies
                                    .get(b)
                                    .map(RigidBody::position)
                                    .map(Isometry::to_matrix)
                                    .map(Into::<[[f32; 4]; 4]>::into)
                                    .unwrap(),
                            )
                        })
                        .for_each(|(e, i)| instances.entry(e).or_default().push(i));
                        Some(instances.into_iter())
                    })
                    .finish(),
            )
            /* ---- GETTING LIGHTS ---- */
            .zip({
                let (ambient_light, directional_lights) = engine_state
                    .start_access()
                    .get(|LegionState { world, .. }| {
                        (
                            <&AmbientLight>::query()
                                .filter(component::<DrawMarker>())
                                .iter(world)
                                .cloned()
                                .next()
                                .unwrap_or_default(),
                            <&DirectionalLight>::query()
                                .filter(component::<DrawMarker>())
                                .iter(world)
                                .cloned()
                                .collect_vec(),
                        )
                    })
                    .finish();
                let (point_lights, spot_lights) = engine_state
                    .start_access()
                    .get(|LegionState { world, .. }| {
                        (
                            <(&PointLight, &RigidBodyHandle)>::query()
                                .filter(component::<DrawMarker>())
                                .iter(world)
                                .map(|(l, b)| (l.clone(), b.clone()))
                                .collect_vec()
                                .into_iter(),
                            <(&SpotLight, &RigidBodyHandle)>::query()
                                .filter(component::<DrawMarker>())
                                .iter(world)
                                .map(|(l, b)| (l.clone(), b.clone()))
                                .collect_vec()
                                .into_iter(),
                        )
                    })
                    .then_get(|(point_lights, spot_lights), PhysicsState { bodies, .. }| {
                        (
                            point_lights
                                .map(|(l, b)| (l, bodies.get(b).unwrap()))
                                .map(|(l, b)| (l, b.clone()))
                                .collect_vec(),
                            spot_lights
                                .map(|(l, b)| (l, bodies.get(b).unwrap()))
                                .map(|(l, b)| (l, b.clone()))
                                .collect_vec(),
                        )
                    })
                    .finish();
                Some((ambient_light, directional_lights, point_lights, spot_lights))
            })
            /* ---- SETTING UP WORLD UNIFORMS ---- */
            .map(
                |(
                    ((camera, camera_body), instanced_data),
                    (ambient_light, directional_lights, point_lights, spot_lights),
                )| {
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
                                    camera.into(),
                                    camera_body.position().inverse().to_matrix().into(),
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
                                    (camera.clone(), camera_body.clone()),
                                    ambient_light,
                                    directional_lights,
                                    point_lights,
                                    spot_lights,
                                ))
                                .unwrap(),
                        ))
                        .unwrap();
                        Arc::new(set.build().unwrap())
                    };
                    (
                        instanced_data,
                        vertex_uniform_set,
                        fragment_world_uniform_set,
                    )
                },
            )
            /* ---- SETTING UP PER-INSTANCE UNIFORMS AND WRITING DRAW CALLS ---- */
            .map(
                |(instanced_data, vertex_uniform_set, fragment_world_uniform_set)| {
                    let (
                        graphics_state
                        @
                        GraphicsState {
                            subpass,
                            device,
                            queue,
                            ..
                        },
                        InnerState {
                            buffered_meshes,
                            pipeline,
                            fragment_uniform_mesh_pool,
                            texture_sampler,
                            default_texture,
                            ..
                        },
                    ) = (graphics_state, self.0.get_init_mut());
                    let mut cmd_builder = AutoCommandBufferBuilder::secondary_graphics(
                        device.clone(),
                        queue.family(),
                        CommandBufferUsage::OneTimeSubmit,
                        subpass.clone(),
                    )
                    .unwrap();
                    let cmd = Some(&mut cmd_builder)
                        .map(|cmd| {
                            cmd.bind_pipeline_graphics(pipeline.clone())
                                .bind_descriptor_sets(
                                    vulkano::pipeline::PipelineBindPoint::Graphics,
                                    pipeline.layout().clone(),
                                    0,
                                    (
                                        vertex_uniform_set.clone(),
                                        fragment_world_uniform_set.clone(),
                                    ),
                                )
                        })
                        .unwrap();
                    instanced_data.fold(cmd, |cmd, (mesh_id, instances)| {
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
                                Arc::new(
                                    ImageView::new(
                                        mesh.texture
                                            .as_ref()
                                            .map(Clone::clone)
                                            .unwrap_or_else(|| default_texture.clone()),
                                    )
                                    .unwrap(),
                                ),
                                texture_sampler.clone(),
                            )
                            .unwrap();
                            Arc::new(set.build().unwrap())
                        };
                        cmd.bind_descriptor_sets(
                            vulkano::pipeline::PipelineBindPoint::Graphics,
                            pipeline.layout().clone(),
                            2,
                            (
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
                        .unwrap()
                    });
                    cmd_builder.build().unwrap()
                },
            )
            .unwrap()
    }

    fn termination(&mut self, _: &EngineState<S>, _: &GraphicsState) {}
}

mod buffered_mesh;
mod inner_state;
mod state_requirements;

impl<I> Default for StaticMeshDrawer<I>
where
    I: AssetStorageKey,
{
    fn default() -> Self {
        Self(Default::default())
    }
}
