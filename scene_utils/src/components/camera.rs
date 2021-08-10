use physics::prelude::nalgebra::{Matrix4, Orthographic3, Perspective3};

#[derive(Debug, Clone, Copy)]
pub enum Camera {
    Perspective(Perspective3<f32>),
    Orthographic(Orthographic3<f32>),
}

impl Into<Matrix4<f32>> for Camera {
    fn into(self) -> Matrix4<f32> {
        match self {
            Camera::Perspective(x) => x.into(),
            Camera::Orthographic(x) => x.into(),
        }
    }
}

impl Into<[[f32; 4]; 4]> for Camera {
    fn into(self) -> [[f32; 4]; 4] {
        Into::<Matrix4<f32>>::into(self).into()
    }
}
