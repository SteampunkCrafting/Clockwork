use kernel::abstract_runtime::{ClockworkState, EngineState};
use vulkano::command_buffer::SecondaryAutoCommandBuffer;

use crate::state::GraphicsState;

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
