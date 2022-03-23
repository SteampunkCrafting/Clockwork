use kernel::math::{Vec3, Vec4};
use std::convert::{TryFrom, TryInto};

#[derive(Default, Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct Color {
    red: u8,
    green: u8,
    blue: u8,
    alpha: u8,
}

impl kernel::graphics::light_components::Color for Color {
    fn color(&self) -> Vec3 {
        self.clone().into()
    }
}

impl TryFrom<Vec4> for Color {
    type Error = &'static str;

    fn try_from(x: Vec4) -> Result<Self, Self::Error> {
        let x: [f32; 4] = x.into();
        x.try_into()
    }
}

impl Into<Vec4> for Color {
    fn into(self) -> Vec4 {
        let x: [f32; 4] = self.into();
        x.into()
    }
}

impl TryFrom<Vec3> for Color {
    type Error = &'static str;

    fn try_from(x: Vec3) -> Result<Self, Self::Error> {
        let x: [f32; 3] = x.into();
        x.try_into()
    }
}

impl Into<Vec3> for Color {
    fn into(self) -> Vec3 {
        let x: [f32; 3] = self.into();
        x.into()
    }
}

impl From<[u8; 4]> for Color {
    fn from([red, green, blue, alpha]: [u8; 4]) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }
}

impl From<[u8; 3]> for Color {
    fn from([red, green, blue]: [u8; 3]) -> Self {
        [red, green, blue, 255].into()
    }
}

impl TryFrom<[f32; 4]> for Color {
    type Error = &'static str;

    fn try_from(arr: [f32; 4]) -> Result<Self, Self::Error> {
        match arr {
            [red, green, blue, alpha] if arr.iter().cloned().all(|x| 0.0 <= x && x <= 1.0) => {
                Ok([red as u8, green as u8, blue as u8, alpha as u8].into())
            }
            _ => {
                Err("In order to create a color, all float values must be clipped from 0.0 to 1.0")
            }
        }
    }
}

impl TryFrom<[f32; 3]> for Color {
    type Error = &'static str;

    fn try_from([red, green, blue]: [f32; 3]) -> Result<Self, Self::Error> {
        [red, green, blue, 1.0].try_into()
    }
}

impl Into<[u8; 4]> for Color {
    fn into(self) -> [u8; 4] {
        let Self {
            red,
            green,
            blue,
            alpha,
        } = self;
        [red, green, blue, alpha]
    }
}

impl Into<[f32; 4]> for Color {
    fn into(self) -> [f32; 4] {
        let vec: Vec<_> = Into::<[u8; 4]>::into(self)
            .iter()
            .cloned()
            .map(|x| x as f32)
            .map(|x| x / 255f32)
            .collect();
        [vec[0], vec[1], vec[2], vec[3]]
    }
}

impl Into<[f32; 3]> for Color {
    fn into(self) -> [f32; 3] {
        let vec: Vec<_> = Into::<[u8; 4]>::into(self)
            .iter()
            .cloned()
            .map(|x| x as f32)
            .map(|x| x / 255f32)
            .collect();
        [vec[0], vec[1], vec[2]]
    }
}

impl Into<[u8; 3]> for Color {
    fn into(self) -> [u8; 3] {
        let Self {
            red, green, blue, ..
        } = self;
        [red, green, blue]
    }
}
