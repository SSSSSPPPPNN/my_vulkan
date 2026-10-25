//!创建帧缓冲

use anyhow::Context;
use ash::{Device, vk};

pub fn create_framebuffers(
    device: &Device,
    render_pass: vk::RenderPass,
    image_views: &[vk::ImageView],
    extent: vk::Extent2D,
) -> anyhow::Result<Vec<vk::Framebuffer>> {
    image_views
        .iter()
        .map(|&view| {
            let framebuffer_info = vk::FramebufferCreateInfo {
                s_type: vk::StructureType::FRAMEBUFFER_CREATE_INFO,
                p_next: std::ptr::null(),
                flags: vk::FramebufferCreateFlags::empty(),
                render_pass,
                attachment_count: 1,
                p_attachments: &view,
                width: extent.width,
                height: extent.height,
                layers: 1,
                ..Default::default()
            };

            unsafe { device.create_framebuffer(&framebuffer_info, None) }
                .context("Failed to create framebuffer")
        })
        .collect()
}
