use super::{
    DescriptorSet, Framebuffer, IndexBuffer, Material, Mesh, Pipeline, RenderPass, VertexBuffer,
};

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
        let clear_values = [
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [clear_color.x, clear_color.y, clear_color.z, clear_color.w],
                },
            },
            vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: 1.0,
                    stencil: 0,
                },
            },
        ];

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

    /// Binds a vertex buffer separately
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

    /// Binds an index buffer separately
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

    /// Binds a mesh containing a vertex buffer and index buffer
    /// Does the equivalent of binding the mesh's vertex and index buffer
    pub fn bind_mesh(&self, mesh: &Mesh) {
        self.bind_vertexbuffer(mesh.vertexbuffer());
        self.bind_indexbuffer(mesh.indexbuffer());
    }

    /// Binds a material and the relevant descriptor sets
    /// Since binding a material most likely will change the pipeline, the global set (set=0) mut
    /// be provided and bound again
    /// Parameter image_index tells which descriptor set in the material to use since there is one
    /// for each swapchain image
    pub fn bind_material(&self, material: &Material, global_set: &DescriptorSet, image_index: u32) {
        self.bind_pipeline(material.pipeline());
        self.bind_descriptorsets(
            &material.pipeline(),
            &[
                global_set,
                &material.descriptor_sets()[image_index as usize],
            ],
        )
    }

    /// Binds one or more descriptor sets
    pub fn bind_descriptorsets(&self, pipeline: &Pipeline, descriptor_sets: &[&DescriptorSet]) {
        unsafe {
            let sets: Vec<vk::DescriptorSet> = descriptor_sets.iter().map(|set| set.vk()).collect();
            self.device.cmd_bind_descriptor_sets(
                self.commandbuffer,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline.layout(),
                0,
                &sets,
                &[],
            )
        }
    }

    /// Sets oush constants to the shaders
    pub fn push_contants<T>(
        &self,
        pipeline_layout: vk::PipelineLayout,
        stages: vk::ShaderStageFlags,
        offset: u32,
        constants: &T,
    ) {
        let data: *const T = constants;
        unsafe {
            self.device.cmd_push_constants(
                self.commandbuffer,
                pipeline_layout,
                stages,
                offset,
                std::slice::from_raw_parts(data as *const u8, std::mem::size_of::<T>()),
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

    /// Resets/Clears the commandbuffer allowing you to once again record commands
    // Normal comment
    pub fn reset(&self) -> Result<()> {
        Ok(unsafe { self.device.reset_command_buffer(self.commandbuffer, Default::default()) }?)
    }

    pub fn vk(&self) -> vk::CommandBuffer {
        self.commandbuffer
    }
}
