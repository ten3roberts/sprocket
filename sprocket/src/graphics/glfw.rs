#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
use std::ffi;
#[link(name = "glfw")]
#[no_mangle]
extern "C" {
    pub fn glfwInit() -> i32;
    pub fn glfwGetVersion(major: *mut i32, minor: *mut i32, rev: *mut i32);
    pub fn glfwCreateWindow(
        width: i32,
        height: i32,
        title: *const i8,
        monitor: *const ffi::c_void,
        share: *const ffi::c_void,
    ) -> *mut ffi::c_void;
    pub fn glfwDestroyWindow(window: *mut GLFWwindow);
    pub fn glfwWindowShouldClose(window: *mut GLFWwindow) -> i32;
    pub fn glfwPollEvents();
    pub fn glfwWindowHint(hint: i32, value: i32);
    pub fn glfwGetPrimaryMonitor() -> *const GLFWmonitor;
    pub fn glfwGetVideoMode(monitor: *const GLFWmonitor) -> *const GLFWvidmode;

    pub fn glfwSetWindowUserPointer(window: *mut GLFWwindow, pointer: *mut ffi::c_void);
    pub fn glfwGetWindowUserPointer(window: *mut GLFWwindow) -> *mut ffi::c_void;
    // Callbacks

    pub fn glfwSetWindowCloseCallback(
        window: *mut GLFWwindow,
        callback: extern "C" fn(*mut GLFWwindow),
    );

    pub fn glfwSetKeyCallback(
        window: *mut GLFWwindow,
        callback: extern "C" fn(
            window: *mut GLFWwindow,
            key: i32,
            scancode: i32,
            action: i32,
            mods: i32,
        ),
    );

    pub fn glfwSetMouseButtonCallback(
        window: *mut GLFWwindow,
        callback: extern "C" fn(window: *mut GLFWwindow, button: i32, action: i32),
    );

    pub fn glfwSetScrollCallback(
        window: *mut GLFWwindow,
        callback: extern "C" fn(window: *mut GLFWwindow, xoffset: f64, yoffset: f64),
    );
    pub fn glfwSetCursorPosCallback(
        window: *mut GLFWwindow,
        callback: extern "C" fn(window: *mut GLFWwindow, xpos: f64, ypos: f64),
    );
    pub fn glfwSetWindowSizeCallback(
        window: *mut GLFWwindow,
        callback: extern "C" fn(window: *mut GLFWwindow, width: i32, height: i32),
    );
    pub fn glfwSetWindowFocusCallback(
        window: *mut GLFWwindow,
        callback: extern "C" fn(window: *mut GLFWwindow, focused: i32),
    );
    pub fn glfwSetCharCallback(
        window: *mut GLFWwindow,
        callback: extern "C" fn(window: *mut GLFWwindow, codepoint: u32),
    );
}

pub type GLFWwindow = ffi::c_void;
pub type GLFWmonitor = ffi::c_void;
#[repr(C)]
pub struct GLFWvidmode {
    pub width: i32,
    pub height: i32,
    pub redBits: i32,
    pub greenBits: i32,
    pub blueBits: i32,
    pub refreshRate: i32,
}

#[link(name = "glfw")]
pub const GLFW_FOCUSED: i32 = 0x00020001;
pub const GLFW_ICONIFIED: i32 = 0x00020002;
pub const GLFW_RESIZABLE: i32 = 0x00020003;
pub const GLFW_VISIBLE: i32 = 0x00020004;
pub const GLFW_DECORATED: i32 = 0x00020005;
pub const GLFW_AUTO_ICONIFY: i32 = 0x00020006;
pub const GLFW_FLOATING: i32 = 0x00020007;
pub const GLFW_MAXIMIZED: i32 = 0x00020008;
pub const GLFW_CENTER_CURSOR: i32 = 0x00020009;
pub const GLFW_TRANSPARENT_FRAMEBUFFER: i32 = 0x0002000A;
pub const GLFW_HOVERED: i32 = 0x0002000B;
pub const GLFW_FOCUS_ON_SHOW: i32 = 0x0002000C;
pub const GLFW_RED_BITS: i32 = 0x00021001;
pub const GLFW_GREEN_BITS: i32 = 0x00021002;
pub const GLFW_BLUE_BITS: i32 = 0x00021003;
pub const GLFW_ALPHA_BITS: i32 = 0x00021004;
pub const GLFW_DEPTH_BITS: i32 = 0x00021005;
pub const GLFW_STENCIL_BITS: i32 = 0x00021006;
pub const GLFW_ACCUM_RED_BITS: i32 = 0x00021007;
pub const GLFW_ACCUM_GREEN_BITS: i32 = 0x00021008;
pub const GLFW_ACCUM_BLUE_BITS: i32 = 0x00021009;
pub const GLFW_ACCUM_ALPHA_BITS: i32 = 0x0002100A;
pub const GLFW_AUX_BUFFERS: i32 = 0x0002100B;
pub const GLFW_STEREO: i32 = 0x0002100C;
pub const GLFW_SAMPLES: i32 = 0x0002100D;
pub const GLFW_SRGB_CAPABLE: i32 = 0x0002100E;
pub const GLFW_REFRESH_RATE: i32 = 0x0002100F;
pub const GLFW_DOUBLEBUFFER: i32 = 0x00021010;
pub const GLFW_CLIENT_API: i32 = 0x00022001;
pub const GLFW_CONTEXT_VERSION_MAJOR: i32 = 0x00022002;
pub const GLFW_CONTEXT_VERSION_MINOR: i32 = 0x00022003;
pub const GLFW_CONTEXT_REVISION: i32 = 0x00022004;
pub const GLFW_CONTEXT_ROBUSTNESS: i32 = 0x00022005;
pub const GLFW_OPENGL_FORWARD_COMPAT: i32 = 0x00022006;
pub const GLFW_CONTEXT_DEBUG: i32 = 0x00022007;
pub const GLFW_OPENGL_DEBUG_CONTEXT: i32 = GLFW_CONTEXT_DEBUG;
pub const GLFW_OPENGL_PROFILE: i32 = 0x00022008;
pub const GLFW_CONTEXT_RELEASE_BEHAVIOR: i32 = 0x00022009;
pub const GLFW_CONTEXT_NO_ERROR: i32 = 0x0002200A;
pub const GLFW_CONTEXT_CREATION_API: i32 = 0x0002200B;
pub const GLFW_SCALE_TO_MONITOR: i32 = 0x0002200C;
pub const GLFW_COCOA_RETINA_FRAMEBUFFER: i32 = 0x00023001;
pub const GLFW_COCOA_FRAME_NAME: i32 = 0x00023002;
pub const GLFW_COCOA_GRAPHICS_SWITCHING: i32 = 0x00023003;
pub const GLFW_X11_CLASS_NAME: i32 = 0x00024001;
pub const GLFW_X11_INSTANCE_NAME: i32 = 0x00024002;
pub const GLFW_WIN32_KEYBOARD_MENU: i32 = 0x00025001;
pub const GLFW_NO_API: i32 = 0;
pub const GLFW_OPENGL_API: i32 = 0x00030001;
pub const GLFW_OPENGL_ES_API: i32 = 0x00030002;
pub const GLFW_NO_ROBUSTNESS: i32 = 0;
pub const GLFW_NO_RESET_NOTIFICATION: i32 = 0x00031001;
pub const GLFW_LOSE_CONTEXT_ON_RESET: i32 = 0x00031002;

pub const GLFW_RELEASE: i32 = 0;
pub const GLFW_PRESS: i32 = 1;
pub const GLFW_REPEAT: i32 = 2;
