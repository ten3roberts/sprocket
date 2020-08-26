use super::enums::*;
use super::Result;
use ash::version::DeviceV1_0;
use ash::vk;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
/// Specifies how to create a renderpass
pub struct RenderPassSpec {
    pub subpasses: Vec<Subpass>,
    pub dependencies: Vec<SubpassDependency>,
    /// All attachments part of a renderpass, both color and depth attachments
    /// Their use is specified by index in subpasses
    pub attachments: Vec<Attachment>,
}

#[derive(Serialize, Deserialize)]
pub struct Subpass {
    pub color_attachments: Vec<usize>,
    pub depth_attachment: Option<usize>,
}

#[derive(Serialize, Deserialize)]
pub struct SubpassDependency {
    pub src_subpass: u32,
    pub dst_subpass: u32,
    pub src_stage: PipelineStage,
    pub dst_stage: PipelineStage,
    pub src_access: AccessFlags,
    pub dst_access: AccessFlags,
}

#[derive(Serialize, Deserialize)]
/// Describes an attachment in a renderpass
/// Also describes information in AttachmentReference like layout, but not index
/// The order in RenderPassSpec describes the attachments index
pub struct Attachment {
    pub store_op: AttachmentStoreOp,
    pub load_op: AttachmentLoadOp,
    pub initial_layout: ImageLayout,
    pub final_layout: ImageLayout,
    // The layout while in shader processing
    pub layout: ImageLayout,
    pub sample_count: u32,
    pub format: ImageFormat,
}

impl Attachment {
    pub fn to_vk(
        &self,
        color_format: vk::Format,
        depth_format: vk::Format,
    ) -> vk::AttachmentDescription {
        vk::AttachmentDescription {
            flags: Default::default(),
            format: match self.format {
                ImageFormat::Color => color_format,
                ImageFormat::Depth => depth_format,
                ImageFormat::Undefined => vk::Format::UNDEFINED,
            },
            samples: vk::SampleCountFlags::from_raw(self.sample_count),
            load_op: self.load_op.into(),
            store_op: self.store_op.into(),
            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
            initial_layout: self.initial_layout.into(),
            final_layout: self.final_layout.into(),
        }
    }
}

#[derive(Serialize, Deserialize, Copy, Clone)]
pub enum ImageFormat {
    Undefined,
    Color,
    Depth,
}

pub struct RenderPass {
    device: ash::Device,
    renderpass: vk::RenderPass,
}

impl RenderPass {
    pub fn new(
        device: &ash::Device,
        spec: RenderPassSpec,
        color_format: vk::Format,
        depth_format: vk::Format,
    ) -> Result<RenderPass> {
        let attachments: Vec<_> = spec
            .attachments
            .iter()
            .map(|attachment| attachment.to_vk(color_format, depth_format))
            .collect();

        let attachment_refs: Vec<_> = spec
            .attachments
            .iter()
            .enumerate()
            .map(|(i, attachment)| vk::AttachmentReference {
                attachment: i as u32,
                layout: attachment.layout.into(),
            })
            .collect();

        let subpass_color_attachments: Vec<Vec<_>> = spec
            .subpasses
            .iter()
            .map(|subpass| {
                subpass
                    .color_attachments
                    .iter()
                    .map(|index| attachment_refs[*index])
                    .collect()
            })
            .collect();
        let subpass_depth_attachment: Vec<_> = spec
            .subpasses
            .iter()
            .map(|subpass| match subpass.depth_attachment {
                Some(index) => &attachment_refs[index],
                None => std::ptr::null(),
            })
            .collect();

        let subpasses: Vec<_> = (0..spec.subpasses.len())
            .map(|i| vk::SubpassDescription {
                flags: Default::default(),
                pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
                input_attachment_count: 0,
                p_input_attachments: std::ptr::null(),
                color_attachment_count: subpass_color_attachments[i].len() as u32,
                p_color_attachments: subpass_color_attachments[i].as_ptr(),
                p_resolve_attachments: std::ptr::null(),
                p_depth_stencil_attachment: subpass_depth_attachment[i],
                preserve_attachment_count: 0,
                p_preserve_attachments: std::ptr::null(),
            })
            .collect();

        let dependencies: Vec<_> = spec
            .dependencies
            .iter()
            .map(|dependency| vk::SubpassDependency {
                src_subpass: dependency.src_subpass,
                dst_subpass: dependency.dst_subpass,
                src_stage_mask: dependency.src_stage.into(),
                dst_stage_mask: dependency.dst_stage.into(),
                src_access_mask: dependency.src_access.into(),
                dst_access_mask: dependency.dst_access.into(),
                dependency_flags: Default::default(),
            })
            .collect();

        let renderpass_info = vk::RenderPassCreateInfo::builder()
            .attachments(&attachments)
            .subpasses(&subpasses)
            .dependencies(&dependencies);

        let renderpass = unsafe { device.create_render_pass(&renderpass_info, None)? };

        Ok(RenderPass {
            device: device.clone(),
            renderpass,
        })
    }

    // Returns the internal vulkan renderpass
    pub fn vk(&self) -> vk::RenderPass {
        self.renderpass
    }
}

impl Drop for RenderPass {
    fn drop(&mut self) {
        unsafe { self.device.destroy_render_pass(self.renderpass, None) };
    }
}
