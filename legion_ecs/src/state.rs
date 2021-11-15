use kernel::{abstract_runtime::ClockworkState, util::derive_builder::Builder};
use legion::{storage::IntoComponentSource, systems::Resource, Resources, World};

/// Legion State
///
/// A ClockworkState, which stores a single Legion ECS world alongside
/// with a set of its resources.
///
/// This state is required by the LegionSystems Mechanism.
#[derive(Builder)]
#[builder(pattern = "owned")]
pub struct LegionState {
    /// A set of entities of this world
    #[builder(private, default)]
    pub world: World,

    /// A set of entities
    #[builder(private, default)]
    pub resources: Resources,
}

impl LegionState {
    pub fn builder() -> LegionStateBuilder {
        Default::default()
    }
}

impl ClockworkState for LegionState {}

impl LegionStateBuilder {
    pub fn add_entity<T>(mut self, entity: T) -> Self
    where
        Option<T>: IntoComponentSource,
    {
        self.world
            .get_or_insert_with(|| Default::default())
            .push(entity);
        self
    }

    pub fn add_entities(mut self, entities: impl IntoComponentSource) -> Self {
        self.world
            .get_or_insert_with(|| Default::default())
            .extend(entities);
        self
    }

    pub fn add_resource(mut self, resource: impl Resource) -> Self {
        self.resources
            .get_or_insert_with(|| Default::default())
            .insert(resource);
        self
    }
}
