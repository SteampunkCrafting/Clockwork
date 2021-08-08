use std::sync::Arc;

use legion_ecs::prelude::Read;
use physics::prelude::{nalgebra::Perspective3, RigidBodyHandle};
use scene_utils::{mesh::Mesh, prelude::ColoredMesh};
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer},
    device::Device,
    impl_vertex,
};

pub struct DrawMarker;
pub struct Camera(pub Perspective3<f32>);

pub type CameraEntity = (Read<Camera>, Read<RigidBodyHandle>);
pub type DrawableEntity<MeshID> = (Read<DrawMarker>, Read<MeshID>, Read<RigidBodyHandle>);
