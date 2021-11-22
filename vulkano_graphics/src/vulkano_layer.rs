use kernel::abstract_runtime::{ClockworkState, EngineState};
use vulkano::command_buffer::{
    pool::standard::StandardCommandPoolAlloc, AutoCommandBufferBuilder, PrimaryAutoCommandBuffer,
    SecondaryAutoCommandBuffer,
};

use crate::state::GraphicsState;

#[deprecated]
pub trait OldVulkanoLayer<S>
where
    S: ClockworkState,
{
    fn draw(
        &mut self,
        state: &S,
        command_buffer: &mut AutoCommandBufferBuilder<
            PrimaryAutoCommandBuffer<StandardCommandPoolAlloc>,
        >,
    );
}

impl<T, S> OldVulkanoLayer<S> for T
where
    T: FnMut(&S, &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer<StandardCommandPoolAlloc>>),
    S: ClockworkState,
{
    fn draw(
        &mut self,
        state: &S,
        command_buffer: &mut AutoCommandBufferBuilder<
            PrimaryAutoCommandBuffer<StandardCommandPoolAlloc>,
        >,
    ) {
        self(state, command_buffer)
    }
}

pub trait VulkanoLayer<S>
where
    S: ClockworkState,
{
    fn initialization(&mut self, engine_state: &EngineState<S>, graphics_state: &GraphicsState);

    fn window_resize(&mut self, engine_state: &EngineState<S>, graphics_state: &GraphicsState);

    fn draw(
        &mut self,
        engine_state: &EngineState<S>,
        graphics_state: &GraphicsState,
    ) -> SecondaryAutoCommandBuffer;

    fn termination(&mut self, engine_state: &EngineState<S>, graphics_state: &GraphicsState);
}
