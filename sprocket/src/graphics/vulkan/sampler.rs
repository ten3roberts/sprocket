use ash::version::DeviceV1_0;
use ash::vk;

use super::Result;

pub struct Sampler {
    device: ash::Device,
    sampler: vk::Sampler,
}

impl Sampler {
    pub fn new(device: &ash::Device) -> Result<Sampler> {
        let sampler_info = vk::SamplerCreateInfo {
            s_type: vk::StructureType::SAMPLER_CREATE_INFO,
            mag_filter: vk::Filter::LINEAR,
            min_filter: vk::Filter::LINEAR,
            address_mode_u: vk::SamplerAddressMode::REPEAT,
            address_mode_v: vk::SamplerAddressMode::REPEAT,
            address_mode_w: vk::SamplerAddressMode::REPEAT,
            anisotropy_enable: vk::TRUE,
            max_anisotropy: 16.0,
            border_color: vk::BorderColor::INT_OPAQUE_BLACK,
            unnormalized_coordinates: vk::FALSE,
            compare_enable: vk::FALSE,
            compare_op: vk::CompareOp::ALWAYS,
            mipmap_mode: vk::SamplerMipmapMode::LINEAR,
            mip_lod_bias: 0.0,
            min_lod: 0.0,
            max_lod: 0.0,
            flags: Default::default(),
            p_next: std::ptr::null(),
        };

        let sampler = unsafe { device.create_sampler(&sampler_info, None)? };

        Ok(Sampler {
            device: device.clone(),
            sampler,
        })
    }

    pub fn vk(&self) -> vk::Sampler {
        self.sampler
    }
}

impl Drop for Sampler {
    fn drop(&mut self) {
        unsafe { self.device.destroy_sampler(self.sampler, None) }
    }
}
