use crate::mechanism::*;
use log::*;
use std::*;

/// A set of constraints, which every valid Clockwork state should satisfy.
pub trait ClockworkState: Sized + 'static {}
impl<T> ClockworkState for T where T: Sized + 'static {}

/// A set of constraints, which every valid Clockwork event type should satisfy.
pub trait ClockworkEvent: Send + Clone + Eq + hash::Hash + fmt::Debug + 'static {}
impl<T> ClockworkEvent for T where T: Send + Clone + Eq + hash::Hash + fmt::Debug + 'static {}

/// A substate of a clockwork state.
///
/// In order to provide statically-resolved, yet generic system of states
/// (so that we can toss around mechanisms), we require to heavily rely on
/// delegations. For this purpose, a `Substate` trait is being introduced.
///
/// Being a `Substate<S>` basically means one of:
/// - Being `S`
/// - Containing `S`
///
/// (Remember that having these two conditions satisfied is a logical contradiction)
///
/// A first thing to notice is that a valid `ClockworkState` `S` automatically
/// implies `Substate<S>`:
///
/// ```
/// # use spc_clockwork_core::prelude::*;
/// // Assume that a certain clockwork mechanism requires an i32 counter
/// struct A(i32);
/// let a: &A = A(0).substate();
/// let a_mut: &mut A = A(0).substate_mut();
/// ```
///
/// This means that if there is no composite state in our program, we do not have to implement
/// substates to utilize generic mechanisms.
///
/// Obviously, there would be no point in using substates, if we didn't compose them:
/// ```
/// # use spc_clockwork_core::prelude::*;
/// /* ---- Event section ---- */
/// type Event = ();
///
/// /* ---- State section ---- */
/// struct A(i32); // A state of one mechanism
///
/// struct B(Result<f32, &'static str>); // A state of another mechanism
///
/// struct S(A, B); // A combined state of both mechanisms
/// impl Substate<A> for S {
///     fn substate(&self) -> &A { self.0.substate() }
///     fn substate_mut(&mut self) -> &mut A { self.0.substate_mut() }
/// }
/// impl Substate<B> for S {
///     fn substate(&self) -> &B { self.1.substate() }
///     fn substate_mut(&mut self) -> &mut B { self.1.substate_mut() }
/// }
///
/// /* ---- Mechanism section ---- */
/// struct MechanismA;
/// impl<S> Mechanism<S, Event> for MechanismA
/// where
///     S: Substate<A>
/// {
///     fn name(&self) -> &'static str { "Mechanism A" }
///     fn clink(&mut self,state: &mut S,event: ()) {
///         let state: &mut A = state.substate_mut();
///         // ...
///     }
/// }
///
/// struct MechanismB;
/// impl<S> ReadMechanism<S, Event> for MechanismB
/// where
///     S: Substate<B>
/// {
///     fn name(&self) -> &'static str { "Mechanism B" }
///     fn clink(&mut self, state: &S, event: ()) {
///         let state: &B = state.substate();
///         // ...
///     }
/// }
///
/// /* ---- Main loop section ---- */
/// let mut state: S = S( // Creating state
///     A(128),
///     B(Ok(3.14))
/// );
///
/// let mut mechanism_a = MechanismA;
/// let mut mechanism_b = MechanismB;
///
/// mechanism_a.clink(&mut state, ());
/// mechanism_b.clink(&state, ());
/// ```
///
/// Note: this chaining of states may produce a lot of unnecessary code due
/// to lots of trait delegations. In the future, this should be overcome by
/// providing macros.
pub trait Substate<S>: CallbackSubstate<S> + ClockworkState
where
    S: ClockworkState,
{
    /// Gets an immutable reference to the substate
    fn substate(&self) -> &S;
    /// Gets a mutable reference to the substate
    fn substate_mut(&mut self) -> &mut S;
}
impl<S> Substate<S> for S
where
    S: ClockworkState,
{
    fn substate(&self) -> &S {
        self
    }

    fn substate_mut(&mut self) -> &mut S {
        self
    }
}

/// A substate of a clockwork state, accessible via callback.
///
/// This is a more general type trait, which should be requested by
/// mechanisms.
pub trait CallbackSubstate<S>: ClockworkState
where
    S: ClockworkState,
{
    /// Executes provided callback, supplying its substate reference
    fn callback_substate(&self, callback: impl FnOnce(&S));
    /// Executes provided callback, supplying its mutable substate reference
    fn callback_substate_mut(&mut self, callback: impl FnOnce(&mut S));
}

/// Having substate always implies a possibility
/// to execute a callback on this substate.
/// This is not always true the other way.
impl<T, S> CallbackSubstate<S> for T
where
    T: Substate<S>,
    S: ClockworkState,
{
    fn callback_substate(&self, callback: impl FnOnce(&S)) {
        callback(self.substate())
    }

    fn callback_substate_mut(&mut self, callback: impl FnOnce(&mut S)) {
        callback(self.substate_mut())
    }
}

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
/// # use spc_clockwork_core::prelude::*;
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

