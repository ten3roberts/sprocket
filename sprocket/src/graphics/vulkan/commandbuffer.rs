use super::Framebuffer;
use super::Pipeline;
use super::RenderPass;
use ash::version::DeviceV1_0;
use ash::vk;
use std::borrow::Cow;

pub struct CommandPool {
    device: ash::Device,
    pool: vk::CommandPool,
    transient: bool,
    partial_reset: bool,
}

/// Represents a commandbuffer that stores graphics command to later be executed
impl CommandPool {
    /// Creates a command pool for recording command buffers
    /// The transient options specifies that the command buffer
    pub fn new(
        device: &ash::Device,
        queue_family: u32,
        transient: bool,
        partial_reset: bool,
    ) -> Result<CommandPool, Cow<'static, str>> {
        let mut flags = vk::CommandPoolCreateFlags::default();
        if transient {
            flags |= vk::CommandPoolCreateFlags::TRANSIENT;
        }

        if partial_reset {
            flags |= vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER;
        }

        let pool_info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(queue_family)
            .flags(flags)
            .build();
        let pool = unwrap_or_return!("Failed to create command pool", unsafe {
            device.create_command_pool(&pool_info, None)
        });

        Ok(CommandPool {
            device: device.clone(),
            pool,
            transient,
            partial_reset,
        })
    }
}

impl Drop for CommandPool {
    fn drop(&mut self) {
        unsafe { self.device.destroy_command_pool(self.pool, None) }
    }
}

pub struct CommandBuffer {
    device: ash::Device,
    commandbuffer: vk::CommandBuffer,
    recording: bool,
}

impl CommandBuffer {
    pub fn new_primary(
        device: &ash::Device,
        commandpool: &CommandPool,
        count: usize,
    ) -> Result<Vec<CommandBuffer>, Cow<'static, str>> {
        let alloc_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(commandpool.pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(count as u32)
            .build();
        let commandbuffers = unwrap_or_return!("Failed to create commandbuffer", unsafe {
            device.allocate_command_buffers(&alloc_info)
        });

        if commandbuffers.len() != count {
            return errfmt!(
                "Commandbuffer count does not match requested count. Requested {}, acquired {}",
                count,
                commandbuffers.len()
            );
        }

        Ok(commandbuffers
            .into_iter()
            .map(|commandbuffer| CommandBuffer {
                device: device.clone(),
                commandbuffer,
                recording: false,
            })
            .collect())
    }

    pub fn begin(&mut self) -> Result<(), Cow<'static, str>> {
        let begin_info = vk::CommandBufferBeginInfo::builder().build();
        unwrap_or_return!("Failed to begin recording of command buffer", unsafe {
            self.device
                .begin_command_buffer(self.commandbuffer, &begin_info)
        });

        self.recording = true;
        Ok(())
    }

    pub fn end(&mut self) -> Result<(), Cow<'static, str>> {
        if !self.recording {
            return Err("Cannot end commandbuffer that's not recording".into());
        }

        unwrap_or_return!("Failed to end recording of command buffer", unsafe {
            self.device.end_command_buffer(self.commandbuffer)
        });

        self.recording = false;

        Ok(())
    }

    pub fn submit(
        device: &ash::Device,
        commandbuffers: &[&CommandBuffer],
        queue: &vk::Queue,
        wait_semaphores: &[vk::Semaphore],
        wait_stages: &[vk::PipelineStageFlags],
        signal_semaphores: &[vk::Semaphore],
    ) -> Result<(), Cow<'static, str>> {
        let commandbuffers: Vec<vk::CommandBuffer> = commandbuffers
            .iter()
            .map(|commandbuffer| commandbuffer.commandbuffer)
            .collect();
        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(wait_semaphores)
            .wait_dst_stage_mask(wait_stages)
            .command_buffers(&commandbuffers)
            .signal_semaphores(signal_semaphores)
            .build();

        unwrap_and_return!("Failed to submit command buffers", unsafe {
            device.queue_submit(*queue, &[submit_info], vk::Fence::null())
        })
    }

    pub fn begin_renderpass(&mut self, renderpass: &RenderPass, framebuffer: &Framebuffer) {
        let clear_values = [vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.5, 0.5, 0.5, 1.0],
            },
        }];

        let renderpass_info = vk::RenderPassBeginInfo::builder()
            .render_pass(renderpass.vk())
            .framebuffer(framebuffer.vk())
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: framebuffer.extent().into(),
            })
            .clear_values(&clear_values)
            .build();

        unsafe {
            self.device.cmd_begin_render_pass(
                self.commandbuffer,
                &renderpass_info,
                vk::SubpassContents::INLINE,
            );
        };
    }

    pub fn end_renderpass(&self) {
        unsafe { self.device.cmd_end_render_pass(self.commandbuffer) };
    }

    pub fn bind_pipeline(&self, pipeline: &Pipeline) {
        unsafe {
            self.device.cmd_bind_pipeline(
                self.commandbuffer,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline.vk(),
            )
        };
    }

    pub fn draw(&self) {
        unsafe {
            self.device.cmd_draw(self.commandbuffer, 3, 1, 0, 0);
        };
    }
}
