use crate::math::Mat4;

/// A component of the camera, which holds a projection matrix
#[ambassador::delegatable_trait]
pub trait ProjectionMatrix {
    /// Gets the projection matrix of the camera,
    /// i.e. a linear transformation from local space to
    /// a camera local space, which is later projected onto
    /// a 2D plane.
    fn projection_matrix(&self) -> Mat4;
}
