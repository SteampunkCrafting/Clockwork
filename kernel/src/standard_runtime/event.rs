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
/// convertible from and into the BaseEvent.
///
/// Many mechanisms, as well as the default main loop, provided in the main repository
/// requires this trait to be implemented on the event type.
pub trait FromIntoStandardEvent
where
    Self: ClockworkEvent + Into<StandardEvent> + From<StandardEvent>,
{
}
impl<E> FromIntoStandardEvent for E where
    E: ClockworkEvent + Into<StandardEvent> + From<StandardEvent>
{
}
