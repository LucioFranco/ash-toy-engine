use ash::version::{DeviceV1_0, V1_0};
use ash::vk;
use ash::Device;
use shader::Shader;
use std::default::Default;
use std::ptr;

pub struct Pipeline {
    pub graphics_pipelines: Vec<vk::Pipeline>,
    pub render_pass: RenderPass,
    pub layout: PipelineLayout,
    pub shaders: Vec<Shader>,
}

impl Pipeline {
    pub fn build() -> PipelineBuilder {
        PipelineBuilder::new()
    }
}

#[derive(Default)]
pub struct PipelineBuilder {
    shaders: Vec<Shader>,
    vertex_input_state: Option<vk::PipelineVertexInputStateCreateInfo>,
    input_assembly_state: Option<vk::PipelineInputAssemblyStateCreateInfo>,
    viewport_state: Option<Viewport>,
    rasterizer: Option<vk::PipelineRasterizationStateCreateInfo>,
    multisample: Option<vk::PipelineMultisampleStateCreateInfo>,
    color_blend_state: Option<ColorBlend>,
    dynamic_state: Option<vk::PipelineDynamicStateCreateInfo>,
    layout: Option<PipelineLayout>,
    render_pass: Option<RenderPass>,
}

impl PipelineBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_shader_stage(mut self, shader: Shader) -> Self {
        self.shaders.push(shader);
        self
    }

    pub fn with_vertex_input_state(mut self) -> Self {
        let vertex_input_state = vk::PipelineVertexInputStateCreateInfo {
            s_type: vk::StructureType::PipelineVertexInputStateCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            vertex_binding_description_count: 0,
            p_vertex_binding_descriptions: ptr::null(),
            vertex_attribute_description_count: 0,
            p_vertex_attribute_descriptions: ptr::null(),
        };

        self.vertex_input_state = Some(vertex_input_state);
        self
    }

    pub fn with_input_assembly_state(mut self) -> Self {
        let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo {
            s_type: vk::StructureType::PipelineInputAssemblyStateCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            topology: vk::PrimitiveTopology::TriangleList,
            primitive_restart_enable: 0,
        };

        self.input_assembly_state = Some(input_assembly_state);
        self
    }

    pub fn with_viewport(mut self, surface_resolution: vk::Extent2D) -> Self {
        self.viewport_state = Some(Viewport::new(surface_resolution));
        self
    }

    pub fn with_rasterizer(mut self) -> Self {
        let rasterizer = vk::PipelineRasterizationStateCreateInfo {
            s_type: vk::StructureType::PipelineRasterizationStateCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            depth_clamp_enable: 0,
            rasterizer_discard_enable: 0,
            polygon_mode: vk::PolygonMode::Fill,
            line_width: 1.0,
            cull_mode: vk::CULL_MODE_BACK_BIT,
            front_face: vk::FrontFace::Clockwise,
            depth_bias_enable: 0,
            depth_bias_constant_factor: 0.0,
            depth_bias_clamp: 0.0,
            depth_bias_slope_factor: 0.0,
        };

        self.rasterizer = Some(rasterizer);
        self
    }

    pub fn with_multisample(mut self) -> Self {
        let multisample = vk::PipelineMultisampleStateCreateInfo {
            s_type: vk::StructureType::PipelineMultisampleStateCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            sample_shading_enable: 0,
            rasterization_samples: vk::SAMPLE_COUNT_1_BIT,
            min_sample_shading: 1.0,
            p_sample_mask: ptr::null(),
            alpha_to_coverage_enable: 0,
            alpha_to_one_enable: 0,
        };

        self.multisample = Some(multisample);
        self
    }

    pub fn with_color_blend(mut self) -> Self {
        self.color_blend_state = Some(ColorBlend::new());
        self
    }

    pub fn with_dynamic_state(mut self) -> Self {
        let dynamic_state = [vk::DynamicState::Viewport, vk::DynamicState::Scissor];
        let dynamic_state_info = vk::PipelineDynamicStateCreateInfo {
            s_type: vk::StructureType::PipelineDynamicStateCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            dynamic_state_count: dynamic_state.len() as u32,
            p_dynamic_states: dynamic_state.as_ptr(),
        };

        self.dynamic_state = Some(dynamic_state_info);
        self
    }

    pub fn with_layout(mut self, layout: PipelineLayout) -> Self {
        self.layout = Some(layout);
        self
    }

    pub fn with_render_pass(mut self, render_pass: RenderPass) -> Self {
        self.render_pass = Some(render_pass);
        self
    }

    pub fn create(self, device: &Device<V1_0>) -> Result<Pipeline, ()> {
        let vertex_input_state = self.vertex_input_state.expect("vertex_input_state");
        let input_assembly_state = self.input_assembly_state.expect("input_assembly_state");
        let viewport_state = self.viewport_state.expect("viewport");
        let rasterizer = self.rasterizer.expect("reasterizer");
        let multisample = self.multisample.expect("multisample");
        let color_blend_state_create_info = self.color_blend_state.expect("color_blend_state");
        let layout = self.layout.expect("layout");
        let render_pass = self.render_pass.expect("render_pass");

        let shader_stages = self
            .shaders
            .iter()
            .map(|ref shader| shader.create_stage())
            .collect::<Vec<_>>();

        let pipeline_info = vk::GraphicsPipelineCreateInfo {
            s_type: vk::StructureType::GraphicsPipelineCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            stage_count: 2,
            p_stages: shader_stages.as_ptr(),
            p_vertex_input_state: &vertex_input_state,
            p_input_assembly_state: &input_assembly_state,
            p_viewport_state: &viewport_state.create(),
            p_rasterization_state: &rasterizer,
            p_multisample_state: &multisample,
            p_depth_stencil_state: ptr::null(),
            p_color_blend_state: &color_blend_state_create_info.create(),
            p_dynamic_state: ptr::null(),
            layout: layout.layout,
            render_pass: render_pass.render_pass,
            subpass: 0,
            base_pipeline_handle: vk::Pipeline::null(),
            base_pipeline_index: 0,
            p_tessellation_state: ptr::null(),
        };

        let graphics_pipelines = unsafe {
            device
                .create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info], None)
                .expect("Unable to create graphics pipeline")
        };

        Ok(Pipeline {
            graphics_pipelines,
            layout: layout,
            render_pass: render_pass,
            shaders: self.shaders,
        })
    }
}

