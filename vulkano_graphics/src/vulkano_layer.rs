use crate::state::GraphicsState;
use clockwork_core::clockwork::ClockworkState;
use vulkano::command_buffer::{
    pool::standard::StandardCommandPoolAlloc, AutoCommandBufferBuilder, PrimaryAutoCommandBuffer,
};

pub trait VulkanoLayer<S>
where
    S: ClockworkState,
{
    fn draw(
        &mut self,
        engine_state: &S,
        graphics_state: &GraphicsState,
        command_buffer: &mut AutoCommandBufferBuilder<
            PrimaryAutoCommandBuffer<StandardCommandPoolAlloc>,
        >,
    );
}

impl<T, S> VulkanoLayer<S> for T
where
    T: FnMut(
        &S,
        &GraphicsState,
        &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer<StandardCommandPoolAlloc>>,
    ),
    S: ClockworkState,
{
    fn draw(
        &mut self,
        engine_state: &S,
        graphics_state: &GraphicsState,
        command_buffer: &mut AutoCommandBufferBuilder<
            PrimaryAutoCommandBuffer<StandardCommandPoolAlloc>,
        >,
    ) {
        self(engine_state, graphics_state, command_buffer)
    }
}
