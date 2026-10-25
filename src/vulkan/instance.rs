//!创建vulkan实例

use anyhow::Context;
use ash::ext::debug_utils;
use ash::vk::API_VERSION_1_0;
use ash::{Entry, Instance, vk};
use log::{info, warn};
use raw_window_handle::HasDisplayHandle;
use std::ffi::{CStr, c_char};
use tao::window::Window;

pub fn create_instance(
    entry: &Entry,
    enable_validation: bool,
    window: &Window,
) -> anyhow::Result<Instance> {
    //查询驱动支持的最大 vulkan 版本
    let api_version = unsafe {
        match entry.try_enumerate_instance_version() {
            Ok(Some(ver)) => {
                //成功获取
                info!(
                    "系统最高允许 vulkan 版本: {}.{}.{}",
                    vk::api_version_major(ver),
                    vk::api_version_minor(ver),
                    vk::api_version_patch(ver)
                );
                ver
            }
            Ok(None) => {
                warn!("驱动不支持查询版本，退回到 vulkan 1.0");
                API_VERSION_1_0
            }
            Err(e) => {
                warn!("查询失败 ({:?})，退回到 vulkan 1.0", e);
                API_VERSION_1_0
            }
        }
    };

    //创建vulkan实例
    let app_name_cstr =
        CStr::from_bytes_with_nul(b"Vulkan Rust Engine\0").expect("Invalid application name");
    let engine_name_cstr =
        CStr::from_bytes_with_nul(b"Rust Engine\0").expect("Invalid engine name");

    let app_name = vk::ApplicationInfo {
        s_type: vk::StructureType::APPLICATION_INFO,
        p_next: std::ptr::null(),
        p_application_name: app_name_cstr.as_ptr(),
        application_version: vk::make_api_version(0, 1, 0, 0),
        p_engine_name: engine_name_cstr.as_ptr(),
        engine_version: vk::make_api_version(0, 1, 0, 0),
        api_version,
        ..Default::default()
    };

    //验证层需要的调试扩展
    //使用ash-window 查询当前平台所需的扩展
    let mut extension_names: Vec<*const c_char> = Vec::new();

    //获取显示句柄和窗口句柄
    let display_handle = window
        .display_handle()
        .context("Failed to get display handle")?;

    //返回平台所需扩展列表
    let surface_extensions = ash_window::enumerate_required_extensions(display_handle.as_raw())
        .context("Failed to enumerate surface extensions")?;

    for ext in surface_extensions {
        extension_names.push(*ext)
    }

    //添加调试扩展
    if enable_validation {
        extension_names.push(debug_utils::NAME.as_ptr());
    }

    let layer_names: Vec<*const i8> = if enable_validation {
        vec![
            CStr::from_bytes_with_nul(b"VK_LAYER_KHRONOS_validation\0")
                .unwrap()
                .as_ptr(),
        ]
    } else {
        vec![]
    };

    let instance_create_info = vk::InstanceCreateInfo {
        s_type: vk::StructureType::INSTANCE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::InstanceCreateFlags::empty(),
        p_application_info: &app_name,
        enabled_layer_count: layer_names.len() as u32,
        pp_enabled_layer_names: layer_names.as_ptr(), // 0.38 字段名变更
        enabled_extension_count: extension_names.len() as u32,
        pp_enabled_extension_names: extension_names.as_ptr(), // 0.38 字段名变更
        ..Default::default()
    };

    let result = unsafe { entry.create_instance(&instance_create_info, None) }
        .context("Failed to create Vulkan instance")?;

    Ok(result)
}
