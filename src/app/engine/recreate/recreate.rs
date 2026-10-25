use crate::{
    config::Config,
    app::engine::{
        recreate::recreate_error::RecreateError,
        core::{VulkanEngine, MAX_FRAMES_IN_FLIGHT},
    },
    vulkan,
    
};

impl VulkanEngine {
    pub fn recreate_swapchain(
        &mut self,
        new_width: u32,
        new_height: u32,
        config: Config,
    ) -> Result<(), RecreateError> {
        //保证有效性
        if new_width == 0 || new_height == 0 {
            return Ok(());
        }
        //等待GPU空闲
        let device = self.device.as_ref().unwrap();
        if let Err(e) = unsafe { device.device_wait_idle() } {
            return Err(RecreateError::DeviceWaitIdleFailed(e));
        }

        let old_swapchain = self.swapchain.unwrap_or(ash::vk::SwapchainKHR::null());

        //创建新的交换链
        let result = vulkan::create_swapchain(
            self.entry.as_ref().unwrap(),
            self.instance.as_ref().unwrap(),
            device,
            self.physical_device,
            self.surface.unwrap(),
            new_width,
            new_height,
            config.clone(),
            old_swapchain,
        );
        let (new_swapchain, new_images, new_extent, format, _) = match result {
            Ok(v) => v,
            Err(e) => {
                return Err(RecreateError::SwapchainCreateFailed(e));
            }
        };

        //更新成员变量
        self.height = new_height;
        self.width = new_width;
        self.swapchain = Some(new_swapchain);

        //销毁
        //销毁旧的帧缓冲
        for fb in self.framebuffers.drain(..) {
            unsafe { device.destroy_framebuffer(fb, None) };
        }
        //销毁旧的图像视图
        for view in self.swapchain_image_views.drain(..) {
            unsafe { device.destroy_image_view(view, None) };
        }
        //销毁旧的交换链
        let swapchain_loader = self.swapchain_loader.as_ref().unwrap();
        if old_swapchain != ash::vk::SwapchainKHR::null() {
            unsafe { swapchain_loader.destroy_swapchain(old_swapchain, None) };
        }

        //重建图像视图
        self.swapchain_image_views = match vulkan::create_image_views(&device, &new_images, format)
        {
            Ok(swapchain) => swapchain,
            Err(e) => {
                return Err(RecreateError::ImageViewCreateFailed(e));
            }
        };

        //重建帧缓冲
        self.framebuffers = match vulkan::create_framebuffers(
            &device,
            self.render_pass.unwrap(),
            &self.swapchain_image_views,
            new_extent,
        ) {
            Ok(frame_buffer) => frame_buffer,
            Err(e) => {
                return Err(RecreateError::FramebufferCreateFailed(e));
            }
        };

        //销毁并重建信号量
        for &sem in &self.render_finished_semaphores {
            unsafe { device.destroy_semaphore(sem, None) };
        }
        for &sem in &self.image_available_semaphores {
            unsafe { device.destroy_semaphore(sem, None) };
        }

        self.render_finished_semaphores.clear();
        self.image_available_semaphores.clear();

        let sem_info = ash::vk::SemaphoreCreateInfo::default();

        for _ in 0..new_images.len() {
            let sem = match unsafe { device.create_semaphore(&sem_info, None) } {
                Ok(semaphore) => semaphore,
                Err(e) => {
                    return Err(RecreateError::SemaphoreCreateFailed(e.into()));
                }
            };
            self.render_finished_semaphores.push(sem);
        }
        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            let sem = unsafe { device.create_semaphore(&sem_info, None) }
                .map_err(|e| RecreateError::SemaphoreCreateFailed(e.into()))?;
            self.image_available_semaphores.push(sem);
        }
        self.swapchain_out_of_data = false;
        Ok(())
    }
}
