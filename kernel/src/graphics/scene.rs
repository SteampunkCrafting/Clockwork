use crate::abstract_runtime::ClockworkState;

use super::{
    layer_key::RenderingLayerKey, scene_object, AmbientLight, DirectionalLight, PointLight,
    SceneObject, SpotLight,
};

/// A local type for iterators
pub type Iter<T> = Box<dyn Iterator<Item = T>>;

/// A trait for the collection of objects on the scene
pub trait Scene
where
    Self: ClockworkState,
{
    /// An associated type, which sets the LayerKey,
    /// used by the scene to select and pass renderables
    /// to the layers.
    type LayerKey: RenderingLayerKey;
}

/// A trait of Scene with SceneObjects of a certain type
pub trait SceneObjects<T>
where
    Self: Scene,
    T: SceneObject,
{
    /// Gets an iterator over the scene instances
    fn scene_objects(&self, layer_key: Self::LayerKey) -> Iter<T>;
}

/// A trait of Scene with at least one camera, which is
/// guaranteed to exist at the draw call.
///
/// In the case, when there is more than one camera,
/// it is up to the scene to decide, which camera
/// is provided.
pub trait PrimaryCamera<T>
where
    Self: Scene,
    T: scene_object::Camera,
{
    /// Gets the primary camera of the scene
    fn primary_camera(&self, layer_key: Self::LayerKey) -> T;
}

/// A trait of Scene with all types of lights
pub trait Lights<A, D, P, S>
where
    Self: Scene,
    A: AmbientLight,
    D: DirectionalLight,
    P: PointLight,
    S: SpotLight,
{
    /// Gets the scene ambient light
    fn ambient_light(&self, layer_key: Self::LayerKey) -> A;

    /// Gets an iterator over scene directional lights
    fn directional_lights(&self, layer_key: Self::LayerKey) -> Iter<D>;

    /// Gets an iterator over scene point lights
    fn point_lights(&self, layer_key: Self::LayerKey) -> Iter<P>;

    /// Gets an iterator over scene spot lights
    fn spot_lights(&self, layer_key: Self::LayerKey) -> Iter<S>;
}
