use asset_storage::asset_storage::AssetStorageKey;
use kernel::abstract_runtime::Substate;
use legion_ecs::state::LegionState;
use physics::state::PhysicsState;
use scene_utils::prelude::{PhongMaterialStorage, TexturedMeshStorage};

/// State requirements for this layer in order to be considered renderable
pub trait StateRequirements<I>:
    Substate<PhysicsState>
    + Substate<LegionState>
    + Substate<TexturedMeshStorage<I>>
    + Substate<PhongMaterialStorage<I>>
where
    I: AssetStorageKey,
{
}

impl<T, I> StateRequirements<I> for T
where
    T: Substate<PhysicsState>
        + Substate<LegionState>
        + Substate<TexturedMeshStorage<I>>
        + Substate<PhongMaterialStorage<I>>,
    I: AssetStorageKey,
{
}
