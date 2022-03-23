use crate::math::Vec3;

#[ambassador::delegatable_trait]
pub trait Color {
    /// Gets the light color
    fn color(&self) -> Vec3;
}

#[ambassador::delegatable_trait]
pub trait Intensity {
    /// Gets the light intensity
    fn intensity(&self) -> f32;
}

#[ambassador::delegatable_trait]
pub trait Attenuation {
    /// Gets the light attenuation
    fn attenuation(&self) -> Vec3;
}

#[ambassador::delegatable_trait]
pub trait OpeningAngle {
    /// Gets the light opening angle in radians
    fn opening_angle_rad(&self) -> f32;

    /// Gets the opening angle in degrees
    fn opening_angle_deg(&self) -> f32 {
        f32::to_degrees(self.opening_angle_rad())
    }
}

#[ambassador::delegatable_trait]
pub trait Direction {
    /// Gets the light direction
    fn direction(&self) -> Vec3;
}

/// A presense on the scene (Isometry) automatically implies Direction
impl<T> Direction for T
where
    T: super::SceneObject,
{
    fn direction(&self) -> Vec3 {
        let [x, y, z, ..]: [f32; 4] =
            (self.normal_matrix() * [0f32, 0f32, -1f32, 0f32].into()).into();
        [x, y, z].into()
    }
}
