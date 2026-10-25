//!创建图形管线

use crate::vulkan::buffer::Vertex;
use ash::{Device, vk};
use std::mem::size_of;

pub fn create_graphics_pipeline(
    device: &Device,
    render_pass: vk::RenderPass,
    pipeline_layout: vk::PipelineLayout,
    vert_module: vk::ShaderModule,
    frag_module: vk::ShaderModule,
    extent: vk::Extent2D,
) -> anyhow::Result<vk::Pipeline> {
    let main_name = std::ffi::CString::new("main").unwrap();

    //着色器阶段
    let vert_stage = vk::PipelineShaderStageCreateInfo {
        s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineShaderStageCreateFlags::empty(),
        stage: vk::ShaderStageFlags::VERTEX,
        module: vert_module,
        p_name: main_name.as_ptr(),
        p_specialization_info: std::ptr::null(),
        ..Default::default()
    };
    let frag_stage = vk::PipelineShaderStageCreateInfo {
        s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineShaderStageCreateFlags::empty(),
        stage: vk::ShaderStageFlags::FRAGMENT,
        module: frag_module,
        p_name: main_name.as_ptr(),
        p_specialization_info: std::ptr::null(),
        ..Default::default()
    };

    let stages = [vert_stage, frag_stage];

    //顶点输入
    let binding_desc = vk::VertexInputBindingDescription {
        binding: 0,
        stride: size_of::<Vertex>() as u32,
        input_rate: vk::VertexInputRate::VERTEX,
    };

    let attr_pos = vk::VertexInputAttributeDescription {
        location: 0,
        binding: 0,
        format: vk::Format::R32G32B32_SFLOAT,
        offset: 0,
    };

    let attr_color = vk::VertexInputAttributeDescription {
        location: 1,
        binding: 0,
        format: vk::Format::R32G32B32_SFLOAT,
        offset: 12, // 3 * 4
    };

    let attr_descs = [attr_pos, attr_color];

    let vertex_input = vk::PipelineVertexInputStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineVertexInputStateCreateFlags::empty(),
        vertex_binding_description_count: 1,
        p_vertex_binding_descriptions: &binding_desc,
        vertex_attribute_description_count: attr_descs.len() as u32,
        p_vertex_attribute_descriptions: attr_descs.as_ptr(),
        ..Default::default()
    };

    //输入装配
    let input_assembly = vk::PipelineInputAssemblyStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineInputAssemblyStateCreateFlags::empty(),
        topology: vk::PrimitiveTopology::TRIANGLE_LIST,
        primitive_restart_enable: vk::FALSE,
        ..Default::default()
    };

    //视口和裁剪
    let viewport = vk::Viewport {
        x: 0.0,
        y: 0.0,
        width: extent.width as f32,
        height: extent.height as f32,
        min_depth: 0.0,
        max_depth: 1.0,
    };

    let scissor = vk::Rect2D {
        offset: vk::Offset2D { x: 0, y: 0 },
        extent,
    };

    let viewport_state = vk::PipelineViewportStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_VIEWPORT_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineViewportStateCreateFlags::empty(),
        viewport_count: 1,
        p_viewports: &viewport,
        scissor_count: 1,
        p_scissors: &scissor,
        ..Default::default()
    };

    //光栅化
    let rasterization = vk::PipelineRasterizationStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineRasterizationStateCreateFlags::empty(),
        depth_clamp_enable: vk::FALSE,
        rasterizer_discard_enable: vk::FALSE,
        polygon_mode: vk::PolygonMode::FILL,
        cull_mode: vk::CullModeFlags::NONE,
        front_face: vk::FrontFace::CLOCKWISE,
        depth_bias_enable: vk::FALSE,
        depth_bias_constant_factor: 0.0,
        depth_bias_clamp: 0.0,
        depth_bias_slope_factor: 0.0,
        line_width: 1.0,
        ..Default::default()
    };

    //多重采样
    let multisample = vk::PipelineMultisampleStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineMultisampleStateCreateFlags::empty(),
        rasterization_samples: vk::SampleCountFlags::TYPE_1,
        sample_shading_enable: vk::FALSE,
        min_sample_shading: 0.0,
        p_sample_mask: std::ptr::null(),
        alpha_to_coverage_enable: vk::FALSE,
        alpha_to_one_enable: vk::FALSE,
        ..Default::default()
    };

    //颜色混合
    let blend_attachment = vk::PipelineColorBlendAttachmentState {
        blend_enable: vk::FALSE,
        src_color_blend_factor: vk::BlendFactor::ZERO,
        dst_color_blend_factor: vk::BlendFactor::ZERO,
        color_blend_op: vk::BlendOp::ADD,
        src_alpha_blend_factor: vk::BlendFactor::ZERO,
        dst_alpha_blend_factor: vk::BlendFactor::ZERO,
        alpha_blend_op: vk::BlendOp::ADD,
        color_write_mask: vk::ColorComponentFlags::R
            | vk::ColorComponentFlags::G
            | vk::ColorComponentFlags::B
            | vk::ColorComponentFlags::A,
    };

    let color_blend = vk::PipelineColorBlendStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineColorBlendStateCreateFlags::empty(),
        logic_op_enable: vk::FALSE,
        logic_op: vk::LogicOp::COPY,
        attachment_count: 1,
        p_attachments: &blend_attachment,
        blend_constants: [0.0, 0.0, 0.0, 0.0],
        ..Default::default()
    };

    //定义动态状态
    let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
    let dynamic_state_info = vk::PipelineDynamicStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_DYNAMIC_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineDynamicStateCreateFlags::empty(),
        dynamic_state_count: dynamic_states.len() as u32,
        p_dynamic_states: dynamic_states.as_ptr(),
        ..Default::default()
    };

    //主创建信息
    let pipeline_info = vk::GraphicsPipelineCreateInfo {
        s_type: vk::StructureType::GRAPHICS_PIPELINE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineCreateFlags::empty(),
        stage_count: stages.len() as u32,
        p_stages: stages.as_ptr(),
        p_vertex_input_state: &vertex_input,
        p_input_assembly_state: &input_assembly,
        p_tessellation_state: std::ptr::null(),
        p_viewport_state: &viewport_state,
        p_rasterization_state: &rasterization,
        p_multisample_state: &multisample,
        p_depth_stencil_state: std::ptr::null(),
        p_color_blend_state: &color_blend,
        p_dynamic_state: &dynamic_state_info,
        layout: pipeline_layout,
        render_pass,
        subpass: 0,
        base_pipeline_handle: vk::Pipeline::null(),
        base_pipeline_index: -1,
        ..Default::default()
    };

    let pipelines = unsafe {
        device.create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info], None)
    };

    match pipelines {
        Ok(mut ps) => {
            let pipeline = ps.remove(0);
            Ok(pipeline)
        }
        Err((_, err)) => Err(anyhow::anyhow!(
            "Failed to create graphics pipelines: {:?}",
            err
        )),
    }
}
