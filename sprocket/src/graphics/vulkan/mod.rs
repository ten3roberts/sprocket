#![allow(dead_code)]
use crate::graphics::glfw;
use crate::*;
use std::borrow::Cow;
use std::collections::HashSet;
use std::ffi::{c_void, CStr, CString};
use std::ptr;

use ash::extensions::khr::Surface;
use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0};
use ash::{vk, vk::Handle, Entry};

mod swapchain;
use swapchain::Swapchain;

pub struct VulkanContext {
    entry: ash::Entry,
    instance: ash::Instance,
    debug_messenger: vk::DebugUtilsMessengerEXT,
    surface: vk::SurfaceKHR,
    device: ash::Device,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,
    swapchain: Swapchain,
}

pub struct QueueFamilies {
    pub graphics: Option<u32>,
    pub present: Option<u32>,
    pub compute: Option<u32>,
    pub present_support: bool,
}

impl QueueFamilies {
    unsafe fn find(
        instance: &ash::Instance,
        physical_device: &vk::PhysicalDevice,
        surface_loader: &Surface,
        surface: &vk::SurfaceKHR,
    ) -> QueueFamilies {
        let families = instance.get_physical_device_queue_family_properties(*physical_device);
        let mut graphics_family = None;
        let mut presentation_family = None;
        let mut compute_family = None;
        let mut present_support = false;
        for (i, family) in families.iter().enumerate() {
            if family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                graphics_family = Some(i as u32);
            }
            if surface_loader
                .get_physical_device_surface_support(*physical_device, i as u32, *surface)
                .unwrap_or(false)
            {
                presentation_family = Some(i as u32);
                present_support = surface_loader
                    .get_physical_device_surface_support(*physical_device, i as u32, *surface)
                    .unwrap_or(false);
            }
            if family.queue_flags.contains(vk::QueueFlags::COMPUTE) {
                compute_family = Some(i as u32);
            }
        }

        QueueFamilies {
            graphics: graphics_family,
            present: presentation_family,
            compute: compute_family,
            present_support,
        }
    }
}

pub fn init(window: &Window) -> Result<VulkanContext, Cow<'static, str>> {
    unsafe {
        let entry = unwrap_or_return!("Failed to create vulkan entry", Entry::new());

        let validation_layers = ["VK_LAYER_KHRONOS_validation"];
        let device_extensions = ["VK_KHR_swapchain"];

        // Ensure all requested layers exist
        check_validation_layer_support(&entry, &validation_layers)?;
        let instance = create_instance(&entry, &validation_layers)?;

        let debug_messenger = create_debug_messenger(&entry, &instance)?;
        let surface = create_surface(&instance, &window)?;
        // Choose physical devices

        let surface_loader = Surface::new(&entry, &instance);
        let (physical_device, queue_families) =
            find_physical_device(&instance, &surface_loader, &surface, &device_extensions)?;

        let device = create_device(
            &instance,
            physical_device,
            &queue_families,
            &device_extensions,
        )?;
        let graphics_queue = device.get_device_queue(queue_families.graphics.unwrap(), 0);
        let present_queue = device.get_device_queue(queue_families.present.unwrap(), 0);
        let swapchain = unwrap_or_return!(
            "Failed to create swapchain",
            Swapchain::new(
                &instance,
                &physical_device,
                &device,
                &surface_loader,
                &surface,
                &queue_families,
                window.width(),
                window.height(),
            )
        );
        Ok(VulkanContext {
            entry,
            instance,
            debug_messenger,
            surface,
            device,
            graphics_queue,
            present_queue,
            swapchain,
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
    let app_name = CString::new("Sprocket").unwrap();
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
    let layers = utils::vec_to_null_terminated(layers);
    let layers = utils::vec_to_carray(&layers);

    let create_info = vk::InstanceCreateInfo::builder()
        .application_info(&app_info)
        .enabled_layer_names(&layers)
        .enabled_extension_names(&extensions);
    unwrap_and_return!(
        "Failed to create instance",
        entry.create_instance(&create_info, None)
    )
}

fn check_validation_layer_support(
    entry: &ash::Entry,
    layers: &[&str],
) -> Result<(), Cow<'static, str>> {
    let available_layers = unwrap_or_return!(
        "Could not enumerate supported layers",
        entry.enumerate_instance_layer_properties()
    );

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
        unwrap_and_return!(
            "Failed to create debug utils messenger",
            debug_utils.create_debug_utils_messenger(&create_info, None)
        )
    }
}

