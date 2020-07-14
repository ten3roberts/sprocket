use super::CommandBuffer;
use super::CommandPool;
use ash::version::{DeviceV1_0, InstanceV1_0};
use ash::vk;

// Creates a new low level vulkan buffer
pub fn create(
    instance: &ash::Instance,
    device: &ash::Device,
    physical_device: vk::PhysicalDevice,
    size: vk::DeviceSize,
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

// Copies the contents of one buffer to another
pub fn copy(
    device: &ash::Device,
    queue: vk::Queue,
    commandpool: &CommandPool,
    src_buffer: vk::Buffer,
    dst_buffer: vk::Buffer,
    size: vk::DeviceSize,
) {
    let commandbuffer = &mut CommandBuffer::new_primary(device, commandpool, 1).unwrap()[0];
    // let commandbuffer = &mut commandbuffer[0];

    commandbuffer
        .begin(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
        .unwrap();

    let region = vk::BufferCopy::builder()
        .src_offset(0)
        .dst_offset(0)
        .size(size)
        .build();
    unsafe { device.cmd_copy_buffer(commandbuffer.vk(), src_buffer, dst_buffer, &[region]) }

    commandbuffer.end().unwrap();

    CommandBuffer::submit(
        device,
        &[commandbuffer],
        queue,
        &[],
        &[],
        &[],
        vk::Fence::null(),
    )
    .unwrap();

    unsafe { device.queue_wait_idle(queue).unwrap() }
}

pub fn destroy(device: &ash::Device, buffer: vk::Buffer, memory: vk::DeviceMemory) {
    unsafe {
        device.destroy_buffer(buffer, None);
        device.free_memory(memory, None);
    }
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
