use super::VulkanContext;
use super::*;
use crate::graphics::vulkan;
use math::Mat4;
use std::sync::Arc;

const MAX_FRAMES_IN_FLIGHT: usize = 2;

pub struct Renderer {
    context: Arc<VulkanContext>,
    image_available_semaphores: Vec<vk::Semaphore>,
    render_finished_semaphores: Vec<vk::Semaphore>,
    in_flight_fences: Vec<vk::Fence>,
    images_in_flight: Vec<vk::Fence>,
    current_frame: usize,
    data: Data,
    frame_count: usize,
}

struct Data {
    swapchain: Swapchain,
    renderpass: RenderPass,
    commandpool: CommandPool,
    commandbuffers: Vec<CommandBuffer>,
    pipeline: Pipeline,
    framebuffers: Vec<Framebuffer>,
    model: Model,
    uniformbuffers: Vec<UniformBuffer>,
    set_layout: DescriptorSetLayout,
    descriptor_pool: DescriptorPool,
    descriptors: Vec<DescriptorSet>,
    texture: Texture,
    sampler: Sampler,
}

impl Renderer {
    pub fn new(context: Arc<VulkanContext>, window: &Window) -> Result<Renderer> {
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

        let data = Self::create_data(&context, window)?;

        for _ in 0..data.swapchain.image_count() {
            images_in_flight.push(vk::Fence::null());
        }

        Ok(Renderer {
            context,
            image_available_semaphores,
            render_finished_semaphores,
            in_flight_fences,
            images_in_flight,
            current_frame: 0,
            data,
            frame_count: 0,
        })
    }

