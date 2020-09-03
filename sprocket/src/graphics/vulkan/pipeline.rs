use super::vertexbuffer::Vertex;
use super::{resources::Resource, DescriptorSetLayout, DescriptorSetLayoutSpec, Error, Result};

use ash::version::DeviceV1_0;
use ash::vk;
use ex::fs;
use serde::{Deserialize, Serialize};
use std::ffi::CStr;

#[derive(Serialize, Deserialize, Clone)]
pub struct PipelineSpec {
    pub vertex_shader: String,
    pub fragment_shader: String,
    pub geometry_shader: String,
    pub renderpass: String,
    pub layouts: Vec<DescriptorSetLayoutSpec>,
}

pub struct Pipeline {
    device: ash::Device,
    layout: vk::PipelineLayout,
    set_layouts: Vec<DescriptorSetLayout>,
    pipeline: vk::Pipeline,
    spec: PipelineSpec,
}

impl Resource for Pipeline {
    fn load(resourcemanager: &super::ResourceManager, path: &str) -> Result<Self> {
        let spec: PipelineSpec = serde_json::from_str(&ex::fs::read_to_string(path)?)?;
        let context = resourcemanager.context();

        Self::new(&context.device, spec, resourcemanager)
    }
}

impl Pipeline {
    pub fn new(
        device: &ash::Device,
        spec: PipelineSpec,
        resourcemanager: &super::ResourceManager,
    ) -> Result<Self> {
        let shader_entry_point = unsafe { CStr::from_ptr("main\0".as_ptr() as _) };

        // Shader stages
        let vertex_shader_module = create_shader_module(device, &spec.vertex_shader)?;

        let fragment_shader_module = create_shader_module(device, &spec.fragment_shader)?;

        let vertex_shader_info = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vertex_shader_module)
            .name(&shader_entry_point)
            .build();

        let fragment_shader_info = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(fragment_shader_module)
            .name(&shader_entry_point)
            .build();

        let shader_stages = [vertex_shader_info, fragment_shader_info];

        // Vertex input
        let binding_descriptions = [Vertex::binding_description()];
        let attribute_descriptions = Vertex::attribute_descriptions();
        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(&binding_descriptions)
            .vertex_attribute_descriptions(&attribute_descriptions);

        // Input assembly
        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        let extent = resourcemanager.get_swapchain().unwrap().extent();

