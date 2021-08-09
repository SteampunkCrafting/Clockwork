use self::{buffered_mesh::BufferedMesh, instance_data::InstanceData, vertex::Vertex};
use asset_storage::{asset_storage::AssetStorageKey, prelude::AssetStorage};
use clockwork_core::clockwork::Substate;
use graphics::{
    graphics_state::GraphicsState,
    prelude::VulkanoLayer,
    vulkano::{
        command_buffer::AutoCommandBufferBuilder,
        framebuffer::Subpass,
        pipeline::{
            vertex::OneVertexOneInstanceDefinition, GraphicsPipeline, GraphicsPipelineAbstract,
        },
    },
};
use legion_ecs::{prelude::*, state::LegionState};
use physics::state::RapierState3D;
use scene_utils::prelude::ColoredMesh;
use std::{collections::HashMap, sync::Arc};
pub use util::*;
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer, CpuBufferPool},
    descriptor::descriptor_set::PersistentDescriptorSet,
};

mod buffered_mesh;
mod fragment_shader;
mod instance_data;
mod util;
mod vertex;
mod vertex_shader;

pub struct ColoredMeshDrawer<I>(
    Option<(
        Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
        CpuBufferPool<vertex_shader::ty::Data>,
    )>,
    HashMap<I, BufferedMesh>,
)
where
    I: AssetStorageKey;

impl<S, I> VulkanoLayer<S> for ColoredMeshDrawer<I>
where
    S: Substate<RapierState3D> + Substate<LegionState> + Substate<AssetStorage<I, ColoredMesh>>,
    I: AssetStorageKey,
{
    fn draw(
        &mut self,
        state: &S,
        graphics_state: &GraphicsState,
        command_buffer: &mut AutoCommandBufferBuilder,
    ) {
        let GraphicsState {
            dynamic_state,
            render_pass,
            device,
        } = graphics_state;
        let LegionState { world, .. } = state.substate();
        let meshes: &AssetStorage<I, ColoredMesh> = state.substate();
        let physics: &RapierState3D = state.substate();

        match (self, CameraEntity::query().iter(world).next()) {
            (Self(inner_state @ None, ..), _) => {
                // INITIALIZATION
                *inner_state = Some((
                    Arc::new(
                        GraphicsPipeline::start()
                            .vertex_input(
                                OneVertexOneInstanceDefinition::<Vertex, InstanceData>::new(),
                            )
                            .vertex_shader(
                                vertex_shader::Shader::load(device.clone())
                                    .unwrap()
                                    .main_entry_point(),
                                (),
                            )
                            .triangle_list()
                            .viewports_dynamic_scissors_irrelevant(1)
                            .fragment_shader(
                                fragment_shader::Shader::load(device.clone())
                                    .unwrap()
                                    .main_entry_point(),
                                (),
                            )
                            .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
                            .build(device.clone())
                            .unwrap(),
                    ),
                    CpuBufferPool::new(device.clone(), BufferUsage::all()),
                ));
            }
            (
                Self(Some((pipeline, uniform_buffer)), buffered_meshes),
                Some((camera, camera_body)),
            ) => {
                // DRAWING

                /* ---- ACQUIRING BODY SET ---- */
                let bodies = physics.user_locks().1;
                let bodies = bodies.lock();

                /* ---- GETTING CAMERA ENTITY ---- */
                let projection_matrix: [[f32; 4]; 4] = camera.0.as_matrix().clone().into();
                let view_matrix: [[f32; 4]; 4] = bodies
                    .get(camera_body.clone())
                    .unwrap()
                    .position()
                    .inverse()
                    .to_matrix()
                    .into();

                /* ---- GETTING DRAWABLES ---- */
                let instanced_data = {
                    let mut instances: HashMap<I, Vec<[[f32; 4]; 4]>> = Default::default();
                    DrawableEntity::<I>::query()
                        .iter(world)
                        .map(|(_, e, b)| (e.clone(), bodies.get(b.clone()).unwrap().position()))
                        .map(|(e, i)| (e, i.to_matrix()))
                        .map(|(e, i)| (e, Into::<[[f32; 4]; 4]>::into(i)))
                        .for_each(|(e, i)| instances.entry(e).or_default().push(i));
                    instances
                };

                /* ---- RENDERING ---- */
                let set = Arc::new(
                    PersistentDescriptorSet::start(
                        pipeline.descriptor_set_layout(0).unwrap().clone(),
                    )
                    .add_buffer(
                        uniform_buffer
                            .next(vertex_shader::ty::Data {
                                projection: projection_matrix,
                                view: view_matrix,
                            })
                            .unwrap(),
                    )
                    .unwrap()
                    .build()
                    .unwrap(),
                );

                for (mesh_id, instances) in instanced_data {
                    let BufferedMesh { vertices, indices } =
                        buffered_meshes.entry(mesh_id.clone()).or_insert_with(|| {
                            (device.clone(), &*meshes.get(mesh_id.clone()).lock()).into()
                        });
                    command_buffer
                        .draw_indexed(
                            pipeline.clone(),
                            dynamic_state,
                            vec![
                                vertices.clone(),
                                CpuAccessibleBuffer::from_iter(
                                    device.clone(),
                                    BufferUsage::all(),
                                    false,
                                    instances.into_iter(),
                                )
                                .unwrap(),
                            ],
                            indices.clone(),
                            set.clone(),
                            (),
                        )
                        .unwrap();
                }
            }
            (_, None) => (),
        }
    }
}

impl<I> Default for ColoredMeshDrawer<I>
where
    I: AssetStorageKey,
{
    fn default() -> Self {
        Self(None, Default::default())
    }
}