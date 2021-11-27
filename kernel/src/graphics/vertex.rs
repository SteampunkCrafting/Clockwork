use crate::math::Vector;

/// A marker trait, which indicates that this type
/// can act as a vertex of an instance mesh.
pub trait Vertex {}

/// A trait, which indicates that this vertex type
/// is a source of position.
pub trait Position<const N: usize>
where
    Self: Vertex,
{
    /// Gets vertex position in the space of a model.
    fn position(&self) -> Vector<N>;
}

/// A trait, which indicates that this vertex type
/// is also a source of a surface normal.
pub trait Normal<const N: usize>
where
    Self: Vertex,
{
    /// Gets vertex normal in the space of a model.
    fn normal(&self) -> Vector<N>;
}

/// A trait, which indicates that this vertex type
/// is also a source of a texture coordinate.
pub trait TextureCoordinate<const N: usize>
where
    Self: Vertex,
{
    /// Gets texture coordinate
    /// in the space of a texture.
    fn texture_coordinate(&self) -> Vector<N>;
}

/// A trait, which indicates that this vertex type is
/// also a source of a Color.
pub trait Color<const N: usize>
where
    Self: Vertex,
{
    /// Gets color.
    fn color(&self) -> Vector<N>;
}

/// A trait, which indicates that this vertex type is also
/// a source of joint weights, which is used for skeletal
/// animations.
pub trait JointWeights<const N: usize>
where
    Self: Vertex,
{
    fn joint_weights(&self) -> Vector<N>;
}
