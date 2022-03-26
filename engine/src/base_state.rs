use crate::base_state::scene::Iter;
use crate::base_state::scene::PrimaryCamera;
use crate::base_state::scene::SceneObjects;
use crate::base_state::scene_instance::SceneCamera;
use crate::base_state::scene_instance::SceneInstance;
use ::graphics::state::GuiState;
use ::scene::prelude::{ColoredMeshStorage, PhongMaterialStorage, TexturedMeshStorage};
use asset_storage::asset_storage::AssetStorageKey;
use derive_builder::Builder;
use ecs::prelude::LegionState;
use kernel::graphics::scene::Lights;
use kernel::*;
use kernel::{
    abstract_runtime::{ClockworkState, Delegate, Substate},
    graphics::*,
    prelude::StandardEvent,
};
use main_loop::state::{InitWinitState, InputState, MainLoopStatistics};
use physics::prelude::PhysicsState;

pub use assets_wrapper::Assets;
use legion_rapier_wrapper::ECSWrapper;

use self::scene_instance::SceneAmbientLight;
use self::scene_instance::SceneDirectionalLight;
use self::scene_instance::ScenePointLight;
use self::scene_instance::SceneSpotLight;
mod assets_wrapper;
mod legion_rapier_wrapper;
mod scene_instance;

#[derive(Builder, Delegate)]
#[delegate(Substate<ColoredMeshStorage<AssetT>>, target = "assets")]
#[delegate(Substate<TexturedMeshStorage<AssetT>>, target = "assets")]
#[delegate(Substate<PhongMaterialStorage<AssetT>>, target = "assets")]
#[delegate(Substate<LegionState>, target = "ecs")]
#[delegate(Substate<GuiState>, target="ecs")]
#[delegate(Substate<MainLoopStatistics>, target = "ecs")]
#[delegate(Substate<InputState>, target = "ecs")]
#[delegate(Substate<PhysicsState>, target = "ecs")]
// #[delegate(Scene<LayerKey = u32>, target = "ecs")]
#[delegate(SceneObjects<SceneInstance<AssetT>>, target = "ecs")]
#[delegate(PrimaryCamera<SceneCamera>, target = "ecs")]
// #[delegate(Lights<SceneAmbientLight, SceneDirectionalLight, ScenePointLight, SceneSpotLight>, target = "ecs")]
#[delegate(Substate<InitWinitState<StandardEvent>>, target = "main_loop_state")]
#[builder(pattern = "owned", setter(into, prefix = "with"), build_fn(skip))]
pub struct BaseState<AssetT>
where
    AssetT: AssetStorageKey,
{
    #[builder(setter(skip))]
    ecs: ECSWrapper,

    #[builder(setter(skip))]
    main_loop_state: InitWinitState<StandardEvent>,

    assets: Assets<AssetT>,
}

impl<T: AssetStorageKey> Scene for BaseState<T> {
    type LayerKey = u32;
}

impl<AssetT> ClockworkState for BaseState<AssetT> where AssetT: AssetStorageKey {}

impl<AssetT> BaseState<AssetT>
where
    AssetT: AssetStorageKey,
{
    pub fn builder() -> BaseStateBuilder<AssetT> {
        Default::default()
    }
}

impl<AssetT> BaseStateBuilder<AssetT>
where
    AssetT: AssetStorageKey,
{
    pub fn build(self) -> Result<BaseState<AssetT>, String> {
        let Self { assets, .. } = self;
        let main_loop_state = InitWinitState::builder().build().unwrap();
        Ok(BaseState {
            ecs: ECSWrapper::new(main_loop_state.proxy().clone()),
            main_loop_state,
            assets: assets.ok_or("Missing assets")?,
        })
    }
}

impl<AssetT: AssetStorageKey>
    Lights<SceneAmbientLight, SceneDirectionalLight, ScenePointLight, SceneSpotLight>
    for BaseState<AssetT>
{
    fn ambient_light(&self, layer_key: Self::LayerKey) -> SceneAmbientLight {
        self.ecs.ambient_light(layer_key)
    }

    fn directional_lights(&self, layer_key: Self::LayerKey) -> Iter<SceneDirectionalLight> {
        self.ecs.directional_lights(layer_key)
    }

    fn point_lights(&self, layer_key: Self::LayerKey) -> Iter<ScenePointLight> {
        self.ecs.point_lights(layer_key)
    }

    fn spot_lights(&self, layer_key: Self::LayerKey) -> Iter<SceneSpotLight> {
        self.ecs.spot_lights(layer_key)
    }
}
