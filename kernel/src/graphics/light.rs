use super::light_components::*;
use super::SceneObject;

/// An abstract source of light
#[ambassador::delegatable_trait]
pub trait Light: Color {}

/// A source of light, which illuminates from within the space.
#[ambassador::delegatable_trait]
pub trait AmbientLight: Light {}

/// A source of light, which illuminates from a point.
#[ambassador::delegatable_trait]
pub trait PointLight: SceneObject + Light + Intensity + Attenuation {}

/// A source of light, which illuminates from outside the scene.
#[ambassador::delegatable_trait]
pub trait DirectionalLight: Light + Direction {}

/// A source of light, which illuminates from within the point,
/// and has some opening angle and direction.
#[ambassador::delegatable_trait]
pub trait SpotLight:
    SceneObject + OpeningAngle + Light + Intensity + Attenuation + Direction
{
}
