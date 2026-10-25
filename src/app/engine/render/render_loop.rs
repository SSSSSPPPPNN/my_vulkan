use tokio::sync::mpsc::Sender;
use crate::app::{
    command::RenderCommand,
    engine::core::VulkanEngine,
};
use crate::config::Config;

pub async fn draw_frame_and_handle_errors(
    engine: &mut VulkanEngine,
    error_tx: &Sender<String>,
) -> bool {
    if let Err(e) = engine.draw_frame() {
        if !e.is_recoverable() {
            let _ = error_tx.send(format!("渲染出错 {:#}", e)).await;
            return true;
        }
    }
    false
}
pub async fn handle_command(
    cmd: RenderCommand,
    engine: &mut VulkanEngine,
    error_tx: &Sender<String>,
    config: Config,
) -> bool {
    match cmd {
        RenderCommand::Resized { width, height } => {
            if width == 0 || height == 0 {
                // 无效尺寸
                return false;
            }

            if let Err(e) = engine.recreate_swapchain(width, height, config) {
                let _ = error_tx.send(format!("重建失败 {:#}", e)).await;
                return true;
            }

            false
        }
        RenderCommand::Exit => true,
    }
}
