use super::{Texture, VkAllocator};
use crate::graphics::Extent2D;
use crate::*;
use ash::vk;
use std::cmp::{max, min};

use super::Result;

pub struct Swapchain {
    swapchain: vk::SwapchainKHR,
    swapchain_loader: ash::extensions::khr::Swapchain,
    images: Vec<Texture>,
    depth_image: Texture,
    format: vk::Format,
    extent: Extent2D,
}

impl Swapchain {
    pub fn new(
        instance: &ash::Instance,
        physical_device: &vk::PhysicalDevice,
        device: &ash::Device,
        allocator: &VkAllocator,
        surface_loader: &ash::extensions::khr::Surface,
        surface: &vk::SurfaceKHR,
        queue_families: &graphics::vulkan::QueueFamilies,
        extent: Extent2D,
    ) -> Result<Swapchain> {
        unsafe {
            let (capabilities, formats, present_modes) =
                Self::query_support(physical_device, surface_loader, surface)?;

            let format = Self::pick_format(formats);
            let present_mode = Self::pick_present_mode(present_modes);
            let extent = Self::pick_extent(&capabilities, extent);

            debug!(
                "Swapchain images: {}<>{}",
                capabilities.min_image_count, capabilities.max_image_count
            );

            let min_image_count = 3;

            let swapchain_loader = ash::extensions::khr::Swapchain::new(instance, device);

            let mut create_info = vk::SwapchainCreateInfoKHR::builder()
                .surface(*surface)
                .min_image_count(min_image_count)
                .image_color_space(format.color_space)
                .image_format(format.format)
                .image_extent(extent)
                .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
                .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
                .pre_transform(capabilities.current_transform)
                .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
                .present_mode(present_mode)
                .clipped(true)
                .image_array_layers(1);

            let queue_family_indices = [
                queue_families.graphics.unwrap(),
                queue_families.present.unwrap(),
            ];

            if queue_families.graphics != queue_families.present {
                debug!("Sharing mode concurrent");
                create_info.image_sharing_mode = vk::SharingMode::CONCURRENT;
                create_info.queue_family_index_count = 2;
                create_info = create_info.queue_family_indices(&queue_family_indices);
            } else {
                debug!("Sharing mode exclusive");
                create_info.image_sharing_mode = vk::SharingMode::EXCLUSIVE;
                create_info.queue_family_index_count = 0;
            }

            let swapchain = swapchain_loader.create_swapchain(&create_info, None)?;

            // Create textures from the images in swapchain
            let images = swapchain_loader.get_swapchain_images(swapchain)?;
            debug!("Swapchain image count: {}", images.len());

            let mut swapchain_images = Vec::with_capacity(images.len());
            for image in images {
                swapchain_images.push(Texture::new_from_image(
                    device,
                    extent.into(),
                    image,
                    format.format,
                    vk::ImageLayout::UNDEFINED,
                )?)
            }

            let depth_image = Texture::new_depth(allocator, device, extent.into())?;

            Ok(Swapchain {
                swapchain,
                swapchain_loader,
                images: swapchain_images,
                depth_image,
                format: format.format,
                extent: extent.into(),
            })
        }
    }

    pub fn format(&self) -> vk::Format {
        self.format
    }

    fn pick_format(formats: Vec<vk::SurfaceFormatKHR>) -> vk::SurfaceFormatKHR {
        for format in &formats {
            if format.format == vk::Format::B8G8R8A8_SRGB
                && format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            {
                return *format;
            }
        }
        formats[0]
    }

    fn pick_present_mode(present_modes: Vec<vk::PresentModeKHR>) -> vk::PresentModeKHR {
        for mode in &present_modes {
            if *mode == vk::PresentModeKHR::MAILBOX {
                info!("Choosing MAILBOX present mode");
                return *mode;
            }
        }
        info!("Choosing IMMEDIATE present mode");
        vk::PresentModeKHR::IMMEDIATE
    }

    fn pick_extent(capabilities: &vk::SurfaceCapabilitiesKHR, extent: Extent2D) -> vk::Extent2D {
        if capabilities.current_extent.width != std::u32::MAX {
            capabilities.current_extent
        } else {
            vk::Extent2D {
                width: max(
                    capabilities.min_image_extent.width,
                    min(capabilities.max_image_extent.width, extent.width),
                ),
                height: max(
                    capabilities.min_image_extent.height,
                    min(capabilities.max_image_extent.height, extent.height),
                ),
            }
        }
    }

    pub fn image_count(&self) -> usize {
        self.images.len()
    }

    pub fn image(&self, index: usize) -> &Texture {
        &self.images[index]
    }

    pub fn extent(&self) -> Extent2D {
        self.extent
    }

    pub fn depth_image(&self) -> &Texture {
        &self.depth_image
    }

    /// Returns the index to the next available image in the swapchain
    pub fn acquire_next_image(&self, semaphore: &vk::Semaphore) -> Result<(u32, bool)> {
        unsafe {
            self.swapchain_loader
                .acquire_next_image(self.swapchain, std::u64::MAX, *semaphore, vk::Fence::null())
                .map_err(|e| e.into())
        }
    }

    // Presents an image on the swapchain
    pub fn present(
        &self,
        image_index: u32,
        queue: vk::Queue,
        wait_semaphores: &[vk::Semaphore],
    ) -> Result<bool> {
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(wait_semaphores)
            .swapchains(&[self.swapchain])
            .image_indices(&[image_index])
            .build();

        unsafe {
            self.swapchain_loader
                .queue_present(queue, &present_info)
                .map_err(|e| e.into())
        }
    }

    pub fn vk(&self) -> vk::SwapchainKHR {
        self.swapchain
    }

    pub fn loader(&self) -> &ash::extensions::khr::Swapchain {
        &self.swapchain_loader
    }

    pub unsafe fn query_support(
        physical_device: &vk::PhysicalDevice,
        surface_loader: &ash::extensions::khr::Surface,
        surface: &vk::SurfaceKHR,
    ) -> Result<(
        vk::SurfaceCapabilitiesKHR,
        Vec<vk::SurfaceFormatKHR>,
        Vec<vk::PresentModeKHR>,
    )> {
        let capabilities =
            surface_loader.get_physical_device_surface_capabilities(*physical_device, *surface)?;

        let formats =
            surface_loader.get_physical_device_surface_formats(*physical_device, *surface)?;

        let present_modes =
            surface_loader.get_physical_device_surface_present_modes(*physical_device, *surface)?;

        Ok((capabilities, formats, present_modes))
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        unsafe {
            self.images.clear();
            self.swapchain_loader
                .destroy_swapchain(self.swapchain, None);
        };
    }
}
