use physics::prelude::nalgebra::UnitVector3;

use crate::fields::{Attenuation, Color};

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct AmbientLight {
    pub color: Color,
}

#[derive(Debug, Clone, Copy)]
pub struct DirectionalLight {
    pub color: Color,
    pub direction: UnitVector3<f32>,
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct PointLight {
    pub color: Color,
    pub attenuation: Attenuation,
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct SpotLight {
    pub opening_angle: f32,
    pub color: Color,
    pub attenuation: Attenuation,
}

impl Default for DirectionalLight {
    fn default() -> Self {
        Self {
            color: Default::default(),
            direction: UnitVector3::new_unchecked([0.0, -1.0, 0.0].into()),
        }
    }
}
