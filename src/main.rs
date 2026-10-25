//release模式下隐藏控制台
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod vulkan;
mod memory;
mod config;
mod app;

use std::fs::OpenOptions;

use crate::app::{App, run_app};
use env_logger::{Builder, Target};
use log::{LevelFilter, info};

fn build() {
    //追加模式写入日志
    let log_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("engine.log")
        .expect("Failed to open log file");
    //构建并初始化日志器
    Builder::new()
        .target(Target::Pipe(Box::new(log_file))) // 将输出定向到文件
        .filter_level(LevelFilter::Info) // 设置日志级别
        .init();

    info!("程序开始运行");

    let app = App::new();

    info!("准备创建实例并运行");
    run_app(app);
}

fn main() {
    build();
}