/// `Clockwork` is a type, which represents the game engine.
pub struct Clockwork<S, E>
where
    S: ClockworkState,
    E: ClockworkEvent,
{
    main_loop: Box<dyn MainLoop<S, E>>,
    state: S,
    mechanisms: Mechanisms<S, E>,
}
impl<S, E> Clockwork<S, E>
where
    S: ClockworkState,
    E: ClockworkEvent,
{
    /// Initializes the game engine, and starts the main loop.
    pub fn set_the_clock(self) {
        let Self {
            main_loop,
            state,
            mechanisms,
        } = self;
        info!("Starting Clockwork Engine");
        main_loop(state, mechanisms);
        info!("Terminating Clockwork Engine");
    }

    /// Creates a new builder struct, which is used for the Clockwork assembly.
    pub fn builder() -> ClockworkBuilder<S, E> {
        info!("Constructing Clockwork builder");
        Default::default()
    }
}

/// A builder of the Clockwork game engine.
pub struct ClockworkBuilder<S, E>(
    Option<Box<dyn MainLoop<S, E>>>,
    Option<S>,
    MechanismsBuilder<S, E>,
)
where
    S: ClockworkState,
    E: ClockworkEvent;
impl<S, E> ClockworkBuilder<S, E>
where
    S: ClockworkState,
    E: ClockworkEvent,
{
    /// Sets the main loop to the engine
    pub fn with_main_loop(self, main_loop: impl MainLoop<S, E> + 'static) -> Self {
        info!("Adding Main Loop");
        Self {
            0: Some(Box::new(main_loop)),
            ..self
        }
    }

    /// Sets the initial engine state
    pub fn with_state(self, state: impl Into<S>) -> Self {
        info!("Setting initial engine state");
        Self {
            1: Some(state.into()),
            ..self
        }
    }

    /// Adds a mechanism, and subscribes it to the events provided.
    ///
    /// This mechanism is allowed to do both: reading and writing the
    /// state of the game. If the mechanism only intends to read the game
    /// state, try using `with_read_mechanism`.
    pub fn with_mechanism(self, mechanism: impl Mechanism<S, E> + 'static) -> Self {
        info!("Adding mechanism: {}", mechanism.name());
        let Self(main_loop, state, mechanisms) = self;
        Self(main_loop, state, mechanisms.with_mechanism(mechanism))
    }

    /// Adds a read-only mechanism, and subscribes it to the events provided.
    ///
    /// If it is also required for the mechanism to write into the game state,
    /// use `with_mechanism` instead.
    pub fn with_read_mechanism(self, read_mechanism: impl ReadMechanism<S, E> + 'static) -> Self {
        info!("Adding read-only mechanism: {}", read_mechanism.name());
        let Self(main_loop, state, mechanisms) = self;
        Self(
            main_loop,
            state,
            mechanisms.with_read_mechanism(read_mechanism),
        )
    }

    /// Finalizes Clockwork building process, and returns a result,
    /// which is either the Clockwork instance, or a string,
    /// which explains the reason of the building failure.
    ///
    /// As for now, there exist only two reasons for this method to fail:
    /// - Missing main loop
    /// - Missing initial state
    ///
    /// Both errors can be overcome by means of calling the `with_main_loop`,
    /// and `with_state` methods.
    ///
    /// Example 1: Normal execution
    /// ```
    /// # use spc_clockwork_core::prelude::*;
    ///
    /// use std::process::exit;
    /// assert!(Clockwork::<(), ()>::builder()
    ///     .with_state(())
    ///     .with_main_loop(|_, _| exit(0))
    ///     // We are free to add some mechanisms here
    ///     .build()
    ///     .is_ok()); // We expect the result to be the Clockwork instance
    /// ```
    ///
    /// Example 2: Missing main loop
    /// ```
    /// # use spc_clockwork_core::prelude::*;
    /// assert!(Clockwork::<(), ()>::builder()
    ///     .with_state(())
    ///     // We miss the main loop here
    ///     .build()
    ///     .map_err(|err| {
    ///         // We expect the error to be related to the
    ///         // main loop (if the result of the build is error)
    ///         assert_eq!(err, "Missing main loop");
    ///         err
    ///     })
    ///     .is_err()); // We expect the result to be error
    /// ```
    ///
    /// Example 3: Missing initial state
    /// ```
    /// # use spc_clockwork_core::prelude::*;
    /// use std::process::exit;
    ///
    /// assert!(Clockwork::<(), ()>::builder()
    ///     .with_main_loop(|_, _| exit(0))
    ///     // We miss the initial state here
    ///     .build()
    ///     .map_err(|err| {
    ///         // We expect the error to be related to the
    ///         // initial state (if the result of the build is error)
    ///         assert_eq!(err, "Missing initial state");
    ///         err
    ///     })
    ///     .is_err()); // We expect the result to be error
    /// ```
    pub fn build(self) -> Result<Clockwork<S, E>, &'static str> {
        info!("Assembling Clockwork");
        let Self(main_loop, state, mechanisms) = self;
        Ok(Clockwork {
            main_loop: main_loop.ok_or("Missing main loop")?,
            state: state.ok_or("Missing initial state")?,
            mechanisms: mechanisms.build()?,
        })
    }
}
impl<S, E> Default for ClockworkBuilder<S, E>
where
    S: ClockworkState,
    E: ClockworkEvent,
{
    fn default() -> Self {
        Self(None, None, Default::default())
    }
}
