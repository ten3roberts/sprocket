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

        let buffer_info = vk::BufferCreateInfo::builder()
            .size(buffer_size)
            .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let buffer = unwrap_or_return!("Failed to create vertex buffer", unsafe {
            device.create_buffer(&buffer_info, None)
        });

        let memory_requirements = unsafe { device.get_buffer_memory_requirements(buffer) };
        let memory_type_index = match find_memory_type(
            instance,
            physical_device,
            memory_requirements.memory_type_bits,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        ) {
            Some(v) => v,
            None => return Err("Failed to find suitable memory type for vertex buffer".into()),
        };

        let alloc_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(memory_requirements.size)
            .memory_type_index(memory_type_index);

        let memory = unwrap_or_return!("Failed to allocate vertex buffer memory", unsafe {
            device.allocate_memory(&alloc_info, None)
        });

        unwrap_or_return!("Failed to bind vertex buffer memory", unsafe {
            device.bind_buffer_memory(buffer, memory, 0)
        });

        // Copy the data into the buffer
        let data = unwrap_or_return!("Failed to map vertex buffer memory", unsafe {
            device.map_memory(memory, 0, buffer_info.size, vk::MemoryMapFlags::default())
        });

        unsafe {
            std::ptr::copy_nonoverlapping(
                vertices.as_ptr() as *const std::ffi::c_void,
                data,
                buffer_info.size as usize,
            )
        };

        unsafe { device.unmap_memory(memory) }

        Ok(VertexBuffer {
            device: device.clone(),
            buffer,
            memory,
            size: buffer_info.size,
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
