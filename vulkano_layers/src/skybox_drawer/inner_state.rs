use asset_storage::asset_storage::AssetStorageKey;
use graphics::state::GraphicsState;
use scene_utils::components::{AmbientLight, PhongMaterial};
use std::{collections::HashMap, sync::Arc};
use vulkano::{
    buffer::{BufferUsage, CpuBufferPool},
    format::Format,
    image::{ImmutableImage, MipmapsCount},
    memory::pool::{PotentialDedicatedAllocation, StdMemoryPoolAlloc},
    pipeline::{vertex::BuffersDefinition, GraphicsPipeline},
    sampler::{Filter, MipmapMode, Sampler, SamplerAddressMode},
};

use super::buffered_mesh::{BufferedMesh, InstanceData, Vertex};

pub struct InnerState<I>
where
    I: AssetStorageKey,
{
    pub buffered_meshes: HashMap<I, BufferedMesh>,
    pub pipeline: Arc<GraphicsPipeline>,
    pub vertex_uniform_pool: CpuBufferPool<vertex_shader::ty::Data>,
    pub fragment_uniform_mesh_pool: CpuBufferPool<fragment_shader::ty::DataMesh>,
    pub fragment_uniform_world_pool: CpuBufferPool<fragment_shader::ty::DataWorld>,
    pub texture_sampler: Arc<Sampler>,
    pub default_texture: Arc<ImmutableImage<PotentialDedicatedAllocation<StdMemoryPoolAlloc>>>,
}

impl<I> From<&GraphicsState> for InnerState<I>
where
    I: AssetStorageKey,
{
    fn from(
        GraphicsState {
            subpass,
            device,
            queue,
            ..
        }: &GraphicsState,
    ) -> Self {
        Self {
            pipeline: Arc::new(
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
                    .depth_stencil_disabled()
                    .depth_write(false)
                    .cull_mode_front()
                    .render_pass(subpass.clone())
                    .build(device.clone())
                    .unwrap(),
            ),
            vertex_uniform_pool: CpuBufferPool::new(device.clone(), BufferUsage::all()),
            fragment_uniform_mesh_pool: CpuBufferPool::new(device.clone(), BufferUsage::all()),
            fragment_uniform_world_pool: CpuBufferPool::new(device.clone(), BufferUsage::all()),
            buffered_meshes: Default::default(),
            texture_sampler: Sampler::new(
                device.clone(),
                Filter::Linear,
                Filter::Linear,
                MipmapMode::Nearest,
                SamplerAddressMode::Repeat,
                SamplerAddressMode::Repeat,
                SamplerAddressMode::Repeat,
                0.0,
                1.0,
                0.0,
                0.0,
            )
            .unwrap(),
            default_texture: ImmutableImage::from_iter(
                vec![0f32, 0f32, 0f32, 0f32].iter().cloned(),
                vulkano::image::ImageDimensions::Dim2d {
                    width: 1,
                    height: 1,
                    array_layers: 1,
                },
                MipmapsCount::One,
                Format::R8G8B8A8_SINT,
                queue.clone(),
            )
            .unwrap()
            .0,
        }
    }
}

mod vertex_shader {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "glsl/skybox_drawer.vert"
    }
}
mod fragment_shader {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "glsl/skybox_drawer.frag"
    }
}

pub fn make_vertex_uniforms(
    projection: [[f32; 4]; 4],
    view: [[f32; 4]; 4],
) -> vertex_shader::ty::Data {
    vertex_shader::ty::Data { projection, view }
}

pub fn make_mesh_fragment_uniforms(material: PhongMaterial) -> fragment_shader::ty::DataMesh {
    use fragment_shader::ty as fs;
    match material {
        PhongMaterial::Colored {
            ambient,
            diffuse,
            specular,
            specular_power,
        } => fs::DataMesh {
            material: fs::PhongMaterial {
                ambient: ambient.into(),
                diffuse: diffuse.into(),
                specular: specular.into(),
                specular_power: specular_power.into(),
            },
            is_textured: 0,
            _dummy0: Default::default(),
        },
        PhongMaterial::Textured { specular_power, .. } => fs::DataMesh {
            material: fs::PhongMaterial {
                ambient: Default::default(),
                diffuse: Default::default(),
                specular: Default::default(),
                specular_power: specular_power.into(),
            },
            is_textured: 1,
            _dummy0: Default::default(),
        },
    }
}

pub fn make_world_fragment_uniforms(
    AmbientLight { color }: AmbientLight,
) -> fragment_shader::ty::DataWorld {
    use fragment_shader::ty::*;
    fragment_shader::ty::DataWorld {
        ambient_light: AmbientLight {
            color: color.clone().into(),
        },
    }
}
