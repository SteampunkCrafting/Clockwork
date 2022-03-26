use ::graphics::state::GuiState;
use asset_storage::asset_storage::AssetStorageKey;
use ecs::prelude::*;
use ecs::state::LegionState;
use kernel::abstract_runtime::ClockworkEvent;
use kernel::abstract_runtime::ClockworkState;
use kernel::abstract_runtime::Delegate;
use kernel::abstract_runtime::Substate;
use kernel::graphics::scene::Lights;
use kernel::graphics::scene::PrimaryCamera;
use kernel::graphics::scene::SceneObjects;
use kernel::graphics::Scene;
use kernel::prelude::Itertools;
use kernel::*;
use main_loop::state::InputState;
use main_loop::state::MainLoopStatistics;
use main_loop::state::WinitLoopProxy;
use physics::prelude::RigidBodyHandle;
use physics::state::PhysicsState;

use super::scene_instance::SceneAmbientLight;
use super::scene_instance::SceneCamera;
use super::scene_instance::SceneDirectionalLight;
use super::scene_instance::SceneInstance;
use super::scene_instance::ScenePointLight;
use super::scene_instance::SceneSpotLight;

#[derive(Delegate)]
#[delegate(Substate<LegionState>)]
pub struct ECSWrapper(LegionState);

impl ECSWrapper {
    pub fn new<E: ClockworkEvent>(winit_proxy: WinitLoopProxy<E>) -> Self {
        Self(
            LegionState::builder()
                .add_resource(winit_proxy)
                .add_resource(PhysicsState::builder().build().unwrap())
                .add_resource(InputState::builder().build().unwrap())
                .add_resource(MainLoopStatistics::builder().build().unwrap())
                .add_resource(GuiState::default())
                .build()
                .unwrap(),
        )
    }
}

impl ClockworkState for ECSWrapper {}

impl Substate<MainLoopStatistics> for ECSWrapper {
    fn substate<R>(&self, callback: impl FnOnce(&MainLoopStatistics) -> R) -> R {
        callback(&self.0.resources.get().unwrap())
    }

    fn substate_mut<R>(&mut self, callback: impl FnOnce(&mut MainLoopStatistics) -> R) -> R {
        callback(&mut self.0.resources.get_mut().unwrap())
    }
}

impl Substate<InputState> for ECSWrapper {
    fn substate<R>(&self, callback: impl FnOnce(&InputState) -> R) -> R {
        callback(&self.0.resources.get().unwrap())
    }

    fn substate_mut<R>(&mut self, callback: impl FnOnce(&mut InputState) -> R) -> R {
        callback(&mut self.0.resources.get_mut().unwrap())
    }
}

impl Substate<PhysicsState> for ECSWrapper {
    fn substate<R>(&self, callback: impl FnOnce(&PhysicsState) -> R) -> R {
        callback(&self.0.resources.get().unwrap())
    }

    fn substate_mut<R>(&mut self, callback: impl FnOnce(&mut PhysicsState) -> R) -> R {
        callback(&mut self.0.resources.get_mut().unwrap())
    }
}

impl Substate<GuiState> for ECSWrapper {
    fn substate<R>(&self, callback: impl FnOnce(&GuiState) -> R) -> R {
        callback(&self.0.resources.get().unwrap())
    }

    fn substate_mut<R>(&mut self, callback: impl FnOnce(&mut GuiState) -> R) -> R {
        callback(&mut self.0.resources.get_mut().unwrap())
    }
}

impl Scene for ECSWrapper {
    type LayerKey = u32;
}

impl<I: AssetStorageKey> SceneObjects<SceneInstance<I>> for ECSWrapper {
    fn scene_objects(&self, layer_key: Self::LayerKey) -> graphics::scene::Iter<SceneInstance<I>> {
        Box::new(
            <(&Self::LayerKey, &I, &RigidBodyHandle)>::query()
                .iter(&self.0.world)
                .filter_map(|(layer, asset, body)| {
                    if layer == &layer_key {
                        Some((asset, body))
                    } else {
                        None
                    }
                })
                .map(|(asset_id, body_handle)| {
                    (
                        asset_id.clone(),
                        self.0
                            .resources
                            .get::<PhysicsState>()
                            .unwrap()
                            .bodies
                            .get(body_handle.clone())
                            .unwrap()
                            .clone(),
                    )
                })
                .map(Into::into)
                .collect_vec()
                .into_iter(),
        )
    }
}

