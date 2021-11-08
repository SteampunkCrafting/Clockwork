use asset_storage::asset_storage::AssetStorageKey;
use derive_builder::Builder;
use ecs::prelude::LegionState;
use graphics::{prelude::Gui, state::OptionGraphicsState};
use kernel::{
    prelude::ClockworkState,
    state::{CallbackSubstate, Substate},
};
use main_loop::{
    prelude::{IOState, OptionGui},
    state::MainLoopState,
};
use physics::state::PhysicsState;
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

#[derive(Builder)]
#[builder(pattern = "owned", setter(into, prefix = "with"), build_fn(skip))]
pub struct BaseState<C>
where
    C: AssetStorageKey,
{
    #[builder(setter(skip))]
    ecs: LegionState,

    #[builder(setter(skip))]
    main_loop_state: MainLoopState,

    assets: Assets<C>,

    graphics_state: OptionGraphicsState,
}

impl<C> ClockworkState for BaseState<C> where C: AssetStorageKey {}

impl<C> Assets<C>
where
    C: AssetStorageKey,
{
    pub fn builder() -> AssetsBuilder<C> {
        Default::default()
    }
}

impl<C> BaseState<C>
where
    C: AssetStorageKey,
{
    pub fn builder() -> BaseStateBuilder<C> {
        Default::default()
    }
}

impl<C> BaseStateBuilder<C>
where
    C: AssetStorageKey,
{
    pub fn build(self) -> Result<BaseState<C>, String> {
        /* ---- ALLOCATING MEMORY ---- */
        let Self { assets, .. } = self;
        let mut base_state = BaseState {
            ecs: LegionState::default(),
            assets: assets.ok_or("Missing assets")?,
            main_loop_state: Default::default(),
            graphics_state: Default::default(),
        };
        let BaseState {
            ecs: LegionState { resources, .. },
            ..
        } = &mut base_state;

        /* ---- INITIALIZING PHYSICS ---- */
        resources.insert(PhysicsState::default());

        /* ---- INITIALIZING IO ---- */
        resources.insert(IOState::default());

        /* ---- INITIALIZING GUI ---- */
        resources.insert(Option::<Gui>::None);

        /* ---- RETURNING ---- */
        Ok(base_state)
    }
}

impl<C> Substate<OptionGraphicsState> for BaseState<C>
where
    C: AssetStorageKey,
{
    fn substate(&self) -> &OptionGraphicsState {
        &self.graphics_state
    }

    fn substate_mut(&mut self) -> &mut OptionGraphicsState {
        &mut self.graphics_state
    }
}

impl<C> CallbackSubstate<PhysicsState> for BaseState<C>
where
    C: AssetStorageKey,
{
    fn callback_substate(&self, callback: impl FnOnce(&PhysicsState)) {
        let Self {
            ecs: LegionState { resources, .. },
            ..
        } = self;
        callback(&resources.get().unwrap());
    }

    fn callback_substate_mut(&mut self, callback: impl FnOnce(&mut PhysicsState)) {
        let Self {
            ecs: LegionState { resources, .. },
            ..
        } = self;
        callback(&mut resources.get_mut().unwrap());
    }
}

impl<C> CallbackSubstate<IOState> for BaseState<C>
where
    C: AssetStorageKey,
{
    fn callback_substate(&self, callback: impl FnOnce(&IOState)) {
        let Self {
            ecs: LegionState { resources, .. },
            ..
        } = &self;
        callback(&resources.get().unwrap());
    }

    fn callback_substate_mut(&mut self, callback: impl FnOnce(&mut IOState)) {
        let Self {
            ecs: LegionState { resources, .. },
            ..
        } = self;
        callback(&mut resources.get_mut().unwrap())
    }
}

impl<C> Substate<LegionState> for BaseState<C>
where
    C: AssetStorageKey,
{
    fn substate(&self) -> &LegionState {
        &self.ecs
    }

    fn substate_mut(&mut self) -> &mut LegionState {
        &mut self.ecs
    }
}

impl<C> Substate<ColoredMeshStorage<C>> for BaseState<C>
where
    C: AssetStorageKey,
{
    fn substate(&self) -> &ColoredMeshStorage<C> {
        &self.assets.colored_meshes
    }

    fn substate_mut(&mut self) -> &mut ColoredMeshStorage<C> {
        &mut self.assets.colored_meshes
    }
}

impl<C> Substate<TexturedMeshStorage<C>> for BaseState<C>
where
    C: AssetStorageKey,
{
    fn substate(&self) -> &TexturedMeshStorage<C> {
        &self.assets.static_meshes
    }

    fn substate_mut(&mut self) -> &mut TexturedMeshStorage<C> {
        &mut self.assets.static_meshes
    }
}

impl<C> Substate<MainLoopState> for BaseState<C>
where
    C: AssetStorageKey,
{
    fn substate(&self) -> &MainLoopState {
        &self.main_loop_state
    }

    fn substate_mut(&mut self) -> &mut MainLoopState {
        &mut self.main_loop_state
    }
}

impl<C> Substate<PhongMaterialStorage<C>> for BaseState<C>
where
    C: AssetStorageKey,
{
    fn substate(&self) -> &PhongMaterialStorage<C> {
        &self.assets.materials
    }

    fn substate_mut(&mut self) -> &mut PhongMaterialStorage<C> {
        &mut self.assets.materials
    }
}

impl<C> CallbackSubstate<OptionGui> for BaseState<C>
where
    C: AssetStorageKey,
{
    fn callback_substate(&self, callback: impl FnOnce(&OptionGui)) {
        callback(&self.ecs.resources.get().unwrap())
    }

    fn callback_substate_mut(&mut self, callback: impl FnOnce(&mut OptionGui)) {
        callback(&mut self.ecs.resources.get_mut().unwrap())
    }
}
