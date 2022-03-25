use ecs::state::LegionState;
use kernel::abstract_runtime::ClockworkState;
use kernel::abstract_runtime::Delegate;
use kernel::abstract_runtime::Substate;
use kernel::*;

#[derive(Delegate)]
#[delegate(Substate<LegionState>)]
pub struct ECSWrapper(LegionState);

impl ClockworkState for ECSWrapper {}
