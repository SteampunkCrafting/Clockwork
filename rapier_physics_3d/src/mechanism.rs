use crate::state::RapierState3D;
use ::core::prelude::{Mechanism, Substate};
use main_loop::prelude::Event;
use rapier3d::{dynamics::IntegrationParameters, na::*, pipeline::PhysicsPipeline};

pub struct Rapier3DTicker(PhysicsPipeline, IntegrationParameters);
impl<S> Mechanism<S, Event> for Rapier3DTicker
where
    S: Substate<RapierState3D>,
{
    fn name(&self) -> &'static str {
        "Rapier 3D Physics Ticker"
    }

    fn clink(&mut self, state: &mut S, event: Event) {
        match event {
            Event::Tick(delta_time) => {
                let Rapier3DTicker(pipeline, integration_parameters) = self;
                let RapierState3D {
                    bodies,
                    joints,
                    colliders,
                    broad_phase,
                    narrow_phase,
                    ccd_solver,
                    islands,
                    ..
                } = state.substate_mut();
                let gra = Vector3::zeros();
                integration_parameters.dt = delta_time.as_secs_f32();
                pipeline.step(
                    &gra,
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
                )
            }
            _ => (),
        }
    }
}
