use super::VulkanContext;
use super::*;
use crate::graphics::vulkan;
use ecs::{ComponentArray, Entity};
use math::Mat4;
use physics::Transform;
use std::sync::Arc;

const MAX_FRAMES_IN_FLIGHT: usize = 2;

struct EntityData {
    mvp: Mat4,
}

pub struct Renderer {
    context: Arc<VulkanContext>,
    resourcemanager: Arc<ResourceManager>,
    image_available_semaphores: Vec<vk::Semaphore>,
    render_finished_semaphores: Vec<vk::Semaphore>,
    in_flight_fences: Vec<vk::Fence>,
    images_in_flight: Vec<vk::Fence>,
    current_frame: usize,
    data: Data,
    frame_count: usize,
    entities: ComponentArray<Transform>,
}

struct Data {
    swapchain: Arc<Swapchain>,
    commandpool: CommandPool,
    commandbuffers: Vec<CommandBuffer>,
    framebuffers: Vec<Framebuffer>,
    material: Arc<Material>,
    model: Arc<Model>,
    uniformbuffers: Vec<UniformBuffer>,
    descriptor_pool: DescriptorPool,
    global_descriptors: Vec<DescriptorSet>,
    renderpass: Arc<RenderPass>,
}

impl Renderer {
    pub fn insert_entity(&mut self, entity: Entity, transform: Transform) {
        self.entities.insert_component(entity, transform);
    }

