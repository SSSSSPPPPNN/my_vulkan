//!创建vulkan表面

use anyhow::Context;
use ash::{Entry, Instance, vk};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use tao::window::Window;

///创建vulkan表面：建立vulkan与tao窗口的链接
pub fn create_surface(
    entry: &Entry,
    instance: &Instance,
    window: &Window,
) -> anyhow::Result<vk::SurfaceKHR> {
    let display_handle = window
        .display_handle()
        .expect("Failed to get display handle");
    let window_handle = window.window_handle().expect("Failed to get window handle");

    let result = unsafe {
        ash_window::create_surface(
            entry,
            instance,
            display_handle.as_raw(),
            window_handle.as_raw(),
            None,
        )
        .context("Failed to create surface")?
    };

    Ok(result)
}
