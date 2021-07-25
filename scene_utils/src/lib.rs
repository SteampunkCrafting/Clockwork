pub mod mesh;
pub mod mesh_vertex;

pub mod prelude {
    use crate::mesh::*;
    use crate::mesh_vertex::*;
    use asset_storage::asset_storage::AssetStorage;

    /// A Mesh with colored vertices
    pub type ColoredMesh = Mesh<ColoredVertex>;

    /// An AssetStorage of colored meshes
    pub type ColoredMeshStorage<K> = AssetStorage<K, ColoredMesh>;

    /// A Mesh with textured vertices
    pub type TexturedMesh = Mesh<TexturedVertex>;

    /// An AssetStorage of textured meshes
    pub type TexturedMeshStorage<K> = AssetStorage<K, TexturedMesh>;
}
