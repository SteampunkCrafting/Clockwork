use std::sync::Arc;

use crate::{graphics_state::GraphicsState, prelude::VulkanoLayer};
use clockwork_core::clockwork::ClockworkState;
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer},
    command_buffer::AutoCommandBufferBuilder,
    framebuffer::Subpass,
    impl_vertex,
    pipeline::{
        vertex::OneVertexOneInstanceDefinition, GraphicsPipeline, GraphicsPipelineAbstract,
    },
};

struct PrivateState {
    pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
}

#[deprecated]
pub struct TriangleLayer(Option<PrivateState>);

impl<S> VulkanoLayer<S> for TriangleLayer
where
    S: ClockworkState,
{
    fn draw(
        &mut self,
        _: &S,
        graphics_state: &GraphicsState,
        command_buffer: &mut AutoCommandBufferBuilder,
    ) {
        let GraphicsState {
            dynamic_state,
            render_pass,
            device,
        } = graphics_state;
        match self {
            s @ Self(None) => {
                *s = Self(Some(PrivateState {
                    pipeline: Arc::new(
                        GraphicsPipeline::start()
                            .vertex_input(
                                OneVertexOneInstanceDefinition::<Vertex, InstanceData>::new(),
                            )
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
                    ),
                }))
            }
            Self(Some(PrivateState { pipeline })) => {
                let (vertices, instances) = get_triangles();
                let vertices = CpuAccessibleBuffer::from_iter(
                    device.clone(),
                    BufferUsage::all(),
                    false,
                    vertices.iter().cloned(),
                )
                .unwrap();
                let instances = CpuAccessibleBuffer::from_iter(
                    device.clone(),
                    BufferUsage::all(),
                    false,
                    instances.iter().cloned(),
                )
                .unwrap();

                command_buffer
                    .draw(
                        pipeline.clone(),
                        &dynamic_state,
                        vec![vertices, instances],
                        (),
                        (),
                    )
                    .unwrap();
            }
        }
    }
}

impl Default for TriangleLayer {
    fn default() -> Self {
        Self(None)
    }
}

fn get_triangles() -> (Vec<Vertex>, Vec<InstanceData>) {
    (
        vec![
            Vertex {
                position: [-0.5, -0.25],
            },
            Vertex {
                position: [0.0, 0.5],
            },
            Vertex {
                position: [0.25, -0.1],
            },
        ],
        {
            let rows = 10;
            let cols = 10;
            let n_instances = rows * cols;
            let mut data = vec![];
            for c in 0..cols {
                for r in 0..rows {
                    let half_cell_w = 0.5 / cols as f32;
                    let half_cell_h = 0.5 / rows as f32;
                    let x = half_cell_w + (c as f32 / cols as f32) * 2.0 - 1.0;
                    let y = half_cell_h + (r as f32 / rows as f32) * 2.0 - 1.0;
                    let position_offset = [x, y];
                    let scale = (2.0 / rows as f32) * (c * rows + r) as f32 / n_instances as f32;
                    data.push(InstanceData {
                        position_offset,
                        scale,
                    });
                }
            }
            data
        },
    )
}

mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: "
                #version 450
                // The triangle vertex positions.
                layout(location = 0) in vec2 position;
                // The per-instance data.
                layout(location = 1) in vec2 position_offset;
                layout(location = 2) in float scale;
                void main() {
                    // Apply the scale and offset for the instance.
                    gl_Position = vec4(position * scale + position_offset, 0.0, 1.0);
                }
            "
    }
}

mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: "
                #version 450
                layout(location = 0) out vec4 f_color;
                void main() {
                    f_color = vec4(1.0, 0.0, 0.0, 1.0);
                }
            "
    }
}

#[derive(Default, Debug, Clone)]
struct Vertex {
    position: [f32; 2],
}
impl_vertex!(Vertex, position);

#[derive(Default, Debug, Clone)]
struct InstanceData {
    position_offset: [f32; 2],
    scale: f32,
}
impl_vertex!(InstanceData, position_offset, scale);
