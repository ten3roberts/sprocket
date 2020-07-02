#![allow(dead_code)]
use crate::graphics::glfw;
use crate::*;
use std::borrow::Cow;
use std::ffi::{c_void, CStr, CString};
use std::ptr;

use ash::extensions::khr::Surface;
use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0};
use ash::{vk, vk::Handle, Device, Entry, Instance};
pub struct VulkanContext {
    entry: ash::Entry,
    instance: ash::Instance,
    debug_messenger: vk::DebugUtilsMessengerEXT,
    surface: vk::SurfaceKHR,
}

pub fn init(window: &Window) -> Result<VulkanContext, Cow<'static, str>> {
    unsafe {
        let entry = match Entry::new() {
            Ok(entry) => entry,
            Err(e) => return errfmt!("Failed to create vulkan entry {}", e),
        };

        let validation_layers = ["VK_LAYER_KHRONOS_validation"];

        // Ensure all requested layers exist
        check_validation_layer_support(&entry, &validation_layers)?;
        let instance = create_instance(&entry, &validation_layers)?;

        let debug_messenger = create_debug_messenger(&entry, &instance)?;
        let surface = create_surface(&entry, &instance, &window)?;
        // Choose physical devices

        let pdevices = instance.enumerate_physical_devices().unwrap_or(Vec::new());
        let surface_loader = Surface::new(&entry, &instance);

        let (pdevice, queue_family_index) = pdevices
            .iter()
            .map(|pdevice| {
                instance
                    .get_physical_device_queue_family_properties(*pdevice)
                    .iter()
                    .enumerate()
                    .filter_map(|(index, ref info)| {
                        let supports_graphic_and_surface =
                            info.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                                && surface_loader
                                    .get_physical_device_surface_support(
                                        *pdevice,
                                        index as u32,
                                        surface,
                                    )
                                    .unwrap();
                        if supports_graphic_and_surface {
                            Some((*pdevice, index as u32))
                        } else {
                            None
                        }
                    })
                    .next()
            })
            .filter_map(|v| v)
            .next()
            .expect("Couldn't find suitable device.");

        let device = create_device(&instance, pdevice, queue_family_index)?;
        Ok(VulkanContext {
            entry,
            instance,
            debug_messenger,
            surface,
        })
    }

    // // Find physical devices
    // let pdevices = instance..enumerate_physical_devices()?;
    //
}

unsafe fn create_instance(
    entry: &ash::Entry,
    layers: &[&str],
) -> Result<ash::Instance, Cow<'static, str>> {
    let app_name = CString::new("VulkanTriangle").unwrap();
    let app_info = vk::ApplicationInfo::builder()
        .application_name(&app_name)
        .application_version(0)
        .engine_name(&app_name)
        .engine_version(0)
        .api_version(vk::make_version(1, 0, 0));

    // Extension support
    let mut glfw_extension_count = 0;
    let glfw_extensions = glfw::glfwGetRequiredInstanceExtensions(&mut glfw_extension_count);

    let mut extensions = Vec::with_capacity(glfw_extension_count as usize);
    for i in 0..glfw_extension_count {
        let extension = *glfw_extensions.offset(i as isize);
        extensions.push(extension);
    }
    extensions.push(b"VK_EXT_debug_utils\0".as_ptr() as *const i8);

    info!("Extensions: {:?}", extensions);

    // Convert the slice to *const *const null terminated
    let layers: Vec<CString> = layers
        .iter()
        .map(|layer| CString::new(*layer).expect("Failed to convert layer to c_str"))
        .collect();

    let layers: Vec<*const i8> = layers.iter().map(|layer| layer.as_ptr()).collect();

    let create_info = vk::InstanceCreateInfo::builder()
        .application_info(&app_info)
        .enabled_layer_names(&layers)
        .enabled_extension_names(&extensions);
    match entry.create_instance(&create_info, None) {
        Ok(instance) => Ok(instance),
        Err(e) => errfmt!("Failed to create instance {}", e),
    }
}

fn check_validation_layer_support(
    entry: &ash::Entry,
    layers: &[&str],
) -> Result<(), Cow<'static, str>> {
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
) -> Result<vk::DebugUtilsMessengerEXT, Cow<'static, str>> {
    let create_info = vk::DebugUtilsMessengerCreateInfoEXT {
        s_type: vk::StructureType::DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT,
        message_severity: vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
            | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
            | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
        message_type: vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
            | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
            | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
        pfn_user_callback: Some(debug_callback),
        p_user_data: ptr::null_mut(),
        p_next: ptr::null(),
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

unsafe fn create_surface(
    entry: &ash::Entry,
    instance: &ash::Instance,
    window: &Window,
) -> Result<vk::SurfaceKHR, Cow<'static, str>> {
    let raw_window = window.get_raw();
    let mut surface_handle = 0;
    let instance = instance.handle();

    match glfw::glfwCreateWindowSurface(
        instance.as_raw(),
        raw_window,
        ptr::null(),
        &mut surface_handle,
    ) {
        vk::Result::SUCCESS => {}
        _ => return errfmt!("Failed to create window surface"),
    }
    Ok(vk::SurfaceKHR::from_raw(surface_handle))
}
unsafe fn create_device(
    instance: &ash::Instance,
    pdevice: vk::PhysicalDevice,
    queue_family_index: u32,
) -> Result<ash::Device, Cow<'static, str>> {
    let priorities = [1.0];

    let queue_info = [vk::DeviceQueueCreateInfo::builder()
        .queue_family_index(queue_family_index)
        .queue_priorities(&priorities)
        .build()];

    let features = vk::PhysicalDeviceFeatures {
        shader_clip_distance: 1,
        ..Default::default()
    };
    let device_create_info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queue_info)
        .enabled_features(&features);

    match instance.create_device(pdevice, &device_create_info, None) {
        Ok(device) => Ok(device),
        Err(e) => errfmt!("Failed to create logical device {}", e),
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
