use super::texture::Texture;
use crate::*;
use ash::vk;
use std::borrow::Cow;
use std::cmp::{max, min};

pub struct Swapchain {
    swapchain: vk::SwapchainKHR,
    swapchain_loader: ash::extensions::khr::Swapchain,
    images: Vec<Texture>,
}

impl Swapchain {
    pub fn new(
        instance: &ash::Instance,
        physical_device: &vk::PhysicalDevice,
        device: &ash::Device,
        surface_loader: &ash::extensions::khr::Surface,
        surface: &vk::SurfaceKHR,
        queue_families: &graphics::vulkan::QueueFamilies,
        width: i32,
        height: i32,
    ) -> Result<Swapchain, Cow<'static, str>> {
        unsafe {
            let (capabilities, formats, present_modes) = unwrap_or_return!(
                "Failed to query swapchain support",
                Self::query_support(physical_device, surface_loader, surface)
            );

            let format = Self::pick_format(formats);
            let present_mode = Self::pick_present_mode(present_modes);
            let extent = Self::pick_extent(&capabilities, width, height);
            info!(
                "Swapchain minimum image count: {}",
                capabilities.min_image_count
            );
            let image_count = graphics::SWAPCHAIN_IMAGE_COUNT;

            let swapchain_loader = ash::extensions::khr::Swapchain::new(instance, device);

            let mut create_info = vk::SwapchainCreateInfoKHR::builder()
                .surface(*surface)
                .min_image_count(image_count)
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

            if queue_families.graphics == queue_families.present {
                debug!("Sharing mode concurrent");
                create_info.image_sharing_mode = vk::SharingMode::CONCURRENT;
                create_info.queue_family_index_count = 2;
                create_info = create_info.queue_family_indices(&queue_family_indices);
            } else {
                debug!("Sharing mode exclusive");
                create_info.image_sharing_mode = vk::SharingMode::EXCLUSIVE;
                create_info.queue_family_index_count = 0;
            }

            let swapchain = unwrap_or_return!(
                "Failed to create swapchain",
                swapchain_loader.create_swapchain(&create_info, None)
            );

            // Create textures from the images in swapchain
            let images = unwrap_or_return!(
                "Failed to get swapchain images",
                swapchain_loader.get_swapchain_images(swapchain)
            );

            let images = images
                .into_iter()
                .map(|image| {
                    Texture::new_from_image(
                        device,
                        width as u32,
                        height as u32,
                        image,
                        format.format,
                    )
                })
                .collect();

            Ok(Swapchain {
                swapchain,
                swapchain_loader,
                images,
            })
        }
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
        info!("Choosing FIFO present mode");
        vk::PresentModeKHR::FIFO
    }

    fn pick_extent(
        capabilities: &vk::SurfaceCapabilitiesKHR,
        width: i32,
        height: i32,
    ) -> vk::Extent2D {
        if capabilities.current_extent.width != std::u32::MAX {
            capabilities.current_extent
        } else {
            vk::Extent2D {
                width: max(
                    capabilities.min_image_extent.width,
                    min(capabilities.max_image_extent.width, width as u32),
                ),
                height: max(
                    capabilities.min_image_extent.height,
                    min(capabilities.max_image_extent.height, height as u32),
                ),
            }
        }
    }

    pub unsafe fn query_support(
        physical_device: &vk::PhysicalDevice,
        surface_loader: &ash::extensions::khr::Surface,
        surface: &vk::SurfaceKHR,
    ) -> Result<
        (
            vk::SurfaceCapabilitiesKHR,
            Vec<vk::SurfaceFormatKHR>,
            Vec<vk::PresentModeKHR>,
        ),
        Cow<'static, str>,
    > {
        let capabilities = unwrap_or_return!(
            "Unable to get physical device surface capabilities",
            surface_loader.get_physical_device_surface_capabilities(*physical_device, *surface)
        );

        let formats = unwrap_or_return!(
            "Failed to get surface formats for swapchain",
            surface_loader.get_physical_device_surface_formats(*physical_device, *surface)
        );

        let present_modes = unwrap_or_return!(
            "Failed to get present modes from surface",
            surface_loader.get_physical_device_surface_present_modes(*physical_device, *surface)
        );

        Ok((capabilities, formats, present_modes))
    }

    // Destroys the swapchain and textures
    pub unsafe fn destroy(&mut self) {
        self.images.clear();
        self.swapchain_loader
            .destroy_swapchain(self.swapchain, None);
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        unsafe {
            self.destroy();
        };
    }
}
