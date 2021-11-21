use kernel::abstract_runtime::{CallbackSubstate, EngineState};
use vulkano::command_buffer::{
    pool::standard::StandardCommandPoolAlloc, AutoCommandBufferBuilder, PrimaryAutoCommandBuffer,
    SecondaryAutoCommandBuffer,
};

use crate::state::{GraphicsInitState, GraphicsState};

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

pub trait NewVulkanoLayer<S>
where
    S: StateRequirements,
{
    fn initialization(&mut self, engine_state: &EngineState<S>, window_dimensions: [u32; 2]);

    fn window_resize(
        &mut self,
        engine_state: &EngineState<S>,
        graphics_state: &GraphicsState,
        window_dimensions: [u32; 2],
    );

    fn draw(
        &mut self,
        engine_state: &EngineState<S>,
        graphics_state: &GraphicsState,
    ) -> SecondaryAutoCommandBuffer;

    fn termination(&mut self, engine_state: &EngineState<S>, graphics_state: &GraphicsState);
}