impl PrimaryCamera<SceneCamera> for ECSWrapper {
    fn primary_camera(&self, layer_key: Self::LayerKey) -> SceneCamera {
        <(
            &Self::LayerKey,
            &scene::components::Camera,
            &RigidBodyHandle,
        )>::query()
        .iter(&self.0.world)
        .filter_map(|(layer, camera, body)| {
            if layer == &layer_key {
                Some((camera, body))
            } else {
                None
            }
        })
        .map(|(camera, body_handle)| {
            (
                camera.clone(),
                self.0
                    .resources
                    .get::<PhysicsState>()
                    .unwrap()
                    .bodies
                    .get(body_handle.clone())
                    .unwrap()
                    .clone(),
            )
        })
        .map(Into::into)
        .next()
        .expect(&format!(
            "Failed to get a primary camera for layer {:?}",
            layer_key,
        ))
    }
}

impl Lights<SceneAmbientLight, SceneDirectionalLight, ScenePointLight, SceneSpotLight>
    for ECSWrapper
{
    fn ambient_light(&self, layer_key: Self::LayerKey) -> SceneAmbientLight {
        <(&Self::LayerKey, &scene::components::AmbientLight)>::query()
            .iter(&self.0.world)
            .filter_map(|(layer, ambient)| {
                if layer == &layer_key {
                    Some(ambient)
                } else {
                    None
                }
            })
            .cloned()
            .map(Into::into)
            .next()
            .expect(&format!(
                "Failed to get a requested Ambient Light for layer {:?}",
                layer_key
            ))
    }

    fn directional_lights(
        &self,
        layer_key: Self::LayerKey,
    ) -> kernel::graphics::scene::Iter<SceneDirectionalLight> {
        Box::new(
            <(&Self::LayerKey, &scene::components::DirectionalLight)>::query()
                .iter(&self.0.world)
                .filter_map(|(layer, light)| {
                    if layer == &layer_key {
                        Some(light)
                    } else {
                        None
                    }
                })
                .cloned()
                .map(Into::into)
                .collect_vec()
                .into_iter(),
        )
    }

    fn point_lights(
        &self,
        layer_key: Self::LayerKey,
    ) -> kernel::graphics::scene::Iter<ScenePointLight> {
        Box::new(
            <(
                &Self::LayerKey,
                &scene::components::PointLight,
                &RigidBodyHandle,
            )>::query()
            .iter(&self.0.world)
            .filter_map(|(layer, light, body)| {
                if layer == &layer_key {
                    Some((light, body))
                } else {
                    None
                }
            })
            .map(|(light, body_handle)| {
                (
                    light.clone(),
                    self.0
                        .resources
                        .get::<PhysicsState>()
                        .unwrap()
                        .bodies
                        .get(body_handle.clone())
                        .unwrap()
                        .clone(),
                )
            })
            .map(Into::into)
            .collect_vec()
            .into_iter(),
        )
    }

    fn spot_lights(
        &self,
        layer_key: Self::LayerKey,
    ) -> kernel::graphics::scene::Iter<SceneSpotLight> {
        Box::new(
            <(
                &Self::LayerKey,
                &scene::components::SpotLight,
                &RigidBodyHandle,
            )>::query()
            .iter(&self.0.world)
            .filter_map(|(layer, light, body)| {
                if layer == &layer_key {
                    Some((light, body))
                } else {
                    None
                }
            })
            .map(|(light, body_handle)| {
                (
                    light.clone(),
                    self.0
                        .resources
                        .get::<PhysicsState>()
                        .unwrap()
                        .bodies
                        .get(body_handle.clone())
                        .unwrap()
                        .clone(),
                )
            })
            .map(Into::into)
            .collect_vec()
            .into_iter(),
        )
    }
}
