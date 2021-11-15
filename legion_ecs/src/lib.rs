/* ---- PRELUDE ---- */
pub mod prelude {
    pub use crate::mechanism::LegionSystems;
    pub use crate::state::LegionState;
    pub use legion::*;
}

/* ---- MODULES ---- */
/// Mechanism description
pub mod mechanism;

/// State description
pub mod state;

/// Utilities
pub mod util {
    /* ---- REEXPORTS ---- */
    pub use legion;
}