    pub fn draw_frame(&mut self, window: &Window) {
        let device = &self.context.device;

        vulkan::wait_for_fences(device, &[self.in_flight_fences[self.current_frame]], true);

        // Update uniform buffer for this frame

        let (image_index, suboptimal) = match self
            .data
            .swapchain
            .acquire_next_image(&self.image_available_semaphores[self.current_frame])
        {
            Ok(v) => v,
            Err(Error::VulkanError(vk::Result::ERROR_OUT_OF_DATE_KHR)) => {
                self.recreate(window);
                return;
            }
            Err(e) => {
                error!("Failed to present to swapchain '{}'", e);
                return;
            }
        };

        if suboptimal {
            self.recreate(window);
            return;
        }

        let ub_data = UniformBufferObject {
            model:
            // Mat4::rotate_z(self.frame_count as f32 / 30.0),
            Mat4::rotate_y(self.frame_count as f32 / 150.0)
            * Mat4::translate(Vec3::new(0.0, 0.0, -5.0)),
            view: Mat4::identity(),
            proj: Mat4::perspective(window.aspect(), 1.0, 0.1, 10.0),
            // proj: Mat4::ortho(window.aspect() as f32 * 2.0, 2 as f32, 0.0, 10.0),
        };

        iferr!(
            "Failed to write to uniformbuffer",
            self.data.uniformbuffers[image_index as usize].write(&ub_data, None, None)
        );

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
                &[&self.data.commandbuffers[image_index as usize]],
                self.context.graphics_queue,
                &wait_semaphores,
                &wait_stages,
                &signal_semaphores,
                self.in_flight_fences[self.current_frame],
            )
        );

        // Present it to the swapchain
        let suboptimal = match self.data.swapchain.present(
            image_index,
            self.context.present_queue,
            &signal_semaphores,
        ) {
            Ok(v) => v,
            Err(Error::VulkanError(vk::Result::ERROR_OUT_OF_DATE_KHR)) => {
                self.recreate(window);
                return;
            }
            Err(e) => {
                error!("Failed to present to swapchain '{}'", e);
                return;
            }
        };

        if suboptimal {
            self.recreate(window);
            return;
        }

        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
        self.frame_count += 1;
    }

    fn recreate(&mut self, window: &Window) {
        info!("Recreating renderer");
        unsafe {
            iferr!(
                "Failed to wait for device",
                self.context.device.device_wait_idle()
            );
        }

        self.data = iferr!(
            "Failed to recreate renderer",
            Self::create_data(&self.context, window)
        );
    }

    fn create_data(context: &Arc<VulkanContext>, window: &Window) -> Result<Data> {
        let swapchain = Swapchain::new(
            &context.instance,
            &context.physical_device,
            &context.device,
            &context.allocator,
            &context.surface_loader,
            &context.surface,
            &context.queue_families,
            window.extent(),
        )?;

        let renderpass = RenderPass::new(
            &context.device,
            swapchain.format(),
            swapchain.depth_image().format(),
        )?;

        let pipeline_spec = pipeline::PipelineSpec {
            vertex_shader: "./data/shaders/default.vert.spv".into(),
            fragment_shader: "./data/shaders/default.frag.spv".into(),
            geometry_shader: "".into(),
        };

        let mut uniformbuffers = Vec::new();
        for _ in 0..swapchain.image_count() {
            uniformbuffers.push(UniformBuffer::new(
                &context.allocator,
                std::mem::size_of::<UniformBufferObject>() as u64,
            )?);
        }

        let descriptor_bindings = [
            vk::DescriptorSetLayoutBinding {
                binding: 0,
                descriptor_count: 1,
                descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                p_immutable_samplers: std::ptr::null(),
                stage_flags: vk::ShaderStageFlags::VERTEX,
            },
            vk::DescriptorSetLayoutBinding {
                binding: 1,
                descriptor_count: 1,
                descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                p_immutable_samplers: std::ptr::null(),
                stage_flags: vk::ShaderStageFlags::FRAGMENT,
            },
        ];

        let set_layout = DescriptorSetLayout::new(&context.device, &descriptor_bindings)?;

        let descriptor_pool = DescriptorPool::new(
            &context.device,
            &[
                vk::DescriptorPoolSize {
                    descriptor_count: swapchain.image_count() as u32,
                    ty: vk::DescriptorType::UNIFORM_BUFFER,
                },
                vk::DescriptorPoolSize {
                    descriptor_count: swapchain.image_count() as u32,
                    ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                },
            ],
            swapchain.image_count() as u32,
        )?;

        // Create descriptor set for mvp data
        let descriptors = DescriptorSet::new(
            &context.device,
            &descriptor_pool,
            &set_layout,
            swapchain.image_count() as u32,
        )?;

        let pipeline = Pipeline::new(
            &context.device,
            pipeline_spec,
            window.extent(),
            &renderpass,
            &[&set_layout],
        )?;

        let commandpool = CommandPool::new(
            &context.device,
            context.queue_families.graphics.unwrap(),
            false,
            false,
        )?;

        let texture = Texture::load(
            &context.allocator,
            &context.device,
            context.graphics_queue,
            &commandpool,
            "./data/textures/color_grid.png",
        )?;

        let sampler = Sampler::new(&context.device)?;

        DescriptorSet::write(
            &context.device,
            &descriptors,
            &descriptor_bindings,
            &uniformbuffers,
            &[&texture].repeat(3),
            &[&sampler].repeat(3),
        )?;

        let mut framebuffers = Vec::with_capacity(swapchain.image_count());
        for i in 0..swapchain.image_count() {
            framebuffers.push(Framebuffer::new(
                &context.device,
                &[swapchain.image(i), swapchain.depth_image()],
                &renderpass,
                swapchain.extent(),
            )?)
        }

        let mut commandbuffers =
            CommandBuffer::new_primary(&context.device, &commandpool, swapchain.image_count())?;

        let model = Model::load(
            "./data/models/suzanne.dae",
            &context.allocator,
            &context.device,
            context.graphics_queue,
            &commandpool,
        )?;
        let mesh = model.get_mesh_index(0).unwrap();
        // let mesh = Mesh::new(
        //     &context.allocator,
        //     &context.device,
        //     context.graphics_queue,
        //     &commandpool,
        //     &vertices,
        //     &indices,
        // )?;

        // info!(
        //     "Mesh: v count: {},i count: {}",
        //     mesh.vertex_count(),
        //     mesh.index_count()
        // );

        // Prerecord commandbuffers
        for (i, commandbuffer) in commandbuffers.iter_mut().enumerate() {
            commandbuffer.begin(Default::default())?;
            commandbuffer.begin_renderpass(
                &renderpass,
                &framebuffers[i],
                math::Vec4::new(0.0, 0.0, 0.01, 1.0),
            );
            commandbuffer.bind_pipeline(&pipeline);
            commandbuffer.bind_descriptorsets(&pipeline, &[&descriptors[i]]);
            commandbuffer.bind_mesh(mesh);
            commandbuffer.draw_indexed(mesh.index_count());
            commandbuffer.end_renderpass();
            commandbuffer.end()?;
        }

        Ok(Data {
            swapchain,
            renderpass,
            commandpool,
            commandbuffers,
            pipeline,
            framebuffers,
            model,
            uniformbuffers,
            set_layout,
            descriptor_pool,
            descriptors,
            texture,
            sampler,
        })
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
