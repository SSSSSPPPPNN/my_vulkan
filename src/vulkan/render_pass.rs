//!渲染通道

use anyhow::Context;
use ash::{Device, vk};

pub fn create_render_pass(device: &Device, format: vk::Format) -> anyhow::Result<vk::RenderPass> {
    //定义颜色附件描述
    let color_attachment = vk::AttachmentDescription {
        flags: vk::AttachmentDescriptionFlags::empty(), //显式设置flags
        format,                                         //交换链查到的格式
        samples: vk::SampleCountFlags::TYPE_1,          //1个采样点(不开多重采样抗锯齿)
        load_op: vk::AttachmentLoadOp::CLEAR,           //开始渲染将图像清空为指定颜色
        store_op: vk::AttachmentStoreOp::STORE,         //渲染结束后将图像保存用于呈现
        stencil_load_op: vk::AttachmentLoadOp::DONT_CARE, //无模板缓冲忽略
        stencil_store_op: vk::AttachmentStoreOp::DONT_CARE, //无模板缓冲忽略
        initial_layout: vk::ImageLayout::UNDEFINED,     //不关心旧内容
        final_layout: vk::ImageLayout::PRESENT_SRC_KHR, //渲染结束让图像布局改为可呈现
    };

    //定义颜色附件引用
    let color_attachment_ref = vk::AttachmentReference {
        attachment: 0,                                     //一个附件，索引为零
        layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL, //这个子通道用作颜色附件，使用最优颜色布局
    };

    //定义子通道
    let subpass = vk::SubpassDescription {
        flags: vk::SubpassDescriptionFlags::empty(),
        pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS, //图形子通道
        input_attachment_count: 0,
        p_input_attachments: std::ptr::null(), //没有输入附件（0.38 字段名变更）
        color_attachment_count: 1,
        p_color_attachments: &color_attachment_ref, //传入颜色引用（0.38 字段名变更）
        p_resolve_attachments: std::ptr::null(),    //没有解析附件（0.38 字段名变更）
        p_depth_stencil_attachment: std::ptr::null(), //没有深度模板附件（0.38 字段名变更）
        preserve_attachment_count: 0,
        p_preserve_attachments: std::ptr::null(), //没有需要保留的附件（0.38 字段名变更）
        ..Default::default()
    };

    //组装渲染通道
    let render_pass_info = vk::RenderPassCreateInfo {
        s_type: vk::StructureType::RENDER_PASS_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::RenderPassCreateFlags::empty(),
        attachment_count: 1,
        p_attachments: &color_attachment, //附件描述
        subpass_count: 1,
        p_subpasses: &subpass,
        dependency_count: 0,
        p_dependencies: std::ptr::null(), //没有子通道依赖
        ..Default::default()
    };

    let result = unsafe { device.create_render_pass(&render_pass_info, None) }
        .context("Failed to create render pass")?;

    Ok(result)
}
