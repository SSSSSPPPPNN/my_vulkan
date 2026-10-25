//!创建逻辑设备

use anyhow::Context;
use ash::khr::swapchain;
use ash::{Device, Instance, vk};

pub fn create_logical_device(
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
    queue_family_index: u32,
) -> anyhow::Result<(Device, vk::Queue)> {
    //创建DeviceQueueCreateInfo
    let queue_priority = [1.0f32]; //队列优先级
    let queue_create_info = vk::DeviceQueueCreateInfo {
        s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::DeviceQueueCreateFlags::empty(),
        queue_family_index,
        queue_count: 1,
        p_queue_priorities: queue_priority.as_ptr(),
        ..Default::default()
    };

    //指定设备扩展（交换链扩展）
    let device_extension = [swapchain::NAME.as_ptr()];
    //创建设备创建信息
    let device_create_info = vk::DeviceCreateInfo {
        s_type: vk::StructureType::DEVICE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::DeviceCreateFlags::empty(),
        queue_create_info_count: 1,
        p_queue_create_infos: &queue_create_info,
        enabled_extension_count: device_extension.len() as u32,
        pp_enabled_extension_names: device_extension.as_ptr(),
        p_enabled_features: std::ptr::null(),
        // 以下两个字段已废弃，不再需要
        // enabled_layer_count: 0,
        // pp_enabled_layer_names: std::ptr::null(),
        ..Default::default()
    };

    //创建逻辑设备
    let device = unsafe { instance.create_device(physical_device, &device_create_info, None) }
        .context("Failed to create logical device")?;

    //获取图形队列，返回vk::Queue
    let graphics_queue = unsafe { device.get_device_queue(queue_family_index, 0) };

    Ok((device, graphics_queue))
}
