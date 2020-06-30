use super::glfw::*;
use crate::event::Event;
use log::{error, info};
use std::ptr;
use std::sync::mpsc;

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
            let c_title =
                std::ffi::CString::new(title).expect("Failed to convert window title to c_str");

            glfwCreateWindow(width, height, c_title.as_ptr(), monitor, ptr::null())
        };

        let window = Window {
            title: String::from(title),
            width,
            height,
            raw_window,
        };

        unsafe {
            // Set callbacks
            glfwSetWindowCloseCallback(raw_window, close_callback);
            glfwSetWindowUserPointer(raw_window, ptr::null_mut());
        }

        window
    }

    pub fn set_event_sender(&mut self, sender: mpsc::Sender<Event>) {
        unsafe {
            glfwSetWindowUserPointer(
                self.raw_window,
                Box::into_raw(Box::new(sender)) as *mut std::ffi::c_void,
            );
        }
    }

    pub fn should_close(&self) -> bool {
        unsafe { glfwWindowShouldClose(self.raw_window) != 0 }
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

// Returns the sender from window user pointer
unsafe fn get_sender(window: *mut GLFWwindow) -> Option<*mut mpsc::Sender<Event>> {
    let sender = glfwGetWindowUserPointer(window) as *mut mpsc::Sender<Event>;

    if sender == ptr::null_mut() {
        error!("Invalid window event sender");
        None
    } else {
        Some(&mut *sender)
    }
}

#[no_mangle]
extern "C" fn close_callback(window: *mut GLFWwindow) {
    unsafe {
        if let Some(sender) = get_sender(window) {
            (*sender)
                .send(Event::WindowClose)
                .expect("Failed to send window close event");
        };
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        unsafe {
            glfwDestroyWindow(self.raw_window);

            // Reclaim event sender and drop it

            if let Some(sender) = get_sender(self.raw_window) {
                Box::from_raw(sender);
            }
        }
    }
}
