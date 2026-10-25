//!shader模块

use anyhow::Context;
use ash::{Device, vk};

pub fn create_shader_modules(
    device: &Device,
) -> anyhow::Result<(vk::ShaderModule, vk::ShaderModule)> {
    let vert_spv = include_bytes!("../shaders/vert.spv");
    let frag_spv = include_bytes!("../shaders/frag.spv");

    // 转换为 Vec<u32>（自动对齐到 4 字节）
    let vert_u32 = vert_spv
        .chunks_exact(4)
        .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect::<Vec<u32>>();
    let frag_u32 = frag_spv
        .chunks_exact(4)
        .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect::<Vec<u32>>();

    // 创建模块时使用 &[u32]
    let vert_module = unsafe {
        device.create_shader_module(
            &vk::ShaderModuleCreateInfo {
                s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
                p_next: std::ptr::null(),
                flags: vk::ShaderModuleCreateFlags::empty(),
                code_size: vert_u32.len() * 4, // 字节长度
                p_code: vert_u32.as_ptr(),
                ..Default::default()
            },
            None,
        )
    }
    .context("Failed to create vertex shader module")?;

    let frag_module = unsafe {
        device.create_shader_module(
            &vk::ShaderModuleCreateInfo {
                s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
                p_next: std::ptr::null(),
                flags: vk::ShaderModuleCreateFlags::empty(),
                code_size: frag_u32.len() * 4,
                p_code: frag_u32.as_ptr(),
                ..Default::default()
            },
            None,
        )
    }
    .context("Failed to create fragment shader module")?;

    Ok((vert_module, frag_module))
}
