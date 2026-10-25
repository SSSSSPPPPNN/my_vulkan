//!创建命令池和命令缓冲

use anyhow::Context;
use ash::{Device, vk};

pub fn create_command_pool_and_buffer(
    device: &Device,
    queue_family_index: u32,
) -> anyhow::Result<(vk::CommandPool, vk::CommandBuffer)> {
    //创建命令池
    //flags的vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER意思是允许单独重置命令缓冲
    let pool_info = vk::CommandPoolCreateInfo {
        s_type: vk::StructureType::COMMAND_POOL_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
        queue_family_index,
        ..Default::default()
    };

    let command_pool = unsafe { device.create_command_pool(&pool_info, None) }
        .context("Failed to create command pool")?;

    //从池中分配一个主命令缓冲
    let alloc_info = vk::CommandBufferAllocateInfo {
        s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
        p_next: std::ptr::null(),
        command_pool,
        level: vk::CommandBufferLevel::PRIMARY, //主缓冲，可直接提交到队列
        command_buffer_count: 1,
        ..Default::default()
    };

    let command_buffer = unsafe { device.allocate_command_buffers(&alloc_info) }.unwrap()[0];

    Ok((command_pool, command_buffer))
}
