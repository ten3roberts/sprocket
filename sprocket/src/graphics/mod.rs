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
            Err("Invalid Graphics API".into())
        }
    }
}

pub struct Extent2D {
    width: u32,
    height: u32,
}

impl Extent2D {
    pub fn new(width: u32, height: u32) -> Extent2D {
        Extent2D { width, height }
    }
}
impl Clone for Extent2D {
    fn clone(&self) -> Self {
        Extent2D { ..*self }
    }
}

impl std::convert::From<ash::vk::Extent2D> for Extent2D {
    fn from(vkextent: ash::vk::Extent2D) -> Self {
        Extent2D {
            width: vkextent.width,
            height: vkextent.height,
        }
    }
}

impl Copy for Extent2D {}
