use asset_storage::asset_storage::AssetStorageKey;
use ecs::prelude::*;
use kernel::{
    graphics::{
        scene::{Lights, PrimaryCamera, SceneObjects},
        Scene,
    },
    prelude::Itertools,
};
use physics::{prelude::RigidBodyHandle, state::PhysicsState};
use scene::components::{Camera, DirectionalLight, PointLight, SpotLight};

use super::{
    scene_instance::{
        SceneAmbientLight, SceneCamera, SceneDirectionalLight, SceneInstance, ScenePointLight,
        SceneSpotLight,
    },
    BaseState,
};

impl<I: AssetStorageKey> Scene for BaseState<I> {
    type LayerKey = u32;
}

impl<'a, I: AssetStorageKey> SceneObjects<SceneInstance<I>> for BaseState<I> {
    fn scene_objects(
        &self,
        layer_key: Self::LayerKey,
    ) -> Box<dyn Iterator<Item = SceneInstance<I>>> {
        Box::new(
            <(&Self::LayerKey, &I, &RigidBodyHandle)>::query()
                .iter(&self.ecs.world)
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
                        self.ecs
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

impl<I: AssetStorageKey> PrimaryCamera<SceneCamera> for BaseState<I> {
    fn primary_camera(&self, layer_key: Self::LayerKey) -> SceneCamera {
        <(&Self::LayerKey, &Camera, &RigidBodyHandle)>::query()
            .iter(&self.ecs.world)
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
                    self.ecs
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

impl<I: AssetStorageKey>
    Lights<SceneAmbientLight, SceneDirectionalLight, ScenePointLight, SceneSpotLight>
    for BaseState<I>
{
    fn ambient_light(&self, layer_key: Self::LayerKey) -> SceneAmbientLight {
        <(&Self::LayerKey, &scene::components::AmbientLight)>::query()
            .iter(&self.ecs.world)
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
            <(&Self::LayerKey, &DirectionalLight)>::query()
                .iter(&self.ecs.world)
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
            <(&Self::LayerKey, &PointLight, &RigidBodyHandle)>::query()
                .iter(&self.ecs.world)
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
                        self.ecs
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
            <(&Self::LayerKey, &SpotLight, &RigidBodyHandle)>::query()
                .iter(&self.ecs.world)
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
                        self.ecs
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
