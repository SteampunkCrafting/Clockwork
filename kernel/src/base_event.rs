use std::{fmt::Debug, marker::PhantomData};

use crate::{
    event::ClockworkEvent,
    prelude::{ClockworkState, EngineState, Mechanism},
};

/// A base event of the Clockwork.
///
/// Even if it is not required to blindly use this exact
/// event type for all usecases, the variants of this enumeration
/// represent the most important event types of every game engine runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BaseEvent {
    /// During this event, all mechanisms initialize their
    /// internal states, as well as the parts of a shared state,
    /// which belong to them.
    Initialization,

    /// During this event, all mechanisms update their internal states,
    /// as well as the parts of shared state, which belong to them.
    /// An example of an action, taken during Tick, is a physics step computation.
    Tick,

    /// During this event, all mechanisms perform IO-bound operations,
    /// such as rendering and presenting images, playing sounds, and writing to files.
    Draw,

    /// During this event, all mechanisms dispose their internal states, as well as their
    /// parts of a shared state.
    ///
    /// > For majority of cases, Rust language disposes all objects automatically, so this
    ///   kind of event does not have to be handled in all cases except for manual memory allocation.
    Termination,
}

/// A trait, which is automatically implemented for every custom ClockworkEvent,
/// convertible from and into the BaseEvent.
///
/// Many mechanisms, as well as the default main loop, provided in the main repository
/// requires this trait to be implemented on the event type.
pub trait FromIntoBaseEvent: ClockworkEvent + Into<BaseEvent> + From<BaseEvent> {}
impl<E> FromIntoBaseEvent for E where E: ClockworkEvent + Into<BaseEvent> + From<BaseEvent> {}

/// A subset of Mechanisms, which is meant to work with the BaseEvent.
pub trait BaseEventMechanism<S>
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
    fn handled_events(&self) -> Option<Vec<BaseEvent>>;
}

/// A wrapper for the BaseEventMechanism.
///
/// This structure is used in order to give impl Mechanism to every instance
/// of BaseEventMechanism
pub(crate) struct BaseEventMechanismWrapper<T, S>(T, PhantomData<S>)
where
    T: BaseEventMechanism<S>,
    S: ClockworkState;

impl<T, S> From<T> for BaseEventMechanismWrapper<T, S>
where
    T: BaseEventMechanism<S>,
    S: ClockworkState,
{
    fn from(mechanism: T) -> Self {
        Self(mechanism, Default::default())
    }
}

impl<T, S, E> Mechanism<S, E> for BaseEventMechanismWrapper<T, S>
where
    T: BaseEventMechanism<S>,
    S: ClockworkState,
    E: FromIntoBaseEvent,
{
    fn clink(&mut self, state: &mut EngineState<S>, event: E) {
        (match Into::<BaseEvent>::into(event) {
            BaseEvent::Initialization => T::initialization,
            BaseEvent::Tick => T::tick,
            BaseEvent::Draw => T::draw,
            BaseEvent::Termination => T::termination,
        })(&mut self.0, state)
    }

    fn handled_events(&self) -> Option<Vec<E>> {
        BaseEventMechanism::handled_events(&self.0)
            .map(IntoIterator::into_iter)
            .map(|el| el.map(Into::into))
            .map(Iterator::collect)
    }
}
