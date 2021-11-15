/* ---- PRELUDE ---- */
pub mod prelude {
    pub use crate::base_event::BaseEvent;
    pub use crate::clockwork::*;
    pub use crate::main_loop::MainLoop;
    pub use crate::mechanism::*;
    pub use crate::state::*;
    pub use crate::sync::Lock;
}

/* ---- REEXPORTS ---- */
pub extern crate derive_builder;
pub extern crate getset;
pub extern crate itertools;
pub extern crate log;

/* ---- MODULES ---- */
pub mod base_event;
pub mod clockwork;
pub mod event;
pub mod main_loop;
pub mod mechanism;
pub mod state;
pub mod sync;
pub mod util {
    pub mod init_state;
}
