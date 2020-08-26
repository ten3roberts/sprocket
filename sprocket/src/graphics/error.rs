//! Describes a graphics related error
use super::glfw;
use ash::vk;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    IoError(ex::io::Error),
    VulkanError(vk::Result),
    VMAError(vk_mem::Error),
    GLFWError(glfw::Error),
    InstanceError(ash::InstanceError),
    MissingLayer(String),
    UnsupportedAPI(super::Api),
    UnsupportedGPU(super::Api),
    SPVReadError(std::io::Error, String),
    ImageReadError(String),
    NotRecording,
    MissingMemoryType(vk::MemoryPropertyFlags),
    MismatchedBinding(vk::DescriptorType, u32, u32),
    NoAllocator,
    UnsupportedTransition(vk::ImageLayout, vk::ImageLayout),
    XMLError(simple_xml::Error),
    JSONError(serde_json::Error),
    ParseError,
    UnimplementedFeature(&'static str),
}

impl From<vk::Result> for Error {
    fn from(error: vk::Result) -> Self {
        Error::VulkanError(error)
    }
}

impl From<vk_mem::Error> for Error {
    fn from(error: vk_mem::Error) -> Self {
        Error::VMAError(error)
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

impl From<simple_xml::Error> for Error {
    fn from(error: simple_xml::Error) -> Self {
        Error::XMLError(error)
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Error::JSONError(error)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IoError(e) => write!(f, "Io error {:?}", e),
            Error::VulkanError(e) => write!(f, "Vulkan error {:?}", e),
            Error::VMAError(e) => write!(f, "Vulkan allocation error {:?}", e),
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
            Error::ImageReadError(path) => {
                write!(f, "Failed to read image from file '{:?}'", path)
            }
            Error::NotRecording => write!(f, "Command buffer is not in recording state"),
            Error::MissingMemoryType(properties) => {
                write!(f, "Cannot find GPU memory type supporting {:?}", properties)
            }
            Error::MismatchedBinding(ty, binding_count, supplied_count) => write!(f, "Descriptor set bindings count do not match supplied count for {:?}. Expected {}, supplied {}", ty, binding_count, supplied_count),
            Error::NoAllocator => write!(f, "The specified resource has no allocator associated with it"),
            Error::UnsupportedTransition(src, dst) => write!(f, "The image transition from {:?} to {:?} is not supported", src, dst),
            Error::XMLError(e) => write!(f, "Failed to read xml file {:?}", e),
            Error::JSONError(e) => write!(f, "Failed to parse json file {:?}", e),
            Error::ParseError => write!(f, "Failed to parse string into a type"),
            Error::UnimplementedFeature(e) => write!(f, "Feature {} is not yet implemented", e),
        }
    }
}
