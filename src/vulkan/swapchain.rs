//!创建交换链

use crate::config::Config;
use anyhow::Context;
use ash::khr::{surface, swapchain};
use ash::{Device, Entry, Instance, vk};
use log::warn;

pub fn create_swapchain(
    entry: &Entry,                       //用于查询表面能力
    instance: &Instance,                 //用于查询表面能力
    device: &Device,                     //用于创建交换链
    physical_device: vk::PhysicalDevice, //用于查询能力
    surface: vk::SurfaceKHR,             //表面句柄
    width: u32,
    height: u32,
    config: Config,
    old_swapchain: vk::SwapchainKHR,
) -> anyhow::Result<(
    vk::SwapchainKHR,
    Vec<vk::Image>,
    vk::Extent2D,
    vk::Format,
    swapchain::Device,
)> {
    //创建surface加载器并查询能力
    let surface_loader = surface::Instance::new(entry, instance);
    //查询表面能力
    let capabilities = unsafe {
        surface_loader.get_physical_device_surface_capabilities(physical_device, surface)
    }
    .context("Failed to get surface capabilities")?;

    //保证有效性
    if capabilities.current_extent.height == 0 || capabilities.current_extent.width == 0 {
        return Err(anyhow::anyhow!("无效尺寸"));
    }

    //查询支持的格式
    let surface_formats =
        unsafe { surface_loader.get_physical_device_surface_formats(physical_device, surface) }
            .context("Failed to get surface formats")?;
    //查询支持的呈现格式
    let present_modes = unsafe {
        surface_loader.get_physical_device_surface_present_modes(physical_device, surface)
    }
    .context("Failed to get present modles")?;

    //选择表面格式优先选择 B8G8R8A8_SRGB 和 SRGB_NONLINEAR 颜色空间，这是最常见的 LCD 显示器标准
    let surface_format = surface_formats
        .iter()
        .find(|f| {
            f.format == vk::Format::B8G8R8A8_SRGB
                && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
        })
        .or_else(|| {
            warn!("没有使用B8G8R8A8_SRGB和SRGB_NONLINEAR颜色空间，使用第一个默认颜色空间");
            surface_formats.first()
        })
        .context("No suitable surface format")?;

    //选择呈现格式
    let vertical_sync = config.vertical_sync;
    let present_mode = present_modes
        .iter()
        .find(|&&m| {
            let mode = if vertical_sync {
                //vk::PresentModeKHR::MAILBOX
                vk::PresentModeKHR::FIFO
            } else {
                vk::PresentModeKHR::IMMEDIATE
            };
            m == mode
        })
        .or_else(|| {
            present_modes
                .iter()
                .find(|&&m| m == vk::PresentModeKHR::FIFO)
        })
        .unwrap_or(&vk::PresentModeKHR::FIFO)
        .clone();

    //确定交换链范围
    let swap_extent = if capabilities.current_extent.width == u32::MAX {
        //窗口系统允许应用程序自行决定尺寸
        vk::Extent2D {
            width: width.max(1),
            height: height.max(1),
        }
    } else {
        //固定模式
        capabilities.current_extent
    };

    //确定图像数量
    let mut image_count = capabilities.min_image_count + 1; //三缓冲
    if capabilities.max_image_count > 0 && image_count > capabilities.max_image_count {
        image_count = capabilities.max_image_count;
    }
    if image_count < 2 {
        image_count = 2;
    }

    //创建交换链
    let swapchain_create_info = vk::SwapchainCreateInfoKHR {
        s_type: vk::StructureType::SWAPCHAIN_CREATE_INFO_KHR,
        p_next: std::ptr::null(),
        flags: vk::SwapchainCreateFlagsKHR::empty(),
        surface,
        min_image_count: image_count,
        image_format: surface_format.format,
        image_color_space: surface_format.color_space,
        image_extent: swap_extent,
        image_array_layers: 1,
        image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
        image_sharing_mode: vk::SharingMode::EXCLUSIVE,
        queue_family_index_count: 0,
        p_queue_family_indices: std::ptr::null(),
        pre_transform: capabilities.current_transform,
        composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
        present_mode,
        clipped: vk::TRUE,
        old_swapchain,
        ..Default::default()
    };

    //加载交换链创建函数
    let swapchain_loader = swapchain::Device::new(instance, device);
    let swapchain = unsafe { swapchain_loader.create_swapchain(&swapchain_create_info, None) }
        .context("Failed to create Swapchain")?;

    //获取交换链中的图像
    let images = unsafe { swapchain_loader.get_swapchain_images(swapchain) }
        .context("Failed to get swapchain images")?;

    Ok((
        swapchain,
        images,
        swap_extent,
        surface_format.format,
        swapchain_loader,
    ))
}
