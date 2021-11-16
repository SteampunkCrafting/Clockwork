use crate::abstract_runtime::ClockworkState;
use std::time::{Duration, Instant};

/// Runtime statistics is a trait,
/// which some Mechanisms are expecting to be implemented
/// on one of the substates of the Clockwork superstate.
///
/// It is used by the mechanisms, which require time tracking
/// between the events (e.g. physics simulators).
pub trait StandardRuntimeStatistics
where
    Self: ClockworkState,
{
    /// A frequency measurement, usually a floating point number.
    type Frequency;

    /// A count measurement, usually an unsigned integer number.
    type Count;

    /// A function, which converts `Duration` to `Self::Frequency`.
    ///
    /// For an `f32` frequency, the conversion would look like:
    /// `1f32 / duration.as_sec_f32()`
    fn duration_to_freq(duration: Duration) -> Self::Frequency;

    /// Gets the actual tick delta.
    ///
    /// Tick delta is the time duration between previous
    /// tick start, and a current tick start.
    fn current_tick_delta(&self) -> Duration;

    /// Gets the actual ticks-per-second ratio.
    ///
    /// The method has a default implementation.
    fn current_tps(&self) -> Self::Frequency {
        <Self as StandardRuntimeStatistics>::duration_to_freq(self.current_tick_delta())
    }

    /// Gets the desired average tick delta
    /// (to which the system is trying to get close to).
    ///
    /// Tick delta is the time duration between previous
    /// tick start, and a current tick start.
    fn desired_avg_tick_delta(&self) -> Duration;

    /// Gets the desired average tick frequency.
    ///
    /// The method has a default implementation.
    fn desired_avg_tps(&self) -> Self::Frequency {
        <Self as StandardRuntimeStatistics>::duration_to_freq(self.desired_avg_tick_delta())
    }

    /// Gets the actual draw delta.
    ///
    /// Draw delta is the time duration between previous
    /// draw call start, and a current draw call start.
    fn current_draw_delta(&self) -> Duration;

    /// Gets the actual frames-per-second ratio.
    ///
    /// The method has a default implementation.
    fn current_fps(&self) -> Self::Frequency {
        <Self as StandardRuntimeStatistics>::duration_to_freq(self.current_draw_delta())
    }

    /// Gets the desired minimum draw delta
    /// (to which the system is trying to get close to, but never get smaller).
    ///
    /// Draw delta is the time duration between previous
    /// draw call start, and a current draw call start.
    fn desired_min_draw_period(&self) -> Duration;

    /// Gets the desired maximum frames per second
    /// (to which the system is trying to get close to, but never get higher).
    ///
    /// This method has a default implementation.
    fn desired_max_fps(&self) -> Self::Frequency {
        <Self as StandardRuntimeStatistics>::duration_to_freq(self.desired_min_draw_period())
    }

    /// Gets the time, at which the runtime has been initialized.
    fn init_time(&self) -> Instant;

    /// Gets the total amount of ticks since runtime start.
    fn ticks_total(&self) -> Self::Count;

    /// Gets the total amount of draw calls since runtime start.
    fn frames_total(&self) -> Self::Count;
}
