use asset_storage::asset_storage::AssetStorageKey;
use asset_storage::prelude::AssetStorage;
use clockwork_core::{clockwork::Substate, sync::ReadLock};
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
use physics::{
    prelude::{RigidBodyHandle, RigidBodySet},
    state::RapierState3D,
};
use scene_utils::{mesh::Mesh, mesh_vertex::ColoredVertex, prelude::ColoredMesh};
use std::{collections::HashMap, sync::Arc};
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer},
    device::Device,
    impl_vertex,
};

pub struct DrawMarker;

pub type DrawableEntity<MeshID> = (Read<DrawMarker>, Read<MeshID>, Read<RigidBodyHandle>);

pub struct ColoredMeshDrawer<I>(
    Option<Arc<dyn GraphicsPipelineAbstract + Send + Sync>>,
    HashMap<I, BufferedMesh>,
)
where
    I: AssetStorageKey;

#[derive(Clone, Default, Debug)]
struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
    color: [f32; 4],
}

#[derive(Clone, Default, Debug)]
struct InstanceData {
    transformation: [[f32; 4]; 4],
}

struct BufferedMesh {
    vertices: Arc<CpuAccessibleBuffer<[Vertex]>>,
    indices: Arc<CpuAccessibleBuffer<[u32]>>,
}

mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: "
#version 450

// Input (Vertex Data)
layout (location = 0) in vec3 position;
layout (location = 1) in vec3 normal;
layout (location = 2) in vec4 color;

// Input (Instance Data)
layout (location = 3) in mat4 transformation;

// Output (Color)
layout (location = 0) out vec4 vert_color;

void main() {
    gl_Position = vec4(position, 1.0);
    vert_color = color;
}
        "
    }
}

mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: "
#version 450
layout (location = 0) in vec4 vert_color;

layout (location = 0) out vec4 frag_color;

void main() {
    frag_color = vert_color;
}
        "
    }
}

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
        let LegionState { world, resources } = state.substate();
        let meshes: &AssetStorage<I, ColoredMesh> = state.substate();

        match self {
            Self(pipeline @ None, ..) => {
                // INITIALIZATION
                *pipeline = Some(Arc::new(
                    GraphicsPipeline::start()
                        .vertex_input(OneVertexOneInstanceDefinition::<Vertex, InstanceData>::new())
                        .vertex_shader(
                            vs::Shader::load(device.clone()).unwrap().main_entry_point(),
                            (),
                        )
                        .triangle_list()
                        .viewports_dynamic_scissors_irrelevant(1)
                        .fragment_shader(
                            fs::Shader::load(device.clone()).unwrap().main_entry_point(),
                            (),
                        )
                        .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
                        .build(device.clone())
                        .unwrap(),
                ));
            }
            Self(Some(pipeline), buffered_meshes) => {
                // DRAWING
                let bodies = resources.get::<ReadLock<RigidBodySet>>().unwrap();
                let bodies = bodies.lock();
                for (mesh_id, transform) in DrawableEntity::<I>::query()
                    .iter(world)
                    .map(|(_, e, b)| (e, bodies.get(b.clone()).unwrap().position().clone()))
                    .map(|(e, i)| (e, i.to_matrix()))
                    .map(|(e, i)| (e, Into::<[[f32; 4]; 4]>::into(i)))
                {
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
                                    vec![transform].into_iter(),
                                )
                                .unwrap(),
                            ],
                            indices.clone(),
                            (),
                            (),
                        )
                        .unwrap();
                }
            }
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

impl_vertex!(Vertex, position, normal, color);

impl From<ColoredVertex> for Vertex {
    fn from(
        ColoredVertex {
            position,
            normal,
            color,
        }: ColoredVertex,
    ) -> Self {
        Self {
            position,
            normal,
            color,
        }
    }
}

impl_vertex!(InstanceData, transformation);

impl From<[[f32; 4]; 4]> for InstanceData {
    fn from(transformation: [[f32; 4]; 4]) -> Self {
        Self { transformation }
    }
}

impl From<(Arc<Device>, &ColoredMesh)> for BufferedMesh {
    fn from((dev, Mesh { indices, vertices }): (Arc<Device>, &ColoredMesh)) -> Self {
        Self {
            vertices: CpuAccessibleBuffer::from_iter(
                dev.clone(),
                BufferUsage::all(),
                false,
                vertices.iter().cloned().map(From::from),
            )
            .unwrap(),
            indices: CpuAccessibleBuffer::from_iter(
                dev,
                BufferUsage::all(),
                false,
                indices.iter().cloned().map(|i| i as u32),
            )
            .unwrap(),
        }
    }
}
