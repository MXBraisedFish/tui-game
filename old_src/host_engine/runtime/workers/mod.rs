//! 后台工作线程服务。
//!
//! 工作线程只负责昂贵或阻塞任务，并通过 `EngineEvent` 把不可变结果交回主线程。
//! RuntimeWorld/状态机/包管理器的实际修改仍必须在主线程完成。

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::host_engine::boot::preload::init_environment::HostInputEvent;
use crate::host_engine::package::package_id::PackageKind;
use crate::host_engine::package::package_manager::PackageManager;
use crate::host_engine::runtime::event_dispatch::EngineEvent;

const WORKER_RECV_TIMEOUT_MS: u64 = 50;

/// Runtime 后台工作线程池。
pub struct WorkerPool {
    input_poller: Option<JoinHandle<()>>,
    input_shutdown: Option<Arc<AtomicBool>>,
    package_scanner: Option<JoinHandle<()>>,
    image_loader: Option<JoinHandle<()>>,
    event_sender: Sender<EngineEvent>,
    event_receiver: Receiver<EngineEvent>,
    shutdown: Arc<AtomicBool>,
}

impl WorkerPool {
    pub fn new() -> Self {
        let (event_sender, event_receiver) = mpsc::channel();
        Self {
            input_poller: None,
            input_shutdown: None,
            package_scanner: None,
            image_loader: None,
            event_sender,
            event_receiver,
            shutdown: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn start_input_poller(&mut self, input_receiver: Receiver<HostInputEvent>) {
        self.stop_input_poller();
        let sender = self.event_sender.clone();
        let input_shutdown = Arc::new(AtomicBool::new(false));
        let thread_shutdown = Arc::clone(&input_shutdown);
        self.input_shutdown = Some(input_shutdown);
        self.input_poller = thread::Builder::new()
            .name("tg-input-poller".to_string())
            .spawn(move || {
                while !thread_shutdown.load(Ordering::Relaxed) {
                    match input_receiver.recv_timeout(Duration::from_millis(WORKER_RECV_TIMEOUT_MS))
                    {
                        Ok(event) => {
                            if sender.send(EngineEvent::from(event)).is_err() {
                                break;
                            }
                        }
                        Err(RecvTimeoutError::Timeout) => {}
                        Err(RecvTimeoutError::Disconnected) => break,
                    }
                }
            })
            .ok();
    }

    pub fn scan_packages_in_background(&mut self, kind: PackageKind) {
        self.join_package_scanner();
        let sender = self.event_sender.clone();
        let shutdown = Arc::clone(&self.shutdown);
        self.package_scanner = Some(thread::spawn(move || {
            if shutdown.load(Ordering::Relaxed) {
                return;
            }
            PackageManager::scan_in_background(kind, sender);
        }));
    }

    pub fn start_image_loader<F>(&mut self, task: F)
    where
        F: FnOnce(Sender<EngineEvent>) + Send + 'static,
    {
        self.join_image_loader();
        let sender = self.event_sender.clone();
        let shutdown = Arc::clone(&self.shutdown);
        self.image_loader = Some(thread::spawn(move || {
            if !shutdown.load(Ordering::Relaxed) {
                task(sender);
            }
        }));
    }

    pub fn event_receiver(&self) -> &Receiver<EngineEvent> {
        &self.event_receiver
    }

    pub fn event_sender(&self) -> Sender<EngineEvent> {
        self.event_sender.clone()
    }

    pub fn stop(mut self) {
        self.shutdown.store(true, Ordering::Relaxed);
        self.join_all();
    }

    fn stop_input_poller(&mut self) {
        if let Some(input_shutdown) = &self.input_shutdown {
            input_shutdown.store(true, Ordering::Relaxed);
        }
        if let Some(handle) = self.input_poller.take() {
            let _ = handle.join();
        }
        self.input_shutdown = None;
    }

    fn join_package_scanner(&mut self) {
        if let Some(handle) = self.package_scanner.take() {
            let _ = handle.join();
        }
    }

    fn join_image_loader(&mut self) {
        if let Some(handle) = self.image_loader.take() {
            let _ = handle.join();
        }
    }

    fn join_all(&mut self) {
        if let Some(handle) = self.input_poller.take() {
            if let Some(input_shutdown) = &self.input_shutdown {
                input_shutdown.store(true, Ordering::Relaxed);
            }
            let _ = handle.join();
        }
        self.input_shutdown = None;
        if let Some(handle) = self.package_scanner.take() {
            let _ = handle.join();
        }
        self.join_image_loader();
    }
}

impl Default for WorkerPool {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for WorkerPool {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Relaxed);
        self.join_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_poller_forwards_host_input_events() {
        let (input_sender, input_receiver) = mpsc::channel();
        let mut workers = WorkerPool::new();
        workers.start_input_poller(input_receiver);

        input_sender
            .send(HostInputEvent::Key {
                key: "enter".to_string(),
                status: "press".to_string(),
            })
            .unwrap();

        let event = workers
            .event_receiver()
            .recv_timeout(Duration::from_secs(1))
            .unwrap();
        assert_eq!(
            event,
            EngineEvent::Key {
                key: "enter".to_string(),
                status: "press".to_string()
            }
        );

        workers.stop();
    }

    #[test]
    fn package_scanner_emits_typed_refresh_event() {
        let mut workers = WorkerPool::new();
        workers.scan_packages_in_background(PackageKind::UiPack);

        let event = workers
            .event_receiver()
            .recv_timeout(Duration::from_secs(1))
            .unwrap();
        assert_eq!(event, EngineEvent::PackagesRefreshed(PackageKind::UiPack));

        workers.stop();
    }

    #[test]
    fn image_loader_task_can_emit_engine_events() {
        let mut workers = WorkerPool::new();
        workers.start_image_loader(|sender| {
            let _ = sender.send(EngineEvent::Tick(1));
        });

        let event = workers
            .event_receiver()
            .recv_timeout(Duration::from_secs(1))
            .unwrap();
        assert_eq!(event, EngineEvent::Tick(1));

        workers.stop();
    }
}
