use crate::prelude::{ClockworkEvent, ClockworkState, Mechanisms};

/// A main loop trait.
///
/// Main loop is a function, which is called by a Clockwork core after its initialization.
/// Its purpose is to push the execution forward, track time, manage threads
/// (if there are more than one), and `clink` Mechanisms -- the Clockwork event handlers,
/// which implement some game logic.
///
/// Example:
/// ```
/// use std::process::exit;
/// # use spc_clockwork_kernel::prelude::*;
///
/// #[derive(Eq, PartialEq, Debug)]
/// struct State(u8);
///
/// #[derive(Clone, Eq, PartialEq, Hash, Debug)]
/// enum Event {
///     Tick
/// }
///
/// fn main_loop(
///     mut state: State,
///     mut mechanisms: Mechanisms<State, Event>
/// ) {
///     loop {
///         assert_eq!(state, State(0)); // Checking the state
///         mechanisms.clink_event(
///             &mut state,
///             Event::Tick,
///         ); // Emitting event, so that mechanisms,
///            // which are subscribed to it, do some actions
///         exit(0);
///     }
/// }
///
///
/// # fn main() -> Result<(), &'static str> {
/// Clockwork::<State, Event>::builder()
///     .with_main_loop(main_loop)
///     .with_state(State(0))
///     .build()?
///     .set_the_clock();
/// # Ok(())
/// # }
/// ```
///
/// Note: The function is expected to never return (i.e. the main loop ends with the termination
/// of a program), however, due to `never_type` not being stabilized in the language (`RFC 1216`),
/// the `MainLoop` should return `()`. After the stabilization occurs,
/// the main loop will be returning `!`, which might become a breaking change.
/// For more information, see this [issue](https://github.com/rust-lang/rust/issues/35121)
pub trait MainLoop<S, E>: FnOnce(S, Mechanisms<S, E>)
where
    S: ClockworkState,
    E: ClockworkEvent,
{
}
impl<T, S, E> MainLoop<S, E> for T
where
    T: FnOnce(S, Mechanisms<S, E>),
    S: ClockworkState,
    E: ClockworkEvent,
{
}
