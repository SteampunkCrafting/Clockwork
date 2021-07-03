use core::prelude::Lock;
use core::sync::WriteLock;

use legion_ecs::state::LegionStateBuilder;
use rapier3d::{
    dynamics::{CCDSolver, JointSet, RigidBodySet},
    geometry::{BroadPhase, ColliderSet, NarrowPhase},
    math::Vector,
    prelude::IslandManager,
};

#[derive(Clone, Copy, Debug, Default)]
pub struct Gravity(pub Vector<f32>);

pub struct RapierState3D {
    pub(crate) gravity: WriteLock<Gravity>,
    pub(crate) bodies: WriteLock<RigidBodySet>,
    pub(crate) colliders: WriteLock<ColliderSet>,
    pub(crate) joints: WriteLock<JointSet>,
    pub(crate) islands: WriteLock<IslandManager>,
    pub(crate) broad_phase: WriteLock<BroadPhase>,
    pub(crate) narrow_phase: WriteLock<NarrowPhase>,
    pub(crate) ccd_solver: WriteLock<CCDSolver>,
}

impl<T> From<T> for Gravity
where
    T: Into<Vector<f32>>,
{
    fn from(x: T) -> Self {
        Self(x.into())
    }
}

impl RapierState3D {
    pub fn install_into_legion(&self, ecs_state_builder: LegionStateBuilder) -> LegionStateBuilder {
        let Self {
            gravity,
            bodies,
            colliders,
            joints,
            islands,
            broad_phase,
            narrow_phase,
            ccd_solver,
        } = self;
        ecs_state_builder
            .with_resource(gravity.downgrade_to_user_lock())
            .with_resource(bodies.downgrade_to_user_lock())
            .with_resource(colliders.downgrade_to_user_lock())
            .with_resource(joints.downgrade_to_user_lock())
            .with_resource(islands.downgrade_to_user_lock())
            .with_resource(broad_phase.downgrade_to_user_lock())
            .with_resource(narrow_phase.downgrade_to_user_lock())
            .with_resource(ccd_solver.downgrade_to_user_lock())
    }
}

impl Default for RapierState3D {
    fn default() -> Self {
        RapierState3D {
            gravity: Gravity::default().into(),
            bodies: RigidBodySet::new().into(),
            colliders: ColliderSet::new().into(),
            joints: JointSet::new().into(),
            islands: IslandManager::new().into(),
            broad_phase: BroadPhase::new().into(),
            narrow_phase: NarrowPhase::new().into(),
            ccd_solver: CCDSolver::new().into(),
        }
    }
}
