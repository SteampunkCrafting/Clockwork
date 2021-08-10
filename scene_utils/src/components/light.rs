use crate::fields::{Attenuation, Color, Vector3f};

pub struct AmbientLight {
    pub color: Color,
}

pub struct DirectionalLight {
    pub color: Color,
    pub direction: Vector3f,
    pub attenuation: Attenuation,
}

pub struct PointLight {
    pub color: Color,
    pub attenuation: Attenuation,
}

pub struct SpotLight {
    pub color: Color,
    pub attenuation: Attenuation,
}
