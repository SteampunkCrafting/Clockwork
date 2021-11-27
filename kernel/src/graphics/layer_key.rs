use std::hash::Hash;

/// `LayerKey` is a generic type of a hint for the `Scene` subtypes.
///
/// These keys are provided by the rendering layers, so that the scene handlers
/// could properly select the appropriate scene objects to be retrieved.
pub trait RenderingLayerKey: Send + Sync + Sized + Clone + Hash + Eq + 'static {}
impl<T> RenderingLayerKey for T where T: Send + Sync + Sized + Clone + Hash + Eq + 'static {}
