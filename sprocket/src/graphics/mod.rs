mod glfw;
pub use log::{debug, error, info, trace, warn};
use std::sync::Arc;
use window::Window;

pub mod error;
pub mod vulkan;
pub mod window;

pub use error::{Error, Result};

const SWAPCHAIN_IMAGE_COUNT: u32 = 3;

pub enum GraphicsContext {
    Vulkan(Arc<vulkan::VulkanContext>),
    OpenGL,
}

#[derive(Debug)]
pub enum Api {
    Vulkan,
    // Opengl is not implemented yet
    OpenGL,
}

/// Initializes the graphics api and returns a context
pub fn init(api: Api, window: &Window) -> Result<GraphicsContext> {
    match api {
        Api::Vulkan => match vulkan::init(window) {
            Ok(context) => Ok(GraphicsContext::Vulkan(Arc::new(context))),
            Err(f) => Err(f),
        },
        Api::OpenGL => Err(Error::UnsupportedAPI(api)),
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

impl From<ash::vk::Extent2D> for Extent2D {
    fn from(vkextent: ash::vk::Extent2D) -> Self {
        Extent2D {
            width: vkextent.width,
            height: vkextent.height,
        }
    }
}

impl From<Extent2D> for ash::vk::Extent2D {
    fn from(extent: Extent2D) -> Self {
        ash::vk::Extent2D {
            width: extent.width,
            height: extent.height,
        }
    }
}

impl From<(u32, u32)> for Extent2D {
    fn from(t: (u32, u32)) -> Self {
        Extent2D {
            width: t.0,
            height: t.1,
        }
    }
}

impl From<(i32, i32)> for Extent2D {
    fn from(t: (i32, i32)) -> Self {
        Extent2D {
            width: t.0 as u32,
            height: t.1 as u32,
        }
    }
}

impl Copy for Extent2D {}
