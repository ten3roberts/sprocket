use super::{Error, Result};
use super::{Sampler, Texture, UniformBuffer};
use ash::version::DeviceV1_0;
use ash::vk;
use serde::{Deserialize, Serialize};
use std::ptr;

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct DescriptorSetLayoutSpec {
    pub bindings: Vec<DescriptorSetLayoutBinding>,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct DescriptorSetLayoutBinding {
    pub slot: u32,
    pub ty: DescriptorType,
    pub count: u32,
    pub stages: Vec<ShaderStage>,
}

impl DescriptorSetLayoutBinding {
    pub fn to_vk(&self) -> vk::DescriptorSetLayoutBinding {
        vk::DescriptorSetLayoutBinding {
            binding: self.slot,
            descriptor_type: self.ty.into(),
            descriptor_count: self.count,
            stage_flags: vk::ShaderStageFlags::from_raw(
                self.stages.iter().fold(0, |acc, val| acc | ((*val) as u32)),
            ),
            p_immutable_samplers: std::ptr::null(),
        }
    }
}

#[derive(Serialize, Deserialize, Copy, Clone, Eq, PartialEq)]
/// Represents a descriptor type
/// Commented types are not yet implemented
pub enum DescriptorType {
    // Sampler= 0,
    CombinedImageSampler = 1,
    // SampledImage= 2,
    // StorageImage= 3,
    // UniformTexelBuffer= 4,
    // StorageTexelBuffer= 5,
    UniformBuffer = 6,
    // StorageBuffer= 7,
    // UniformBufferDynamic= 8,
    // StorageBufferDynamic= 9,
    // InputAttachment= 10,
}

impl From<DescriptorType> for vk::DescriptorType {
    fn from(ty: DescriptorType) -> Self {
        Self::from_raw(ty as i32)
    }
}

#[derive(Serialize, Deserialize, Copy, Clone, Eq, PartialEq)]
pub enum ShaderStage {
    Vertex = 0b1,
    TessellationControl = 0b10,
    TessellationEvaluation = 0b100,
    Geometry = 0b1000,
    Fragment = 0b1_0000,
    Compute = 0b10_0000,
    AllGraphics = 0x0000_001F,
    All = 0x7FFF_FFFF,
}

pub struct DescriptorSetLayout {
    device: ash::Device,
    layout: vk::DescriptorSetLayout,
    spec: DescriptorSetLayoutSpec,
}

impl DescriptorSetLayout {
    /// Creates a new descriptorset layout
    /// The spec is saved into the structure and can be retrieved with .spec()
    /// Useful for creating descriptor sets from it
    pub fn new(device: &ash::Device, spec: DescriptorSetLayoutSpec) -> Result<Self> {
        let bindings: Vec<_> = spec
            .bindings
            .iter()
            .map(|binding| binding.to_vk())
            .collect();
        let layout_info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(&bindings);
        let layout = unsafe { device.create_descriptor_set_layout(&layout_info, None)? };

        Ok(DescriptorSetLayout {
            device: device.clone(),
            spec,
            layout,
        })
    }

    pub fn vk(&self) -> vk::DescriptorSetLayout {
        self.layout
    }

    /// Returns the specification that was used to create the descriptor set layout
    pub fn spec(&self) -> &DescriptorSetLayoutSpec {
        &self.spec
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

    /// Updates the specified descriptors taking into account the bindings and provided data
    /// The number of supplied uniform buffers should match that of the bindings
    /// The number of supplied textures should match the bindings
    /// The number of samplers should be the same as the number of textures
    /// Sampler and textures are combined so that texture [2] uses sampler [2]
    pub fn write(
        device: &ash::Device,
        sets: &[DescriptorSet],
        spec: &DescriptorSetLayoutSpec,
        buffers: &[UniformBuffer],
        textures: &[&Texture],
        samplers: &[&Sampler],
    ) -> Result<()> {
        let bindings = &spec.bindings;
        // The number of uniform buffers specified in the bindings
        let ub_count = bindings
            .iter()
            .filter(|binding| binding.ty == DescriptorType::UniformBuffer)
            .count()
            * sets.len();

        let image_count = bindings
            .iter()
            .filter(|binding| binding.ty == DescriptorType::CombinedImageSampler)
            .count()
            * sets.len();

        if ub_count != buffers.len() {
            return Err(Error::MismatchedBinding(
                vk::DescriptorType::UNIFORM_BUFFER,
                ub_count as u32,
                buffers.len() as u32,
            ));
        }

        if image_count != textures.len() {
            return Err(Error::MismatchedBinding(
                vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                image_count as u32,
                textures.len() as u32,
            ));
        }

        if image_count != samplers.len() {
            return Err(Error::MismatchedBinding(
                vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                image_count as u32,
                samplers.len() as u32,
            ));
        }

        let mut descriptor_writes = Vec::with_capacity(bindings.len() * sets.len());
        let mut buffer_infos = Vec::with_capacity(ub_count);
        let mut image_infos = Vec::with_capacity(image_count);

        for set in sets {
            for binding in bindings {
                match binding.ty {
                    DescriptorType::UniformBuffer => {
                        let buffer = &buffers[buffer_infos.len()];
                        buffer_infos.push(vk::DescriptorBufferInfo {
                            buffer: buffer.buffer(),
                            range: buffer.size(),
                            offset: 0,
                        });

                        descriptor_writes.push(vk::WriteDescriptorSet {
                            s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
                            dst_set: set.set,
                            dst_binding: binding.slot,
                            dst_array_element: 0,
                            descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                            descriptor_count: 1,
                            p_buffer_info: &buffer_infos[buffer_infos.len() - 1],
                            p_image_info: ptr::null(),
                            p_texel_buffer_view: ptr::null(),
                            p_next: ptr::null(),
                        })
                    }
                    DescriptorType::CombinedImageSampler => {
                        let texture = &textures[image_infos.len()];
                        image_infos.push(vk::DescriptorImageInfo {
                            image_layout: texture.layout(),
                            image_view: texture.image_view(),
                            sampler: samplers[image_infos.len()].vk(),
                        });

                        descriptor_writes.push(vk::WriteDescriptorSet {
                            s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
                            dst_set: set.set,
                            dst_binding: binding.slot,
                            dst_array_element: 0,
                            descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                            descriptor_count: 1,
                            p_buffer_info: ptr::null(),
                            p_image_info: &image_infos[image_infos.len() - 1],
                            p_texel_buffer_view: ptr::null(),
                            p_next: ptr::null(),
                        })
                    }
                }
            }
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
