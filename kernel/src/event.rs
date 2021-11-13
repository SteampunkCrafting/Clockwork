use std::{fmt::Debug, hash::Hash};

/// A set of constraints, which every valid Clockwork event type should satisfy.
pub trait ClockworkEvent: Send + Clone + Eq + Hash + Debug + 'static {}
impl<T> ClockworkEvent for T where T: Send + Clone + Eq + Hash + Debug + 'static {}
