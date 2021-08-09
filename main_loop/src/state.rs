use crate::event::Event;
use derive_builder::Builder;
use getset::Getters;
use std::{collections::HashSet, time};
use winit::event::VirtualKeyCode;
use winit::event_loop::EventLoop;

#[derive(Getters, Builder)]
#[builder(pattern = "owned", setter(into, skip, prefix = "with"))]
pub struct IOState {
    /// A desired average tick period (an inverse of fps)
    #[builder(setter, default = "std::time::Duration::from_secs_f32(1f32 / 20f32)")]
    pub desired_tick_period: time::Duration,

    /// A desired minimum draw period (an inverse of max fps)
    #[builder(setter, default = "std::time::Duration::from_secs_f32(1f32 / 60f32)")]
    pub desired_min_draw_period: time::Duration,

    /// An input table
    pub input: Input,

    /// Runtime statistics
    pub statistics: Statistics,

    /// Winit event loop.
    /// This field is `Some` (and hence the event loop is available to mechanisms)
    /// only at the initialization stage.
    pub event_loop: Option<EventLoop<Event>>,
}
impl IOState {
    /// Creates new builder of the IOState
    pub fn builder() -> IOStateBuilder {
        Default::default()
    }

    /// Sets desired average ticks per second rate
    pub fn set_desired_tps(&mut self, tps: impl Into<f32>) {
        self.desired_tick_period = time::Duration::from_secs_f32(1f32 / tps.into());
    }

    /// Sets desired max frames per second rate
    pub fn set_desired_max_fps(&mut self, fps: impl Into<f32>) {
        self.desired_min_draw_period = std::time::Duration::from_secs_f32(1f32 / fps.into());
    }
}
impl IOStateBuilder {
    /// Sets desired average ticks per second rate
    pub fn with_desired_tps(self, fps: impl Into<f32>) -> Self {
        self.with_desired_tick_period(time::Duration::from_secs_f32(1f32 / fps.into()))
    }

    /// Sets desired max frames per second rate
    pub fn with_desired_max_fps(self, fps: impl Into<f32>) -> Self {
        self.with_desired_min_draw_period(time::Duration::from_secs_f32(1f32 / fps.into()))
    }
}

impl Default for IOState {
    fn default() -> Self {
        Self::builder()
            .with_desired_max_fps(90f32)
            .with_desired_tps(60f32)
            .build()
            .unwrap()
    }
}

#[derive(Default, Clone)]
pub struct Input {
    pub pressed_keys: HashSet<VirtualKeyCode>,
}

/// Main loop statistics
#[derive(Getters, Clone)]
pub struct Statistics {
    /// A start time of the execution
    #[getset(get = "pub")]
    pub(crate) init_time: time::Instant,
    /// The amount of ticks, passed by now
    #[getset(get = "pub")]
    pub(crate) ticks_total: u128,
    /// The amount of frames, drawn by now
    #[getset(get = "pub")]
    pub(crate) frames_total: u128,
    /// The estimated tick period
    #[getset(get = "pub")]
    pub(crate) tick_period: time::Duration,
    /// The estimated draw period
    #[getset(get = "pub")]
    pub(crate) draw_period: time::Duration,
}
impl Statistics {
    /// Gets the estimated tps value of the execution
    pub fn ticks_per_second(&self) -> f32 {
        1f32 / self.tick_period.as_secs_f32()
    }

    /// Gets the estimated fps value of the execution
    pub fn frames_per_second(&self) -> f32 {
        1f32 / self.draw_period.as_secs_f32()
    }
}
impl Default for Statistics {
    fn default() -> Self {
        Self {
            init_time: time::Instant::now(),
            ticks_total: Default::default(),
            frames_total: Default::default(),
            tick_period: time::Duration::from_secs(0),
            draw_period: time::Duration::from_secs(0),
        }
    }
}
