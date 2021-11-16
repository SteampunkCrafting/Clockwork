/* ---- PRELUDE ---- */
pub mod prelude {
    pub use crate::clockwork::Clockwork;
    pub use crate::standard_runtime::StandardEvent;
    pub use crate::util::itertools::*;
    pub use crate::util::log::*;
}

/* ---- MODULES ---- */
/// A set of very basic definitions,
/// required for every clockwork operation.
pub mod abstract_runtime {
    /* ---- PRIVATE ---- */
    /// Abstract Clockwork Event definitions.
    mod event;
    /// Abstract Clockwork main loop definition.
    pub mod main_loop;
    /// Abstract Clockwork Mechanism definitions.
    mod mechanism;
    /// Abstract Clockwork State definitions.
    mod state;

    /* ---- PUBLIC ---- */
    pub use event::*;
    pub use main_loop::*;
    pub use mechanism::*;
    pub use state::*;
}

/// Clockwork object definitions.
pub mod clockwork;

/// A set of utilities for a standard Clockwork runtime.
pub mod standard_runtime {
    /* ---- PRIVATE ---- */
    mod event;
    mod mechanism;
    mod statistics;

    /* ---- PUBLIC ---- */
    pub use event::*;
    pub use mechanism::*;
    pub use statistics::*;
}

/// Utilities
pub mod util {
    /* ---- LOCAL ---- */
    pub mod init_state;
    pub mod sync;

    /* ---- REEXPORTS ---- */
    pub use derive_builder;
    pub use getset;
    pub use itertools;
    pub use log;
}
