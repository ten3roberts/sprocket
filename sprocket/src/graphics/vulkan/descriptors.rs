use super::UniformBuffer;
use super::{Error, Result};
use ash::version::DeviceV1_0;
use ash::vk;
use std::ptr;

use log::debug;

pub struct DescriptorSetLayout {
    device: ash::Device,
    layout: vk::DescriptorSetLayout,
}

impl DescriptorSetLayout {
    pub fn new(device: &ash::Device, bindings: &[vk::DescriptorSetLayoutBinding]) -> Result<Self> {
        let layout_info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(bindings);
        let layout = unsafe { device.create_descriptor_set_layout(&layout_info, None)? };

        Ok(DescriptorSetLayout {
            device: device.clone(),
            layout,
        })
    }

    pub fn vk(&self) -> vk::DescriptorSetLayout {
        self.layout
    }
}

impl Drop for DescriptorSetLayout {
    fn drop(&mut self) {
        unsafe { self.device.destroy_descriptor_set_layout(self.layout, None) };
    }
}

pub struct DescriptorPool {
    device: ash::Device,
    pool: vk::DescriptorPool,
    sizes: Vec<vk::DescriptorPoolSize>,
}

impl DescriptorPool {
    pub fn new(
        device: &ash::Device,
        sizes: &[vk::DescriptorPoolSize],
        max_sets: u32,
    ) -> Result<DescriptorPool> {
        let pool_info = vk::DescriptorPoolCreateInfo::builder()
            .pool_sizes(sizes)
            .max_sets(max_sets);

        let pool = unsafe { device.create_descriptor_pool(&pool_info, None)? };
        Ok(DescriptorPool {
            device: device.clone(),
            pool,
            sizes: sizes.into(),
        })
    }
}

impl Drop for DescriptorPool {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_descriptor_pool(self.pool, None);
        }
    }
}

pub struct DescriptorSet {
    set: vk::DescriptorSet,
}

impl DescriptorSet {
    /// Allocated one or more descriptor sets
    pub fn new(
        device: &ash::Device,
        pool: &DescriptorPool,
        layout: &DescriptorSetLayout,
        count: u32,
    ) -> Result<Vec<DescriptorSet>> {
        let layouts: Vec<vk::DescriptorSetLayout> = (0..count).map(|_| layout.layout).collect();

        let alloc_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(pool.pool)
            .set_layouts(&layouts);

        let sets = unsafe { device.allocate_descriptor_sets(&alloc_info)? };
        Ok(sets.into_iter().map(|set| DescriptorSet { set }).collect())
    }

    pub fn write(
        device: &ash::Device,
        sets: &[DescriptorSet],
        bindings: &[vk::DescriptorSetLayoutBinding],
        buffers: &[UniformBuffer],
    ) -> Result<()> {
        // The number of uniform buffers specified in the bindings
        let ub_count = bindings
            .iter()
            .filter(|binding| binding.descriptor_type == vk::DescriptorType::UNIFORM_BUFFER)
            .count()
            * sets.len();

        if ub_count != buffers.len() {
            return Err(Error::MismatchedBinding(
                vk::DescriptorType::UNIFORM_BUFFER,
                ub_count as u32,
                buffers.len() as u32,
            ));
        }

        let mut descriptor_writes = Vec::with_capacity(bindings.len() * sets.len());
        let mut buffer_infos = Vec::with_capacity(ub_count);

        for set in sets {
            for binding in bindings {
                match binding.descriptor_type {
                    vk::DescriptorType::UNIFORM_BUFFER => {
                        let buffer = &buffers[buffer_infos.len()];
                        buffer_infos.push(vk::DescriptorBufferInfo {
                            buffer: buffer.buffer(),
                            range: buffer.size(),
                            offset: 0,
                        });

                        descriptor_writes.push(vk::WriteDescriptorSet {
                            s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
                            dst_set: set.set,
                            dst_binding: binding.binding,
                            dst_array_element: 0,
                            descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                            descriptor_count: 1,
                            p_buffer_info: &buffer_infos[buffer_infos.len() - 1],
                            p_image_info: ptr::null(),
                            p_texel_buffer_view: ptr::null(),
                            p_next: ptr::null(),
                        })
                    }
                    ty => return Err(Error::UnsupportedDescriptorType(ty)),
                }
                debug!("Binding");
            }
            debug!("Set");
        }
        unsafe { device.update_descriptor_sets(&descriptor_writes, &[]) };
        Ok(())
    }

    pub fn vk(&self) -> vk::DescriptorSet {
        self.set
    }
}

// DescriptorSets are freed when the pool is freed
// impl Drop for DescriptorSet
