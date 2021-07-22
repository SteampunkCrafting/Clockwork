use crate::graphics_state::GraphicsState;
use clockwork_core::clockwork::ClockworkState;
use vulkano::command_buffer::AutoCommandBufferBuilder;

pub trait VulkanoLayer<S>
where
    S: ClockworkState,
{
    fn draw(
        &mut self,
        state: &S,
        graphics_state: &GraphicsState,
        command_buffer: &mut AutoCommandBufferBuilder,
    );
}

impl<T, S> VulkanoLayer<S> for T
where
    T: FnMut(&S, &GraphicsState, &mut AutoCommandBufferBuilder),
    S: ClockworkState,
{
    fn draw(
        &mut self,
        state: &S,
        graphics_state: &GraphicsState,
        command_buffer: &mut AutoCommandBufferBuilder,
    ) {
        self(state, graphics_state, command_buffer)
    }
}
