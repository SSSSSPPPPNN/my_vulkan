//渲染错误
#[derive(thiserror::Error, Debug)]
pub enum DrawFrameError {
    #[error("窗口尺寸无效")]
    InvalidSize,

    #[error("无交换链")]
    NoSwapchain,

    #[error("交换链已过期，须重建")]
    SwapchainOutOfDate,

    #[error("交换链次优，须重建")]
    SwapchainSuboptimal,

    #[error("Vulkan 调用失败: {0:?}")]
    Vulkan(ash::vk::Result),

    #[error("其他错误: {0:?}")]
    OtherError(String),
}

impl DrawFrameError {
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::SwapchainOutOfDate | Self::SwapchainSuboptimal | Self::InvalidSize
        )
    }
}
