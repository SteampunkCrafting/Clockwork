use asset_storage::asset_storage::AssetStorageKey;
use clockwork_core::prelude::Substate;
use derive_builder::Builder;
use ecs::prelude::LegionState;
use main_loop::prelude::IOState;
use physics::prelude::RapierState3D;
use scene::prelude::ColoredMeshStorage;

#[derive(Builder)]
#[builder(pattern = "owned", setter(into, prefix = "with"))]
pub struct Assets<C>
where
    C: AssetStorageKey,
{
    pub colored_meshes: ColoredMeshStorage<C>,
}

#[derive(Builder)]
#[builder(pattern = "owned", setter(into, prefix = "with"), build_fn(skip))]
pub struct BaseState<C>
where
    C: AssetStorageKey,
{
    #[builder(setter(skip))]
    physics: RapierState3D,

    #[builder(setter(skip))]
    ecs: LegionState,

    #[builder(setter(skip), default = "IOState::builder().build().unwrap()")]
    io: IOState,

    assets: Assets<C>,
}

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
            physics: RapierState3D::default(),
            ecs: LegionState::default(),
            io: IOState::builder().build().unwrap(),
            assets: assets.ok_or("Missing assets")?,
        };

        /* ---- CONNECTING PHYSICS TO ECS ---- */
        let (g, b, c, j, i, bp, np, ccd) = base_state.physics.user_locks();
        let BaseState {
            ecs: LegionState { resources: res, .. },
            ..
        } = &mut base_state;
        res.insert(g);
        res.insert(b);
        res.insert(c);
        res.insert(j);
        res.insert(i);
        res.insert(bp);
        res.insert(np);
        res.insert(ccd);

        /* ---- RETURNING ---- */
        Ok(base_state)
    }
}

impl<C> Substate<RapierState3D> for BaseState<C>
where
    C: AssetStorageKey,
{
    fn substate(&self) -> &RapierState3D {
        &self.physics
    }

    fn substate_mut(&mut self) -> &mut RapierState3D {
        &mut self.physics
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

impl<C> Substate<IOState> for BaseState<C>
where
    C: AssetStorageKey,
{
    fn substate(&self) -> &IOState {
        &self.io
    }

    fn substate_mut(&mut self) -> &mut IOState {
        &mut self.io
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
