use kernel::{
    abstract_runtime::{ClockworkEvent, ClockworkState},
    util::{
        derive_builder::Builder,
        init_state::InitState,
        sync::{ReadLock, WriteLock},
    },
};
use std::panic;
use winit::{
    event::Event,
    event_loop::{EventLoop, EventLoopProxy},
};

/// Container for Winit event loop.
///
/// Required for the initialization of mechanisms, whose functionality depends
/// on the window system.
///
/// When uninitialized, can be a source of winit `EventLoop` reference.
#[derive(Builder)]
#[builder(pattern = "owned", setter(skip), build_fn(skip))]
pub struct InitWinitState<E>
where
    E: ClockworkEvent,
{
    /// Inner (initializable) state
    inner: InitState<(EventLoop<E>, WinitLoopProxy<E>), WinitLoopProxy<E>>,
}

impl<E> InitWinitState<E>
where
    E: ClockworkEvent,
{
    pub fn builder() -> InitWinitStateBuilder<E> {
        Default::default()
    }
}

impl<E> InitWinitStateBuilder<E>
where
    E: ClockworkEvent,
{
    pub fn build(self) -> Result<InitWinitState<E>, InitWinitStateBuilderError> {
        let event_loop = EventLoop::with_user_event();
        let event_proxy = event_loop.create_proxy();
        Ok(InitWinitState {
            inner: InitState::Uninit((
                event_loop,
                WinitLoopProxy {
                    event_loop_proxy: event_proxy.into(),
                    callbacks: Vec::default().into(),
                },
            )),
        })
    }
}

impl<E> InitWinitState<E>
where
    E: ClockworkEvent,
{
    /// Getter for the event loop.
    ///
    /// # Panics
    /// The method panics, if called outside initialization stage.
    pub fn uninit_event_loop(&self) -> &EventLoop<E> {
        &self.inner.get_uninit().0
    }

    /// Proxy getter
    pub fn proxy(&self) -> &WinitLoopProxy<E> {
        match &self.inner {
            InitState::Uninit((_, p)) => p,
            InitState::Init(p) => p,
            InitState::Terminated => panic!("The MainLoopState is terminated."),
        }
    }

    /// Adds event callback
    ///
    /// This is a way to subscribe for winit events, if mechanisms depend on it.
    ///
    /// # Panics
    /// The method will panic, if called after the main loop is terminated.
    pub fn add_event_callback(&mut self, callback: impl FnMut(&Event<E>) + 'static) {
        match &mut self.inner {
            InitState::Uninit((_, proxy)) => proxy,
            InitState::Init(proxy) => proxy,
            InitState::Terminated => panic!("The MainLoopState is terminated."),
        }
        .add_event_callback(callback)
    }

    /// Triggers a clockwork event.
    ///
    /// This call will make the winit-based main loop
    /// to wake up all Mechanisms and ECS-systems, attached
    /// to this event.
    ///
    /// # Panics
    /// The method will panic, if called after the main loop is terminated.
    pub fn trigger_event(&self, event: E) {
        match &self.inner {
            InitState::Uninit((_, proxy)) => proxy,
            InitState::Init(proxy) => proxy,
            InitState::Terminated => panic!("The MainLoopState is terminated."),
        }
        .trigger_event(event)
    }

    /// Notifies subscribers about the winit event.
    pub(crate) fn notify(&mut self, event: &Event<E>) {
        self.inner.get_init_mut().notify(event)
    }

    /// Performs state initialization.
    ///
    /// Returns event loop and proxy.
    pub(crate) fn initialize(&mut self) -> (EventLoop<E>, EventLoopProxy<E>) {
        let mut ep = None;
        self.inner.initialize(|(event_loop, proxy)| {
            ep = Some((event_loop, proxy.event_loop_proxy.lock().clone()));
            proxy
        });
        ep.unwrap()
    }
}

impl<E> ClockworkState for InitWinitState<E> where E: ClockworkEvent {}

/// An initialized MainLoopState
#[derive(Clone)]
pub struct WinitLoopProxy<E>
where
    E: ClockworkEvent,
{
    /// A proxy of the main loop.
    ///
    /// Capable of dispatching custom events.
    event_loop_proxy: ReadLock<EventLoopProxy<E>>,

    /// Listeners, which are getting notified on every winit event.
    callbacks: WriteLock<Vec<Box<dyn FnMut(&Event<E>)>>>,
}

impl<E> WinitLoopProxy<E>
where
    E: ClockworkEvent,
{
    /// Notifies subscribers about the winit event.
    pub(crate) fn notify(&mut self, event: &Event<E>) {
        self.callbacks
            .lock_mut()
            .iter_mut()
            .for_each(|callback| callback(event))
    }

    /// Triggers a clockwork event.
    ///
    /// This call will make the winit-based main loop
    /// to wake up all Mechanisms and ECS-systems, attached
    /// to this event.
    ///
    /// # Panics
    /// The method will panic, if called after the main loop is terminated.
    pub fn trigger_event(&self, event: E) {
        self.event_loop_proxy.lock().send_event(event).unwrap()
    }

    /// Adds event callback
    ///
    /// This is a way to subscribe for winit events, if mechanisms depend on it.
    ///
    /// # Panics
    /// The method will panic, if called after the main loop is terminated.
    pub fn add_event_callback(&mut self, callback: impl FnMut(&Event<E>) + 'static) {
        self.callbacks.lock_mut().push(Box::new(callback))
    }
}
