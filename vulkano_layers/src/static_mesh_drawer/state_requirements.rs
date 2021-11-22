use asset_storage::asset_storage::AssetStorageKey;
use kernel::abstract_runtime::CallbackSubstate;
use legion_ecs::state::LegionState;
use physics::state::PhysicsState;
use scene_utils::prelude::{PhongMaterialStorage, TexturedMeshStorage};

/// State requirements for this layer in order to be considered renderable
pub trait StateRequirements<I>:
    CallbackSubstate<PhysicsState>
    + CallbackSubstate<LegionState>
    + CallbackSubstate<TexturedMeshStorage<I>>
    + CallbackSubstate<PhongMaterialStorage<I>>
where
    I: AssetStorageKey,
{
}

impl<T, I> StateRequirements<I> for T
where
    T: CallbackSubstate<PhysicsState>
        + CallbackSubstate<LegionState>
        + CallbackSubstate<TexturedMeshStorage<I>>
        + CallbackSubstate<PhongMaterialStorage<I>>,
    I: AssetStorageKey,
{
}
