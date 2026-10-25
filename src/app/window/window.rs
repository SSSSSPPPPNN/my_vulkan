use std::sync::atomic::Ordering;
use crate::{
    app::{
        command::RenderCommand,
        engine::{create_draw_frame_handle, VulkanEngine},
        my_app::App,
    }
};
use log::{error, info};
use std::time::Instant;
use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use tokio::sync::mpsc;

pub fn run_app(mut app: App) {
    //创建事件循环和窗口
    let event_loop = EventLoop::new();

    let (width, height) = (app.config.width, app.config.height);

    let window_attributes = WindowBuilder::new()
        .with_title(&app.config.title)
        .with_inner_size(tao::dpi::PhysicalSize::new(width, height))
        .with_resizable(app.config.resizable)
        .with_visible(true);

    let window = match window_attributes.build(&event_loop) {
        Ok(w) => w,
        Err(e) => {
            error!("创建窗口出现错误 {:#}", e);
            return;
        }
    };

    info!("创建窗口成功，尺寸: {}*{}", width, height);

    //创建引擎
    info!("正在创建引擎");
    let engine = match VulkanEngine::new(&window, width, height) {
        Ok(e) => {
            info!("创建引擎成功");
            e
        }
        Err(err) => {
            error!("创建引擎出现错误 {:#}", err);
            return;
        }
    };

    info!("起始渲染线程");
    let (tx, rx) = mpsc::channel::<RenderCommand>(32);
    let (error_tx, error_rx) = mpsc::channel::<String>(1);
    app.error_rx = Some(error_rx);

    info!("准备在渲染线程执行每帧绘画");
    let handle =
        create_draw_frame_handle(engine, rx, error_tx, app.config.clone(), &app.should_exit);

    app.sender = Some(tx);
    app.join_handle = Some(handle);
    app.window = Some(window);

    info!("进行事件循环");
    event_loop.run(move |event, _event_loop, control_flow| {
        //每次回调根据状态设置下次等待策略
        if app.pending_size.is_some() {
            let deadline = app.last_resize_time + app.debounce_delay;
            *control_flow = ControlFlow::WaitUntil(deadline);
        } else {
            *control_flow = ControlFlow::Wait;
        }

        // if app.should_exit.load(Ordering::Relaxed) {
        //     if let Some(tx) = &app.sender {
        //         let _ = tx.try_send(RenderCommand::Exit);
        //     }
        //     if let Some(handle) = app.join_handle.take() {
        //         let _ = handle.join();
        //     }
        //     *control_flow = ControlFlow::Exit;
        //     return;
        // }

        match event {
            //窗口事件
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    if let Some(tx) = &app.sender {
                        let _ = tx.try_send(RenderCommand::Exit);
                    }
                    app.should_exit.store(true, Ordering::Relaxed);
                    if let Some(handle) = app.join_handle.take() {
                        let _ = handle.join();
                    }
                    *control_flow = ControlFlow::Exit;
                    info!("程序安全退出\n");
                }

                WindowEvent::Resized(new_size) => {
                    app.pending_size = Some((new_size.width, new_size.height));
                    app.last_resize_time = Instant::now();
                }

                _ => {}
            },

            //准备请求重绘
            Event::MainEventsCleared => {
                //resize 稳定后再通知渲染线程
                if let Some((width, height)) = app.pending_size {
                    if app.last_resize_time.elapsed() >= app.debounce_delay {
                        if let Some(tx) = &app.sender {
                            let _ = tx.try_send(RenderCommand::Resized { width, height });
                        }
                        app.pending_size = None;
                    }
                }

                //检查渲染线程错误
                if let Some(error_rx) = &mut app.error_rx {
                    if let Ok(msg) = error_rx.try_recv() {
                        error!("渲染线程出错 {}", msg);
                        app.should_exit.store(true, Ordering::Relaxed);
                    }
                }

                //请求重绘
                if let Some(window) = &app.window {
                    window.request_redraw();
                }
            }

            // Event::RedrawRequested(_window_id) => {
            //     渲染线程负责实际绘制
            // }
            _ => {}
        }
    });
}
