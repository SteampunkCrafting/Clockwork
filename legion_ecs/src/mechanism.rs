use std::collections::HashMap;

use crate::state::LegionState;
use clockwork_core::prelude::*;
pub use legion::system;
use legion::{systems::Builder, systems::ParallelRunnable, Schedule};

pub struct LegionSystems<E>(HashMap<E, Schedule>)
where
    E: ClockworkEvent;
impl<S, E> Mechanism<S, E> for LegionSystems<E>
where
    S: Substate<LegionState>,
    E: ClockworkEvent,
{
    fn name(&self) -> &'static str {
        "Legion systems"
    }

    fn clink(&mut self, state: &mut S, event: E) {
        let LegionState {
            world, resources, ..
        } = state.substate_mut();
        if let Some(schedule) = self.0.get_mut(&event) {
            schedule.execute(world, resources)
        }
    }
}
impl<E> LegionSystems<E>
where
    E: ClockworkEvent,
{
    pub fn builder() -> LegionSystemsBuilder<E> {
        LegionSystemsBuilder(Default::default())
    }
}

pub struct LegionSystemsBuilder<E>(HashMap<E, Builder>)
where
    E: ClockworkEvent;

impl<E> LegionSystemsBuilder<E>
where
    E: ClockworkEvent,
{
    pub fn with_system(mut self, event: E, system: impl ParallelRunnable + 'static) -> Self {
        self.0.entry(event).or_default().add_system(system);
        self
    }

    pub fn build(mut self) -> LegionSystems<E> {
        LegionSystems(
            self.0
                .iter_mut()
                .map(|(e, b)| (e.clone(), b.build()))
                .collect(),
        )
    }
}
