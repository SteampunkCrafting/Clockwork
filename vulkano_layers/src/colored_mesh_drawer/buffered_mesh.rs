use std::sync::Arc;

use scene_utils::{mesh::Mesh, prelude::ColoredMesh};
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer},
    device::Device,
};

use super::vertex::Vertex;

pub(super) struct BufferedMesh {
    pub vertices: Arc<CpuAccessibleBuffer<[Vertex]>>,
    pub indices: Arc<CpuAccessibleBuffer<[u32]>>,
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
                BufferUsage {
                    transfer_source: true,
                    transfer_destination: true,
                    uniform_texel_buffer: true,
                    storage_texel_buffer: true,
                    uniform_buffer: true,
                    storage_buffer: true,
                    index_buffer: true,
                    vertex_buffer: true,
                    indirect_buffer: true,
                    device_address: true,
                },
                false,
                indices.iter().cloned().map(|i| i as u32),
            )
            .unwrap(),
        }
    }
}
