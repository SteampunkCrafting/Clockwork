use asset_storage::asset_storage::AssetStorageKey;
use derive_builder::Builder;
use scene::prelude::{ColoredMeshStorage, PhongMaterialStorage, TexturedMeshStorage};

#[derive(Builder)]
#[builder(pattern = "owned", setter(into, prefix = "with"))]
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
