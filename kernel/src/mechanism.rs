use crate::{
    event::ClockworkEvent,
    prelude::{ClockworkState, EngineState},
};
use itertools::Itertools;
use std::collections::HashMap;

/// Mechanism is an event handler to clockwork events.
///
/// Is able to read from, and write into the game state.
pub trait Mechanism<S, E>
where
    S: ClockworkState,
    E: ClockworkEvent,
{
    /// Defines a reaction of the mechanism on the event
    fn clink(&mut self, state: &mut EngineState<S>, event: E);

    /// Defines a set of events, which this mechanism is handling.
    /// The method is called once during the mechanisms assembly.
    /// If None is returned, then the mechanism will be clinked upon every event.
    /// It is recommended to implement this manually, as it might save some cpu
    /// resources, especially if there is a complex event system, or big amount of
    /// mechanisms.
    fn handled_events(&self) -> Option<Vec<E>>;
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
    events_to_mechanisms: HashMap<E, Vec<usize>>,

    /// A set of mechanisms, which respond to every event
    any_event_mechanisms: Vec<usize>,
}

impl<S, E> Default for Mechanisms<S, E>
where
    S: ClockworkState,
    E: ClockworkEvent,
{
    fn default() -> Self {
        Mechanisms {
            all_mechanisms: Default::default(),
            events_to_mechanisms: Default::default(),
            any_event_mechanisms: Default::default(),
        }
    }
}

impl<S, E> Mechanisms<S, E>
where
    S: ClockworkState,
    E: ClockworkEvent,
{
    /// Adds a mechanism to the struct.
    ///
    /// This method is crate-private.
    /// Use `ClockworkBuilder::add_mechanism` to add mechanism into clockwork.
    pub(crate) fn add_mechanism(&mut self, mechanism: impl Mechanism<S, E> + 'static) {
        let Self {
            all_mechanisms,
            events_to_mechanisms,
            any_event_mechanisms,
            ..
        } = self;
        let mechanism_id = all_mechanisms.len();
        mechanism
            .handled_events()
            .map(IntoIterator::into_iter)
            .map(Itertools::unique)
            .map(|it| {
                it.for_each(|event| {
                    events_to_mechanisms
                        .entry(event.clone())
                        .or_default()
                        .push(mechanism_id)
                })
            })
            .or_else(|| Some(any_event_mechanisms.push(mechanism_id)))
            .map(|_| all_mechanisms.push(Box::from(mechanism)))
            .unwrap()
    }

    /// Gets a mutable reference to the state, and event,
    /// then calls `clink` on every `Mechanism`, and `ReadMechanism`
    /// instance, which has been subscribed to the event of this kind.
    pub fn clink_event(&mut self, state: &mut EngineState<S>, event: E) {
        let Self {
            ref mut all_mechanisms,
            ref events_to_mechanisms,
            ref any_event_mechanisms,
        } = self;
        events_to_mechanisms
            .get(&event)
            .map(|x| x.clone())
            .unwrap_or_default()
            .into_iter()
            .chain(any_event_mechanisms.iter().cloned())
            .for_each(|id| unsafe {
                all_mechanisms
                    .get_unchecked_mut(id)
                    .clink(state, event.clone())
            })
    }
}
