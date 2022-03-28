use serde::{Deserialize, Serialize};

/// Attenuation is a factor of light intensity decay over distance,
/// which contains constant, linear, and quadratic components.
///
/// Default attenuation is 1.0 for constant, and 0.0 for other parts,
/// meaning that the light will not decay over distance.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Attenuation {
    pub constant: f32,
    pub linear: f32,
    pub quadratic: f32,
}

impl kernel::graphics::light_components::Attenuation for Attenuation {
    fn attenuation(&self) -> kernel::math::Vec3 {
        [self.constant, self.linear, self.quadratic].into()
    }
}

impl Default for Attenuation {
    fn default() -> Self {
        Self {
            constant: 1.0,
            linear: 0.0,
            quadratic: 0.0,
        }
    }
}
