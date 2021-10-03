use graphics::graphics_state::GraphicsState;
use scene_utils::{
    components::PhongMaterial, mesh::Mesh, mesh_vertex::TexturedVertex, prelude::TexturedMesh,
};
use std::sync::Arc;
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer},
    format::Format,
    image::{Dimensions, ImmutableImage, MipmapsCount},
};

pub struct BufferedMesh {
    pub vertices: Arc<CpuAccessibleBuffer<[Vertex]>>,
    pub indices: Arc<CpuAccessibleBuffer<[u32]>>,
    pub texture: Option<Arc<ImmutableImage<Format>>>,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
    texture_coord: [f32; 2],
}

#[derive(Clone, Copy, Default, Debug)]
pub(super) struct InstanceData {
    world: [[f32; 4]; 4],
}

impl From<(&GraphicsState, &TexturedMesh, &PhongMaterial)> for BufferedMesh {
    fn from(
        (GraphicsState { device, queue, .. }, Mesh { indices, vertices }, material): (
            &GraphicsState,
            &TexturedMesh,
            &PhongMaterial,
        ),
    ) -> Self {
        Self {
            vertices: CpuAccessibleBuffer::from_iter(
                device.clone(),
                BufferUsage::all(),
                false,
                vertices.iter().cloned().map(From::from),
            )
            .unwrap(),
            indices: CpuAccessibleBuffer::from_iter(
                device.clone(),
                BufferUsage::all(),
                false,
                indices.iter().cloned().map(|i| i as u32),
            )
            .unwrap(),
            texture: match material {
                PhongMaterial::Colored { .. } => None,
                PhongMaterial::Textured { texture, .. } => {
                    let (texture, _) = {
                        let dimensions = Dimensions::Dim2d {
                            width: texture.width() as u32,
                            height: texture.height() as u32,
                        };
                        ImmutableImage::from_iter(
                            texture.data_lock().lock().iter().cloned(),
                            dimensions,
                            MipmapsCount::One,
                            Format::R8G8B8A8Srgb,
                            queue.clone(),
                        )
                        .unwrap()
                    };
                    Some(texture)
                }
            },
        }
    }
}

impl From<TexturedVertex> for Vertex {
    fn from(
        TexturedVertex {
            position,
            normal,
            texture_coord,
        }: TexturedVertex,
    ) -> Self {
        Self {
            position,
            normal,
            texture_coord,
        }
    }
}

vulkano::impl_vertex!(Vertex, position, normal, texture_coord);

impl From<[[f32; 4]; 4]> for InstanceData {
    fn from(world: [[f32; 4]; 4]) -> Self {
        Self { world }
    }
}

vulkano::impl_vertex!(InstanceData, world);
