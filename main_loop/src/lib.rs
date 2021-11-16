pub mod main_loop;
pub mod state {
    /* ---- PRIVATE ---- */
    mod input;
    mod statistics;
    mod winit_loop;

    /* ---- PUBLIC ---- */
    pub use input::*;
    pub use statistics::*;
    pub use winit_loop::*;
}

pub mod prelude {
    pub use crate::main_loop::*;
    pub use crate::state::*;
    pub use winit::event::VirtualKeyCode;
    pub use winit::window::Window;
}
