use super::{
    layer_key::RenderingLayerKey, scene_object, AmbientLight, DirectionalLight, PointLight,
    SceneObject, SpotLight,
};

/// A marker trait for the collection of objects on the scene
pub trait Scene {
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
    /// An iterator type over instances
    type InstanceIter: Iterator<Item = T>;

    /// Gets an iterator over the scene instances
    fn instances(&self, layer_key: Self::LayerKey) -> Self::InstanceIter;
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

pub trait Lights<A, D, P, S>
where
    Self: Scene,
    A: AmbientLight,
    D: DirectionalLight,
    P: PointLight,
    S: SpotLight,
{
    /// Directional light iterator type
    type DirLightIter: Iterator<Item = D>;

    /// Point light iterator type
    type PointLightIter: Iterator<Item = P>;

    /// Spot light iterator type
    type SpotLightIter: Iterator<Item = S>;

    /// Gets the scene ambient light
    fn ambient_light(&self, layer_key: Self::LayerKey) -> A;

    /// Gets scene directional lights
    fn directional_lights(&self, layer_key: Self::LayerKey) -> Self::DirLightIter;

    /// Gets scene point lights
    fn point_lights(&self, layer_key: Self::LayerKey) -> Self::PointLightIter;

    /// Gets scene spot lights
    fn spot_lights(&self, layer_key: Self::LayerKey) -> Self::SpotLightIter;
}
