use kernel::{
    abstract_runtime::ClockworkState,
    prelude::{Deserialize, Serialize},
    util::derive_builder::Builder,
};
use rapier3d::{
    dynamics::{CCDSolver, RigidBodySet},
    geometry::{BroadPhase, ColliderSet, NarrowPhase},
    math::Vector,
    prelude::{ImpulseJointSet, IslandManager, MultibodyJointSet},
};

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct Gravity(pub Vector<f32>);

/// A physics simulation state.
///
/// Represents a world with static/kinematic/dynamic
/// bodies/colliders/joints, which exist in the same
/// space.
#[derive(Builder, Serialize, Deserialize)]
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

    /// A set of impulse joint constraints for rigid bodies.
    #[builder(default = "ImpulseJointSet::new()")]
    pub impulse_joints: ImpulseJointSet,

    /// A set of multibody joint constraints for rigid bodies.
    #[builder(default = "MultibodyJointSet::new()")]
    pub multibody_joints: MultibodyJointSet,

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
