use super::texture::Texture;
use super::RenderPass;
use crate::graphics::Extent2D;
use ash::version::DeviceV1_0;
use ash::vk;
use std::borrow::Cow;

pub struct Framebuffer {
    device: ash::Device,
    framebuffer: vk::Framebuffer,
    extent: Extent2D,
}

impl Framebuffer {
    pub fn new(
        device: &ash::Device,
        attachments: &[&Texture],
        renderpass: &RenderPass,
        extent: Extent2D,
    ) -> Result<Framebuffer, Cow<'static, str>> {
        let attachment_views: Vec<vk::ImageView> = attachments
            .iter()
            .map(|attachment| attachment.image_view())
            .collect();

        let framebuffer_info = vk::FramebufferCreateInfo::builder()
            .render_pass(renderpass.vk())
            .attachments(&attachment_views)
            .width(extent.width)
            .height(extent.height)
            .layers(1)
            .build();

        // let framebuffer = unsafe { device.create_framebuffer(&framebuffer_info, None).unwrap() };

        // let framebuffer = unwrap_or_return!("Failed to create frambuffer", unsafe {
        //     device.create_framebuffer(&framebuffer_info, None).unwrap()
        // });

        let framebuffer = unwrap_or_return!("Failed to create renderpass", unsafe {
            device.create_framebuffer(&framebuffer_info, None)
        });

        Ok(Framebuffer {
            device: device.clone(),
            framebuffer,
            extent,
        })
    }

    pub fn vk(&self) -> vk::Framebuffer {
        self.framebuffer
    }

    pub fn extent(&self) -> Extent2D {
        self.extent
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_framebuffer(self.framebuffer, None);
        }
    }
}
