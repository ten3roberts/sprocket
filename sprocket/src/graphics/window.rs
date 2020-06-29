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

    pub fn new(title: &str, width: i32, height: i32, mode: WindowMode) -> Window {
        unsafe { glfwWindowHint(GLFW_CLIENT_API, GLFW_NO_API) };
        let mut monitor: *const GLFWmonitor = ptr::null();
        let raw_window = unsafe {
            match mode {
                WindowMode::Borderless => glfwWindowHint(GLFW_DECORATED, 0),
                WindowMode::Windowed => {}
                WindowMode::Fullscreen => {
                    glfwWindowHint(GLFW_DECORATED, 0);
                    monitor = glfwGetPrimaryMonitor();
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
