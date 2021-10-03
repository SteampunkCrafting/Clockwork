use crate::fields::{Color, Texture2D};

#[derive(Clone)]
pub enum PhongMaterial {
    Colored {
        ambient: Color,
        diffuse: Color,
        specular: Color,
        specular_power: f32,
    },
    Textured {
        texture: Texture2D,
        specular_power: f32,
    },
}
