use asset_storage::asset_storage::AssetStorageKey;
use graphics::vulkano_layer;
use kernel::clockwork::CallbackSubstate;
use legion_ecs::state::LegionState;
use physics::state::PhysicsState;
use scene_utils::prelude::{PhongMaterialStorage, TexturedMeshStorage};

/// State requirements for this layer in order to be considered renderable
pub trait StateRequirements<I>:
    vulkano_layer::StateRequirements
    + CallbackSubstate<PhysicsState>
    + CallbackSubstate<LegionState>
    + CallbackSubstate<TexturedMeshStorage<I>>
    + CallbackSubstate<PhongMaterialStorage<I>>
where
    I: AssetStorageKey,
{
}

impl<T, I> StateRequirements<I> for T
where
    T: vulkano_layer::StateRequirements
        + CallbackSubstate<PhysicsState>
        + CallbackSubstate<LegionState>
        + CallbackSubstate<TexturedMeshStorage<I>>
        + CallbackSubstate<PhongMaterialStorage<I>>,
    I: AssetStorageKey,
{
}
