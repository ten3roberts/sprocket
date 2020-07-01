#[macro_use]
pub mod macros;
pub mod application;
pub mod event;
pub mod graphics;
pub mod logger;
pub mod math;
pub use application::Application;
pub use graphics::window::{Window, WindowMode};
/// Exports logging macros
pub use log::{debug, error, info, trace, warn};
pub use math::{Vec2, Vec3, Vec4};
