use super::RenderPass;
use crate::graphics::Extent2D;
use ash::version::DeviceV1_0;
use ash::vk;
use std::borrow::Cow;
use std::ffi::CStr;
use std::fs;

pub struct PipelineSpec {
    pub vertex_shader: String,
    pub fragment_shader: String,
    pub geometry_shader: String,
}

pub struct Pipeline {
    device: ash::Device,
    layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
}

impl Pipeline {
    pub fn new(
        device: &ash::Device,
        spec: PipelineSpec,
        extent: Extent2D,
        renderpass: &RenderPass,
    ) -> Result<Pipeline, Cow<'static, str>> {
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
        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(&[])
            .vertex_attribute_descriptions(&[]);

        // Input assembly
        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

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
            .cull_mode(vk::CullModeFlags::FRONT_AND_BACK)
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
            .sample_mask(&[])
            .alpha_to_coverage_enable(false)
            .alpha_to_one_enable(false);

        // Depth and stencil testing
        // TODO

        // Color blending
        let color_blend_attachment = vk::PipelineColorBlendAttachmentState::builder()
            .color_write_mask(
                vk::ColorComponentFlags::R
                    | vk::ColorComponentFlags::G
                    | vk::ColorComponentFlags::B,
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

        // Dynamic state
        // TODO
        // let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::LINE_WIDTH];

        // let dynamic_states =
        //     vk::PipelineDynamicStateCreateInfo::builder().dynamic_states(&dynamic_states);

        // Pipeline layout
        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(&[])
            .push_constant_ranges(&[]);

        let pipeline_layout = unsafe {
            unwrap_or_return!(
                "Failed to create pipeline layout",
                device.create_pipeline_layout(&pipeline_layout_info, None)
            )
        };

        let pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterizer)
            .multisample_state(&multisampling)
            .color_blend_state(&color_blending)
            .layout(pipeline_layout)
            .render_pass(renderpass.vk())
            .subpass(0)
            .base_pipeline_handle(vk::Pipeline::null())
            .base_pipeline_index(-1)
            .build();

        let pipeline = match unsafe {
            device.create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info], None)
        } {
            Ok(pipelines) => pipelines[0],
            Err(e) => return errfmt!("Failed to create graphics pipeline {:?}", e),
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
        })
    }

    pub fn vk(&self) -> vk::Pipeline {
        self.pipeline
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

fn create_shader_module(
    device: &ash::Device,
    filename: &str,
) -> Result<vk::ShaderModule, Cow<'static, str>> {
    let code = match fs::read(filename) {
        Ok(code) => code,
        Err(e) => return errfmt!("Failed to read file {}, '{}'", filename, e),
    };
    let create_info = vk::ShaderModuleCreateInfo {
        code_size: code.len(),
        p_code: unsafe { std::mem::transmute(code.as_ptr()) },
        ..Default::default()
    };

    unwrap_and_return!("Failed to create shader module", unsafe {
        device.create_shader_module(&create_info, None)
    })
}
