use asset_storage::asset_storage::AssetStorageKey;
use derive_builder::Builder;
use kernel::abstract_runtime::{ClockworkState, Delegate, Substate};
use kernel::*;
use scene::prelude::{ColoredMeshStorage, PhongMaterialStorage, TexturedMeshStorage};

#[derive(Builder, Delegate)]
#[delegate(Substate<ColoredMeshStorage<C>>, target="colored_meshes")]
#[delegate(Substate<TexturedMeshStorage<C>>, target="static_meshes")]
#[delegate(Substate<PhongMaterialStorage<C>>, target="materials")]
#[builder(pattern = "owned", setter(into))]
pub struct Assets<C>
where
    C: AssetStorageKey,
{
    pub colored_meshes: ColoredMeshStorage<C>,
    pub static_meshes: TexturedMeshStorage<C>,
    pub materials: PhongMaterialStorage<C>,
}

impl<C> Assets<C>
where
    C: AssetStorageKey,
{
    pub fn builder() -> AssetsBuilder<C> {
        Default::default()
    }
}

impl<C> ClockworkState for Assets<C> where C: AssetStorageKey {}
