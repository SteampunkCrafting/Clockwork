use self::{
    buffered_mesh::BufferedMesh,
    instance_data::InstanceData,
    util::{
        make_mesh_fragment_uniforms, make_vertex_uniforms, make_world_fragment_uniforms,
        AmbientLightEntity, DirectionalLightEntity, PointLightEntity, SpotLightEntity,
    },
    vertex::Vertex,
};
use asset_storage::{asset_storage::AssetStorageKey, prelude::AssetStorage};
use graphics::{
    prelude::VulkanoLayer,
    state::{GraphicsState, OptionGraphicsState},
    vulkano::{command_buffer::AutoCommandBufferBuilder, pipeline::GraphicsPipeline},
    vulkano_layer,
};
use kernel::state::CallbackSubstate;
use legion_ecs::{prelude::*, state::LegionState};
use physics::state::PhysicsState;
use scene_utils::{
    components::PhongMaterial,
    prelude::{ColoredMesh, ColoredMeshStorage, PhongMaterialStorage},
};
use std::{collections::HashMap, sync::Arc};
pub use util::{CameraEntity, DrawMarker, DrawableEntity};
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer, CpuBufferPool, TypedBufferAccess},
    command_buffer::{pool::standard::StandardCommandPoolAlloc, PrimaryAutoCommandBuffer},
    descriptor_set::PersistentDescriptorSet,
    pipeline::vertex::BuffersDefinition,
    render_pass::Subpass,
};

mod buffered_mesh;
mod instance_data;
mod util;
mod vertex;

mod vertex_shader {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "glsl/colored_mesh_drawer.vert"
    }
}
mod fragment_shader {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "glsl/colored_mesh_drawer.frag"
    }
}

pub trait StateRequirements<I>:
    vulkano_layer::StateRequirements
    + CallbackSubstate<PhysicsState>
    + CallbackSubstate<LegionState>
    + CallbackSubstate<AssetStorage<I, ColoredMesh>>
    + CallbackSubstate<AssetStorage<I, PhongMaterial>>
where
    I: AssetStorageKey,
{
}

impl<T, I> StateRequirements<I> for T
where
    T: vulkano_layer::StateRequirements
        + CallbackSubstate<PhysicsState>
        + CallbackSubstate<LegionState>
        + CallbackSubstate<AssetStorage<I, ColoredMesh>>
        + CallbackSubstate<AssetStorage<I, PhongMaterial>>,
    I: AssetStorageKey,
{
}

pub struct ColoredMeshDrawer<I>(
    Option<(
        Arc<GraphicsPipeline>,
        CpuBufferPool<vertex_shader::ty::Data>,
        CpuBufferPool<fragment_shader::ty::DataMesh>,
        CpuBufferPool<fragment_shader::ty::DataWorld>,
    )>,
    HashMap<I, BufferedMesh>,
)
where
    I: AssetStorageKey;

