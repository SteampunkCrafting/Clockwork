use crate::math::Vec3;

/// A source of light, which illuminates from a point.
pub trait PointLight {
    /// Gets the light position
    fn position(&self) -> Vec3;

    /// Gets the light color
    fn color(&self) -> Vec3;

    /// Gets the light intensity
    fn intensity(&self) -> f32;

    /// Gets the light attenuation
    fn attenuation(&self) -> Vec3;
}

/// A source of light, which illuminates from outside the scene.
pub trait DirectionalLight {
    /// Gets the light color
    fn color(&self) -> Vec3;

    /// Gets the light direction
    fn direction(&self) -> Vec3;
}

/// A source of light, which illuminates from within the space.
pub trait AmbientLight {
    /// Gets the ambient light color
    fn color(&self) -> Vec3;
}

/// A source of light, which illuminates from within the space,
/// and has some opening angle and direction.
pub trait SpotLight {
    /// Gets the light position
    fn position(&self) -> Vec3;

    /// Gets the light color
    fn color(&self) -> Vec3;

    /// Gets the light intensity
    fn intensity(&self) -> f32;

    /// Gets the light attenuation
    fn attenuation(&self) -> Vec3;

    /// Gets the light direction
    fn direction(&self) -> Vec3;

    /// Gets the opening angle of the light
    fn opening_angle(&self) -> f32;
}
