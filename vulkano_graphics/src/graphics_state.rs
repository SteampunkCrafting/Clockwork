use main_loop::prelude::Window;
use std::sync::Arc;
use vulkano::{
    command_buffer::DynamicState,
    device::{Device, Queue},
    framebuffer::{FramebufferAbstract, RenderPassAbstract},
    pipeline::GraphicsPipelineAbstract,
    swapchain::{Surface, Swapchain},
    sync::GpuFuture,
};

pub struct GraphicsState {
    pub dynamic_state: DynamicState,
    pub swapchain: Arc<Swapchain<Window>>,
    pub surface: Arc<Surface<Window>>,
    pub previous_frame_end: Option<Box<dyn GpuFuture>>,
    pub recreate_swapchain: bool,
    pub framebuffers: Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,
    pub render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
    pub device: Arc<Device>,
    pub pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
    pub queue: Arc<Queue>,
}
