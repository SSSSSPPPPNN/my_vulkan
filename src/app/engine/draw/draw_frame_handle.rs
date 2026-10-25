use crate::config::Config;
use crate::app::{
    engine::{
        core::VulkanEngine,
        render::{draw_frame_and_handle_errors, handle_command},
    },
    command::RenderCommand,
};

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread::{JoinHandle, spawn};
use std::time::Duration;
use tokio::sync::mpsc::{Receiver, Sender};

pub fn create_draw_frame_handle(
    mut engine: VulkanEngine,
    mut rx: Receiver<RenderCommand>,
    error_tx: Sender<String>,
    config: Config,
    should_exit: &Arc<AtomicBool>,
) -> JoinHandle<()> {
    let should_exit = Arc::clone(should_exit);
    spawn(move || {
        let run_time = tokio::runtime::Runtime::new().unwrap();

        run_time.block_on(async {
            let mut interval = tokio::time::interval(Duration::from_secs_f32(1.0 / 60.0));

            if config.vertical_sync {
                loop {
                    tokio::select! {
                        _ = interval.tick() => {
                            if should_exit.load(Ordering::Relaxed) {
                                break;
                            }

                            if draw_frame_and_handle_errors(&mut engine, &error_tx).await {
                                break;
                            }
                        }
                        Some(cmd) = rx.recv() => {

                            if handle_command(cmd, &mut engine, &error_tx, config.clone()).await {
                                break;
                            }
                        }
                    }
                }
            } else {
                loop {
                    tokio::select! {
                        if_break = async {
                            draw_frame_and_handle_errors(&mut engine, &error_tx).await
                        } => {
                            if should_exit.load(Ordering::Relaxed) {
                                break;
                            }
                            if if_break {
                                break;
                            }
                        }
                        Some(cmd) = rx.recv() => {
                            if handle_command(cmd, &mut engine, &error_tx, config.clone()).await {
                                break;
                            }
                        }
                    }
                }
            }
        });
    })
}
