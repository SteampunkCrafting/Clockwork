use crate::fields::Color;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhongMaterial {
    pub ambient: Color,
    pub diffuse: Color,
    pub specular: Color,
    pub specular_power: f32,
}