    pub fn new(
        context: Arc<VulkanContext>,
        window: &Window,
        resourcemanager: Arc<ResourceManager>,
    ) -> Result<Renderer> {
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

        let data = Self::create_data(&context, window, &resourcemanager)?;

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
            resourcemanager,
            entities: ComponentArray::new(),
        })
    }

    pub fn draw_frame(&mut self, window: &Window, _time: &Time) {
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

        // Reset and record command buffers
        let commandbuffer = &mut self.data.commandbuffers[image_index as usize];

        iferr!("Failed to reset commandbuffers", commandbuffer.reset());
        iferr!(
            "Failed to begin recording command buffer",
            commandbuffer.begin(Default::default())
        );

        commandbuffer.begin_renderpass(
            &self.data.renderpass,
            &self.data.framebuffers[image_index as usize],
            math::Vec4::new(0.0, 0.0, 0.01, 1.0),
        );
        // TODO MaterialComponent and MeshComponent
        let material = &self.data.material;
        let mesh = self.data.model.get_mesh_index(0).unwrap();

        // Iterate all entities
        for transform in &mut self.entities.into_iter() {
            commandbuffer.bind_material(
                &material,
                &self.data.global_descriptors[image_index as usize],
                image_index,
            );
            let model = Mat4::translate(transform.position);
            let view = Mat4::translate(Vec3::new(0.0, 0.0, -5.0)); // Camera
            let proj = Mat4::perspective(window.aspect(), 1.0, 0.1, 10.0); // Camera

            // proj: Mat4::ortho(window.aspect() as f32 * 2.0, 2 as f32, 0.0, 10.0),
            let entity_data = EntityData {
                mvp: model * view * proj,
            };

            commandbuffer.push_contants(
                material.pipeline().layout(),
                vk::ShaderStageFlags::VERTEX,
                0,
                &entity_data,
            );

            commandbuffer.bind_mesh(mesh);
            commandbuffer.draw_indexed(mesh.index_count());
        }

        commandbuffer.end_renderpass();

        iferr!(
            "Failed to begin recording command buffer",
            commandbuffer.end()
        );

        // let ub_data = UniformBufferObject {
        //     model:
        //     // Mat4::rotate_z(self.frame_count as f32 / 30.0),
        //     Mat4::rotate_y(time.elapsed_f32())
        //         * Mat4::translate(Vec3::new(0.0, 0.5 * time.elapsed_f32().sin(), 0.0))
        //     * Mat4::translate(Vec3::new(0.0, 0.0, -5.0)),
        //     view: Mat4::identity(),
        //     proj: Mat4::perspective(window.aspect(), 1.0, 0.1, 10.0),
        //     // proj: Mat4::ortho(window.aspect() as f32 * 2.0, 2 as f32, 0.0, 10.0),
        // };

        // iferr!(
        //     "Failed to write to uniformbuffer",
        // self.data.uniformbuffers[image_index as usize].write(&ub_data, None)
        // );

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

        log::info!("Recreating resource manager");
        match self.resourcemanager.recreate() {
            Ok(_) => {}
            Err(e) => log::error!("Failed to recreate resource manager: {}", e),
        };

        self.data = iferr!(
            "Failed to recreate renderer",
            Self::create_data(&self.context, window, &self.resourcemanager)
        );
    }

    fn create_data(
        context: &Arc<VulkanContext>,
        window: &Window,
        resourcemanager: &Arc<ResourceManager>,
    ) -> Result<Data> {
        let swapchain = Arc::new(Swapchain::new(
            &context.instance,
            &context.physical_device,
            &context.device,
            &context.allocator,
            &context.surface_loader,
            &context.surface,
            &context.queue_families,
            window.extent(),
        )?);

        resourcemanager.set_swapchain(Arc::clone(&swapchain));

        let global_descriptor_layout_spec = DescriptorSetLayoutSpec {
            bindings: vec![DescriptorSetLayoutBinding {
                slot: 0,
                ty: DescriptorType::UniformBuffer,
                count: 1,
                stages: vec![ShaderStage::Vertex],
            }],
        };
        let global_descriptor_layout =
            DescriptorSetLayout::new(&context.device, global_descriptor_layout_spec)?;
        let mut uniformbuffers = Vec::new();
        for _ in 0..swapchain.image_count() {
            uniformbuffers.push(UniformBuffer::new(
                &context.allocator,
                std::mem::size_of::<UniformBufferObject>() as u64,
            )?);
        }

        let descriptor_pool = DescriptorPool::new(
            &context.device,
            &[vk::DescriptorPoolSize {
                descriptor_count: swapchain.image_count() as u32,
                ty: vk::DescriptorType::UNIFORM_BUFFER,
            }],
            swapchain.image_count() as u32,
        )?;

        // Create descriptor set for mvp data
        let global_descriptors = DescriptorSet::new(
            &context.device,
            &descriptor_pool,
            &global_descriptor_layout,
            swapchain.image_count() as u32,
        )?;

        // Write global descriptors
        DescriptorSet::write(
            &context.device,
            &global_descriptors,
            &global_descriptor_layout.spec(),
            uniformbuffers.iter(),
            [].iter(),
            [].iter(),
        )?;

        let commandpool = CommandPool::new(
            &context.device,
            context.queue_families.graphics.unwrap(),
            false,
            true,
        )?;

        let material = resourcemanager.load_material("./data/materials/default.json")?;
        let renderpass = resourcemanager.load_renderpass("./data/renderpasses/default.json")?;

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

        let model = resourcemanager.load_model("./data/models/suzanne.dae")?;

        let mesh = model.get_mesh_index(0).unwrap();

        for (i, commandbuffer) in commandbuffers.iter_mut().enumerate() {
            commandbuffer.begin(Default::default())?;
            commandbuffer.begin_renderpass(
                &renderpass,
                &framebuffers[i],
                math::Vec4::new(0.0, 0.0, 0.01, 1.0),
            );
            commandbuffer.bind_material(&material, &global_descriptors[i], i as u32);
            commandbuffer.bind_mesh(mesh);
            commandbuffer.draw_indexed(mesh.index_count());
            commandbuffer.end_renderpass();
            commandbuffer.end()?;
        }

        Ok(Data {
            swapchain,
            commandpool,
            commandbuffers,
            framebuffers,
            material,
            model,
            uniformbuffers,
            descriptor_pool,
            global_descriptors,
            renderpass,
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
