pub mod mesh;
pub mod mesh_vertex;

pub mod components {
    mod camera;
    mod light;

    pub use camera::*;
    pub use light::*;
}

pub mod fields {
    mod attenuation;
    mod color;

    pub use attenuation::*;
    pub use color::*;
    pub type Vector3f = physics::prelude::Vector<f32>;
}

pub mod prelude {
    use crate::mesh::*;
    use crate::mesh_vertex::*;
    use asset_storage::asset_storage::AssetStorage;

    pub use crate::components;
    pub use crate::fields;

    /// A Mesh with colored vertices
    pub type ColoredMesh = Mesh<ColoredVertex>;

    /// An AssetStorage of colored meshes
    pub type ColoredMeshStorage<K> = AssetStorage<K, ColoredMesh>;

    /// A Mesh with textured vertices
    pub type TexturedMesh = Mesh<TexturedVertex>;

    /// An AssetStorage of textured meshes
    pub type TexturedMeshStorage<K> = AssetStorage<K, TexturedMesh>;
}
