pub use core;
pub use ecs;
pub use main_loop;
pub use physics;

pub mod prelude {
    pub use core::prelude::*;
    pub use ecs::prelude::*;
    pub use main_loop::prelude::*;
    pub use physics::prelude::*;
}
