use crate::prelude::{ClockworkEvent, ClockworkState, EngineState};
use std::*;

/// Mechanism is an event handler to clockwork events.
///
/// Is able to read from, and write into the game state.
pub trait Mechanism<S, E>
where
    S: ClockworkState,
    E: ClockworkEvent,
{
    /// Defines a mechanism name
    fn name(&self) -> &'static str;

    /// Defines a reaction of the mechanism on the event
    fn clink(&mut self, state: &mut EngineState<S>, event: E);

    /// Defines a set of events, which this mechanism is handling.
    /// The method is called once during the mechanisms assembly.
    /// Has a default implementation.
    /// If None is returned, then the mechanism will be clinkd upon every event.
    /// It is recommended to implement this manually, as it might save some cpu
    /// resources, especially if there is a complex event system, or big amount of
    /// mechanisms.
    fn handled_events(&self) -> Option<&'static [E]> {
        None
    }
}

/// A read-only version of Mechanism.
///
/// Is not able to write into the game state.
pub trait ReadMechanism<S, E>
where
    S: ClockworkState,
    E: ClockworkEvent,
{
    /// Defines a mechanism name
    fn name(&self) -> &'static str;

    /// Defines a reaction of the mechanism on the event
    fn clink(&mut self, state: &EngineState<S>, event: E);

    /// Defines a set of events, which this mechanism is handling.
    /// The method is called once during the mechanisms assembly.
    /// Has a default implementation.
    /// If None is returned, then the mechanism will be clinkd upon every event.
    /// It is recommended to implement this manually, as it might save some cpu
    /// resources, especially if there is a complex event system, or big amount of
    /// mechanisms.
    fn handled_events(&self) -> Option<&'static [E]> {
        None
    }
}

/// A struct, which is owned by the main loop.
/// Its purpose is to pass events, produced by the main loop,
/// to the mechanisms, but only if the mechanisms are subscribed
/// to this kind of event.
pub struct Mechanisms<S, E>
where
    S: ClockworkState,
    E: ClockworkEvent,
{
    /// Mechanism storage
    all_mechanisms: Vec<Box<dyn Mechanism<S, E>>>,

    /// Mapping from event to mechanism indices
    events_to_mechanisms: collections::HashMap<E, Vec<usize>>,

    /// A set of mechanisms, which respond to every event
    any_event_mechanisms: Vec<usize>,

    /// Read mechanism storage
    all_read_mechanisms: Vec<Box<dyn ReadMechanism<S, E>>>,

    /// Mapping from event to read mechanism indices
    events_to_read_mechanisms: collections::HashMap<E, Vec<usize>>,

    /// A set of read mechanisms, which respond to every event
    any_event_read_mechanisms: Vec<usize>,
}

impl<S, E> Mechanisms<S, E>
where
    S: ClockworkState,
    E: ClockworkEvent,
{
    /// Gets a mutable reference to the state, and event,
    /// then calls `clink` on every `Mechanism`, and `ReadMechanism`
    /// instance, which has been subscribed to the event of this kind.
    pub fn clink_event(&mut self, state: &mut EngineState<S>, event: E) {
        let Self {
            all_mechanisms,
            events_to_mechanisms,
            any_event_mechanisms,
            all_read_mechanisms,
            events_to_read_mechanisms,
            any_event_read_mechanisms,
        } = self;

        for id in any_event_mechanisms {
            unsafe {
                all_mechanisms
                    .get_unchecked_mut(*id)
                    .clink(state, event.clone());
            }
        }

        if let Some(ids) = events_to_mechanisms.get(&event) {
            for id in ids {
                unsafe {
                    all_mechanisms
                        .get_unchecked_mut(*id)
                        .clink(state, event.clone());
                }
            }
        }

        for id in any_event_read_mechanisms {
            unsafe {
                all_read_mechanisms
                    .get_unchecked_mut(*id)
                    .clink(state, event.clone());
            }
        }

        if let Some(ids) = events_to_read_mechanisms.get(&event) {
            for id in ids {
                unsafe {
                    all_read_mechanisms
                        .get_unchecked_mut(*id)
                        .clink(state, event.clone());
                }
            }
        }
    }
}

/// A private builder for the Mechanisms struct
pub(crate) struct MechanismsBuilder<S, E>
where
    S: ClockworkState,
    E: ClockworkEvent,
{
    all_mechanisms: Vec<Box<dyn Mechanism<S, E>>>,
    events_to_mechanisms: collections::HashMap<E, Vec<usize>>,
    any_event_mechanisms: Vec<usize>,
    all_read_mechanisms: Vec<Box<dyn ReadMechanism<S, E>>>,
    events_to_read_mechanisms: collections::HashMap<E, Vec<usize>>,
    any_event_read_mechanisms: Vec<usize>,
}

impl<'a, S, E> MechanismsBuilder<S, E>
where
    S: ClockworkState,
    E: ClockworkEvent,
{
    pub fn with_mechanism(mut self, mechanism: impl Mechanism<S, E> + 'static) -> Self {
        let Self {
            all_mechanisms,
            events_to_mechanisms,
            any_event_mechanisms,
            ..
        } = &mut self;
        let id = all_mechanisms.len();

        if let Some(events) = mechanism.handled_events() {
            for event in events {
                events_to_mechanisms
                    .entry(event.clone())
                    .or_default()
                    .push(id);
            }
        } else {
            any_event_mechanisms.push(id);
        }

        all_mechanisms.push(Box::new(mechanism));
        self
    }

    pub fn with_read_mechanism(
        mut self,
        read_mechanism: impl ReadMechanism<S, E> + 'static,
    ) -> Self {
        let Self {
            all_read_mechanisms,
            events_to_read_mechanisms,
            any_event_read_mechanisms,
            ..
        } = &mut self;
        let id = all_read_mechanisms.len();

        if let Some(events) = read_mechanism.handled_events() {
            for event in events {
                events_to_read_mechanisms
                    .entry(event.clone())
                    .or_default()
                    .push(id);
            }
        } else {
            any_event_read_mechanisms.push(id);
        }

        all_read_mechanisms.push(Box::new(read_mechanism));
        self
    }

    pub fn build(self) -> Result<Mechanisms<S, E>, &'static str> {
        let Self {
            all_mechanisms,
            events_to_mechanisms,
            any_event_mechanisms,
            all_read_mechanisms,
            events_to_read_mechanisms,
            any_event_read_mechanisms,
        } = self;
        Ok(Mechanisms {
            all_mechanisms,
            events_to_mechanisms,
            any_event_mechanisms,
            all_read_mechanisms,
            events_to_read_mechanisms,
            any_event_read_mechanisms,
        })
    }
}

impl<S, E> Default for MechanismsBuilder<S, E>
where
    S: ClockworkState,
    E: ClockworkEvent,
{
    fn default() -> Self {
        MechanismsBuilder {
            all_mechanisms: Default::default(),
            events_to_mechanisms: Default::default(),
            any_event_mechanisms: Default::default(),
            all_read_mechanisms: Default::default(),
            events_to_read_mechanisms: Default::default(),
            any_event_read_mechanisms: Default::default(),
        }
    }
}