impl<S, I> VulkanoLayer<S> for ColoredMeshDrawer<I>
where
    S: StateRequirements<I>,
    I: AssetStorageKey,
{
    fn draw(
        &mut self,
        state: &S,
        command_buffer: &mut AutoCommandBufferBuilder<
            PrimaryAutoCommandBuffer<StandardCommandPoolAlloc>,
        >,
    ) {
        state.callback_substate(|graphics_state: &OptionGraphicsState| {
            state.callback_substate(|LegionState { world, .. }| {
                state.callback_substate(|meshes: &ColoredMeshStorage<I>| {
                    state.callback_substate(|physics: &PhysicsState| {
                        state.callback_substate(|materials: &PhongMaterialStorage<I>| {
                            let GraphicsState {
                                render_pass,
                                device,
                                ..
                            } = graphics_state.as_ref().unwrap();
                            match (self, CameraEntity::query().iter(world).next()) {
                                (Self(inner_state @ None, ..), _) => {
                                    // INITIALIZATION
                                    *inner_state = Some((
                                        Arc::new(
                                            GraphicsPipeline::start()
                                                .vertex_input(
                                                    BuffersDefinition::new()
                                                        .vertex::<Vertex>()
                                                        .instance::<InstanceData>(),
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
                                                .depth_stencil_simple_depth()
                                                .cull_mode_back()
                                                .render_pass(
                                                    Subpass::from(render_pass.clone(), 0).unwrap(),
                                                )
                                                .build(device.clone())
                                                .unwrap(),
                                        ),
                                        CpuBufferPool::new(device.clone(), BufferUsage::all()),
                                        CpuBufferPool::new(device.clone(), BufferUsage::all()),
                                        CpuBufferPool::new(device.clone(), BufferUsage::all()),
                                    ));
                                }
                                (
                                    Self(
                                        Some((
                                            pipeline,
                                            vertex_uniform_buffer,
                                            fragment_mesh_uniform_buffer,
                                            fragment_world_uniform_buffer,
                                        )),
                                        buffered_meshes,
                                    ),
                                    Some((camera, camera_body)),
                                ) => {
                                    // DRAWING

                                    /* ---- ACQUIRING BODY SET ---- */
                                    let bodies = &physics.bodies;

                                    /* ---- GETTING CAMERA ENTITY ---- */
                                    let camera_body = bodies.get(camera_body.clone()).unwrap();
                                    let projection_matrix: [[f32; 4]; 4] = camera.clone().into();
                                    let view_matrix: [[f32; 4]; 4] =
                                        camera_body.position().inverse().to_matrix().into();

                                    /* ---- GETTING DRAWABLES ---- */
                                    let instanced_data = {
                                        let mut instances: HashMap<I, Vec<[[f32; 4]; 4]>> =
                                            Default::default();
                                        DrawableEntity::<I>::query()
                                            .iter(world)
                                            .map(|(_, e, b)| {
                                                (
                                                    e.clone(),
                                                    bodies.get(b.clone()).unwrap().position(),
                                                )
                                            })
                                            .map(|(e, i)| (e, i.to_matrix()))
                                            .map(|(e, i)| (e, Into::<[[f32; 4]; 4]>::into(i)))
                                            .for_each(|(e, i)| {
                                                instances.entry(e).or_default().push(i)
                                            });
                                        instances
                                    };

                                    /* ---- UNIFORM SETUP ---- */
                                    let vertex_set = {
                                        let mut set = PersistentDescriptorSet::start(
                                            pipeline
                                                .layout()
                                                .clone()
                                                .descriptor_set_layouts()
                                                .get(0)
                                                .unwrap()
                                                .clone(),
                                        );
                                        set.add_buffer(Arc::new(
                                            vertex_uniform_buffer
                                                .next(make_vertex_uniforms(
                                                    projection_matrix,
                                                    view_matrix,
                                                ))
                                                .unwrap(),
                                        ))
                                        .unwrap();
                                        Arc::new(set.build().unwrap())
                                    };

                                    let world_fragment_set = {
                                        let mut set = PersistentDescriptorSet::start(
                                            pipeline
                                                .layout()
                                                .descriptor_set_layouts()
                                                .get(1)
                                                .unwrap()
                                                .clone(),
                                        );
                                        set.add_buffer(Arc::new(
                                            fragment_world_uniform_buffer
                                                .next(make_world_fragment_uniforms(
                                                    (camera.clone(), camera_body.clone()),
                                                    AmbientLightEntity::query()
                                                        .iter(world)
                                                        .map(|(_, l)| l)
                                                        .cloned()
                                                        .next()
                                                        .unwrap_or_default(),
                                                    DirectionalLightEntity::query()
                                                        .iter(world)
                                                        .map(|(_, l)| l)
                                                        .cloned()
                                                        .collect(),
                                                    PointLightEntity::query()
                                                        .iter(world)
                                                        .map(|(_, l, b)| (l, b.clone()))
                                                        .map(|(l, b)| (l, bodies.get(b).unwrap()))
                                                        .map(|(l, b)| (l.clone(), b.clone()))
                                                        .collect(),
                                                    SpotLightEntity::query()
                                                        .iter(world)
                                                        .map(|(_, l, b)| (l, b.clone()))
                                                        .map(|(l, b)| (l, bodies.get(b).unwrap()))
                                                        .map(|(l, b)| (l.clone(), b.clone()))
                                                        .collect(),
                                                ))
                                                .unwrap(),
                                        ))
                                        .unwrap();
                                        Arc::new(set.build().unwrap())
                                    };

                                    /* ---- RENDERING ---- */
                                    for (mesh_id, instances) in instanced_data {
                                        let BufferedMesh { vertices, indices } = buffered_meshes
                                            .entry(mesh_id.clone())
                                            .or_insert_with(|| {
                                                (
                                                    device.clone(),
                                                    &*meshes.get(mesh_id.clone()).lock(),
                                                )
                                                    .into()
                                            });
                                        let mesh_fragment_set = {
                                            let mut set = PersistentDescriptorSet::start(
                                                pipeline
                                                    .layout()
                                                    .descriptor_set_layouts()
                                                    .get(2)
                                                    .unwrap()
                                                    .clone(),
                                            );
                                            set.add_buffer(Arc::new(
                                                fragment_mesh_uniform_buffer
                                                    .next(make_mesh_fragment_uniforms(
                                                        materials
                                                            .get(mesh_id.clone())
                                                            .lock()
                                                            .clone(),
                                                    ))
                                                    .unwrap(),
                                            ))
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
                                                    vertex_set.clone(),
                                                    world_fragment_set.clone(),
                                                    mesh_fragment_set.clone(),
                                                ),
                                            )
                                            .bind_vertex_buffers(
                                                0,
                                                (
                                                    vertices.clone(),
                                                    CpuAccessibleBuffer::from_iter(
                                                        device.clone(),
                                                        BufferUsage::all(),
                                                        false,
                                                        instances.iter().cloned(),
                                                    )
                                                    .unwrap(),
                                                ),
                                            )
                                            .bind_index_buffer(indices.clone())
                                            .draw_indexed(
                                                indices.len() as u32,
                                                instances.len() as u32,
                                                0,
                                                0,
                                                0,
                                            )
                                            .unwrap();
                                    }
                                }
                                (_, None) => (),
                            }
                        })
                    });
                })
            });
        });
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
