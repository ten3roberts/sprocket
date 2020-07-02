use crate::graphics::glfw;
use crate::*;
use ash::{version::EntryV1_0, vk, Entry};
use std::ffi::{c_void, CStr, CString};

pub struct VulkanContext {
    entry: ash::Entry,
    instance: ash::Instance,
    debug_messenger: vk::DebugUtilsMessengerEXT,
}

pub fn init() -> Result<VulkanContext, String> {
    let entry = match Entry::new() {
        Ok(entry) => entry,
        Err(e) => return errfmt!("Failed to create vulkan entry {}", e),
    };

    let validation_layers = ["VK_LAYER_KHRONOS_validation"];

    // Ensure all requested layers exist
    check_validation_layer_support(&entry, &validation_layers)?;
    let instance = create_instance(&entry, &validation_layers)?;

    let debug_messenger = create_debug_messenger(&entry, &instance)?;
    Ok(VulkanContext {
        entry,
        instance,
        debug_messenger,
    })
}

fn create_instance(entry: &ash::Entry, layers: &[&str]) -> Result<ash::Instance, String> {
    let app_info = vk::ApplicationInfo {
        api_version: vk::make_version(1, 0, 0),
        ..Default::default()
    };

    // Extension support
    let mut glfw_extension_count = 0;
    let glfw_extensions =
        unsafe { glfw::glfwGetRequiredInstanceExtensions(&mut glfw_extension_count) };

    let mut extensions = Vec::with_capacity(glfw_extension_count as usize);
    unsafe {
        for i in 0..glfw_extension_count {
            let extension = *glfw_extensions.offset(i as isize);
            extensions.push(extension);
        }
        extensions.push(b"VK_EXT_debug_utils\0".as_ptr() as *const i8);
    }

    info!("Extensions: {:?}", extensions);

    // Convert the slice to *const *const null terminated
    let layer_count = layers.len();
    let layers: Vec<CString> = layers
        .iter()
        .map(|layer| CString::new(*layer).expect("Failed to convert layer to c_str"))
        .collect();

    let layers: Vec<*const i8> = layers.iter().map(|layer| layer.as_ptr()).collect();

    let create_info = vk::InstanceCreateInfo {
        p_application_info: &app_info,
        enabled_extension_count: extensions.len() as u32,
        pp_enabled_extension_names: extensions.as_ptr(),
        pp_enabled_layer_names: layers.as_ptr(),
        enabled_layer_count: layer_count as u32,
        ..Default::default()
    };
    let instance = unsafe { entry.create_instance(&create_info, None) };
    match instance {
        Ok(instance) => Ok(instance),
        Err(e) => errfmt!("Failed to create vulkan instance {}", e),
    }
}

fn check_validation_layer_support(entry: &ash::Entry, layers: &[&str]) -> Result<(), String> {
    let available_layers = match entry.enumerate_instance_layer_properties() {
        Ok(layers) => layers,
        Err(e) => return errfmt!("Could not enumerate supported layers {}", e),
    };
    let available_layers: Vec<&CStr> = available_layers
        .iter()
        .map(|layer| unsafe { CStr::from_ptr(layer.layer_name.as_ptr()) })
        .collect();

    // Check if all layers exist
    for layer in layers {
        let mut found = false;
        for available in &available_layers {
            if available.to_string_lossy() == *layer {
                found = true;
                break;
            }
        }
        if !found {
            return errfmt!("Could not find validation layer {}", layer);
        }
    }

    Ok(())
}

fn create_debug_messenger(
    entry: &ash::Entry,
    instance: &ash::Instance,
) -> Result<vk::DebugUtilsMessengerEXT, String> {
    let create_info = vk::DebugUtilsMessengerCreateInfoEXT {
        s_type: vk::StructureType::DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT,
        message_severity: vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
            | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
            | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
        message_type: vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
            | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
            | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
        pfn_user_callback: Some(debug_callback),
        p_user_data: std::ptr::null_mut(),
        p_next: std::ptr::null(),
        flags: vk::DebugUtilsMessengerCreateFlagsEXT::default(),
    };

    unsafe {
        let debug_utils = ash::extensions::ext::DebugUtils::new(entry, instance);
        match debug_utils.create_debug_utils_messenger(&create_info, None) {
            Ok(messenger) => Ok(messenger),
            Err(e) => errfmt!("Failed to create debug utils messenger {}", e),
        }
    }
}

impl Drop for VulkanContext {
    fn drop(&mut self) {
        info!("Dropping vulkan context");
    }
}

#[no_mangle]
unsafe extern "system" fn debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    _message_types: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut c_void,
) -> vk::Bool32 {
    let message = CStr::from_ptr((*p_callback_data).p_message);
    match message_severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => error!("{:?}", message),
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => warn!("{:?}", message),
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => info!("{:?}", message),
        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => info!(
            "VERBOSE: {:?}",
            CStr::from_ptr((*p_callback_data).p_message)
        ),
        _ => info!("Other: {:?}", message),
    }
    vk::FALSE
}
