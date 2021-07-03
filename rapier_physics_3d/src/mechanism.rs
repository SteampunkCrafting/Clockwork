use crate::state::RapierState3D;
use ::core::prelude::{Mechanism, Substate};
use legion_ecs::state::LegionState;
use main_loop::prelude::Event;
use rapier3d::{dynamics::IntegrationParameters, pipeline::PhysicsPipeline};

#[derive(Default)]
pub struct Rapier3DTicker(PhysicsPipeline, IntegrationParameters);

impl<S> Mechanism<S, Event> for Rapier3DTicker
where
    S: Substate<RapierState3D> + Substate<LegionState>,
{
    fn name(&self) -> &'static str {
        "Rapier 3D Physics Ticker"
    }

    fn clink(&mut self, state: &mut S, event: Event) {
        match event {
            Event::Initialization => {
                let (g, b, c, j, i, bp, np, ccd) =
                    Substate::<RapierState3D>::substate(state).user_locks();
                let LegionState { resources: res, .. } = state.substate_mut();
                res.insert(g);
                res.insert(b);
                res.insert(c);
                res.insert(j);
                res.insert(i);
                res.insert(bp);
                res.insert(np);
                res.insert(ccd);
            }
            Event::Tick(delta_time) => {
                let Rapier3DTicker(pipeline, integration_parameters) = self;
                let RapierState3D {
                    gravity,
                    bodies,
                    joints,
                    colliders,
                    broad_phase,
                    narrow_phase,
                    ccd_solver,
                    islands,
                } = state.substate_mut();
                let (
                    gravity,
                    mut bodies,
                    mut joints,
                    mut colliders,
                    mut broad_phase,
                    mut narrow_phase,
                    mut ccd_solver,
                    mut islands,
                ) = (
                    gravity.lock(),
                    bodies.lock_mut(),
                    joints.lock_mut(),
                    colliders.lock_mut(),
                    broad_phase.lock_mut(),
                    narrow_phase.lock_mut(),
                    ccd_solver.lock_mut(),
                    islands.lock_mut(),
                );
                integration_parameters.dt = delta_time.as_secs_f32();
                pipeline.step(
                    &gravity.0,
                    integration_parameters,
                    &mut *islands,
                    &mut *broad_phase,
                    &mut *narrow_phase,
                    &mut *bodies,
                    &mut *colliders,
                    &mut *joints,
                    &mut *ccd_solver,
                    &mut (),
                    &mut (),
                );
            }
            _ => (),
        }
    }
}
