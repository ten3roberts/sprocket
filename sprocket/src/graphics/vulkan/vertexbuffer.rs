use super::buffer;
use super::CommandPool;
use crate::math::*;
use ash::vk;
use std::sync::Arc;

use super::{Result, VkAllocator};

pub struct Vertex {
    position: Vec2,
    texcoord: Vec2,
}

impl Vertex {
    pub fn new(position: Vec2, texcoord: Vec2) -> Vertex {
        Vertex { position, texcoord }
    }

    pub fn binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(std::mem::size_of::<Vertex>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
            .build()
    }

    pub fn attribute_descriptions() -> Vec<vk::VertexInputAttributeDescription> {
        vec![
            // Position
            vk::VertexInputAttributeDescription::builder()
                .binding(0)
                .location(0)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(offsetof!(Vertex, position) as u32)
                .build(),
            // Color
            vk::VertexInputAttributeDescription::builder()
                .binding(0)
                .location(1)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(offsetof!(Vertex, texcoord) as u32)
                .build(),
        ]
    }
}

pub struct VertexBuffer {
    allocator: VkAllocator,
    buffer: vk::Buffer,
    memory: vk_mem::Allocation,
    size: vk::DeviceSize,
    count: u32,
}

const DEFAULT_SIZE: u64 = 1024;

impl VertexBuffer {
    /// Creates and allocated memory for a vertex buffer
    /// The buffer is filled with the supplied vertices
    /// If an empty list of vertices is supplied, DEFAULT_SIZE bytes is allocated
    pub fn new(
        allocator: &VkAllocator,
        device: &ash::Device,
        queue: vk::Queue,
        commandpool: &CommandPool,
        vertices: &[Vertex],
    ) -> Result<VertexBuffer> {
        let buffer_size = match vertices.len() {
            0 => 1024,
            n => (n * std::mem::size_of_val(&vertices[0])) as u64,
        };

        let (staging_buffer, staging_memory, _) = buffer::create_staging(allocator, buffer_size)?;

        // Copy the data into the buffer
        let data = allocator.borrow().map_memory(&staging_memory)?;
        unsafe {
            std::ptr::copy_nonoverlapping(vertices.as_ptr() as _, data, buffer_size as usize);
        }
        allocator.borrow().unmap_memory(&staging_memory)?;

        let (buffer, memory, _) = allocator.borrow().create_buffer(
            &vk::BufferCreateInfo::builder()
                .size(buffer_size)
                .usage(vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER)
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

        Ok(VertexBuffer {
            allocator: Arc::clone(allocator),
            buffer,
            memory,
            size: buffer_size,
            count: vertices.len() as u32,
        })
    }

    pub fn buffer(&self) -> vk::Buffer {
        self.buffer
    }

    /// Returns the number of vertices in the buffer
    pub fn count(&self) -> u32 {
        self.count
    }
}

impl Drop for VertexBuffer {
    fn drop(&mut self) {
        self.allocator
            .borrow()
            .destroy_buffer(self.buffer, &self.memory)
            .expect("Failed to free vulkan memory");
    }
}
