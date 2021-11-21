use std::convert::TryInto;

use crate::abstract_runtime::ClockworkEvent;

/// A base event of the Clockwork.
///
/// Even if it is not required to blindly use this exact
/// event type for all usecases, the variants of this enumeration
/// represent the most important event types of every game engine runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StandardEvent {
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
/// some of which events have a one-to-one correspondence.
///
/// This type instance can be created from the `StandardEvent` -- its subset,
/// and may possibly map to the corresponding instance of `StandardEvent`.
///
/// Because this is a superset relation, `TryInto::Error` is an empty tuple.
pub trait StandardEventSuperset
where
    Self: ClockworkEvent + TryInto<StandardEvent> + From<StandardEvent>,
{
}
impl<E> StandardEventSuperset for E where
    E: ClockworkEvent + TryInto<StandardEvent> + From<StandardEvent>
{
}
