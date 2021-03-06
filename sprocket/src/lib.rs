#[macro_use]
pub mod macros;
pub mod application;
pub mod event;
pub mod graphics;
pub mod logger;
pub mod math;
pub mod utils;
pub use application::Application;
pub use graphics::window::{Window, WindowMode};
/// Exports logging macros
pub use log::{debug, error, info, trace, warn};
pub use math::{Vec2, Vec3, Vec4};

mod time;
pub use time::Time;

mod timer;
pub use timer::Timer;

// Systems
pub mod systems;

// Components
pub mod ecs;

// Physics
pub mod physics;
