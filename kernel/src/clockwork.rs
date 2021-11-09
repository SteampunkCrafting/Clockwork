use crate::main_loop::*;
use crate::mechanism::*;
use crate::state::*;
use log::*;

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
        main_loop(EngineState(state), mechanisms);
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
    /// # use spc_clockwork_kernel::prelude::*;
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
    /// # use spc_clockwork_kernel::prelude::*;
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
    /// # use spc_clockwork_kernel::prelude::*;
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
