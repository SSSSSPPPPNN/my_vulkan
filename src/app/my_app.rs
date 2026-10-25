use std::sync::{Arc, atomic::AtomicBool};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use tao::window::Window;
use tokio::sync::mpsc::{Receiver, Sender};

use super::command::RenderCommand;
use crate::config::{Config, load_config};

pub struct App {
    pub window: Option<Window>,
    pub config: Config,

    pub sender: Option<Sender<RenderCommand>>,
    pub join_handle: Option<JoinHandle<()>>,

    //防抖
    pub debounce_delay: Duration,
    pub pending_size: Option<(u32, u32)>,
    pub last_resize_time: Instant,

    pub error_rx: Option<Receiver<String>>, //接受线程错误
    pub should_exit: Arc<AtomicBool>,
}

impl App {
    pub fn new() -> Self {
        let config = load_config();

        let debounce_ms = config.resize_debounce_ms;
        let debounce_delay = Duration::from_millis(debounce_ms as u64);

        App {
            window: None,
            config: config.clone(),
            sender: None,
            join_handle: None,
            debounce_delay,
            pending_size: None,
            last_resize_time: Instant::now(),
            error_rx: None,
            should_exit: Arc::new(AtomicBool::new(false)),
        }
    }
}
