use super::VulkanContext;
use super::*;
use crate::graphics::vulkan;
use std::borrow::Cow;
use std::sync::Arc;

const MAX_FRAMES_IN_FLIGHT: usize = 2;

pub struct Renderer {
    context: Arc<VulkanContext>,
    image_available_semaphores: Vec<vk::Semaphore>,
    render_finished_semaphores: Vec<vk::Semaphore>,
    in_flight_fences: Vec<vk::Fence>,
    images_in_flight: Vec<vk::Fence>,
    current_frame: usize,
}

impl Renderer {
    pub fn new(context: Arc<VulkanContext>) -> Result<Renderer, Cow<'static, str>> {
        let mut image_available_semaphores = Vec::new();
        let mut render_finished_semaphores = Vec::new();
        let mut in_flight_fences = Vec::new();
        let mut images_in_flight = Vec::new();

        // Create the semaphores
        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            image_available_semaphores.push(vulkan::create_semaphore(&context.device)?);
            render_finished_semaphores.push(vulkan::create_semaphore(&context.device)?);
            in_flight_fences.push(vulkan::create_fence(&context.device)?);
        }

        for _ in 0..context.data.as_ref().unwrap().swapchain.image_count() {
            images_in_flight.push(vk::Fence::null());
        }

        Ok(Renderer {
            context,
            image_available_semaphores,
            render_finished_semaphores,
            in_flight_fences,
            images_in_flight,
            current_frame: 0,
        })
    }

    pub fn draw_frame(&mut self) {
        let data = self.context.data.as_ref().unwrap();
        let device = &self.context.device;

        vulkan::wait_for_fences(device, &[self.in_flight_fences[self.current_frame]], true);

        let (image_index, _) = data
            .swapchain
            .acquire_next_image(&self.image_available_semaphores[self.current_frame]);

        // Check if a previous frame is using this image (i.e. there is its fence to wait on)
        if self.images_in_flight[image_index as usize] != vk::Fence::null() {
            vulkan::wait_for_fences(device, &[self.images_in_flight[image_index as usize]], true)
        }

        self.images_in_flight[image_index as usize] = self.in_flight_fences[self.current_frame];

        // Submit the primary command buffer
        let wait_semaphores = [self.image_available_semaphores[self.current_frame]];
        let signal_semaphores = [self.render_finished_semaphores[self.current_frame]];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];

        vulkan::reset_fences(device, &[self.in_flight_fences[self.current_frame]]);

        iferr!(
            "Failed to submit command buffers for rendering",
            commandbuffer::CommandBuffer::submit(
                device,
                &[&data.commandbuffers[image_index as usize]],
                &self.context.graphics_queue,
                &wait_semaphores,
                &wait_stages,
                &signal_semaphores,
                self.in_flight_fences[self.current_frame],
            )
        );

        // Present it to the swapchain
        let _suboptimal = iferr!(
            "Failed to present to swapchain",
            data.swapchain
                .present(image_index, self.context.present_queue, &signal_semaphores,)
        );

        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            iferr!(
                "Failed to wait on device",
                self.context.device.device_wait_idle()
            );
            for semaphore in &self.image_available_semaphores {
                self.context.device.destroy_semaphore(*semaphore, None);
            }
            for semaphore in &self.render_finished_semaphores {
                self.context.device.destroy_semaphore(*semaphore, None);
            }
            for fence in &self.in_flight_fences {
                self.context.device.destroy_fence(*fence, None);
            }
        }
    }
}
