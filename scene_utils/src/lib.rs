pub mod mesh;
pub mod mesh_vertex;

pub mod components {
    mod camera;
    mod light;
    mod material;

    pub use camera::*;
    pub use light::*;
    pub use material::*;
}

pub mod fields {
    mod attenuation;
    mod color;

    use std::marker::PhantomData;

    use physics::prelude::nalgebra::{ArrayStorage, Vector3};

    pub use attenuation::*;
    pub use color::*;
    pub type Vector3f = Vector3<f32>;
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
