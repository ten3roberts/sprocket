use ash::version::{DeviceV1_0, InstanceV1_0};
use ash::vk;

pub fn create(
    instance: &ash::Instance,
    device: &ash::Device,
    physical_device: vk::PhysicalDevice,
    size: u64,
    usage: vk::BufferUsageFlags,
    properties: vk::MemoryPropertyFlags,
) -> Result<(vk::Buffer, vk::DeviceMemory), vk::Result> {
    let buffer_info = vk::BufferCreateInfo::builder()
        .size(size)
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        .usage(usage);

    let buffer = unsafe { device.create_buffer(&buffer_info, None) }?;

    let memory_requirements = unsafe { device.get_buffer_memory_requirements(buffer) };
    let memory_type_index = match find_memory_type(
        instance,
        physical_device,
        memory_requirements.memory_type_bits,
        properties,
    ) {
        Some(v) => v,
        None => return Err(vk::Result::ERROR_FEATURE_NOT_PRESENT), // Temporary error
    };

    let alloc_info = vk::MemoryAllocateInfo::builder()
        .allocation_size(memory_requirements.size)
        .memory_type_index(memory_type_index);

    let memory = unsafe { device.allocate_memory(&alloc_info, None)? };

    unsafe { device.bind_buffer_memory(buffer, memory, 0)? };

    Ok((buffer, memory))
}

fn find_memory_type(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    type_filter: u32,
    properties: vk::MemoryPropertyFlags,
) -> Option<u32> {
    let mem_properties = unsafe { instance.get_physical_device_memory_properties(physical_device) };
    for i in 0..mem_properties.memory_type_count {
        if type_filter & (1 << i) != 0
            && (mem_properties.memory_types[i as usize]
                .property_flags
                .as_raw()
                & properties.as_raw()
                != 0)
        {
            return Some(i);
        }
    }
    None
}