unsafe fn create_surface(
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

unsafe fn rate_device(
    instance: &ash::Instance,
    device: &vk::PhysicalDevice,
    surface_loader: &Surface,
    surface: &vk::SurfaceKHR,
    extensions: &[&str],
) -> u32 {
    let mut score = 1;
    let properties = instance.get_physical_device_properties(*device);
    // let features = instance.get_physical_device_features(*device);

    let queue_families = QueueFamilies::find(instance, device, surface_loader, surface);

    let available_extensions: Vec<&CStr> =
        match instance.enumerate_device_extension_properties(*device) {
            Ok(extensions) => extensions
                .iter()
                .map(|extension| CStr::from_ptr(extension.extension_name.as_ptr()))
                .collect(),
            Err(e) => {
                error!("Failed to get supported device extensions '{}'", e);
                return 0;
            }
        };

    // Check if all layers exist
    for extension in extensions {
        let mut found = false;
        for available in &available_extensions {
            if available.to_string_lossy() == *extension {
                found = true;
                break;
            }
        }
        if !found {
            return 0;
        }
    }
    if queue_families.graphics.is_none() {
        return 0;
    }
    if queue_families.present.is_none() {
        return 0;
    }

    if !queue_families.present_support {
        return 0;
    }

    // Check adequate swapchain support
    let (capabilities, formats, present_modes) =
        match Swapchain::query_support(device, surface_loader, surface) {
            Ok(v) => v,
            Err(_) => return 0,
        };

    if capabilities.min_image_count > graphics::SWAPCHAIN_IMAGE_COUNT
        || (capabilities.max_image_count != 0
            && capabilities.max_image_count < graphics::SWAPCHAIN_IMAGE_COUNT)
    {
        return 0;
    }

    if formats.is_empty() {
        return 0;
    }
    if present_modes.is_empty() {
        return 0;
    }

    if properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU {
        score += 500
    };

    score += properties.limits.max_framebuffer_height / 10;
    score += properties.limits.max_framebuffer_width / 10;
    score += properties.limits.max_image_dimension2_d / 10;
    score += properties.limits.max_color_attachments;
    score
}

unsafe fn find_physical_device(
    instance: &ash::Instance,
    surface_loader: &Surface,
    surface: &vk::SurfaceKHR,
    device_extensions: &[&str],
) -> Result<(vk::PhysicalDevice, QueueFamilies), Cow<'static, str>> {
    let devices = instance.enumerate_physical_devices().unwrap_or_default();

    let best_device = match devices
        .iter()
        .zip(devices.iter().map(|device| {
            rate_device(instance, device, surface_loader, surface, device_extensions)
        }))
        .filter(|(_, score)| *score > 0)
        .max_by(|(_, prev_score), (_, score)| score.cmp(prev_score))
    {
        Some(device) => device,
        None => return Err("Unable to find suitable GPU".into()),
    };

    let device_properties = instance.get_physical_device_properties(*best_device.0);
    info!(
        "Using device {:?}",
        CStr::from_ptr(device_properties.device_name.as_ptr())
    );

    Ok((
        *best_device.0,
        QueueFamilies::find(instance, best_device.0, surface_loader, surface),
    ))
}

unsafe fn create_device(
    instance: &ash::Instance,
    pdevice: vk::PhysicalDevice,
    queue_families: &QueueFamilies,
    device_extensions: &[&str],
) -> Result<ash::Device, Cow<'static, str>> {
    let priorities = [1.0];

    let mut queue_infos = Vec::new();

    let mut unique_families = HashSet::new();
    unique_families.insert(queue_families.graphics.unwrap());
    unique_families.insert(queue_families.present.unwrap());
    debug!("Unique queue families {}", unique_families.len());

    for queue_family in unique_families {
        let queue_info = vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(queue_family)
            .queue_priorities(&priorities)
            .build();
        queue_infos.push(queue_info);
    }

    let features = vk::PhysicalDeviceFeatures {
        shader_clip_distance: 1,
        ..Default::default()
    };

    // Convert the slice to *const *const null terminated
    let device_extensions = utils::vec_to_null_terminated(device_extensions);
    let device_extensions = utils::vec_to_carray(&device_extensions);

    let device_create_info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queue_infos)
        .enabled_features(&features)
        .enabled_extension_names(&device_extensions);

    unwrap_and_return!(
        "Failed to create logical device",
        instance.create_device(pdevice, &device_create_info, None)
    )
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
