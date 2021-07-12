pub mod mesh;
pub mod mesh_vertex;

pub mod prelude {
    use crate::mesh::*;
    use crate::mesh_vertex::*;

    /// A Mesh with colored vertices
    pub type ColoredMesh = Mesh<ColoredVertex>;

    /// A Mesh with textured vertices
    pub type TexturedMesh = Mesh<TexturedVertex>;
}
