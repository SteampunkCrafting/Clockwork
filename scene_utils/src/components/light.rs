use crate::fields;
use kernel::abstract_runtime::Delegate;
use kernel::ambassador_impl_Attenuation_body_single_struct;
use kernel::ambassador_impl_Color_body_single_struct;
use kernel::graphics::light_components::Intensity;
use kernel::graphics::light_components::OpeningAngle;
use kernel::graphics::light_components::{Attenuation, Color};
use kernel::math::Vec3;
use physics::prelude::nalgebra::UnitVector3;

#[derive(Default, Debug, Clone, Copy, PartialEq, Delegate)]
#[delegate(Color)]
pub struct AmbientLight {
    pub color: fields::Color,
}
impl kernel::graphics::Light for AmbientLight {}
impl kernel::graphics::AmbientLight for AmbientLight {}

#[derive(Debug, Clone, Copy, Delegate)]
#[delegate(Color, target = "color")]
pub struct DirectionalLight {
    pub color: fields::Color,
    pub direction: UnitVector3<f32>,
}
impl kernel::graphics::Light for DirectionalLight {}
impl kernel::graphics::light_components::Direction for DirectionalLight {
    fn direction(&self) -> kernel::math::Vec3 {
        let dir = self.direction.clone();
        [[dir.x], [dir.y], [dir.z]].into()
    }
}
impl kernel::graphics::DirectionalLight for DirectionalLight {}

#[derive(Default, Debug, Clone, Copy, PartialEq, Delegate)]
#[delegate(Color, target = "color")]
#[delegate(Attenuation, target = "attenuation")]
pub struct PointLight {
    pub color: fields::Color,
    pub attenuation: fields::Attenuation,
}
impl kernel::graphics::Light for PointLight {}
impl kernel::graphics::light_components::Intensity for PointLight {
    fn intensity(&self) -> f32 {
        1f32
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Delegate)]
#[delegate(Attenuation, target = "attenuation")]
#[delegate(Color, target = "color")]
pub struct SpotLight {
    pub opening_angle: f32,
    pub color: fields::Color,
    pub attenuation: fields::Attenuation,
}

impl Intensity for SpotLight {
    fn intensity(&self) -> f32 {
        1f32
    }
}

impl OpeningAngle for SpotLight {
    fn opening_angle_rad(&self) -> f32 {
        self.opening_angle
    }
}

impl Default for DirectionalLight {
    fn default() -> Self {
        Self {
            color: Default::default(),
            direction: UnitVector3::new_unchecked([0.0, -1.0, 0.0].into()),
        }
    }
}