        // Viewports and scissors
        let viewports = [vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: extent.width as f32,
            height: extent.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        }];

        let scissors = [vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: vk::Extent2D {
                width: extent.width,
                height: extent.height,
            },
        }];

        let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
            .viewports(&viewports)
            .scissors(&scissors);

        // Rasterizer
        let rasterizer = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .cull_mode(vk::CullModeFlags::NONE)
            .depth_bias_enable(false)
            .front_face(vk::FrontFace::CLOCKWISE)
            .line_width(1.0)
            .depth_bias_enable(false)
            .depth_bias_constant_factor(0.0)
            .depth_bias_clamp(0.0)
            .depth_bias_slope_factor(0.0);

        // Multisampling
        let multisampling = vk::PipelineMultisampleStateCreateInfo::builder()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlags::TYPE_1)
            .min_sample_shading(1.0)
            // .sample_mask(&[vk::SampleMask::MAX])
            .alpha_to_coverage_enable(false)
            .alpha_to_one_enable(false);

        // Depth and stencil testing
        // TODO

        // Color blending
        let color_blend_attachment = vk::PipelineColorBlendAttachmentState::builder()
            .color_write_mask(
                vk::ColorComponentFlags::R
                    | vk::ColorComponentFlags::G
                    | vk::ColorComponentFlags::B
                    | vk::ColorComponentFlags::A,
            )
            .blend_enable(false)
            .src_color_blend_factor(vk::BlendFactor::ONE)
            .dst_color_blend_factor(vk::BlendFactor::ZERO)
            .src_alpha_blend_factor(vk::BlendFactor::ONE)
            .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
            .alpha_blend_op(vk::BlendOp::ADD)
            .build();

        let color_blend_attachments = [color_blend_attachment];
        let color_blending = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .logic_op(vk::LogicOp::COPY)
            .attachments(&color_blend_attachments)
            .blend_constants([0.0; 4]);

        let depth_stencil_state = vk::PipelineDepthStencilStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO,
            depth_test_enable: vk::TRUE,
            depth_write_enable: vk::TRUE,
            depth_compare_op: vk::CompareOp::LESS,
            depth_bounds_test_enable: vk::FALSE,
            min_depth_bounds: 0.0,
            max_depth_bounds: 1.0,
            stencil_test_enable: vk::FALSE,
            front: Default::default(),
            back: Default::default(),
            flags: Default::default(),
            p_next: std::ptr::null(),
        };

        // Dynamic state
        // TODO
        // let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::LINE_WIDTH];

        // let dynamic_states =
        //     vk::PipelineDynamicStateCreateInfo::builder().dynamic_states(&dynamic_states);

        // Pipeline layout
        let mut set_layouts = Vec::with_capacity(spec.layouts.len());

        for layout_spec in &spec.layouts {
            // TODO avoid clone
            set_layouts.push(DescriptorSetLayout::new(device, layout_spec.clone())?)
        }

        let vk_set_layouts: Vec<vk::DescriptorSetLayout> =
            set_layouts.iter().map(|layout| layout.vk()).collect();

        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(&vk_set_layouts)
            .push_constant_ranges(&[]);

        let pipeline_layout =
            unsafe { device.create_pipeline_layout(&pipeline_layout_info, None)? };

        let renderpass = resourcemanager.load_renderpass(&spec.renderpass)?;

        let pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterizer)
            .multisample_state(&multisampling)
            .color_blend_state(&color_blending)
            .depth_stencil_state(&depth_stencil_state)
            .layout(pipeline_layout)
            .render_pass(renderpass.vk())
            .subpass(0)
            .base_pipeline_handle(vk::Pipeline::null())
            .base_pipeline_index(-1)
            .build();

        let pipeline = unsafe {
            device
                .create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info], None)
                .map_err(|e| Error::VulkanError(e.1))?[0]
        };

        // Destroy shader modules
        unsafe {
            device.destroy_shader_module(vertex_shader_module, None);
            device.destroy_shader_module(fragment_shader_module, None);
        }

        Ok(Pipeline {
            device: device.clone(),
            layout: pipeline_layout,
            pipeline,
            set_layouts,
            spec,
        })
    }

    pub fn vk(&self) -> vk::Pipeline {
        self.pipeline
    }

    pub fn layout(&self) -> vk::PipelineLayout {
        self.layout
    }

    pub fn set_layouts(&self) -> &[DescriptorSetLayout] {
        &self.set_layouts[..]
    }

    /// Returns self created again from spec but with updated values
    /// Called when swapchain is recreated
    pub fn recreate(&self, resourcemanager: &super::ResourceManager) -> Result<Pipeline> {
        Self::new(&self.device, self.spec.clone(), resourcemanager)
    }
}

impl Drop for Pipeline {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_pipeline_layout(self.layout, None);
            self.device.destroy_pipeline(self.pipeline, None);
        };
    }
}

fn create_shader_module(device: &ash::Device, filename: &str) -> Result<vk::ShaderModule> {
    let mut file = fs::File::open(filename)?;

    let code = match ash::util::read_spv(&mut file) {
        Ok(code) => code,
        Err(e) => return Err(Error::SPVReadError(e, filename.to_owned())),
    };

    let create_info = vk::ShaderModuleCreateInfo::builder().code(&code);

    unsafe {
        device
            .create_shader_module(&create_info, None)
            .map_err(|e| e.into())
    }
}
