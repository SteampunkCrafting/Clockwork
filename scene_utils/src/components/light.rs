use crate::fields::{Attenuation, Color, Vector3f};

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct AmbientLight {
    pub color: Color,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct DirectionalLight {
    pub color: Color,
    pub direction: Vector3f,
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
