//! This module contains several enums representing and abstracting over vulkan enums
//! All enums here can be serialized and deserialized to strings

use ash::vk;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Copy, Clone)]
pub enum PipelineStage {
    TopOfPipe = 0b1,
    DrawIndirect = 0b10,
    VertexInput = 0b100,
    VertexShader = 0b1000,
    TessellationControlShader = 0b1_0000,
    TessellationEvaluationShader = 0b10_0000,
    GeometryShader = 0b100_0000,
    FragmentShader = 0b1000_0000,
    EarlyFragmentTests = 0b1_0000_0000,
    LateFragmentTests = 0b10_0000_0000,
    ColorAttachmentOutput = 0b100_0000_0000,
    ComputeShader = 0b1000_0000_0000,
    Transfer = 0b1_0000_0000_0000,
    BottomOfPipe = 0b10_0000_0000_0000,
    Host = 0b100_0000_0000_0000,
    AllGraphics = 0b1000_0000_0000_0000,
    AllCommands = 0b1_0000_0000_0000_0000,
}

impl From<PipelineStage> for vk::PipelineStageFlags {
    fn from(stage: PipelineStage) -> Self {
        Self::from_raw(stage as u32)
    }
}

#[derive(Serialize, Deserialize, Copy, Clone)]
pub enum AccessFlags {
    None = 0,
    IndirectCommandRead = 0b1,
    IndexRead = 0b10,
    VertexAttributeRead = 0b100,
    UniformRead = 0b1000,
    InputAttachmentRead = 0b1_0000,
    ShaderRead = 0b10_0000,
    ShaderWrite = 0b100_0000,
    ColorAttachmentRead = 0b1000_0000,
    ColorAttachmentWrite = 0b1_0000_0000,
    DepthStencilAttachmentRead = 0b10_0000_0000,
    DepthStencilAttachmentWrite = 0b100_0000_0000,
    TransferRead = 0b1000_0000_0000,
    TransferWrite = 0b1_0000_0000_0000,
    HostRead = 0b10_0000_0000_0000,
    HostWrite = 0b100_0000_0000_0000,
    MemoryRead = 0b1000_0000_0000_0000,
    MemoryWrite = 0b1_0000_0000_0000_0000,
}

impl From<AccessFlags> for vk::AccessFlags {
    fn from(flags: AccessFlags) -> Self {
        Self::from_raw(flags as u32)
    }
}

#[derive(Serialize, Deserialize, Copy, Clone)]
pub enum AttachmentType {
    Color,
    Depth,
}

#[derive(Serialize, Deserialize, Copy, Clone)]
pub enum AttachmentLoadOp {
    Clear,
    Load,
    DontCare,
}

impl From<AttachmentLoadOp> for vk::AttachmentLoadOp {
    fn from(op: AttachmentLoadOp) -> Self {
        match op {
            AttachmentLoadOp::Clear => Self::CLEAR,
            AttachmentLoadOp::DontCare => Self::DONT_CARE,
            AttachmentLoadOp::Load => Self::LOAD,
        }
    }
}

#[derive(Serialize, Deserialize, Copy, Clone)]
pub enum AttachmentStoreOp {
    Store,
    DontCare,
}

impl From<AttachmentStoreOp> for vk::AttachmentStoreOp {
    fn from(op: AttachmentStoreOp) -> Self {
        match op {
            AttachmentStoreOp::Store => Self::STORE,
            AttachmentStoreOp::DontCare => Self::DONT_CARE,
        }
    }
}

#[derive(Serialize, Deserialize, Copy, Clone)]
pub enum ImageLayout {
    Undefined,
    General,
    ColorAttachment,
    DepthStencilReadOnly,
    DepthStencilAttachment,
    ShaderReadOnly,
    TransferSrc,
    TransferDst,
    Preinitialized,
    PresentSrc,
}

impl From<ImageLayout> for vk::ImageLayout {
    fn from(layout: ImageLayout) -> Self {
        match layout {
            ImageLayout::Undefined => Self::UNDEFINED,
            ImageLayout::General => Self::GENERAL,
            ImageLayout::ColorAttachment => Self::COLOR_ATTACHMENT_OPTIMAL,
            ImageLayout::DepthStencilReadOnly => Self::DEPTH_STENCIL_READ_ONLY_OPTIMAL,
            ImageLayout::DepthStencilAttachment => Self::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            ImageLayout::ShaderReadOnly => Self::SHADER_READ_ONLY_OPTIMAL,
            ImageLayout::TransferSrc => Self::TRANSFER_SRC_OPTIMAL,
            ImageLayout::TransferDst => Self::TRANSFER_DST_OPTIMAL,
            ImageLayout::Preinitialized => Self::PREINITIALIZED,
            ImageLayout::PresentSrc => Self::PRESENT_SRC_KHR,
        }
    }
}
