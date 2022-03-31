use graphics::state::GraphicsState;
use scene_utils::{components::PhongMaterial, mesh_vertex::TexturedVertex, prelude::TexturedMesh};
use std::sync::Arc;
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer},
    format::Format,
    image::{ImageDimensions, ImmutableImage, MipmapsCount},
    memory::pool::{PotentialDedicatedAllocation, StdMemoryPoolAlloc},
};

#[derive(Clone)]
pub struct BufferedMesh {
    pub vertices: Arc<CpuAccessibleBuffer<[Vertex]>>,
    pub indices: Arc<CpuAccessibleBuffer<[u32]>>,
    pub texture: Option<Arc<ImmutableImage<PotentialDedicatedAllocation<StdMemoryPoolAlloc>>>>,
    pub index_count: u32,
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
        (GraphicsState { device, queue, .. }, mesh, material): (
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
                mesh.vertices().iter().cloned().map(From::from),
            )
            .unwrap(),
            indices: CpuAccessibleBuffer::from_iter(
                device.clone(),
                BufferUsage::all(),
                false,
                mesh.indices().iter().cloned().map(|i| i as u32),
            )
            .unwrap(),
            texture: match material {
                PhongMaterial::Colored { .. } => None,
                PhongMaterial::Textured { texture, .. } => {
                    let (texture, _) = {
                        let dimensions = ImageDimensions::Dim2d {
                            width: *texture.width() as u32,
                            height: *texture.height() as u32,
                            array_layers: 1,
                        };
                        ImmutableImage::from_iter(
                            texture.data().iter().cloned(),
                            dimensions,
                            MipmapsCount::One,
                            Format::R8G8B8A8_SRGB,
                            queue.clone(),
                        )
                        .unwrap()
                    };
                    Some(texture)
                }
            },
            index_count: mesh.indices().len() as u32,
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
