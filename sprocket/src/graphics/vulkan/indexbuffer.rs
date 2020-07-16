use super::buffer;
use super::CommandPool;
use super::{Result, VkAllocator};
use ash::vk;
use std::sync::Arc;

pub struct IndexBuffer {
    allocator: VkAllocator,
    buffer: vk::Buffer,
    memory: vk_mem::Allocation,
    size: vk::DeviceSize,
    /// The number of elements in the buffer
    count: u32,
}

impl IndexBuffer {
    pub fn new(
        allocator: &VkAllocator,
        device: &ash::Device,
        queue: vk::Queue,
        commandpool: &CommandPool,
        indices: &[u32],
    ) -> Result<IndexBuffer> {
        let buffer_size = match indices.len() {
            0 => 1024,
            n => (n * std::mem::size_of_val(&indices[0])) as u64,
        };

        let (staging_buffer, staging_memory, _) = allocator.borrow().create_buffer(
            &vk::BufferCreateInfo::builder()
                .size(buffer_size)
                .usage(vk::BufferUsageFlags::TRANSFER_SRC)
                .sharing_mode(vk::SharingMode::EXCLUSIVE)
                .build(),
            &vk_mem::AllocationCreateInfo {
                usage: vk_mem::MemoryUsage::CpuToGpu,
                ..Default::default()
            },
        )?;

        // Copy the data into the buffer
        let data = allocator.borrow().map_memory(&staging_memory)?;
        unsafe {
            std::ptr::copy_nonoverlapping(indices.as_ptr() as _, data, buffer_size as usize);
        }
        allocator.borrow().unmap_memory(&staging_memory)?;

        let (buffer, memory, _) = allocator.borrow().create_buffer(
            &vk::BufferCreateInfo::builder()
                .size(buffer_size)
                .usage(vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER)
                .sharing_mode(vk::SharingMode::EXCLUSIVE)
                .build(),
            &vk_mem::AllocationCreateInfo {
                usage: vk_mem::MemoryUsage::GpuOnly,
                ..Default::default()
            },
        )?;

        buffer::copy(
            device,
            queue,
            commandpool,
            staging_buffer,
            buffer,
            buffer_size,
        )?;

        allocator
            .borrow()
            .destroy_buffer(staging_buffer, &staging_memory)?;

        Ok(IndexBuffer {
            allocator: Arc::clone(allocator),
            buffer: buffer,
            memory: memory,
            size: buffer_size,
            count: indices.len() as u32,
        })
    }
    pub fn buffer(&self) -> vk::Buffer {
        self.buffer
    }

    pub fn index_type(&self) -> vk::IndexType {
        vk::IndexType::UINT32
    }

    pub fn count(&self) -> u32 {
        self.count
    }
}

impl Drop for IndexBuffer {
    fn drop(&mut self) {
        self.allocator
            .borrow()
            .destroy_buffer(self.buffer, &self.memory)
            .expect("Failed to free vulkan memory");
    }
}
