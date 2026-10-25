use crate::app::engine::{
    core::{MAX_FRAMES_IN_FLIGHT, VulkanEngine},
    draw::draw_frame_error::DrawFrameError,
};
use ash::khr::swapchain;
use ash::vk;
use glam::Mat4;
use std::time::Instant;

impl VulkanEngine {
    pub fn draw_frame(&mut self) -> Result<(), DrawFrameError> {
        if self.swapchain_out_of_data {
            return Ok(());
        }
        //保证有效性
        if self.height == 0 || self.width == 0 {
            return Err(DrawFrameError::InvalidSize);
        } else if self.swapchain.is_none() {
            return Err(DrawFrameError::NoSwapchain);
        }

        let frame_index = self.current_frame;
        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;

        let command_buffer = self.command_buffers[frame_index];
        let device = self.device.as_ref().unwrap();
        let fence = &self.in_flight_fences[frame_index];

        //等待要被使用的栅栏完成
        if let Err(e) =
            unsafe { device.wait_for_fences(std::slice::from_ref(fence), true, u64::MAX) }
        {
            return Err(DrawFrameError::Vulkan(e));
        }

        //等待获取下一帧图片索引
        let (image_index, _) = unsafe {
            let swapchain_loader = swapchain::Device::new(self.instance.as_ref().unwrap(), device);
            match swapchain_loader.acquire_next_image(
                self.swapchain.unwrap(),
                u64::MAX,
                self.image_available_semaphores[frame_index],
                vk::Fence::null(),
            ) {
                Ok(result) => result,
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                    //让调用者重建
                    return Err(DrawFrameError::SwapchainOutOfDate);
                }
                Err(e) => {
                    self.swapchain_out_of_data = true;
                    return Err(DrawFrameError::Vulkan(e));
                }
            }
        };

        //完成后，重置栅栏，手动变成未触发状态
        if let Err(e) = unsafe { device.reset_fences(std::slice::from_ref(fence)) } {
            return Err(DrawFrameError::Vulkan(e));
        }

        let render_finished = &self.render_finished_semaphores[image_index as usize];

        //重置并录制命令缓冲
        if let Err(e) = unsafe {
            device.reset_command_buffer(command_buffer, vk::CommandBufferResetFlags::empty())
        } {
            return Err(DrawFrameError::Vulkan(e));
        }

