use core::prelude::Lock;

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
    pub(crate) gravity: Lock<Gravity>,
    pub(crate) bodies: Lock<RigidBodySet>,
    pub(crate) colliders: Lock<ColliderSet>,
    pub(crate) joints: Lock<JointSet>,
    pub(crate) islands: Lock<IslandManager>,
    pub(crate) broad_phase: Lock<BroadPhase>,
    pub(crate) narrow_phase: Lock<NarrowPhase>,
    pub(crate) ccd_solver: Lock<CCDSolver>,
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
            .with_resource(gravity.clone())
            .with_resource(bodies.clone())
            .with_resource(colliders.clone())
            .with_resource(joints.clone())
            .with_resource(islands.clone())
            .with_resource(broad_phase.clone())
            .with_resource(narrow_phase.clone())
            .with_resource(ccd_solver.clone())
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
