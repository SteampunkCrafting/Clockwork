use asset_storage::asset_storage::AssetStorageKey;
use graphics::state::GraphicsState;
use physics::prelude::{nalgebra::Vector4, RigidBody};
use scene_utils::components::{
    AmbientLight, Camera, DirectionalLight, PhongMaterial, PointLight, SpotLight,
};
use std::{collections::HashMap, sync::Arc};
use vulkano::{
    buffer::{BufferUsage, CpuBufferPool},
    format::Format,
    image::{ImageDimensions, ImmutableImage, MipmapsCount},
    memory::pool::{PotentialDedicatedAllocation, StdMemoryPoolAlloc},
    pipeline::{vertex::BuffersDefinition, viewport::Viewport, GraphicsPipeline},
    sampler::{Filter, MipmapMode, Sampler, SamplerAddressMode},
};

use crate::util::partially_init_array;

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
    fn from(graphics_state @ GraphicsState { device, queue, .. }: &GraphicsState) -> Self {
        Self {
            pipeline: generate_pipeline(graphics_state),
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
                ImageDimensions::Dim2d {
                    width: 1,
                    height: 1,
                    array_layers: 1,
                },
                MipmapsCount::One,
                Format::R8G8B8A8_SRGB,
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
        path: "glsl/static_mesh_drawer.vert"
    }
}
mod fragment_shader {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "glsl/static_mesh_drawer.frag"
    }
}

pub fn generate_pipeline(
    GraphicsState {
        subpass,
        device,
        target_image_size: [width, height],
        ..
    }: &GraphicsState,
) -> Arc<GraphicsPipeline> {
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
            .viewports([Viewport {
                origin: [0.0, *height as f32],
                dimensions: [*width as f32, -(*height as f32)],
                depth_range: 0.0..1.0,
            }])
            .fragment_shader(
                fragment_shader::Shader::load(device.clone())
                    .unwrap()
                    .main_entry_point(),
                (),
            )
            .blend_alpha_blending()
            .depth_stencil_simple_depth()
            .cull_mode_back()
            .render_pass(subpass.clone())
            .build(device.clone())
            .unwrap(),
    )
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
    (_, camera_body): (Camera, RigidBody),
    AmbientLight { color }: AmbientLight,
    dir_lights: Vec<DirectionalLight>,
    point_lights: Vec<(PointLight, RigidBody)>,
    spot_lights: Vec<(SpotLight, RigidBody)>,
) -> fragment_shader::ty::DataWorld {
    use fragment_shader::ty::*;
    unsafe {
        fragment_shader::ty::DataWorld {
            num_dir_lights: dir_lights.len() as u32,
            num_point_lights: point_lights.len() as u32,
            num_spot_lights: spot_lights.len() as u32,
            ambient_light: AmbientLight {
                color: color.clone().into(),
            },
            dir_lights: partially_init_array(
                |l| DirectionalLight {
                    view_direction: (camera_body.position().to_matrix().transpose()
                        * l.direction.fixed_resize::<4, 1>(0.0))
                    .fixed_resize::<3, 1>(0.0)
                    .into(),
                    color: l.color.into(),
                    _dummy0: Default::default(),
                    _dummy1: Default::default(),
                },
                dir_lights,
            ),
            point_lights: partially_init_array(
                |(light, body)| PointLight {
                    view_position: camera_body
                        .position()
                        .inv_mul(body.position())
                        .translation
                        .into(),
                    color: light.color.into(),
                    attenuation: Attenuation {
                        constant_component: light.attenuation.constant,
                        linear_component: light.attenuation.linear,
                        quadratic_component: light.attenuation.quadratic,
                    },
                    _dummy0: Default::default(),
                    _dummy1: Default::default(),
                    _dummy2: Default::default(),
                },
                point_lights,
            ),
            spot_lights: partially_init_array(
                |(light, body)| SpotLight {
                    opening_angle_rad: light.opening_angle,
                    view_position: camera_body
                        .position()
                        .inv_mul(body.position())
                        .translation
                        .into(),
                    view_direction: (camera_body
                        .position()
                        .inv_mul(body.position())
                        .inverse()
                        .to_matrix()
                        .transpose()
                        * Vector4::from([0.0, 0.0, -1.0, 0.0]))
                    .fixed_resize::<3, 1>(0.0)
                    .into(),
                    color: light.color.into(),
                    attenuation: Attenuation {
                        constant_component: light.attenuation.constant,
                        linear_component: light.attenuation.linear,
                        quadratic_component: light.attenuation.quadratic,
                    },
                    _dummy0: Default::default(),
                    _dummy1: Default::default(),
                    _dummy2: Default::default(),
                    _dummy3: Default::default(),
                    _dummy4: Default::default(),
                },
                spot_lights,
            ),
            _dummy0: Default::default(),
            _dummy1: Default::default(),
            _dummy2: Default::default(),
            _dummy3: Default::default(),
        }
    }
}
