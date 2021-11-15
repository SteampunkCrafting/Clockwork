use crate::state::LegionState;
use kernel::abstract_runtime::{CallbackSubstate, ClockworkEvent, EngineState, Mechanism};
pub use legion::system;
use legion::{
    systems::{self, ParallelRunnable},
    Schedule,
};
use std::collections::HashMap;

/// A LegionSystems Mechanism -- is a Mechanism, which animates the World
/// of Legion ECS, provided by LegionState through Systems, grouped by Schedules
/// -- one per event type.
///
/// During every handled event, performs a call to the schedule, which does
/// some reading and writing to the world and its resources through queries.
///
/// The World, in this case, may be seen as a database.
///
/// The ECS option should be considered primary, when it comes to application
/// dynamic behavior control.
pub struct LegionSystems<E>
where
    E: ClockworkEvent,
{
    /// A mapping from events to system collections, i.e. schedules.
    events_to_schedules: HashMap<E, Schedule>,
}

impl<E> LegionSystems<E>
where
    E: ClockworkEvent,
{
    pub fn builder() -> LegionSystemsBuilder<E> {
        Default::default()
    }
}

pub struct LegionSystemsBuilder<E>
where
    E: ClockworkEvent,
{
    /// A mapping from events to system collections, i.e. schedules.
    events_to_schedule_builders: HashMap<E, systems::Builder>,
}

impl<E> LegionSystemsBuilder<E>
where
    E: ClockworkEvent,
{
    /// Adds a system to the schedule, executed on this event type.
    pub fn add_system(mut self, event: E, system: impl ParallelRunnable + 'static) -> Self {
        self.events_to_schedule_builders
            .entry(event)
            .or_default()
            .add_system(system);
        self
    }

    /// Combines all systems, provided via `LegionSystemsBuilder::add_system`,
    /// into schedules, then builds
    ///
    /// The `Result`, returned here, is for conventional reasons only.
    /// Its value is always `Ok`.
    pub fn build(self) -> Result<LegionSystems<E>, ()> {
        Ok(LegionSystems {
            events_to_schedules: self
                .events_to_schedule_builders
                .into_iter()
                .map(|(e, mut builder)| (e, builder.build()))
                .collect(),
        })
    }
}

impl<E> Default for LegionSystemsBuilder<E>
where
    E: ClockworkEvent,
{
    fn default() -> Self {
        Self {
            events_to_schedule_builders: Default::default(),
        }
    }
}

impl<S, E> Mechanism<S, E> for LegionSystems<E>
where
    S: CallbackSubstate<LegionState>,
    E: ClockworkEvent,
{
    fn clink(&mut self, state: &mut EngineState<S>, event: E) {
        state
            .start_mutate()
            .get_mut(|LegionState { world, resources }| {
                self.events_to_schedules
                    .get_mut(&event)
                    .map_or((), |schedule| schedule.execute(world, resources))
            })
            .finish()
    }

    fn handled_events(&self) -> Option<Vec<E>> {
        Some(self.events_to_schedules.keys().cloned().collect())
    }
}