pub struct Viewport {
    viewport: vk::Viewport,
    scissor: vk::Rect2D,
}

impl Viewport {
    pub fn new(surface_resolution: vk::Extent2D) -> Self {
        let viewport = vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: surface_resolution.width as f32,
            height: surface_resolution.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        };

        let scissor = vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: surface_resolution.clone(),
        };

        Viewport { viewport, scissor }
    }

    pub fn create(self) -> vk::PipelineViewportStateCreateInfo {
        vk::PipelineViewportStateCreateInfo {
            s_type: vk::StructureType::PipelineViewportStateCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            viewport_count: 1,
            p_viewports: &self.viewport,
            scissor_count: 1,
            p_scissors: &self.scissor,
        }
    }
}

pub struct ColorBlend {
    attachments: [vk::PipelineColorBlendAttachmentState; 1],
}

impl ColorBlend {
    pub fn new() -> Self {
        ColorBlend {
            attachments: [vk::PipelineColorBlendAttachmentState {
                blend_enable: 0,
                src_color_blend_factor: vk::BlendFactor::SrcColor,
                dst_color_blend_factor: vk::BlendFactor::OneMinusDstColor,
                color_blend_op: vk::BlendOp::Add,
                src_alpha_blend_factor: vk::BlendFactor::Zero,
                dst_alpha_blend_factor: vk::BlendFactor::Zero,
                alpha_blend_op: vk::BlendOp::Add,
                color_write_mask: vk::ColorComponentFlags::all(),
            }],
        }
    }

