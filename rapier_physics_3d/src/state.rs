use std::{collections::HashMap, sync::Arc};

use rapier3d::{
    dynamics::{CCDSolver, JointHandle, JointSet, RigidBody, RigidBodyHandle, RigidBodySet},
    geometry::{BroadPhase, ColliderHandle, ColliderSet, NarrowPhase},
    prelude::IslandManager,
};

pub struct RapierState3D {
    pub bodies: RigidBodySet,
    pub colliders: ColliderSet,
    pub joints: JointSet,
    pub islands: IslandManager,
    pub broad_phase: BroadPhase,
    pub narrow_phase: NarrowPhase,
    pub ccd_solver: CCDSolver,
}

#[derive(Default)]
pub struct PhysicalBody<B, C, J> {
    bodies: HashMap<B, RigidBodyHandle>,
    colliders: HashMap<C, ColliderHandle>,
    joints: HashMap<J, JointHandle>,
}

impl<B, C, J> PhysicalBody<B, C, J> {
    // pub fn body(&self, key: &B) -> &RigidBody {}
}
