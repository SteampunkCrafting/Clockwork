use kernel::abstract_runtime::ClockworkState;
use rapier3d::{
    dynamics::{CCDSolver, JointSet, RigidBodySet},
    geometry::{BroadPhase, ColliderSet, NarrowPhase},
    math::Vector,
    prelude::IslandManager,
};

#[derive(Clone, Copy, Debug, Default)]
pub struct Gravity(pub Vector<f32>);

pub struct PhysicsState {
    pub gravity: Gravity,
    pub bodies: RigidBodySet,
    pub colliders: ColliderSet,
    pub joints: JointSet,
    pub islands: IslandManager,
    pub broad_phase: BroadPhase,
    pub narrow_phase: NarrowPhase,
    pub ccd_solver: CCDSolver,
}

impl ClockworkState for PhysicsState {}

impl<T> From<T> for Gravity
where
    T: Into<Vector<f32>>,
{
    fn from(x: T) -> Self {
        Self(x.into())
    }
}

impl Default for PhysicsState {
    fn default() -> Self {
        PhysicsState {
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
