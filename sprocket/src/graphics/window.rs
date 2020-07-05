use super::glfw::*;
use crate::event::Event;
use crate::event::KeyCode;
use log::{debug, error, info, warn};
use std::ptr;
use std::sync::mpsc;

use num_traits::FromPrimitive;

pub enum WindowMode {
    Windowed,
    Borderless,
    Fullscreen,
}
/// This is the userpointer given to the data
/// Needs to be separate so that the address is known and not moved
struct WindowData {
    sender: mpsc::Sender<Event>,
    in_focus: bool,
    width: i32,
    height: i32,
}

pub struct Window {
    title: String,
    raw_window: *mut GLFWwindow,
    data: *mut WindowData,
}

impl Window {
    pub fn init_glfw() {
        debug!("Initializing glfw");

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

    pub fn terminate_glfw() {
        debug!("Terminating glfw");

        unsafe {
            glfwTerminate();
        }
    }

    /// Creates a new window with specified title, width, height, and mode
    /// Mode specified if the window is normal windowed, fullscreen or borderless
    /// if any dimension is -1, it will be set to the native resolution
    pub fn new(
        title: &str,
        mut width: i32,
        mut height: i32,
        mode: WindowMode,
        sender: mpsc::Sender<Event>,
    ) -> Window {
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
            raw_window,
            data: Box::into_raw(Box::new(WindowData {
                width,
                height,
                sender,
                in_focus: false,
            })),
        };

        unsafe {
            glfwSetWindowUserPointer(raw_window, window.data as *mut std::ffi::c_void);
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

    pub fn process_events(&self) {
        unsafe { glfwPollEvents() };
    }

    pub fn in_focus(&self) -> bool {
        unsafe { (*self.data).in_focus }
    }

    pub fn should_close(&self) -> bool {
        unsafe { glfwWindowShouldClose(self.raw_window) != 0 }
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn width(&self) -> i32 {
        unsafe { (*self.data).width }
    }

    pub fn height(&self) -> i32 {
        unsafe { (*self.data).height }
    }

    /// # Safety
    /// Returns the underlying GLFW window
    /// Will fail if glfw_terminate is called with alive windows
    pub unsafe fn get_raw(&self) -> *const GLFWwindow {
        self.raw_window
    }
}

// Returns the sender from window user pointer
unsafe fn get_data(window: *mut GLFWwindow) -> Option<*mut WindowData> {
    let data = glfwGetWindowUserPointer(window) as *mut WindowData;

    if data.is_null() {
        error!("Invalid window event sender");
        return None;
    }
    Some(data)
}

#[no_mangle]
extern "C" fn close_callback(window: *mut GLFWwindow) {
    unsafe {
        if let Some(data) = get_data(window) {
            (*data)
                .sender
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
        if let Some(data) = get_data(window) {
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
            (*data)
                .sender
                .send(event)
                .expect("Failed to send window close event");
        };
    }
}
#[no_mangle]
extern "C" fn mouse_button_callback(window: *mut GLFWwindow, button: i32, action: i32) {
    unsafe {
        if let Some(data) = get_data(window) {
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
            (*data)
                .sender
                .send(event)
                .expect("Failed to send window close event");
        };
    }
}
#[no_mangle]
extern "C" fn scroll_callback(window: *mut GLFWwindow, xoffset: f64, yoffset: f64) {
    unsafe {
        if let Some(data) = get_data(window) {
            // Convert button 0-5 to keycode which starts with mouse buttons after keyboard keys

            (*data)
                .sender
                .send(Event::Scroll(xoffset as i32, yoffset as i32))
                .expect("Failed to send window close event");
        };
    }
}
#[no_mangle]
extern "C" fn mouse_position_callback(window: *mut GLFWwindow, xpos: f64, ypos: f64) {
    unsafe {
        if let Some(data) = get_data(window) {
            (*data)
                .sender
                .send(Event::MousePosition(xpos as i32, ypos as i32))
                .expect("Failed to send window close event");
        };
    }
}
#[no_mangle]
extern "C" fn window_size_callback(window: *mut GLFWwindow, width: i32, height: i32) {
    unsafe {
        if let Some(data) = get_data(window) {
            (*data).width = width;
            (*data).height = height;
            (*data)
                .sender
                .send(Event::WindowResize(width, height))
                .expect("Failed to send window close event");
        };
    }
}
#[no_mangle]
extern "C" fn window_focus_callback(window: *mut GLFWwindow, focus: i32) {
    unsafe {
        if let Some(data) = get_data(window) {
            (*data).in_focus = focus != 0;
            (*data)
                .sender
                .send(Event::WindowFocus(focus != 0))
                .expect("Failed to send window close event");
        };
    }
}
extern "C" fn char_callback(window: *mut GLFWwindow, codepoint: u32) {
    unsafe {
        if let Some(data) = get_data(window) {
            (*data)
                .sender
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

            if let Some(data) = get_data(self.raw_window) {
                Box::from_raw(data);
            }
        }
    }
}
