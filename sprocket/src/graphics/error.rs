//! Describes a graphics related error
use super::glfw;
use ash::vk;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    IoError(ex::io::Error),
    VulkanError(vk::Result),
    GLFWError(glfw::Error),
    InstanceError(ash::InstanceError),
    MissingLayer(String),
    UnsupportedAPI(super::Api),
    UnsupportedGPU(super::Api),
    SPVReadError(std::io::Error, String),
    NotRecording,
    MissingMemoryType(vk::MemoryPropertyFlags),
}

impl From<vk::Result> for Error {
    fn from(error: vk::Result) -> Self {
        Error::VulkanError(error)
    }
}

impl From<ex::io::Error> for Error {
    fn from(error: ex::io::Error) -> Self {
        Error::IoError(error)
    }
}

impl From<glfw::Error> for Error {
    fn from(error: glfw::Error) -> Self {
        Error::GLFWError(error)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IoError(e) => write!(f, "Io error {:?}", e),
            Error::VulkanError(e) => write!(f, "Vulkan error {:?}", e),
            Error::InstanceError(e) => write!(f, "Instance creation error {:?}", e),
            Error::GLFWError(e) => write!(f, "GLFW error {:?}", e),
            Error::MissingLayer(l) => write!(f, "Cannot locate {} on the system", l),
            Error::UnsupportedAPI(a) => write!(f, "{:?} is not supported", a),
            Error::UnsupportedGPU(a) => {
                write!(f, "Unable to find a suitable GPU supporting {:?}", a)
            }
            Error::SPVReadError(e, path) => {
                write!(f, "Failed to read SPV from file {:?}'{:?}'", path, e)
            }
            Error::NotRecording => write!(f, "Command buffer is not in recording state"),
            Error::MissingMemoryType(properties) => {
                write!(f, "Cannot find GPU memory type supporting {:?}", properties)
            }
        }

        // write!(f, "({}, {}, {}, {})", self.x, self.y, self.z, self.w)
    }
}
