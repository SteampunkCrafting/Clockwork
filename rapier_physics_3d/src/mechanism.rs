use kernel::{
    abstract_runtime::{CallbackSubstate, EngineState},
    prelude::StandardEvent,
    standard_runtime::StandardMechanism,
};
use main_loop::state::IOState;
use rapier3d::{dynamics::IntegrationParameters, pipeline::PhysicsPipeline};

use crate::state::PhysicsState;

#[derive(Default)]
pub struct Rapier3DTicker(PhysicsPipeline, IntegrationParameters);

impl<S> StandardMechanism<S> for Rapier3DTicker
where
    S: CallbackSubstate<PhysicsState> + CallbackSubstate<IOState>,
{
    fn tick(&mut self, state: &mut EngineState<S>) {
        let Rapier3DTicker(pipeline, integration_parameters) = self;
        state
            .start_mutate()
            .get_mut(
                |IOState {
                     ref tick_delta_time,
                     ..
                 }| *tick_delta_time,
            )
            .then_get_mut(
                |delta_time,
                 PhysicsState {
                     gravity,
                     bodies,
                     joints,
                     colliders,
                     broad_phase,
                     narrow_phase,
                     ccd_solver,
                     islands,
                 }| {
                    integration_parameters.dt = delta_time.as_secs_f32();
                    pipeline.step(
                        &gravity.0,
                        integration_parameters,
                        islands,
                        broad_phase,
                        narrow_phase,
                        bodies,
                        colliders,
                        joints,
                        ccd_solver,
                        &mut (),
                        &mut (),
                    );
                },
            )
            .finish()
    }

    fn handled_events(&self) -> Option<Vec<StandardEvent>> {
        Some(vec![StandardEvent::Tick])
    }

    fn initialization(&mut self, _: &mut EngineState<S>) {
        unreachable!()
    }

    fn draw(&mut self, _: &mut EngineState<S>) {
        unreachable!()
    }

    fn termination(&mut self, _: &mut EngineState<S>) {
        unreachable!()
    }
}
