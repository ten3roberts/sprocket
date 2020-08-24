use super::{buffer, resources::Resource, CommandBuffer, CommandPool, Error, Result, VkAllocator};
use crate::graphics::Extent2D;
use ash::version::DeviceV1_0;
use ash::vk;
use std::ffi::CString;
use std::sync::Arc;

pub struct Texture {
    allocator: Option<VkAllocator>,
    device: ash::Device,
    image: vk::Image,
    memory: Option<vk_mem::Allocation>,
    view: vk::ImageView,
    format: vk::Format,
    layout: vk::ImageLayout,
    size: vk::DeviceSize,
    extent: Extent2D,
    owns_image: bool,
}

#[link(name = "stb_image", kind = "static")]
extern "C" {
    pub fn stbi_load(
        filename: *const i8,
        x: *mut i32,
        y: *mut i32,
        channels: *mut i32,
        desired_channels: i32,
    ) -> *mut u8;
}

impl Resource for Texture {
    // Load a texture from an image file on disk
    fn load(resourcemanager: &super::ResourceManager, path: &str) -> Result<Self> {
        let context = resourcemanager.context();
        let allocator = &context.allocator;
        let device = &context.device;
        let queue = context.graphics_queue;
        let commandpool = context.generic_pool();
        let filename = CString::new(path).expect("Failed to convert path into CString");
        let mut width = 0;
        let mut height = 0;
        let mut channels = 0;
        let pixels =
            unsafe { stbi_load(filename.as_ptr(), &mut width, &mut height, &mut channels, 4) };

        if pixels.is_null() {
            return Err(Error::ImageReadError(path.to_owned()));
        }

        // The size of the loaded image with alpha channel
        // May differ from the vkimage memrequirement size
        let image_size = width * height * 4;

        let format = vk::Format::R8G8B8A8_SRGB;
        let mut texture = Texture::new(
            allocator,
            device,
            format,
            vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
            vk::ImageAspectFlags::COLOR,
            vk::ImageTiling::OPTIMAL,
            (width, height).into(),
        )?;

        // Transition layout for transfer
        transition_image_layout(
            device,
            commandpool,
            queue,
            texture.image,
            vk::ImageAspectFlags::COLOR,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        )?;

        // Create and copy image pixel data to stagin buffer
        let (staging_buffer, staging_memory, staging_info) =
            buffer::create_staging(allocator, texture.size)?;
        // Copy the data into the staging buffer
        let data = staging_info.get_mapped_data();

        unsafe {
            std::ptr::copy_nonoverlapping(pixels as _, data, image_size as usize);
        }

        // Transfer the staging buffer to the image
        buffer::copy_to_image(
            device,
            queue,
            commandpool,
            staging_buffer,
            texture.image,
            texture.extent,
            vk::ImageAspectFlags::COLOR,
        )?;

        // Transition layout for shader read only optimal
        transition_image_layout(
            device,
            commandpool,
            queue,
            texture.image,
            vk::ImageAspectFlags::COLOR,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        )?;

        texture.layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;

        // Free staging buffer
        allocator
            .borrow()
            .destroy_buffer(staging_buffer, &staging_memory)?;

        // Free the pixels
        unsafe { Box::from_raw(pixels) };
        Ok(texture)
    }
}

