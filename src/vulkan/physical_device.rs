//!选择物理设备

use anyhow::Context;
use ash::khr::surface;
use ash::{Instance, vk};

pub fn pick_physical_device(
    instance: &Instance,
    surface_loader: &surface::Instance,
    surface: &vk::SurfaceKHR,
) -> anyhow::Result<(vk::PhysicalDevice, u32)> {
    //选择物理设备，枚举设备寻找合适的队列族
    let physical_devices = unsafe { instance.enumerate_physical_devices() }
        .context("Failed to enumerate physical devices")?;

    //查找支持图形和呈现的队列族，遍历设备句柄的引用，找到第一个满足的设备
    let (physical_device, queue_family_index) = physical_devices
        .iter()
        .find_map(|&pdev| {
            //查询队列族属性(vk::PhysicalDevice实现了copy类型)
            let queue_families =
                unsafe { instance.get_physical_device_queue_family_properties(pdev) };
            queue_families.iter().enumerate().find_map(|(i, family)| {
                if family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                    let supports_present = unsafe {
                        //查询是否支持呈现
                        surface_loader.get_physical_device_surface_support(pdev, i as u32, *surface)
                    }
                    .unwrap_or(false);
                    if supports_present {
                        log::info!("查询的队列族索引: {i}");
                        return Some((pdev, i as u32));
                    }
                }
                None
            })
        })
        .context("No suitable physical device found")?;
    Ok((physical_device, queue_family_index))
}
