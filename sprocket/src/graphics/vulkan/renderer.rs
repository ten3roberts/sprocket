use super::VulkanContext;
use super::*;
use crate::graphics::vulkan;
use std::borrow::Cow;
use std::sync::Arc;

pub struct Renderer {
    context: Arc<VulkanContext>,
    image_available: vk::Semaphore,
    render_finished: vk::Semaphore,
}

impl Renderer {
    pub fn new(context: Arc<VulkanContext>) -> Result<Renderer, Cow<'static, str>> {
        // Create the semaphores
        let image_available = vulkan::create_semaphore(&context.device)?;
        let render_finished = vulkan::create_semaphore(&context.device)?;
        Ok(Renderer {
            context,
            image_available,
            render_finished,
        })
    }

    pub fn draw_frame(&mut self) {
        unsafe { self.context.device.device_wait_idle() };
        let data = self.context.data.as_ref().unwrap();

        let (image_index, _) = data.swapchain.acquire_next_image(&self.image_available);

        info!("Acquired swapchain image {}", image_index);
        // Submit the primary command buffer
        if let Err(e) = commandbuffer::CommandBuffer::submit(
            &self.context.device,
            &[&data.commandbuffers[image_index as usize]],
            &self.context.graphics_queue,
            &[self.image_available],
            &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT],
            &[self.render_finished],
        ) {
            error!("Failed to submit command buffers for rendering, '{}'", e);
            return;
        }

        // Present it to the swapchain
        let _suboptimal = match data.swapchain.present(
            image_index,
            self.context.present_queue,
            self.render_finished,
        ) {
            Ok(v) => v,
            Err(e) => {
                error!("{}", e);
                return;
            }
        };
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            self.context.device.device_wait_idle();
            self.context
                .device
                .destroy_semaphore(self.image_available, None);
            self.context
                .device
                .destroy_semaphore(self.render_finished, None);
        }
    }
}
