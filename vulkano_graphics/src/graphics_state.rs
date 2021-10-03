use std::sync::Arc;
use vulkano::{
    command_buffer::DynamicState,
    device::{Device, Queue},
    framebuffer::RenderPassAbstract,
};

pub struct GraphicsState {
    pub dynamic_state: DynamicState,
    pub render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
}
