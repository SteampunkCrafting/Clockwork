use legion_ecs::prelude::Read;
use physics::prelude::RigidBodyHandle;
use scene_utils::components::Camera;

pub struct DrawMarker;

pub type CameraEntity = (Read<Camera>, Read<RigidBodyHandle>);
pub type DrawableEntity<MeshID> = (Read<DrawMarker>, Read<MeshID>, Read<RigidBodyHandle>);
