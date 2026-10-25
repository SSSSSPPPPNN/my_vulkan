use anyhow::Context;
use ash::{Entry, vk, khr::surface};
use log::info;
use std::time::Instant;
use tao::window::Window;
use crate::{
    vulkan::{self, Mesh, Vertex},
    config::load_config,
    app::engine::{
        core::{MAX_FRAMES_IN_FLIGHT, VulkanEngine},
        render::RenderObject,
    },
};

impl VulkanEngine {
    pub fn new(window: &Window, width: u32, height: u32) -> anyhow::Result<Self> {
        info!("读取json");
        let config = load_config();

        info!("加载vulkan动态库获取入口函数表（运行时查找vulkan-1.dll）");
        let entry: Entry = unsafe { Entry::load().context("Failed to load Vulkan entry point") }?;

        info!("创建实例");
        let instance = vulkan::create_instance(&entry, config.validation_layers, window)?;

        info!("创建表面");
        let surface = vulkan::create_surface(&entry, &instance, window)?;

        info!("选择物理设备");
        let surface_loader = surface::Instance::new(&entry, &instance);
        let (physical_device, queue_family_index) =
            vulkan::pick_physical_device(&instance, &surface_loader, &surface)?;

        info!("创建逻辑设备");
        let (device, graphics_queue) =
            vulkan::create_logical_device(&instance, physical_device, queue_family_index)?;

        info!("创建交换链");
        let (swapchain, images, extent, format, swapchain_loader) = vulkan::create_swapchain(
            &entry,
            &instance,
            &device,
            physical_device,
            surface,
            width,
            height,
            config.clone(),
            vk::SwapchainKHR::null(),
        )?;
        info!("图像个数:{}", images.len());

        info!("创建交换链视图");
        let image_views = vulkan::create_image_views(&device, &images, format)?;

        info!("创建渲染管道");
        let render_pass = vulkan::create_render_pass(&device, format)?;

        info!("创建管线布局");
        let pipeline_layout = vulkan::create_pipeline_layout(&device)?;

        info!("加载shader模块");
        let (vert_module, frag_module) = vulkan::create_shader_modules(&device)?;

        info!("创建图形管线");
        let graphics_pipeline = vulkan::create_graphics_pipeline(
            &device,
            render_pass,
            pipeline_layout,
            vert_module,
            frag_module,
            extent,
        )?;
        info!("shader模块已传到图形管线内，销毁shader模块");
        unsafe {
            device.destroy_shader_module(vert_module, None);
            device.destroy_shader_module(frag_module, None);
        }

        info!("创建帧缓冲");
        let framebuffers = vulkan::create_framebuffers(&device, render_pass, &image_views, extent)?;

        info!("创建同步原语");
        let image_count = images.len();
        let frames = MAX_FRAMES_IN_FLIGHT;

        info!("创建命令缓冲和命令池");
        let mut command_pools = Vec::with_capacity(frames);
        let mut command_buffers = Vec::with_capacity(frames);

        for _ in 0..frames {
            let (pool, buffer) =
                vulkan::create_command_pool_and_buffer(&device, queue_family_index)?;

            command_buffers.push(buffer);
            command_pools.push(pool);
        }

        info!("创建信号量和栅栏");
        let mut image_available_semaphores = Vec::with_capacity(frames);
        let mut render_finished_semaphores = Vec::with_capacity(image_count);
        let mut in_flight_fences = Vec::with_capacity(frames);

        let sem_info = vk::SemaphoreCreateInfo::default();
        let fence_info = vk::FenceCreateInfo {
            flags: vk::FenceCreateFlags::SIGNALED,
            ..Default::default()
        };

        for _ in 0..frames {
            let sem = unsafe { device.create_semaphore(&sem_info, None) }
                .context("Failed to create semaphore")?;
            image_available_semaphores.push(sem);

            let fence = unsafe { device.create_fence(&fence_info, None) }
                .context("Failed to create fence")?;
            in_flight_fences.push(fence);
        }
        for _ in 0..image_count {
            let sem = unsafe { device.create_semaphore(&sem_info, None) }
                .context("Failed to create semaphore")?;
            render_finished_semaphores.push(sem);
        }

        info!("准备顶点数据");
        let left_verts = [
            Vertex::new([-0.25, 0.25, 0.0], [1.0, 0.0, 0.0]),
            Vertex::new([0.25, 0.25, 0.0], [1.0, 0.0, 0.0]),
            Vertex::new([0.25, -0.25, 0.0], [1.0, 0.0, 0.0]),
            Vertex::new([-0.25, -0.25, 0.0], [1.0, 0.0, 0.0]),
        ];

        let right_verts = [
            Vertex::new([-0.25, 0.25, 0.0], [0.0, 0.0, 1.0]),
            Vertex::new([0.25, 0.25, 0.0], [0.0, 0.0, 1.0]),
            Vertex::new([0.25, -0.25, 0.0], [0.0, 0.0, 1.0]),
            Vertex::new([-0.25, -0.25, 0.0], [0.0, 0.0, 1.0]),
        ];

        let rect_indices: [u32; 6] = [0, 1, 2, 2, 3, 0];

        let left_mesh = Mesh::new(
            &device,
            &instance,
            physical_device,
            &left_verts,
            &rect_indices,
        )?;
        let right_mesh = Mesh::new(
            &device,
            &instance,
            physical_device,
            &right_verts,
            &rect_indices,
        )?;

        let objects = vec![
            RenderObject::new(
                left_mesh,
                glam::Vec3::new(-0.5, 0.0, 0.0),
                config.rotation_speed[0],
            ), // 顺时针
            RenderObject::new(
                right_mesh,
                glam::Vec3::new(0.5, 0.0, 0.0),
                -config.rotation_speed[1],
            ), // 逆时针
        ];

        info!("全部创建完毕，new函数准备返回VulkanEngine实例");
        Ok(VulkanEngine {
            entry: Some(entry),
            instance: Some(instance),
            surface: Some(surface),
            surface_loader: Some(surface_loader),
            physical_device,
            device: Some(device),
            graphics_queue,
            swapchain_loader: Some(swapchain_loader),
            swapchain: Some(swapchain),
            swapchain_image_views: image_views,
            render_pass: Some(render_pass),
            pipeline_layout: Some(pipeline_layout),
            pipeline: Some(graphics_pipeline),
            framebuffers,
            command_pools,
            command_buffers,
            image_available_semaphores,
            render_finished_semaphores,
            in_flight_fences,
            current_frame: 0,
            objects,
            width,
            height,
            last_frame_time: Instant::now(),
            total_time: 0f32,
            swapchain_out_of_data: false,
        })
    }
}
