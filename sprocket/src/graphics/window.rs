use super::glfw::*;
use crate::event::Event;
use crate::event::KeyCode;
use log::{error, info, warn};
use std::ptr;
use std::sync::mpsc;

use num_traits::FromPrimitive;

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
            glfwSetWindowUserPointer(raw_window, ptr::null_mut());
            // Set callbacks
            glfwSetWindowCloseCallback(raw_window, close_callback);
            glfwSetKeyCallback(raw_window, key_callback);
            glfwSetMouseButtonCallback(raw_window, mouse_button_callback);
            glfwSetScrollCallback(raw_window, scroll_callback);
            glfwSetCursorPosCallback(raw_window, mouse_position_callback);
            glfwSetWindowSizeCallback(raw_window, window_size_callback);
            glfwSetWindowFocusCallback(raw_window, window_focus_callback);
            glfwSetCharCallback(raw_window, char_callback);
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
#[no_mangle]
extern "C" fn key_callback(
    window: *mut GLFWwindow,
    key: i32,
    _scancode: i32,
    action: i32,
    _mods: i32,
) {
    unsafe {
        if let Some(sender) = get_sender(window) {
            let key = KeyCode::from_i32(key).unwrap_or(KeyCode::Invalid);
            let event = match action {
                GLFW_PRESS => Event::KeyPress(key),
                GLFW_RELEASE => Event::KeyRelease(key),
                GLFW_REPEAT => Event::KeyRepeat(key),
                _ => {
                    warn!("Unknown key action {}", action);
                    return;
                }
            };
            (*sender)
                .send(event)
                .expect("Failed to send window close event");
        };
    }
}
#[no_mangle]
extern "C" fn mouse_button_callback(window: *mut GLFWwindow, button: i32, action: i32) {
    unsafe {
        if let Some(sender) = get_sender(window) {
            // Convert button 0-5 to keycode which starts with mouse buttons after keyboard keys
            let key =
                KeyCode::from_i32(button + KeyCode::Mouse0 as i32).unwrap_or(KeyCode::Invalid);
            let event = match action {
                GLFW_PRESS => Event::KeyPress(key),
                GLFW_RELEASE => Event::KeyRelease(key),
                GLFW_REPEAT => Event::KeyRepeat(key),
                _ => {
                    warn!("Unknown key action {}", action);
                    return;
                }
            };
            (*sender)
                .send(event)
                .expect("Failed to send window close event");
        };
    }
}
#[no_mangle]
extern "C" fn scroll_callback(window: *mut GLFWwindow, xoffset: f64, yoffset: f64) {
    unsafe {
        if let Some(sender) = get_sender(window) {
            // Convert button 0-5 to keycode which starts with mouse buttons after keyboard keys

            (*sender)
                .send(Event::Scroll(xoffset as i32, yoffset as i32))
                .expect("Failed to send window close event");
        };
    }
}
#[no_mangle]
extern "C" fn mouse_position_callback(window: *mut GLFWwindow, xpos: f64, ypos: f64) {
    unsafe {
        if let Some(sender) = get_sender(window) {
            (*sender)
                .send(Event::MousePosition(xpos as i32, ypos as i32))
                .expect("Failed to send window close event");
        };
    }
}
#[no_mangle]
extern "C" fn window_size_callback(window: *mut GLFWwindow, width: i32, height: i32) {
    unsafe {
        if let Some(sender) = get_sender(window) {
            (*sender)
                .send(Event::WindowResize(width, height))
                .expect("Failed to send window close event");
        };
    }
}
#[no_mangle]
extern "C" fn window_focus_callback(window: *mut GLFWwindow, focus: i32) {
    unsafe {
        if let Some(sender) = get_sender(window) {
            (*sender)
                .send(Event::WindowFocus(focus != 0))
                .expect("Failed to send window close event");
        };
    }
}
extern "C" fn char_callback(window: *mut GLFWwindow, codepoint: u32) {
    unsafe {
        if let Some(sender) = get_sender(window) {
            (*sender)
                .send(Event::CharacterType(
                    std::char::from_u32(codepoint).unwrap_or('\u{fffd}'),
                ))
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
