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
    /// Standard Clockwork Event definitions.
    mod event;
    /// Standard mechanism definitions.
    mod mechanism;
    /// Standard statistics trait.
    mod statistics;

    /* ---- PUBLIC ---- */
    pub use event::*;
    pub use mechanism::*;
    pub use statistics::*;
}

/// A set of structs and traits for computer graphics
/// and rendering.
pub mod graphics {
    /* ---- PRIVATE ---- */
    /// A module, which defines the trait for the
    mod layer_key;
    /// A module, which defines the trait for the lights
    mod light;
    /// A module, which contains a clockwork substate trait, which
    /// can act as a renderable collection, providing renderers
    /// with a camera, assets and instances, required for drawing.
    mod scene;

    /* ---- PUBLIC ---- */
    /// A module, which defines a trait of an abstract object of the scene,
    /// as well as some additional parts it contain.
    mod scene_object;
    /// A module, which contains a `Vertex` trait
    pub mod vertex;

    pub use layer_key::RenderingLayerKey;
    pub use light::*;
    pub use scene::Scene;
    pub use scene_object::SceneObject;
    pub use vertex::Vertex;
}

/// Math utilities.
///
/// > This module may eventually become a separate crate
pub mod math {
    /* ---- PRIVATE ---- */
    mod matrix;

    /* ---- PUBLIC ---- */
    pub use matrix::Matrix;
    pub type Vector<const N: usize> = Matrix<f32, N, 1>;
    pub type Mat2 = Matrix<f32, 2, 2>;
    pub type Mat3 = Matrix<f32, 3, 3>;
    pub type Mat4 = Matrix<f32, 4, 4>;
    pub type Vec2 = Matrix<f32, 2, 1>;
    pub type Vec3 = Matrix<f32, 3, 1>;
    pub type Vec4 = Matrix<f32, 4, 1>;
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