        //提交命令缓冲信息
        let begin_info = vk::CommandBufferBeginInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
            p_next: std::ptr::null(),
            flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
            p_inheritance_info: std::ptr::null(),
            ..Default::default()
        };
        //置于录制状态，只提交一次
        if let Err(e) = unsafe { device.begin_command_buffer(command_buffer, &begin_info) } {
            return Err(DrawFrameError::Vulkan(e));
        }

        //开始渲染通道
        let render_pass = self.render_pass.unwrap();
        let framebuffer = match self.framebuffers.get(image_index as usize) {
            Some(v) => v,
            None => {
                return Err(DrawFrameError::OtherError(
                    "Framebuffer index out of bounds".to_string(),
                ));
            }
        };
        let clear_color = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 1.0], //黑色
            },
        };
        let render_area = vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: vk::Extent2D {
                width: self.width,
                height: self.height,
            },
        };
        let render_pass_begin = vk::RenderPassBeginInfo {
            s_type: vk::StructureType::RENDER_PASS_BEGIN_INFO,
            p_next: std::ptr::null(),
            render_pass,
            framebuffer: *framebuffer,
            render_area,
            clear_value_count: 1,
            p_clear_values: std::slice::from_ref(&clear_color).as_ptr(),
            ..Default::default()
        };

        unsafe {
            device.cmd_begin_render_pass(
                command_buffer,
                &render_pass_begin,
                vk::SubpassContents::INLINE,
            );
        }

        //绑定图形管线
        unsafe {
            device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline.unwrap(),
            );
        }

        //动态设置视口
        let viewport = vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: self.width as f32,
            height: self.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        };
        unsafe { device.cmd_set_viewport(command_buffer, 0, &[viewport]) };

        //动态设置裁剪矩阵
        let scissor = vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: vk::Extent2D {
                width: self.width,
                height: self.height,
            },
        };
        unsafe { device.cmd_set_scissor(command_buffer, 0, &[scissor]) };

        //计算时间
        let now = Instant::now();
        let delta_time = now - self.last_frame_time;
        self.last_frame_time = now;
        self.total_time += delta_time.as_secs_f32();

        for obj in &self.objects {
            let angle = self.total_time * obj.rotation_speed;
            let model = Mat4::from_translation(obj.center) * Mat4::from_rotation_z(angle);
            let model_matrix = model.to_cols_array();

            //推常量
            let model_bytes = unsafe {
                std::slice::from_raw_parts(
                    model_matrix.as_ptr() as *const u8,
                    std::mem::size_of_val(&model_matrix),
                )
            };
            unsafe {
                device.cmd_push_constants(
                    command_buffer,
                    self.pipeline_layout.unwrap(),
                    vk::ShaderStageFlags::VERTEX,
                    0,
                    model_bytes,
                );
            }

            //绑定顶点缓冲
            let vert_buffers = [obj.mesh.vertices.buffer];
            let vert_offsets = [0u64];
            unsafe {
                device.cmd_bind_vertex_buffers(command_buffer, 0, &vert_buffers, &vert_offsets);
            }

            //绑定索引缓冲
            unsafe {
                device.cmd_bind_index_buffer(
                    command_buffer,
                    obj.mesh.indices.buffer,
                    0,
                    vk::IndexType::UINT32,
                );
            }

            //索引绘制
            unsafe {
                device.cmd_draw_indexed(
                    command_buffer,
                    obj.mesh.index_count, //索引数量
                    1,                    //实例数
                    0,                    //first_index
                    0,                    //vertex_offset
                    0,                    //first_instance
                );
            }
        }

        //结束渲染管道和命令缓冲
        if let Err(e) = unsafe {
            device.cmd_end_render_pass(command_buffer);
            device.end_command_buffer(command_buffer)
        } {
            return Err(DrawFrameError::Vulkan(e));
        }

        //提交命令缓冲
        let submit_info = vk::SubmitInfo {
            s_type: vk::StructureType::SUBMIT_INFO,
            p_next: std::ptr::null(),
            //在命令开始执行前，等待这些信号量被触发，保证交换链已就绪
            wait_semaphore_count: 1,
            p_wait_semaphores: &self.image_available_semaphores[frame_index],
            //表示图像只有在渲染到达颜色附件阶段时才认为已就绪
            p_wait_dst_stage_mask: &vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            command_buffer_count: 1,
            p_command_buffers: &command_buffer as *const _,
            //执行完命令触发这些信号量
            signal_semaphore_count: 1,
            p_signal_semaphores: render_finished,
            ..Default::default()
        };

        if let Err(e) = unsafe { device.queue_submit(self.graphics_queue, &[submit_info], *fence) }
        {
            return Err(DrawFrameError::Vulkan(e));
        }

        //呈现到屏幕
        let present_info = vk::PresentInfoKHR {
            s_type: vk::StructureType::PRESENT_INFO_KHR,
            p_next: std::ptr::null(),
            wait_semaphore_count: 1,
            p_wait_semaphores: render_finished,
            swapchain_count: 1,
            p_swapchains: &self.swapchain.unwrap(),
            p_image_indices: &image_index,
            p_results: std::ptr::null_mut(),
            ..Default::default()
        };

        unsafe {
            let swapchain_loader = swapchain::Device::new(self.instance.as_ref().unwrap(), device);
            match swapchain_loader.queue_present(self.graphics_queue, &present_info) {
                Ok(result) => result,
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                    return Err(DrawFrameError::SwapchainOutOfDate);
                }
                Err(vk::Result::SUBOPTIMAL_KHR) => {
                    return Err(DrawFrameError::SwapchainSuboptimal);
                }
                Err(e) => return Err(DrawFrameError::Vulkan(e)),
            }
        };

        Ok(())
    }
}
