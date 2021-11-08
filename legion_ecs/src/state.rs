use kernel::prelude::ClockworkState;
use legion::{storage::IntoComponentSource, systems::Resource, Resources, World};

#[derive(Default)]
pub struct LegionState {
    pub world: World,
    pub resources: Resources,
}
impl LegionState {
    pub fn builder() -> LegionStateBuilder {
        Default::default()
    }
}
impl ClockworkState for LegionState {}

#[derive(Default)]
pub struct LegionStateBuilder {
    world: World,
    resources: Resources,
}
impl LegionStateBuilder {
    pub fn with_entity<T>(mut self, entity: T) -> Self
    where
        Option<T>: IntoComponentSource,
    {
        self.world.push(entity);
        self
    }

    pub fn with_entities(mut self, entities: impl IntoComponentSource) -> Self {
        self.world.extend(entities);
        self
    }

    pub fn with_resource(mut self, resource: impl Resource) -> Self {
        self.resources.insert(resource);
        self
    }

    pub fn build(self) -> LegionState {
        let Self { world, resources } = self;
        LegionState { world, resources }
    }
}
