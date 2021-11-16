use kernel::{
    abstract_runtime::ClockworkState,
    util::{derive_builder::Builder, getset::Getters},
};
use std::collections::HashSet;
use winit::event::VirtualKeyCode;

/// Input state
#[derive(Builder, Clone, Getters)]
#[builder(pattern = "owned", setter(skip))]
pub struct InputState {
    /// A set of currently pressed keys
    #[builder(default)]
    #[getset(get = "pub")]
    pub(crate) pressed_keys: HashSet<VirtualKeyCode>,
}

impl ClockworkState for InputState {}

impl InputState {
    pub fn builder() -> InputStateBuilder {
        Default::default()
    }
}
