mod glfw;
pub use log::{debug, error, info, trace, warn};
pub mod vulkan;
pub mod window;
use std::borrow::Cow;
use window::Window;

const SWAPCHAIN_IMAGE_COUNT: u32 = 3;

pub enum GraphicsContext {
    Vulkan(vulkan::VulkanContext),
}

pub enum Api {
    Vulkan,
    // Opengl is not implemented yet
    OpenGL,
}

/// Initializes the graphics api and returns a context
pub fn init(api: Api, window: &Window) -> Result<GraphicsContext, Cow<'static, str>> {
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
