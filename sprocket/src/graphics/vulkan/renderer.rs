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

    swapchain: Swapchain,
    renderpass: RenderPass,
    commandpool: CommandPool,
    commandbuffers: Vec<CommandBuffer>,
    pipeline: Pipeline,
    framebuffers: Vec<Framebuffer>,
}

impl Renderer {
    pub fn new(
        context: Arc<VulkanContext>,
        window: &Window,
    ) -> Result<Renderer, Cow<'static, str>> {
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

        let swapchain = unwrap_or_return!(
            "Failed to create swapchain",
            Swapchain::new(
                &context.instance,
                &context.physical_device,
                &context.device,
                &context.surface_loader,
                &context.surface,
                &context.queue_families,
                window.extent()
            )
        );

        for _ in 0..swapchain.image_count() {
            images_in_flight.push(vk::Fence::null());
        }

        let renderpass = RenderPass::new(&context.device, swapchain.format())?;

        let pipeline_spec = pipeline::PipelineSpec {
            vertex_shader: "./data/shaders/default.vert.spv".into(),
            fragment_shader: "./data/shaders/default.frag.spv".into(),
            geometry_shader: "".into(),
        };

        let pipeline = Pipeline::new(&context.device, pipeline_spec, window.extent(), &renderpass)?;

        let commandpool = CommandPool::new(
            &context.device,
            context.queue_families.graphics.unwrap(),
            false,
            false,
        )?;

        let mut framebuffers = Vec::with_capacity(swapchain.image_count());
        for i in 0..swapchain.image_count() {
            framebuffers.push(Framebuffer::new(
                &context.device,
                &[swapchain.image(i)],
                &renderpass,
                swapchain.extent(),
            )?)
        }

        let mut commandbuffers =
            CommandBuffer::new_primary(&context.device, &commandpool, swapchain.image_count())?;

        // Prerecord commandbuffers
        for (i, commandbuffer) in commandbuffers.iter_mut().enumerate() {
            commandbuffer.begin()?;
            commandbuffer.begin_renderpass(
                &renderpass,
                &framebuffers[i],
                math::Vec4::new(0.0, 0.0, 0.01, 1.0),
            );
            commandbuffer.bind_pipeline(&pipeline);
            commandbuffer.draw();
            commandbuffer.end_renderpass();
            commandbuffer.end()?;
        }

        Ok(Renderer {
            context,
            image_available_semaphores,
            render_finished_semaphores,
            in_flight_fences,
            images_in_flight,
            current_frame: 0,
            pipeline,
            swapchain,
            commandbuffers,
            commandpool,
            renderpass,
            framebuffers,
        })
    }

    pub fn draw_frame(&mut self) {
        let device = &self.context.device;

        vulkan::wait_for_fences(device, &[self.in_flight_fences[self.current_frame]], true);

        let (image_index, _) = self
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
                &[&self.commandbuffers[image_index as usize]],
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
            self.swapchain
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
