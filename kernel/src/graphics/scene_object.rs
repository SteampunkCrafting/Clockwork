use crate::math::Mat4;

/// A trait, which is implemented by every spacial object with geometry.
pub trait SceneObject {
    /// Gets the homogeneous world matrix, i.e. a linear transformation
    /// from the local space of the object to the world space.
    fn world_matrix(&self) -> Mat4;

    /// Gets the homogeneous view matrix, i.e. a linear transformation
    /// from the world space to the local space of the object.
    fn view_matrix(&self) -> Mat4;
}

/// A subtype of `SceneObject`, which represents
/// a Camera, and contains a projection matrix.
pub trait Camera
where
    Self: SceneObject,
{
    /// Gets the projection matrix of the camera,
    /// i.e. a linear transformation from local space to
    /// a camera local space, which is later projected onto
    /// a 2D plane.
    fn projection_matrix(&self) -> Mat4;
}

/// A subtype of `SceneObject`, which
/// contains a normal matrix
pub trait NormalMatrix
where
    Self: SceneObject,
{
    /// Get the homogeneous normal matrix, i.e. a model matrix for normals.
    ///
    /// For the case of a uniform scaling, it is just a world matrix,
    /// but in the general case, the normal matrix is defined as
    /// `transpose(inverse(world_matrix))`
    fn normal_matrix(&self) -> Mat4;
}

/// A subtype of a `SceneObject`,
/// which contains some mesh geometry.
pub trait Mesh<MeshID>
where
    Self: SceneObject,
{
    /// Gets a mesh id of the object
    fn mesh_id(&self) -> MeshID;
}

/// A subtype of a `SceneObject`,
/// which contains some material.
pub trait Material<MaterialID>
where
    Self: SceneObject,
{
    /// Gets the material id of the object
    fn material_id(&self) -> MaterialID;
}

/// A subtype of a `SceneObject`,
/// which contains some skeletal animations
pub trait Skeletal<SkeletonID, AnimationID = SkeletonID>
where
    Self: SceneObject,
{
    /// Gets the skeleton id of the object.
    fn skeleton_id(&self) -> SkeletonID;

    /// Gets the animation id of the object;
    fn animation_id(&self) -> AnimationID;

    /// Gets the animation time of the object;
    fn animation_time(&self) -> f32;
}
