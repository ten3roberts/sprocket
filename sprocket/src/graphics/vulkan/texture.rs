use crate::graphics::Extent2D;
use ash::version::DeviceV1_0;
use ash::vk;

pub struct Texture {
    device: ash::Device,
    image: vk::Image,
    view: vk::ImageView,
    format: vk::Format,
    size: vk::DeviceSize,
    extent: Extent2D,
    owns_image: bool,
}

impl Texture {
    pub fn new(device: &ash::Device, extent: Extent2D) -> Texture {
        let format = vk::Format::R8G8B8_SRGB;
        let image_create_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .extent(vk::Extent3D {
                width: extent.width,
                height: extent.height,
                depth: 1,
            })
            .mip_levels(1)
            .array_layers(1)
            .format(format)
            .tiling(vk::ImageTiling::OPTIMAL)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .samples(vk::SampleCountFlags::TYPE_1);

        let image = unsafe {
            device
                .create_image(&image_create_info, None)
                .expect("Failed to create image")
        };
        Texture::new_from_image(device, extent, image, format)
    }

    /// Creates a texture with an already existing image view
    pub fn new_from_image(
        device: &ash::Device,
        extent: Extent2D,
        image: vk::Image,
        format: vk::Format,
    ) -> Texture {
        let view_create_info = vk::ImageViewCreateInfo::builder()
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(format)
            .components(vk::ComponentMapping {
                r: vk::ComponentSwizzle::R,
                g: vk::ComponentSwizzle::G,
                b: vk::ComponentSwizzle::B,
                a: vk::ComponentSwizzle::A,
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

        Texture {
            device: device.clone(),
            image,
            view,
            format,
            extent,
            size,
            owns_image: false,
        }
    }

    pub fn image_view(&self) -> vk::ImageView {
        self.view
    }

    pub fn image(&self) -> vk::Image {
        self.image
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            if self.owns_image {
                self.device.destroy_image(self.image, None);
            }
            self.device.destroy_image_view(self.view, None);
        }
    }
}
