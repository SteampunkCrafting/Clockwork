use self::inner_state::InnerState;
use asset_storage::asset_storage::AssetStorageKey;
use graphics::{state::GraphicsState, vulkano_layer::VulkanoLayer};
use legion_ecs::{
    prelude::{component, IntoQuery},
    state::LegionState,
};
use physics::{prelude::RigidBodyHandle, state::PhysicsState};
use scene_utils::{
    components::{AmbientLight, Camera},
    prelude::{PhongMaterialStorage, TexturedMeshStorage},
};
use state_requirements::EngineStateRequirements;
use std::{collections::HashMap, sync::Arc};
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer, TypedBufferAccess},
    command_buffer::{
        pool::standard::StandardCommandPoolAlloc, AutoCommandBufferBuilder,
        PrimaryAutoCommandBuffer,
    },
    descriptor_set::PersistentDescriptorSet,
    image::view::ImageView,
};

pub struct SkyboxDrawer<I>(Option<InnerState<I>>)
where
    I: AssetStorageKey;

pub struct DrawMarker;

impl<S, I> VulkanoLayer<S> for SkyboxDrawer<I>
where
    S: EngineStateRequirements<I>,
    I: AssetStorageKey,
{
    fn draw(
        &mut self,
        engine_state: &S,
        graphics_state: &graphics::state::GraphicsState,
        command_buffer: &mut AutoCommandBufferBuilder<
            PrimaryAutoCommandBuffer<StandardCommandPoolAlloc>,
        >,
    ) {
        /* ---- LAZY INITIALIZATION ---- */
        let InnerState {
            buffered_meshes,
            pipeline,
            vertex_uniform_pool,
            fragment_uniform_mesh_pool,
            fragment_uniform_world_pool,
            texture_sampler,
            default_texture,
        } = self.0.get_or_insert_with(|| graphics_state.into());

        /* ---- STATE DESTRUCTURING ---- */
        let GraphicsState { device, .. } = graphics_state;

        /* ---- GETTING ENGINE STATE AND DRAWING ---- */
        engine_state.callback_substate(|LegionState { world, .. }| {
            engine_state.callback_substate(|meshes: &TexturedMeshStorage<I>| {
                engine_state.callback_substate(|materials: &PhongMaterialStorage<I>| {
                    engine_state.callback_substate(|PhysicsState { bodies, .. }: &PhysicsState| {
                        if let Some((camera, cam_body_handle)) =
                            <(&Camera, &RigidBodyHandle)>::query()
                                .filter(component::<DrawMarker>())
                                .iter(world)
                                .map(|(c, b)| (c.clone(), b.clone()))
                                .next()
                        {
                            /* -- GETTING CAMERA ENTITY -- */
                            let camera_body = bodies.get(cam_body_handle).unwrap();
                            let camera_projection: [[f32; 4]; 4] = camera.into();
                            let camera_view: [[f32; 4]; 4] =
                                camera_body.position().inverse().to_matrix().into();

                            /* -- GETTING DRAWABLE ENTITIES -- */
                            // These are objects' world matrices ordered by their static mesh
                            let instanced_data = {
                                let mut instances: HashMap<I, Vec<[[f32; 4]; 4]>> =
                                    Default::default();
                                <(&I, &RigidBodyHandle)>::query()
                                    .filter(component::<DrawMarker>())
                                    .iter(world)
                                    .map(|(e, b)| {
                                        (e.clone(), bodies.get(b.clone()).unwrap().position())
                                    })
                                    .map(|(e, i)| (e, i.to_matrix()))
                                    .map(|(e, i)| (e, Into::<[[f32; 4]; 4]>::into(i)))
                                    .for_each(|(e, i)| instances.entry(e).or_default().push(i));
                                instances
                            };

                            /* -- SETTING UP WORLD UNIFORMS -- */
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
                                            <&AmbientLight>::query()
                                                .filter(component::<DrawMarker>())
                                                .iter(world)
                                                .cloned()
                                                .next()
                                                .unwrap_or_default(),
                                        ))
                                        .unwrap(),
                                ))
                                .unwrap();
                                Arc::new(set.build().unwrap())
                            };

                            /* -- SETTING UP PER-INSTANCE UNIFORMS AND WRITING DRAW CALLS -- */
                            for (mesh_id, instances) in instanced_data {
                                let mesh =
                                    buffered_meshes.entry(mesh_id.clone()).or_insert_with(|| {
                                        (
                                            graphics_state,
                                            &*meshes.get(mesh_id.clone()).lock(),
                                            &*materials.get(mesh_id.clone()).lock(),
                                        )
                                            .into()
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
                                                materials.get(mesh_id.clone()).lock().clone(),
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

                                command_buffer
                                    .bind_pipeline_graphics(pipeline.clone())
                                    .bind_descriptor_sets(
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
                                                device.clone(),
                                                BufferUsage::all(),
                                                false,
                                                instances.iter().cloned(),
                                            )
                                            .unwrap(),
                                        ),
                                    )
                                    .bind_index_buffer(mesh.indices.clone())
                                    .draw_indexed(
                                        mesh.indices.len() as u32,
                                        instances.len() as u32,
                                        0,
                                        0,
                                        0,
                                    )
                                    .unwrap();
                            }
                        }
                    })
                })
            })
        })
    }
}

mod buffered_mesh;
mod inner_state;
mod state_requirements;

impl<I> Default for SkyboxDrawer<I>
where
    I: AssetStorageKey,
{
    fn default() -> Self {
        Self(None)
    }
}
