#[derive(thiserror::Error, Debug)]
pub enum RecreateError {
    #[error("交换链创建失败: {0}")]
    SwapchainCreateFailed(#[source] anyhow::Error),

    #[error("图像视图创建失败: {0}")]
    ImageViewCreateFailed(#[source] anyhow::Error),

    #[error("帧缓冲创建失败: {0}")]
    FramebufferCreateFailed(#[source] anyhow::Error),

    #[error("信号量创建失败: {0}")]
    SemaphoreCreateFailed(#[source] anyhow::Error),

    #[error("设备等待失败: {0:?}")]
    DeviceWaitIdleFailed(ash::vk::Result),
}
