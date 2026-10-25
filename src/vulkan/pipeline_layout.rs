//!创建管线布局

use anyhow::Context;
use ash::{Device, vk};

pub fn create_pipeline_layout(device: &Device) -> anyhow::Result<vk::PipelineLayout> {
    //定义推常量范围
    let push_constant_range = vk::PushConstantRange {
        stage_flags: vk::ShaderStageFlags::VERTEX, //在顶点着色器中使用
        offset: 0,                                 //起始偏差
        size: std::mem::size_of::<[[f32; 4]; 4]>() as u32, //4x4矩阵大小
    };

    //创建管线布局信息
    let layout_info = vk::PipelineLayoutCreateInfo {
        s_type: vk::StructureType::PIPELINE_LAYOUT_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineLayoutCreateFlags::empty(),
        set_layout_count: 0,
        p_set_layouts: std::ptr::null(), // 没有描述符集布局（0.38 字段名变更）
        push_constant_range_count: 1,
        p_push_constant_ranges: &push_constant_range, // 0.38 字段名变更
        ..Default::default()
    };

    //调用VulkanAPI创建
    let result = unsafe { device.create_pipeline_layout(&layout_info, None) }
        .context("Failed to create pipeline")?;

    Ok(result)
}
