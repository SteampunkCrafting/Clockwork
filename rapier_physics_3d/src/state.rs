use core::{prelude::Lock, sync::WriteLock};

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
    pub fn user_locks(
        &self,
    ) -> (
        Lock<Gravity>,
        Lock<RigidBodySet>,
        Lock<ColliderSet>,
        Lock<JointSet>,
        Lock<IslandManager>,
        Lock<BroadPhase>,
        Lock<NarrowPhase>,
        Lock<CCDSolver>,
    ) {
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

        (
            gravity.downgrade_to_user_lock(),
            bodies.downgrade_to_user_lock(),
            colliders.downgrade_to_user_lock(),
            joints.downgrade_to_user_lock(),
            islands.downgrade_to_user_lock(),
            broad_phase.downgrade_to_user_lock(),
            narrow_phase.downgrade_to_user_lock(),
            ccd_solver.downgrade_to_user_lock(),
        )
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
