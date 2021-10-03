use asset_storage::asset_storage::AssetStorageKey;
use clockwork_core::clockwork::CallbackSubstate;
use legion_ecs::state::LegionState;
use physics::state::PhysicsState;
use scene_utils::prelude::{PhongMaterialStorage, TexturedMeshStorage};

/// State requirements for this layer in order to be considered renderable
pub trait EngineStateRequirements<I>:
    CallbackSubstate<PhysicsState>
    + CallbackSubstate<LegionState>
    + CallbackSubstate<TexturedMeshStorage<I>>
    + CallbackSubstate<PhongMaterialStorage<I>>
where
    I: AssetStorageKey,
{
}

impl<T, I> EngineStateRequirements<I> for T
where
    T: CallbackSubstate<PhysicsState>
        + CallbackSubstate<LegionState>
        + CallbackSubstate<TexturedMeshStorage<I>>
        + CallbackSubstate<PhongMaterialStorage<I>>,
    I: AssetStorageKey,
{
}
