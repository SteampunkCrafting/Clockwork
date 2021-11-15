use kernel::abstract_runtime::CallbackSubstate;
use vulkano::command_buffer::{
    pool::standard::StandardCommandPoolAlloc, AutoCommandBufferBuilder, PrimaryAutoCommandBuffer,
};

use crate::state::GraphicsInitState;

pub trait StateRequirements: CallbackSubstate<GraphicsInitState> {}
impl<T> StateRequirements for T where T: CallbackSubstate<GraphicsInitState> {}

pub trait VulkanoLayer<S>
where
    S: StateRequirements,
{
    fn draw(
        &mut self,
        state: &S,
        command_buffer: &mut AutoCommandBufferBuilder<
            PrimaryAutoCommandBuffer<StandardCommandPoolAlloc>,
        >,
    );
}

impl<T, S> VulkanoLayer<S> for T
where
    T: FnMut(&S, &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer<StandardCommandPoolAlloc>>),
    S: StateRequirements,
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
