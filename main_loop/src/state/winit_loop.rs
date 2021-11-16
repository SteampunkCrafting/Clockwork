use kernel::{
    abstract_runtime::{ClockworkEvent, ClockworkState},
    util::{derive_builder::Builder, init_state::InitState},
};
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
pub struct WinitLoopState<E>
where
    E: ClockworkEvent,
{
    /// Inner (initializable) state
    inner: InitState<
        (
            EventLoop<E>,
            EventLoopProxy<E>,
            Vec<Box<dyn FnMut(&Event<E>)>>,
        ),
        Vec<Box<dyn FnMut(&Event<E>)>>,
    >,
}

impl<E> WinitLoopState<E>
where
    E: ClockworkEvent,
{
    pub fn builder() -> WinitLoopStateBuilder<E> {
        Default::default()
    }
}

impl<E> WinitLoopStateBuilder<E>
where
    E: ClockworkEvent,
{
    pub fn build(self) -> Result<WinitLoopState<E>, WinitLoopStateBuilderError> {
        let event_loop = EventLoop::with_user_event();
        let event_proxy = event_loop.create_proxy();
        Ok(WinitLoopState {
            inner: InitState::Uninit((event_loop, event_proxy, Default::default())),
        })
    }
}

impl<E> WinitLoopState<E>
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

    /// Adds event callback
    ///
    /// This is a way to subscribe for winit events, if mechanisms depend on it.
    pub fn add_event_callback(&mut self, callback: impl FnMut(&Event<E>) + 'static) {
        match &mut self.inner {
            InitState::Uninit((_, _, v)) => v,
            InitState::Init(v) => v,
            InitState::Terminated => panic!("The MainLoopState is terminated."),
        }
        .push(Box::from(callback))
    }

    /// Notifies subscribers about the winit event.
    pub(crate) fn notify(&mut self, event: &Event<E>) {
        self.inner
            .get_init_mut()
            .iter_mut()
            .for_each(|callback| callback(event))
    }

    /// Performs state initialization.
    ///
    /// Returns event loop and proxy.
    pub(crate) fn initialize(&mut self) -> (EventLoop<E>, EventLoopProxy<E>) {
        let mut ep = None;
        self.inner.initialize(|(e, p, v)| {
            ep = Some((e, p));
            v
        });
        ep.unwrap()
    }
}

impl<E> ClockworkState for WinitLoopState<E> where E: ClockworkEvent {}
