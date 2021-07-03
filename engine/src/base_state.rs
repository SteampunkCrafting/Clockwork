use core::prelude::Substate;
use ecs::prelude::LegionState;
use main_loop::prelude::IOState;
use physics::prelude::RapierState3D;

pub struct BaseState {
    physics: RapierState3D,
    ecs: LegionState,
    io: IOState,
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

impl Default for BaseState {
    fn default() -> Self {
        Self {
            physics: RapierState3D::default(),
            ecs: LegionState::default(),
            io: IOState::builder().build().unwrap(),
        }
    }
}
