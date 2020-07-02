mod glfw;
pub use log::{debug, error, info, trace, warn};
pub mod vulkan;
pub mod window;
use window::Window;

pub enum GraphicsContext {
    Vulkan(vulkan::VulkanContext),
}

pub enum Api {
    Vulkan,
    // Opengl is not implemented yet
    OpenGL,
}

/// Initializes the graphics api and returns a context
pub fn init(api: Api, window: &Window) -> Result<GraphicsContext, String> {
    match api {
        Api::Vulkan => match vulkan::init(window) {
            Ok(context) => Ok(GraphicsContext::Vulkan(context)),
            Err(msg) => Err(msg),
        },
        Api::OpenGL => {
            error!("OpenGL graphics is not implemented yet");
            return Err("Invalid Graphics API".into());
        }
    }
}
