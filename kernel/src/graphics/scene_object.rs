use super::scene_object_components::*;
use crate::math::Mat4;

/// A trait, which is implemented by every spacial object with geometry.
#[ambassador::delegatable_trait]
pub trait SceneObject {
    /// Gets the homogeneous world matrix, i.e. a linear transformation
    /// from the local space of the object to the world space.
    fn world_matrix(&self) -> Mat4;

    /// Gets the homogeneous view matrix, i.e. a linear transformation
    /// from the world space to the local space of the object.
    fn view_matrix(&self) -> Mat4;

    /// Get the homogeneous normal matrix, i.e. a model matrix for normals.
    ///
    /// For the case of a uniform scaling, it is just a world matrix,
    /// but in the general case, the normal matrix is defined as
    /// `transpose(inverse(world_matrix))`
    fn normal_matrix(&self) -> Mat4;
}

/// A subtype of `SceneObject`, which represents
/// a Camera, and contains a projection matrix.
#[ambassador::delegatable_trait]
pub trait Camera
where
    Self: SceneObject + ProjectionMatrix,
{
}

/// A subtype of a `SceneObject`,
/// which contains some mesh geometry.
#[ambassador::delegatable_trait]
pub trait Mesh<MeshID>
where
    Self: SceneObject,
{
    /// Gets a mesh id of the object
    fn mesh_id(&self) -> MeshID;
}

/// A subtype of a `SceneObject`,
/// which contains some material.
#[ambassador::delegatable_trait]
pub trait Material<MaterialID>
where
    Self: SceneObject,
{
    /// Gets the material id of the object
    fn material_id(&self) -> MaterialID;
}

/// A subtype of a `SceneObject`,
/// which contains some skeletal animations
#[ambassador::delegatable_trait]
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
