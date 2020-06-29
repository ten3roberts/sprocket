use log::{error, info};
use std::ffi;
use std::ptr;

#[link(name = "glfw")]
extern "C" {
    fn glfwInit() -> i32;
    fn glfwGetVersion(major: *mut i32, minor: *mut i32, rev: *mut i32);
    fn glfwCreateWindow(
        width: i32,
        height: i32,
        title: *const u8,
        monitor: *const ffi::c_void,
        share: *const ffi::c_void,
    ) -> *mut ffi::c_void;
}

type GLFWWindow = ffi::c_void;

pub struct Window {
    title: String,
    width: i32,
    height: i32,
    raw_window: *mut GLFWWindow,
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

    pub fn new(title: &str, width: i32, height: i32) -> Window {
        let raw_window =
            unsafe { glfwCreateWindow(width, height, title.as_ptr(), ptr::null(), ptr::null()) };

        Window {
            title: String::from(title),
            width,
            height,
            raw_window,
        }
    }
}
