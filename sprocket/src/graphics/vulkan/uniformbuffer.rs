use super::Result;
use super::VkAllocator;
use crate::math::Mat4;
use ash::vk;
use std::sync::Arc;

pub struct UniformBufferObject {
    pub model: Mat4,
    pub view: Mat4,
    pub proj: Mat4,
}

pub struct UniformBuffer {
    allocator: VkAllocator,
    buffer: vk::Buffer,
    memory: vk_mem::Allocation,
    size: vk::DeviceSize,
}
impl UniformBuffer {
    pub fn new(allocator: &VkAllocator, size: u64) -> Result<UniformBuffer> {
        let (buffer, memory, _) = allocator.borrow().create_buffer(
            &vk::BufferCreateInfo::builder()
                .size(size)
                .usage(vk::BufferUsageFlags::UNIFORM_BUFFER)
                .sharing_mode(vk::SharingMode::EXCLUSIVE)
                .build(),
            &vk_mem::AllocationCreateInfo {
                usage: vk_mem::MemoryUsage::CpuToGpu,
                ..Default::default()
            },
        )?;

        Ok(UniformBuffer {
            allocator: Arc::clone(allocator),
            buffer,
            memory,
            size,
        })
    }

    /// Writes data to the uniformbuffer in device memory
    pub fn write<T>(&self, data: &T, offset: Option<u64>, size: Option<u64>) -> Result<()> {
        let data: *const T = data;
        let size = size.unwrap_or(self.size);
        let offset = offset.unwrap_or(0);

        // Copy the data into the buffer
        let mapped: *mut u8 = self.allocator.borrow().map_memory(&self.memory)?;
        unsafe {
            std::ptr::copy_nonoverlapping(data as _, mapped.offset(offset as isize), size as usize);
        }
        self.allocator.borrow().unmap_memory(&self.memory)?;

        Ok(())
    }

    pub fn buffer(&self) -> vk::Buffer {
        self.buffer
    }

    /// Returns the size in bytes of the buffers
    pub fn size(&self) -> u64 {
        self.size
    }
}

impl Drop for UniformBuffer {
    fn drop(&mut self) {
        self.allocator
            .borrow()
            .destroy_buffer(self.buffer, &self.memory)
            .expect("Failed to free vulkan memory");
    }
}
