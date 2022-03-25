use std::marker::PhantomData;

use crate::state::PhysicsState;
use kernel::{
    abstract_runtime::{EngineState, Substate},
    prelude::StandardEvent,
    standard_runtime::{StandardMechanism, StandardRuntimeStatistics},
    util::derive_builder::Builder,
};
use rapier3d::{dynamics::IntegrationParameters, pipeline::PhysicsPipeline};

#[derive(Builder)]
#[builder(pattern = "owned")]
pub struct Rapier3DTicker<T>
where
    T: StandardRuntimeStatistics,
{
    /// A physics pipeline of the ticker
    #[builder(private, default)]
    physics_pipeline: PhysicsPipeline,

    /// Integration parameters of the ticker
    #[builder(private, default)]
    integration_parameters: IntegrationParameters,

    /// Phantom data for statistics type
    #[builder(private, default)]
    phantom_data: PhantomData<T>,
}

impl<T> Rapier3DTicker<T>
where
    T: StandardRuntimeStatistics,
{
    pub fn builder() -> Rapier3DTickerBuilder<T> {
        Default::default()
    }
}

impl<S, T> StandardMechanism<S> for Rapier3DTicker<T>
where
    S: Substate<PhysicsState> + Substate<T>,
    T: StandardRuntimeStatistics,
{
    fn tick(&mut self, state: &mut EngineState<S>) {
        let Rapier3DTicker {
            physics_pipeline,
            integration_parameters,
            ..
        } = self;
        state
            .start_mutate()
            .get(T::current_tick_delta)
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
                    physics_pipeline.step(
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
                    )
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
