pub mod event;
pub mod main_loop;
pub mod state;

pub mod prelude {
    pub use crate::event::*;
    pub use crate::main_loop::*;
    pub use crate::state::*;
    pub use winit::event::VirtualKeyCode;
    pub use winit::window::Window;
}
