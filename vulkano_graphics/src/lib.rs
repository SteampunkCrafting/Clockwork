pub mod mechanism;
pub mod state;
pub mod vulkano_layer;

pub use vulkano;
pub use vulkano_shaders;
pub use vulkano_win;

pub mod prelude {
    pub use crate::gui::Gui;
    pub use crate::mechanism::VulkanoGraphics;
    pub use crate::state::GraphicsState;
    pub use crate::vulkano_layer::OldVulkanoLayer;
}

pub mod gui {
    pub use egui_winit_vulkano::*;
}