    pub fn create(self) -> vk::PipelineColorBlendStateCreateInfo {
        vk::PipelineColorBlendStateCreateInfo {
            s_type: vk::StructureType::PipelineColorBlendStateCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            logic_op_enable: 0,
            logic_op: vk::LogicOp::Copy,
            attachment_count: self.attachments.len() as u32,
            p_attachments: self.attachments.as_ptr(),
            blend_constants: [0.0, 0.0, 0.0, 0.0],
        }
    }
}

pub struct RenderPass {
    pub render_pass: vk::RenderPass,
}

impl RenderPass {
    pub fn new(device: &Device<V1_0>, surface_format: vk::SurfaceFormatKHR) -> Self {
        let color_attachment = vk::AttachmentDescription {
            format: surface_format.format,
            flags: vk::AttachmentDescriptionFlags::empty(),
            samples: vk::SAMPLE_COUNT_1_BIT,
            load_op: vk::AttachmentLoadOp::Clear,
            store_op: vk::AttachmentStoreOp::Store,
            stencil_load_op: vk::AttachmentLoadOp::DontCare,
            stencil_store_op: vk::AttachmentStoreOp::DontCare,
            initial_layout: vk::ImageLayout::Undefined,
            final_layout: vk::ImageLayout::PresentSrcKhr,
        };

        let color_attachment_ref = vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::ColorAttachmentOptimal,
        };

        let subpass_description = vk::SubpassDescription {
            flags: Default::default(),
            pipeline_bind_point: vk::PipelineBindPoint::Graphics,
            color_attachment_count: 1,
            p_color_attachments: &color_attachment_ref,
            input_attachment_count: 0,
            p_input_attachments: ptr::null(),
            p_resolve_attachments: ptr::null(),
            p_depth_stencil_attachment: ptr::null(),
            preserve_attachment_count: 0,
            p_preserve_attachments: ptr::null(),
        };

        let dependency = vk::SubpassDependency {
            dependency_flags: Default::default(),
            src_subpass: vk::VK_SUBPASS_EXTERNAL,
            dst_subpass: 0,
            src_stage_mask: vk::PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT,
            src_access_mask: Default::default(),
            dst_stage_mask: vk::PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT,
            dst_access_mask: vk::ACCESS_COLOR_ATTACHMENT_READ_BIT
                | vk::ACCESS_COLOR_ATTACHMENT_WRITE_BIT,
        };

        let render_pass = vk::RenderPassCreateInfo {
            s_type: vk::StructureType::RenderPassCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            attachment_count: 1,
            p_attachments: &color_attachment,
            subpass_count: 1,
            p_subpasses: &subpass_description,
            dependency_count: 1,
            p_dependencies: [dependency].as_ptr(),
        };

        let render_pass = unsafe {
            device
                .create_render_pass(&render_pass, None)
                .expect("unable to create renderpass")
        };

        RenderPass { render_pass }
    }
}

pub struct PipelineLayout {
    pub layout: vk::PipelineLayout,
}

impl PipelineLayout {
    pub fn empty(device: &Device<V1_0>) -> Self {
        let layout_create_info = vk::PipelineLayoutCreateInfo {
            s_type: vk::StructureType::PipelineLayoutCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            set_layout_count: 0,
            p_set_layouts: ptr::null(),
            push_constant_range_count: 0,
            p_push_constant_ranges: ptr::null(),
        };

        let pipeline_layout = unsafe {
            device
                .create_pipeline_layout(&layout_create_info, None)
                .unwrap()
        };

        PipelineLayout {
            layout: pipeline_layout,
        }
    }
}

#[derive(Debug)]
pub enum ShaderType {
    Vertex,
    Fragment,
}

impl ShaderType {
    pub fn to_vulkan(&self) -> vk::ShaderStageFlags {
        match self {
            ShaderType::Vertex => vk::SHADER_STAGE_VERTEX_BIT,
            ShaderType::Fragment => vk::SHADER_STAGE_FRAGMENT_BIT,
        }
    }
}
