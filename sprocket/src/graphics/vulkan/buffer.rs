use super::CommandBuffer;
use super::CommandPool;
use crate::graphics::Extent2D;
use ash::version::{DeviceV1_0, InstanceV1_0};
use ash::vk;

use super::{Error, Result, VkAllocator};

// Creates a staging buffer with specified size
// Buffer is already mapped on creation
pub fn create_staging(
    allocator: &VkAllocator,
    size: u64,
) -> Result<(vk::Buffer, vk_mem::Allocation, vk_mem::AllocationInfo)> {
    allocator
        .borrow()
        .create_buffer(
            &vk::BufferCreateInfo::builder()
                .size(size)
                .usage(vk::BufferUsageFlags::TRANSFER_SRC)
                .sharing_mode(vk::SharingMode::EXCLUSIVE)
                .build(),
            &vk_mem::AllocationCreateInfo {
                usage: vk_mem::MemoryUsage::CpuToGpu,
                flags: vk_mem::AllocationCreateFlags::MAPPED,
                ..Default::default()
            },
        )
        .map_err(|e| e.into())
}

/// Copies the contents of one buffer to another
pub fn copy(
    device: &ash::Device,
    queue: vk::Queue,
    commandpool: &CommandPool,
    src_buffer: vk::Buffer,
    dst_buffer: vk::Buffer,
    size: vk::DeviceSize,
) -> Result<()> {
    let commandbuffer = &mut CommandBuffer::new_primary(device, commandpool, 1)?[0];

    commandbuffer.begin(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)?;

    let region = vk::BufferCopy::builder()
        .src_offset(0)
        .dst_offset(0)
        .size(size)
        .build();
    unsafe { device.cmd_copy_buffer(commandbuffer.vk(), src_buffer, dst_buffer, &[region]) }

    commandbuffer.end()?;

    CommandBuffer::submit(
        device,
        &[commandbuffer],
        queue,
        &[],
        &[],
        &[],
        vk::Fence::null(),
    )?;

    unsafe { device.queue_wait_idle(queue).map_err(|e| e.into()) }
}

pub fn copy_to_image(
    device: &ash::Device,
    queue: vk::Queue,
    commandpool: &CommandPool,
    src_buffer: vk::Buffer,
    dst_image: vk::Image,
    extent: Extent2D,
    aspect: vk::ImageAspectFlags,
) -> Result<()> {
    let region = vk::BufferImageCopy {
        buffer_offset: 0,
        buffer_row_length: 0,
        buffer_image_height: 0,
        image_subresource: vk::ImageSubresourceLayers {
            aspect_mask: aspect,
            mip_level: 0,
            base_array_layer: 0,
            layer_count: 1,
        },
        image_offset: vk::Offset3D { x: 0, y: 0, z: 0 },
        image_extent: vk::Extent3D {
            width: extent.width,
            height: extent.height,
            depth: 1,
        },
    };
    let commandbuffer = &mut CommandBuffer::new_primary(device, commandpool, 1)?[0];

    commandbuffer.begin(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)?;

    unsafe {
        device.cmd_copy_buffer_to_image(
            commandbuffer.vk(),
            src_buffer,
            dst_image,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            &[region],
        )
    }

    commandbuffer.end()?;

    CommandBuffer::submit(
        device,
        &[commandbuffer],
        queue,
        &[],
        &[],
        &[],
        vk::Fence::null(),
    )?;

    unsafe { device.queue_wait_idle(queue).map_err(|e| e.into()) }
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
) -> Result<u32> {
    let mem_properties = unsafe { instance.get_physical_device_memory_properties(physical_device) };
    for i in 0..mem_properties.memory_type_count {
        if type_filter & (1 << i) != 0
            && (mem_properties.memory_types[i as usize]
                .property_flags
                .as_raw()
                & properties.as_raw()
                != 0)
        {
            return Ok(i);
        }
    }
    Err(Error::MissingMemoryType(properties))
}
