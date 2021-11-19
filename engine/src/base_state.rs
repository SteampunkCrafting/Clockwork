use asset_storage::asset_storage::AssetStorageKey;
use derive_builder::Builder;
use ecs::prelude::LegionState;
use graphics::state::{GraphicsInitState, GuiState};
use kernel::{
    abstract_runtime::{CallbackSubstate, ClockworkState, Substate},
    prelude::StandardEvent,
};
use main_loop::state::{InitWinitState, InputState, MainLoopStatistics};
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
    main_loop_state: InitWinitState<StandardEvent>,

    assets: Assets<C>,

    graphics_state: GraphicsInitState,
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
            ecs: LegionState::builder().build().unwrap(),
            assets: assets.ok_or("Missing assets")?,
            main_loop_state: InitWinitState::builder().build().unwrap(),
            graphics_state: Default::default(),
        };
        let BaseState {
            ecs: LegionState { resources, .. },
            main_loop_state,
            ..
        } = &mut base_state;

        /* ---- INITIALIZING MAIN LOOP ---- */
        resources.insert(main_loop_state.proxy().clone());

        /* ---- INITIALIZING PHYSICS ---- */
        resources.insert(PhysicsState::builder().build().unwrap());

        /* ---- INITIALIZING IO ---- */
        resources.insert(InputState::builder().build().unwrap());
        resources.insert(MainLoopStatistics::builder().build().unwrap());

        /* ---- INITIALIZING GUI ---- */
        resources.insert(GuiState::default());

        /* ---- RETURNING ---- */
        Ok(base_state)
    }
}

impl<C> CallbackSubstate<MainLoopStatistics> for BaseState<C>
where
    C: AssetStorageKey,
{
    fn callback_substate<R>(&self, callback: impl FnOnce(&MainLoopStatistics) -> R) -> R {
        let Self {
            ecs: LegionState { resources, .. },
            ..
        } = self;
        callback(&resources.get().unwrap())
    }

    fn callback_substate_mut<R>(
        &mut self,
        callback: impl FnOnce(&mut MainLoopStatistics) -> R,
    ) -> R {
        let Self {
            ecs: LegionState { resources, .. },
            ..
        } = self;
        callback(&mut resources.get_mut().unwrap())
    }
}

impl<C> CallbackSubstate<InputState> for BaseState<C>
where
    C: AssetStorageKey,
{
    fn callback_substate<R>(&self, callback: impl FnOnce(&InputState) -> R) -> R {
        let Self {
            ecs: LegionState { resources, .. },
            ..
        } = self;
        callback(&resources.get().unwrap())
    }

    fn callback_substate_mut<R>(&mut self, callback: impl FnOnce(&mut InputState) -> R) -> R {
        let Self {
            ecs: LegionState { resources, .. },
            ..
        } = self;
        callback(&mut resources.get_mut().unwrap())
    }
}

impl<C> Substate<GraphicsInitState> for BaseState<C>
where
    C: AssetStorageKey,
{
    fn substate(&self) -> &GraphicsInitState {
        &self.graphics_state
    }

    fn substate_mut(&mut self) -> &mut GraphicsInitState {
        &mut self.graphics_state
    }
}

impl<C> CallbackSubstate<PhysicsState> for BaseState<C>
where
    C: AssetStorageKey,
{
    fn callback_substate<R>(&self, callback: impl FnOnce(&PhysicsState) -> R) -> R {
        let Self {
            ecs: LegionState { resources, .. },
            ..
        } = self;
        callback(&resources.get().unwrap())
    }

    fn callback_substate_mut<R>(&mut self, callback: impl FnOnce(&mut PhysicsState) -> R) -> R {
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

impl<C> Substate<InitWinitState<StandardEvent>> for BaseState<C>
where
    C: AssetStorageKey,
{
    fn substate(&self) -> &InitWinitState<StandardEvent> {
        &self.main_loop_state
    }

    fn substate_mut(&mut self) -> &mut InitWinitState<StandardEvent> {
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

impl<C> CallbackSubstate<GuiState> for BaseState<C>
where
    C: AssetStorageKey,
{
    fn callback_substate<R>(&self, callback: impl FnOnce(&GuiState) -> R) -> R {
        callback(&self.ecs.resources.get().unwrap())
    }

    fn callback_substate_mut<R>(&mut self, callback: impl FnOnce(&mut GuiState) -> R) -> R {
        callback(&mut self.ecs.resources.get_mut().unwrap())
    }
}
