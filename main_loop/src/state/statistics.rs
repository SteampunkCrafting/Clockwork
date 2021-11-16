use kernel::{
    abstract_runtime::ClockworkState,
    standard_runtime::StandardRuntimeStatistics,
    util::{derive_builder::Builder, getset::Setters},
};
use std::time;

fn x() {
    time::Duration::from_secs_f32(1f32 / 60f32);
}

/// Main loop statistics
#[derive(Clone, Copy, Debug, Builder, Setters)]
#[builder(pattern = "owned", setter(into, skip))]
pub struct MainLoopStatistics {
    /// The estimated tick period
    pub(crate) current_tick_delta: time::Duration,

    /// The estimated draw period
    pub(crate) current_draw_delta: time::Duration,

    /// A start time of the execution
    #[builder(default = "time::Instant::now()")]
    pub(crate) init_time: time::Instant,

    /// The amount of ticks, passed by now
    pub(crate) ticks_total: u64,

    /// The amount of frames, drawn by now
    pub(crate) frames_total: u64,

    /// The desired tick minimum draw period.
    ///
    /// Can be set at runtime.
    #[getset(set = "pub")]
    #[builder(
        setter(skip = "false"),
        default = "time::Duration::from_secs_f32(1f32 / 60f32)"
    )]
    pub(crate) desired_min_draw_period: time::Duration,

    /// The desired tick minimum draw period.
    ///
    /// Can be set at runtime.
    #[getset(set = "pub")]
    #[builder(
        setter(skip = "false"),
        default = "time::Duration::from_secs_f32(1f32 / 60f32)"
    )]
    pub(crate) desired_avg_tick_period: time::Duration,
}

impl MainLoopStatistics {
    pub fn builder() -> MainLoopStatisticsBuilder {
        Default::default()
    }
}

impl ClockworkState for MainLoopStatistics {}

impl StandardRuntimeStatistics for MainLoopStatistics {
    type Frequency = f32;
    type Count = u64;

    fn duration_to_freq(duration: time::Duration) -> Self::Frequency {
        1f32 / duration.as_secs_f32()
    }

    fn current_tick_delta(&self) -> time::Duration {
        self.current_tick_delta
    }

    fn desired_avg_tick_delta(&self) -> time::Duration {
        self.desired_avg_tick_period
    }

    fn current_draw_delta(&self) -> time::Duration {
        self.current_draw_delta
    }

    fn desired_min_draw_period(&self) -> time::Duration {
        self.desired_min_draw_period
    }

    fn init_time(&self) -> time::Instant {
        self.init_time
    }

    fn ticks_total(&self) -> Self::Count {
        self.ticks_total
    }

    fn frames_total(&self) -> Self::Count {
        self.frames_total
    }
}
