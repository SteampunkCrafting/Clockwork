use kernel::{abstract_runtime::ClockworkState, util::derive_builder::Builder};
use rapier3d::{
    dynamics::{CCDSolver, JointSet, RigidBodySet},
    geometry::{BroadPhase, ColliderSet, NarrowPhase},
    math::Vector,
    prelude::IslandManager,
};

#[derive(Clone, Copy, Debug, Default)]
pub struct Gravity(pub Vector<f32>);

/// A physics simulation state.
///
/// Represents a world with static/kinematic/dynamic
/// bodies/colliders/joints, which exist in the same
/// space.
#[derive(Builder)]
#[builder(pattern = "owned")]
pub struct PhysicsState {
    /// A gravity vector.
    #[builder(default)]
    pub gravity: Gravity,

    /// A set of rigid bodies.
    #[builder(default = "RigidBodySet::new()")]
    pub bodies: RigidBodySet,

    /// A set of colliders.
    #[builder(default = "ColliderSet::new()")]
    pub colliders: ColliderSet,

    /// A set of joint constraints for rigid bodies.
    #[builder(default = "JointSet::new()")]
    pub joints: JointSet,

    /// Resource manager (puts bodies to sleep)
    #[builder(private, default = "IslandManager::new()")]
    pub islands: IslandManager,

    /// Broad phase solver
    #[builder(private, default = "BroadPhase::new()")]
    pub broad_phase: BroadPhase,

    /// Narrow phase solver
    #[builder(private, default = "NarrowPhase::new()")]
    pub narrow_phase: NarrowPhase,

    /// CCD solver
    #[builder(private, default = "CCDSolver::new()")]
    pub ccd_solver: CCDSolver,
}

impl PhysicsState {
    pub fn builder() -> PhysicsStateBuilder {
        Default::default()
    }
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
