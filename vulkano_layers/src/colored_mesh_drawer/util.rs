use legion_ecs::prelude::Read;
use physics::prelude::{nalgebra::Perspective3, RigidBodyHandle};

pub struct DrawMarker;
pub struct Camera(pub Perspective3<f32>);

pub type CameraEntity = (Read<Camera>, Read<RigidBodyHandle>);
pub type DrawableEntity<MeshID> = (Read<DrawMarker>, Read<MeshID>, Read<RigidBodyHandle>);
