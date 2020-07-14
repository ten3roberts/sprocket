use super::buffer;
use crate::math::*;
use ash::version::{DeviceV1_0, InstanceV1_0};
use ash::vk;
use std::borrow::Cow;

pub struct Vertex {
    position: Vec2,
    color: Vec3,
}

impl Vertex {
    pub fn new(position: Vec2, color: Vec3) -> Vertex {
        Vertex { position, color }
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
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(offsetof!(Vertex, color) as u32)
                .build(),
        ]
    }
}

pub struct VertexBuffer {
    device: ash::Device,
    buffer: vk::Buffer,
    memory: vk::DeviceMemory,
    size: vk::DeviceSize,
}

const DEFAULT_SIZE: u64 = 1024;

impl VertexBuffer {
    /// Creates and allocated memory for a vertex buffer
    /// The buffer is filled with the supplied vertices
    /// If an empty list of vertices is supplied, DEFAULT_SIZE bytes is allocated
    pub fn new(
        instance: &ash::Instance,
        device: &ash::Device,
        physical_device: vk::PhysicalDevice,
        vertices: &[Vertex],
    ) -> Result<VertexBuffer, Cow<'static, str>> {
        let buffer_size = match vertices.len() {
            0 => 1024,
            n => (n * std::mem::size_of_val(&vertices[0])) as u64,
        };

        let (buffer, memory) = unwrap_or_return!(
            "Failed to create buffer or memory",
            buffer::create(
                instance,
                device,
                physical_device,
                buffer_size,
                vk::BufferUsageFlags::VERTEX_BUFFER,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT
            )
        );

        // Copy the data into the buffer
        let data = unwrap_or_return!("Failed to map vertex buffer memory", unsafe {
            device.map_memory(memory, 0, buffer_size, vk::MemoryMapFlags::default())
        });

        unsafe {
            std::ptr::copy_nonoverlapping(
                vertices.as_ptr() as *const std::ffi::c_void,
                data,
                buffer_size as usize,
            )
        };

        unsafe { device.unmap_memory(memory) }

        Ok(VertexBuffer {
            device: device.clone(),
            buffer,
            memory,
            size: buffer_size,
        })
    }

    pub fn buffer(&self) -> vk::Buffer {
        self.buffer
    }
}

impl Drop for VertexBuffer {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_buffer(self.buffer, None);
            self.device.free_memory(self.memory, None);
        }
    }
}
