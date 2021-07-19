use clockwork_core::prelude::Substate;
use derive_builder::Builder;
use ecs::prelude::LegionState;
use main_loop::prelude::IOState;
use physics::prelude::RapierState3D;

#[derive(Builder)]
#[builder(pattern = "owned", setter(into, prefix = "with"), build_fn(skip))]
pub struct BaseState {
    #[builder(setter(skip))]
    physics: RapierState3D,

    #[builder(setter(skip))]
    ecs: LegionState,

    #[builder(setter(skip), default = "IOState::builder().build().unwrap()")]
    io: IOState,
}

impl BaseState {
    pub fn builder() -> BaseStateBuilder {
        Default::default()
    }
}

impl BaseStateBuilder {
    pub fn build(self) -> Result<BaseState, String> {
        /* ---- ALLOCATING MEMORY ---- */
        let Self { .. } = self;
        let mut base_state = BaseState {
            physics: RapierState3D::default(),
            ecs: LegionState::default(),
            io: IOState::builder().build().unwrap(),
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

impl Substate<RapierState3D> for BaseState {
    fn substate(&self) -> &RapierState3D {
        &self.physics
    }

    fn substate_mut(&mut self) -> &mut RapierState3D {
        &mut self.physics
    }
}

impl Substate<LegionState> for BaseState {
    fn substate(&self) -> &LegionState {
        &self.ecs
    }

    fn substate_mut(&mut self) -> &mut LegionState {
        &mut self.ecs
    }
}

impl Substate<IOState> for BaseState {
    fn substate(&self) -> &IOState {
        &self.io
    }

    fn substate_mut(&mut self) -> &mut IOState {
        &mut self.io
    }
}
