use super::{FromIntoStandardEvent, StandardEvent};
use crate::abstract_runtime::{ClockworkState, EngineState, Mechanism};
use std::marker::PhantomData;

/// A subset of Mechanisms, which is meant to work with the BaseEvent.
pub trait StandardMechanism<S>
where
    S: ClockworkState,
{
    /// This handler method is expected to be called once per mechanism at
    /// the very beginning of clockwork runtime.
    ///
    /// The invocation of this method is equivalent to `Mechanism::clink`
    /// with `BaseEvent::Initialization` as the event parameter.
    ///
    /// During this event, the mechanism must initialize its internal state
    /// (if its state depends on the shared state),
    /// as well as its part of the shared sate.
    fn initialization(&mut self, state: &mut EngineState<S>);

    /// This handler method is expected to be called repetitively (once per tick) at
    /// the runtime of Clockwork's main loop.
    ///
    /// The invocation of this method is equivalent to `Mechanism::clink`
    /// with `BaseEvent::Tick` as the event parameter.
    ///
    /// During this event, the mechanism must update its internal state,
    /// as well as its part of the shared state, according to its logic.
    fn tick(&mut self, state: &mut EngineState<S>);

    /// This handler method is expected to be called repetitively (once per draw call) at
    /// the runtime of Clockwork's main loop.
    ///
    /// The invocation of this method is equivalent to `Mechanism::clink`
    /// with `BaseEvent::Draw` as the event parameter.
    ///
    /// During this event, the mechanism may update its internal state,
    /// but this stage actually is dedicated for scheduling IO-bound operations,
    /// such as rendering.
    fn draw(&mut self, state: &mut EngineState<S>);

    /// This handler method is expected to be called once at
    /// the end of Clockwork runtime.
    ///
    /// The invocation of this method is equivalent to `Mechanism::clink`
    /// with `BaseEvent::Termination` as the event parameter.
    ///
    /// During this event, the mechanism must dispose its own private state,
    /// as well as the parts of a shared state it is responsible for.
    ///
    /// > Note that for majority of cases, Rust language disposes all objects automatically, so this
    ///   kind of event does not have to be handled in all cases except for manual memory allocation.
    fn termination(&mut self, state: &mut EngineState<S>);

    /// Defines a set of events, which this mechanism is handling.
    /// The method is called once during the mechanisms assembly.
    /// If None is returned, then the mechanism will be clinked upon every event.
    /// It is recommended to implement this manually, as it might save some cpu
    /// resources, especially if there is a complex event system, or big amount of
    /// mechanisms.
    fn handled_events(&self) -> Option<Vec<StandardEvent>>;
}

/// A wrapper for the BaseEventMechanism.
///
/// This structure is used in order to give impl Mechanism to every instance
/// of BaseEventMechanism
pub(crate) struct StandardMechanismWrapper<T, S>(T, PhantomData<S>)
where
    T: StandardMechanism<S>,
    S: ClockworkState;

impl<T, S> From<T> for StandardMechanismWrapper<T, S>
where
    T: StandardMechanism<S>,
    S: ClockworkState,
{
    fn from(mechanism: T) -> Self {
        Self(mechanism, Default::default())
    }
}

impl<T, S, E> Mechanism<S, E> for StandardMechanismWrapper<T, S>
where
    T: StandardMechanism<S>,
    S: ClockworkState,
    E: FromIntoStandardEvent,
{
    fn clink(&mut self, state: &mut EngineState<S>, event: E) {
        (match Into::<StandardEvent>::into(event) {
            StandardEvent::Initialization => T::initialization,
            StandardEvent::Tick => T::tick,
            StandardEvent::Draw => T::draw,
            StandardEvent::Termination => T::termination,
        })(&mut self.0, state)
    }

    fn handled_events(&self) -> Option<Vec<E>> {
        StandardMechanism::handled_events(&self.0)
            .map(IntoIterator::into_iter)
            .map(|el| el.map(Into::into))
            .map(Iterator::collect)
    }
}
