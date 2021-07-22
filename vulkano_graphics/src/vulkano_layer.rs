use crate::mechanism::GraphicsState;
use clockwork_core::clockwork::ClockworkState;
use vulkano::command_buffer::AutoCommandBufferBuilder;

pub trait VulkanoLayer<S>
where
    S: ClockworkState,
{
    fn init(&mut self, state: &S, graphics_state: &GraphicsState);
    fn draw(&mut self, state: &S, command_buffer: &mut AutoCommandBufferBuilder);
}
