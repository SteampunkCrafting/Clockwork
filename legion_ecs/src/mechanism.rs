use crate::state::LegionState;
use clockwork_core::prelude::*;
pub use legion::system;
use legion::{systems::Builder, systems::ParallelRunnable, Schedule};

pub struct LegionSystems(Schedule);
impl<S, E> Mechanism<S, E> for LegionSystems
where
    S: Substate<LegionState>,
    E: ClockworkEvent,
{
    fn name(&self) -> &'static str {
        "Legion systems"
    }

    fn clink(&mut self, state: &mut S, _: E) {
        let LegionState {
            world, resources, ..
        } = state.substate_mut();
        self.0.execute(world, resources)
    }
}
impl LegionSystems {
    pub fn builder() -> LegionSystemsBuilder {
        LegionSystemsBuilder(Schedule::builder())
    }
}

pub struct LegionSystemsBuilder(Builder);
impl LegionSystemsBuilder {
    pub fn with_system(mut self, system: impl ParallelRunnable + 'static) -> Self {
        self.0.add_system(system);
        self
    }

    pub fn build(mut self) -> LegionSystems {
        LegionSystems(self.0.build())
    }
}
