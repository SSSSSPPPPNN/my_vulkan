//!创建图像视图

use anyhow::Context;
use ash::{Device, vk};

pub fn create_image_views(
    device: &Device,
    images: &[vk::Image],
    format: vk::Format,
) -> anyhow::Result<Vec<vk::ImageView>> {
    images
        .iter()
        .map(|&image| {
            let view_info = vk::ImageViewCreateInfo {
                s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
                p_next: std::ptr::null(),
                flags: vk::ImageViewCreateFlags::empty(),
                image,
                view_type: vk::ImageViewType::TYPE_2D,
                format,
                components: vk::ComponentMapping {
                    r: vk::ComponentSwizzle::IDENTITY,
                    g: vk::ComponentSwizzle::IDENTITY,
                    b: vk::ComponentSwizzle::IDENTITY,
                    a: vk::ComponentSwizzle::IDENTITY,
                },
                subresource_range: vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                },
                ..Default::default()
            };

            unsafe { device.create_image_view(&view_info, None) }
                .context("Failed to create image view")
        })
        .collect()
}
