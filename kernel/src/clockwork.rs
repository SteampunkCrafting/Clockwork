use crate::{
    abstract_runtime::{
        ClockworkEvent, ClockworkState, EngineState, MainLoop, Mechanism, Mechanisms,
    },
    standard_runtime::{
        FromIntoStandardEvent, StandardEvent, StandardMechanism, StandardMechanismWrapper,
    },
    util::{derive_builder::Builder, log::*},
};

/// `Clockwork` is a type, which represents the game engine.
#[derive(Builder)]
#[builder(pattern = "owned", setter(into))]
pub struct Clockwork<S, E = StandardEvent>
where
    S: ClockworkState,
    E: ClockworkEvent,
{
    /// The main loop of the Clockwork runtime.
    ///
    /// It is responsible for manipulating the shared state of the application
    /// through a repetitive event emission, and the invocation of the mechanisms.
    #[builder(private, setter(name = "__main_loop", into = "false"))]
    main_loop: Box<dyn MainLoop<S, E>>,

    /// The Clockwork state, which is shared between mechanisms at runtime.
    state: S,

    /// A system of mechanisms.
    ///
    /// This data structure (at Clockwork initialization stored as builder)
    /// is storing and invoking mechanisms by event.
    #[builder(private, setter(name = "__mechanisms", into = "false"))]
    mechanisms: Mechanisms<S, E>,
}

impl<S, E> ClockworkBuilder<S, E>
where
    S: ClockworkState,
    E: ClockworkEvent,
{
    /// The main loop of the Clockwork runtime.
    ///
    /// It is responsible for manipulating the shared state of the application
    /// through a repetitive event emission, and the invocation of the mechanisms.
    pub fn main_loop(self, loop_fn: impl MainLoop<S, E> + 'static) -> Self {
        self.__main_loop(Box::new(loop_fn))
    }

    /// Adds mechanism to the engine.
    pub fn add_mechanism(mut self, mechanism: impl Mechanism<S, E> + 'static) -> Self {
        self.mechanisms
            .get_or_insert(Default::default())
            .add_mechanism(mechanism);
        self
    }

    /// Converts BaseEventMechanism into the instance of Mechanism,
    /// then adds this mechanism to the engine.
    ///
    /// This method is only available for Clockwork, whose events are convertible to BaseEvents.
    pub fn add_standard_mechanism(self, mechanism: impl StandardMechanism<S> + 'static) -> Self
    where
        E: FromIntoStandardEvent,
    {
        self.add_mechanism(StandardMechanismWrapper::from(mechanism))
    }
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
