pub mod skybox_drawer;
pub mod static_mesh_drawer;

pub mod prelude {
    pub use crate::skybox_drawer::SkyboxDrawer;
    pub use crate::static_mesh_drawer::StaticMeshDrawer;
}

pub(crate) mod util;
