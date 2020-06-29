use super::glfw::*;

use log::{error, info};
use std::ptr;

pub enum WindowMode {
    Windowed,
    Borderless,
    Fullscreen,
}

pub struct Window {
    title: String,
    width: i32,
    height: i32,
    raw_window: *mut GLFWwindow,
}

impl Window {
    pub fn init_glfw() {
        info!("Initializing glfw");

        unsafe {
            if glfwInit() != 1 {
                error!("Failed to initialize glfw");
            }
            let mut major = 0;
            let mut minor = 0;
            let mut rev = 0;
            glfwGetVersion(&mut major, &mut minor, &mut rev);
            info!("GLFW version {}.{}.{}", major, minor, rev);
        }
    }

    /// Creates a new window with specified title, width, height, and mode
    /// Mode specified if the window is normal windowed, fullscreen or borderless
    /// if any dimension is -1, it will be set to the native resolution
    pub fn new(title: &str, mut width: i32, mut height: i32, mode: WindowMode) -> Window {
        let mut monitor: *const GLFWmonitor = ptr::null();
        let raw_window = unsafe {
            let primary = glfwGetPrimaryMonitor();
            let vidmode = glfwGetVideoMode(primary);
            if width == -1 {
                width = (*vidmode).width;
            }
            if height == -1 {
                height = (*vidmode).height;
            }

            glfwWindowHint(GLFW_CLIENT_API, GLFW_NO_API);
            match mode {
                WindowMode::Borderless => glfwWindowHint(GLFW_DECORATED, 0),
                WindowMode::Windowed => {}
                WindowMode::Fullscreen => {
                    glfwWindowHint(GLFW_DECORATED, 0);
                    monitor = primary;
                }
            }
            glfwCreateWindow(width, height, title.as_ptr(), monitor, ptr::null())
        };

        Window {
            title: String::from(title),
            width,
            height,
            raw_window,
        }
    }

    pub fn process_events(&self) {
        unsafe { glfwPollEvents() };
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn width(&self) -> i32 {
        self.width
    }
    pub fn height(&self) -> i32 {
        self.height
    }
}