impl Texture {
    // Creates a new empty image and image view with undefined dta
    pub fn new(
        allocator: &VkAllocator,
        device: &ash::Device,
        format: vk::Format,
        usage: vk::ImageUsageFlags,
        image_aspect: vk::ImageAspectFlags,
        tiling: vk::ImageTiling,
        extent: Extent2D,
    ) -> Result<Texture> {
        let image_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .extent(vk::Extent3D {
                width: extent.width,
                height: extent.height,
                depth: 1,
            })
            .mip_levels(1)
            .array_layers(1)
            .format(format)
            .tiling(tiling)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .samples(vk::SampleCountFlags::TYPE_1);

        let image_allocation_info = &vk_mem::AllocationCreateInfo {
            usage: vk_mem::MemoryUsage::GpuOnly,
            ..Default::default()
        };

        let (image, memory, _) = allocator
            .borrow()
            .create_image(&image_info, image_allocation_info)?;

        // Create image view
        let view_info = vk::ImageViewCreateInfo::builder()
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(format)
            .components(vk::ComponentMapping {
                r: vk::ComponentSwizzle::IDENTITY,
                g: vk::ComponentSwizzle::IDENTITY,
                b: vk::ComponentSwizzle::IDENTITY,
                a: vk::ComponentSwizzle::IDENTITY,
            })
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: image_aspect,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            })
            .image(image);

        let view = unsafe { device.create_image_view(&view_info, None)? };
        let size = unsafe { device.get_image_memory_requirements(image).size };

        Ok(Texture {
            allocator: Some(Arc::clone(allocator)),
            device: device.clone(),
            image,
            memory: Some(memory),
            view,
            format,
            extent,
            size,
            owns_image: true,
            layout: vk::ImageLayout::UNDEFINED,
        })
    }

    /// Creates a new texture that can be used as a depth attachment
    /// The contents and layout of the image is undefined
    pub fn new_depth(
        allocator: &VkAllocator,
        device: &ash::Device,
        extent: Extent2D,
    ) -> Result<Texture> {
        let format = vk::Format::D32_SFLOAT;
        let texture = Texture::new(
            allocator,
            device,
            format,
            vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
            vk::ImageAspectFlags::DEPTH,
            vk::ImageTiling::OPTIMAL,
            extent,
        )?;

        Ok(texture)
    }

    /// Creates a texture with an already existing image view
    pub fn new_from_image(
        device: &ash::Device,
        extent: Extent2D,
        image: vk::Image,
        format: vk::Format,
        layout: vk::ImageLayout,
    ) -> Result<Texture> {
        let view_create_info = vk::ImageViewCreateInfo::builder()
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(format)
            .components(vk::ComponentMapping {
                r: vk::ComponentSwizzle::IDENTITY,
                g: vk::ComponentSwizzle::IDENTITY,
                b: vk::ComponentSwizzle::IDENTITY,
                a: vk::ComponentSwizzle::IDENTITY,
            })
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            })
            .image(image);
        let view = unsafe {
            device
                .create_image_view(&view_create_info, None)
                .expect("Failed to create image view")
        };

        let size = unsafe { device.get_image_memory_requirements(image).size };

        Ok(Texture {
            allocator: None,
            device: device.clone(),
            image,
            memory: None,
            view,
            format,
            extent,
            size,
            owns_image: false,
            layout,
        })
    }

    pub fn image_view(&self) -> vk::ImageView {
        self.view
    }

    pub fn image(&self) -> vk::Image {
        self.image
    }

    pub fn layout(&self) -> vk::ImageLayout {
        self.layout
    }

    pub fn format(&self) -> vk::Format {
        self.format
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            if self.owns_image {
                self.allocator
                    .as_ref()
                    .expect("Missing allocator for owned image")
                    .borrow()
                    .destroy_image(self.image, &self.memory.unwrap())
                    .expect("Failed to free image")
            }
            self.device.destroy_image_view(self.view, None);
        }
    }
}

fn transition_image_layout(
    device: &ash::Device,
    commandpool: &CommandPool,
    queue: vk::Queue,
    image: vk::Image,
    image_aspect: vk::ImageAspectFlags,
    old_layout: vk::ImageLayout,
    new_layout: vk::ImageLayout,
) -> Result<()> {
    let commandbuffer = &mut CommandBuffer::new_primary(device, commandpool, 1)?[0];

    commandbuffer.begin(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)?;

    let (src_access_mask, dst_access_mask, src_stage_mask, dst_stage_mask) =
        match (old_layout, new_layout) {
            (vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL) => (
                vk::AccessFlags::default(),
                vk::AccessFlags::TRANSFER_WRITE,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::TRANSFER,
            ),
            (vk::ImageLayout::UNDEFINED, vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL) => (
                vk::AccessFlags::default(),
                vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ
                    | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            ),
            (vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL) => (
                vk::AccessFlags::TRANSFER_WRITE,
                vk::AccessFlags::SHADER_READ,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
            ),

            (src, dst) => return Err(Error::UnsupportedTransition(src, dst)),
        };

    let barrier = vk::ImageMemoryBarrier {
        s_type: vk::StructureType::IMAGE_MEMORY_BARRIER,
        new_layout,
        old_layout,
        src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
        dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
        image,
        subresource_range: vk::ImageSubresourceRange {
            aspect_mask: image_aspect,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        },
        src_access_mask,
        dst_access_mask,
        p_next: std::ptr::null(),
    };

    unsafe {
        device.cmd_pipeline_barrier(
            commandbuffer.vk(),
            src_stage_mask,
            dst_stage_mask,
            vk::DependencyFlags::default(),
            &[],
            &[],
            &[barrier],
        )
    }

    commandbuffer.end()?;

    CommandBuffer::submit(
        device,
        &[&commandbuffer],
        queue,
        &[],
        &[],
        &[],
        vk::Fence::null(),
    )?;

    Ok(())
}

fn has_stencil_component(format: vk::Format) -> bool {
    return format == vk::Format::D32_SFLOAT_S8_UINT || format == vk::Format::D24_UNORM_S8_UINT;
}
