use ash::khr::{surface, swapchain};
use ash::{
    Device,   // 逻辑设备，包含选中的物理显卡，队列，负责创建所有GPU资源
    Entry,    // 加载vulkan-1.dll动态库，并获取底层函数
    Instance, // vulkan实例
    vk,       // vulkan所有常量，结构体，枚举
};
use std::time::Instant;
pub const MAX_FRAMES_IN_FLIGHT: usize = 2;
use crate::app::engine::render::RenderObject;

//内部引擎结构体
#[allow(dead_code)]
pub struct VulkanEngine {
    // vulkan根对象
    pub entry: Option<Entry>,
    pub instance: Option<Instance>,
    pub surface: Option<vk::SurfaceKHR>,
    pub surface_loader: Option<surface::Instance>, // 用于销毁surface

    // 设备与队列
    pub physical_device: vk::PhysicalDevice, // 原始句柄，无所有权，归instance管，仅用于查询，不用销毁
    pub device: Option<Device>,              // 所有资源的依赖
    pub graphics_queue: vk::Queue,           // 原始句柄，无所有权，归device管，只是提交命令的引用，不用销毁

    // 交换链及图像
    pub swapchain_loader: Option<swapchain::Device>, // 持有device的引用，用于调用destroy_swapchain（0.38 类型变更）
    pub swapchain: Option<vk::SwapchainKHR>,         // 原始句柄，需手动销毁
    pub swapchain_image_views: Vec<vk::ImageView>,   // 单独创建，须在Drop逐个销毁

    // 渲染管道，需要销毁
    pub render_pass: Option<vk::RenderPass>,
    pub pipeline_layout: Option<vk::PipelineLayout>,
    pub pipeline: Option<vk::Pipeline>,
    pub framebuffers: Vec<vk::Framebuffer>,

    // 命令缓冲
    pub command_pools: Vec<vk::CommandPool>, // 用于分配command_buffer
    pub command_buffers: Vec<vk::CommandBuffer>,

    // 同步原语,需要销毁
    pub image_available_semaphores: Vec<vk::Semaphore>,
    pub render_finished_semaphores: Vec<vk::Semaphore>,
    pub in_flight_fences: Vec<vk::Fence>,
    pub current_frame: usize, // 轮换索引

    pub objects: Vec<RenderObject>,

    // 窗口尺寸
    pub width: u32,
    pub height: u32,

    // 时间
    pub last_frame_time: Instant, // 记录上一帧时刻
    pub total_time: f32,          // 记录总时间

    pub swapchain_out_of_data: bool,
}

impl Drop for VulkanEngine {
    fn drop(&mut self) {
        // 获取device和instance的所有权
        let device = self.device.take().expect("Device already taken");
        let instance = self.instance.take().expect("Instance already taken");
        let surface = self.surface.take().expect("Surface already taken");

        // 等待设备空闲
        unsafe { device.device_wait_idle() }.expect("Device wait idle failed");

        // 销毁顶点数据
        for mut obj in self.objects.drain(..) {
            unsafe {
                obj.mesh.destroy(&device);
            }
        }

        // 帧缓冲
        for fb in self.framebuffers.drain(..) {
            unsafe { device.destroy_framebuffer(fb, None) };
        }

        // 图形管线
        if let Some(pipeline) = self.pipeline.take() {
            unsafe { device.destroy_pipeline(pipeline, None) };
        }

        // 管线布局
        if let Some(layout) = self.pipeline_layout.take() {
            unsafe { device.destroy_pipeline_layout(layout, None) };
        }

        // 渲染通道
        if let Some(rp) = self.render_pass.take() {
            unsafe { device.destroy_render_pass(rp, None) };
        }

        // 图像视图
        for view in self.swapchain_image_views.drain(..) {
            unsafe { device.destroy_image_view(view, None) };
        }

        // 命令池
        for &pool in &self.command_pools {
            unsafe { device.destroy_command_pool(pool, None) };
        }

        // 栅栏
        for &fence in &self.in_flight_fences {
            unsafe { device.destroy_fence(fence, None) }
        }

        // 销毁信号量
        for &sem in &self.image_available_semaphores {
            unsafe { device.destroy_semaphore(sem, None) };
        }
        for &sem in &self.render_finished_semaphores {
            unsafe { device.destroy_semaphore(sem, None) };
        }

        // 交换链
        if let Some(swapchain) = self.swapchain.take() {
            // 需要 Swapchain 加载器来销毁
            let swapchain_loader = swapchain::Device::new(&instance, &device);
            unsafe { swapchain_loader.destroy_swapchain(swapchain, None) };
        }

        // 表面（需要 Surface 加载器）
        let surface_loader = surface::Instance::new(self.entry.as_ref().unwrap(), &instance);
        unsafe { surface_loader.destroy_surface(surface, None) };

        // 设备
        unsafe { device.destroy_device(None) };

        // 实例
        unsafe { instance.destroy_instance(None) };
    }
}
