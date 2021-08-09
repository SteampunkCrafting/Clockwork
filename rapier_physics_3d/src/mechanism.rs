use clockwork_core::prelude::{CallbackSubstate, Mechanism};
use main_loop::prelude::Event;
use rapier3d::{dynamics::IntegrationParameters, pipeline::PhysicsPipeline};

use crate::state::PhysicsState;

#[derive(Default)]
pub struct Rapier3DTicker(PhysicsPipeline, IntegrationParameters);

impl<S> Mechanism<S, Event> for Rapier3DTicker
where
    S: CallbackSubstate<PhysicsState>,
{
    fn name(&self) -> &'static str {
        "Rapier 3D Physics Ticker"
    }

    fn clink(&mut self, state: &mut S, event: Event) {
        match event {
            Event::Tick(delta_time) => {
                let Rapier3DTicker(pipeline, integration_parameters) = self;
                state.callback_substate_mut(
                    |PhysicsState {
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
                );
            }
            _ => (),
        }
    }
}
