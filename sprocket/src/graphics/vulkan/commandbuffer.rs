use super::{Framebuffer, IndexBuffer, Pipeline, RenderPass, VertexBuffer};
use ash::version::DeviceV1_0;
use ash::vk;

use super::{Error, Result};

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
    ) -> Result<CommandPool> {
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
        let pool = unsafe { device.create_command_pool(&pool_info, None)? };

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
    ) -> Result<Vec<CommandBuffer>> {
        let alloc_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(commandpool.pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(count as u32)
            .build();
        let commandbuffers = unsafe { device.allocate_command_buffers(&alloc_info)? };

        Ok(commandbuffers
            .into_iter()
            .map(|commandbuffer| CommandBuffer {
                device: device.clone(),
                commandbuffer,
                recording: false,
            })
            .collect())
    }

    pub fn begin(&mut self, begin_info: vk::CommandBufferUsageFlags) -> Result<()> {
        let begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(begin_info)
            .build();
        unsafe {
            self.device
                .begin_command_buffer(self.commandbuffer, &begin_info)?
        };

        self.recording = true;
        Ok(())
    }

    pub fn end(&mut self) -> Result<()> {
        if !self.recording {
            return Err(Error::NotRecording);
        }

        unsafe { self.device.end_command_buffer(self.commandbuffer)? };

        self.recording = false;

        Ok(())
    }

    pub fn submit(
        device: &ash::Device,
        commandbuffers: &[&CommandBuffer],
        queue: vk::Queue,
        wait_semaphores: &[vk::Semaphore],
        wait_stages: &[vk::PipelineStageFlags],
        signal_semaphores: &[vk::Semaphore],
        fence: vk::Fence,
    ) -> Result<()> {
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

        unsafe {
            device
                .queue_submit(queue, &[submit_info], fence)
                .map_err(|e| e.into())
        }
    }

    pub fn begin_renderpass(
        &mut self,
        renderpass: &RenderPass,
        framebuffer: &Framebuffer,
        clear_color: crate::math::Vec4,
    ) {
        let clear_values = [vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [clear_color.x, clear_color.y, clear_color.z, clear_color.w],
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

    pub fn bind_vertexbuffer(&self, vertexbuffer: &VertexBuffer) {
        unsafe {
            self.device.cmd_bind_vertex_buffers(
                self.commandbuffer,
                0,
                &[vertexbuffer.buffer()],
                &[0],
            )
        }
    }

    pub fn bind_indexbuffer(&self, indexbuffer: &IndexBuffer) {
        unsafe {
            self.device.cmd_bind_index_buffer(
                self.commandbuffer,
                indexbuffer.buffer(),
                0,
                indexbuffer.index_type(),
            )
        }
    }

    pub fn draw(&self) {
        unsafe {
            self.device.cmd_draw(self.commandbuffer, 3, 1, 0, 0);
        };
    }

    pub fn draw_indexed(&self, index_count: u32) {
        unsafe {
            self.device
                .cmd_draw_indexed(self.commandbuffer, index_count, 1, 0, 0, 0)
        }
    }

    pub fn vk(&self) -> vk::CommandBuffer {
        self.commandbuffer
    }
}
