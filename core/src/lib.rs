use log::*;
pub use mechanism::*;
use std::*;

pub trait ClockworkState: Send + Sized {}
impl<T> ClockworkState for T where T: Send + Sized {}

pub trait ClockworkEvent: Send + Clone + Eq + hash::Hash + fmt::Debug {}
impl<T> ClockworkEvent for T where T: Send + Clone + Eq + hash::Hash + fmt::Debug {}

pub trait MainLoop<S, E>: FnOnce(Box<S>, Mechanisms<S, E>)
where
    S: ClockworkState,
    E: ClockworkEvent,
{
}

impl<T, S, E> MainLoop<S, E> for T
where
    T: FnOnce(Box<S>, Mechanisms<S, E>),
    S: ClockworkState,
    E: ClockworkEvent,
{
}

mod mechanism;

pub struct Clockwork<S, E>
where
    S: ClockworkState,
    E: ClockworkEvent,
{
    main_loop: Box<dyn MainLoop<S, E>>,
    state: Box<S>,
    mechanisms: Mechanisms<S, E>,
}

impl<S, E> Clockwork<S, E>
where
    S: ClockworkState,
    E: ClockworkEvent,
{
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

    pub fn builder() -> ClockworkBuilder<S, E> {
        info!("Constructing Clockwork builder");
        Default::default()
    }
}

pub struct ClockworkBuilder<S, E>(
    Option<Box<dyn MainLoop<S, E>>>,
    Option<Box<S>>,
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
    pub fn with_main_loop(self, main_loop: impl MainLoop<S, E> + 'static) -> Self {
        info!("Adding custom Mail Loop");
        Self {
            0: Some(Box::new(main_loop)),
            ..self
        }
    }

    pub fn with_state(self, state: impl Into<Box<S>>) -> Self {
        info!("Setting initial engine state");
        Self {
            1: Some(state.into()),
            ..self
        }
    }

    pub fn with_mechanism(
        self,
        mechanism: impl Mechanism<S, E> + 'static,
        events: impl IntoIterator<Item = E> + fmt::Debug,
    ) -> Self {
        info!("Adding mechanism: {}", mechanism.name());
        info!(
            "Subscribing mechanism \"{}\" to events: {:?}",
            mechanism.name(),
            events
        );
        let Self(main_loop, state, mechanisms) = self;
        Self(
            main_loop,
            state,
            mechanisms.with_mechanism(mechanism, events),
        )
    }

    pub fn with_read_mechanism(
        self,
        read_mechanism: impl ReadMechanism<S, E> + 'static,
        events: impl IntoIterator<Item = E> + fmt::Debug,
    ) -> Self {
        info!("Adding read-only mechanism: {}", read_mechanism.name());
        info!(
            "Subscribing mechanism \"{}\" to events: {:?}",
            read_mechanism.name(),
            events
        );
        let Self(main_loop, state, mechanisms) = self;
        Self(
            main_loop,
            state,
            mechanisms.with_read_mechanism(read_mechanism, events),
        )
    }

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
