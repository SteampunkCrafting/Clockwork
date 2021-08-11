use self::{
    buffered_mesh::BufferedMesh,
    instance_data::InstanceData,
    util::{make_fragment_uniforms, make_vertex_uniforms},
    vertex::Vertex,
};
use asset_storage::{asset_storage::AssetStorageKey, prelude::AssetStorage};
use clockwork_core::clockwork::CallbackSubstate;
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
use physics::state::PhysicsState;
use scene_utils::{
    components::{AmbientLight, DirectionalLight, PhongMaterial},
    prelude::ColoredMesh,
};
use std::{collections::HashMap, sync::Arc};
pub use util::{CameraEntity, DrawMarker, DrawableEntity};
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer, CpuBufferPool},
    descriptor::descriptor_set::PersistentDescriptorSet,
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

pub struct ColoredMeshDrawer<I>(
    Option<(
        Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
        CpuBufferPool<vertex_shader::ty::Data>,
        CpuBufferPool<fragment_shader::ty::Data>,
    )>,
    HashMap<I, BufferedMesh>,
)
where
    I: AssetStorageKey;

impl<S, I> VulkanoLayer<S> for ColoredMeshDrawer<I>
where
    S: CallbackSubstate<PhysicsState>
        + CallbackSubstate<LegionState>
        + CallbackSubstate<AssetStorage<I, ColoredMesh>>,
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
        state.callback_substate(|LegionState { world, .. }| {
            state.callback_substate(|meshes: &AssetStorage<I, ColoredMesh>| {
                state.callback_substate(|physics: &PhysicsState| {
                    match (self, CameraEntity::query().iter(world).next()) {
                        (Self(inner_state @ None, ..), _) => {
                            // INITIALIZATION
                            *inner_state = Some((
                                Arc::new(
                                    GraphicsPipeline::start()
                                        .vertex_input(OneVertexOneInstanceDefinition::<
                                            Vertex,
                                            InstanceData,
                                        >::new(
                                        ))
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
                                        .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
                                        .build(device.clone())
                                        .unwrap(),
                                ),
                                CpuBufferPool::new(device.clone(), BufferUsage::all()),
                                CpuBufferPool::new(device.clone(), BufferUsage::all()),
                            ));
                        }
                        (
                            Self(
                                Some((pipeline, vertex_uniform_buffer, fragment_uniform_buffer)),
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
                                        (e.clone(), bodies.get(b.clone()).unwrap().position())
                                    })
                                    .map(|(e, i)| (e, i.to_matrix()))
                                    .map(|(e, i)| (e, Into::<[[f32; 4]; 4]>::into(i)))
                                    .for_each(|(e, i)| instances.entry(e).or_default().push(i));
                                instances
                            };

                            /* ---- RENDERING ---- */
                            let vertex_set = Arc::new(
                                PersistentDescriptorSet::start(
                                    pipeline.descriptor_set_layout(0).unwrap().clone(),
                                )
                                .add_buffer(
                                    vertex_uniform_buffer
                                        .next(make_vertex_uniforms(projection_matrix, view_matrix))
                                        .unwrap(),
                                )
                                .unwrap()
                                .build()
                                .unwrap(),
                            );

                            let fragment_set = Arc::new(
                                PersistentDescriptorSet::start(
                                    pipeline.descriptor_set_layout(1).unwrap().clone(),
                                )
                                .add_buffer(
                                    fragment_uniform_buffer
                                        .next(make_fragment_uniforms(
                                            (camera.clone(), camera_body.clone()),
                                            PhongMaterial {
                                                ambient: [255; 3].into(),
                                                diffuse: [255; 3].into(),
                                                specular: [1; 3].into(),
                                                specular_power: 16.0,
                                            },
                                            AmbientLight {
                                                color: [51; 3].into(),
                                            },
                                            &mut [DirectionalLight {
                                                color: [128, 0, 0].into(),
                                                direction: [0.0, 0.0, -1.0].into(),
                                            }]
                                            .iter()
                                            .cloned(),
                                            &mut [].iter().cloned(),
                                            &mut [].iter().cloned(),
                                        ))
                                        .unwrap(),
                                )
                                .unwrap()
                                .build()
                                .unwrap(),
                            );

                            for (mesh_id, instances) in instanced_data {
                                let BufferedMesh { vertices, indices } =
                                    buffered_meshes.entry(mesh_id.clone()).or_insert_with(|| {
                                        (device.clone(), &*meshes.get(mesh_id.clone()).lock())
                                            .into()
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
                                        (vertex_set.clone(), fragment_set.clone()),
                                        (),
                                    )
                                    .unwrap();
                            }
                        }
                        (_, None) => (),
                    }
                });
            })
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
