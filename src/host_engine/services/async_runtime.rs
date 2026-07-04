use std::{
  collections::HashMap,
  fs,
  path::PathBuf,
  sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicU64, Ordering},
  },
  thread::{self, JoinHandle},
  time::Duration,
};

use crossbeam_channel::{Receiver, Sender, unbounded};

use super::{
  image::{ImageConvertParams, ImageService},
  input::{KeyEvent, SystemEvent},
  package::{self, PackageAsyncEvent, PackageTask},
  widget::runtime_object::time::TimeCallbackId,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TaskId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ManagedThreadId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskState {
  Pending,
  Running,
  Finished,
  Failed,
  Cancelled,
}

#[derive(Clone, Debug)]
pub enum FileTask {
  ReadText { path: PathBuf },
  WriteText { path: PathBuf, text: String },
  ReadBytes { path: PathBuf },
  WriteBytes { path: PathBuf, bytes: Vec<u8> },
}

#[derive(Clone, Debug)]
pub enum FileEvent {
  ReadTextFinished {
    task_id: TaskId,
    path: PathBuf,
    text: String,
  },
  WriteTextFinished {
    task_id: TaskId,
    path: PathBuf,
  },
  ReadBytesFinished {
    task_id: TaskId,
    path: PathBuf,
    bytes: Vec<u8>,
  },
  WriteBytesFinished {
    task_id: TaskId,
    path: PathBuf,
  },
  Failed {
    task_id: TaskId,
    path: PathBuf,
    error: String,
  },
}

#[derive(Clone, Debug)]
pub enum ImageTask {
  Convert {
    params: ImageConvertParams,
    cache_dir: Option<PathBuf>,
  },
}

#[derive(Clone, Debug)]
pub enum ImageEvent {
  ConvertFinished { task_id: TaskId, output: String },
  Failed { task_id: TaskId, error: String },
}

#[derive(Clone, Debug)]
pub enum NetworkTask {
  Get { url: String },
}

#[derive(Clone, Debug)]
pub enum NetworkEvent {
  GetFinished {
    task_id: TaskId,
    url: String,
    status: u16,
    body: String,
  },
  Failed {
    task_id: TaskId,
    url: String,
    error: String,
  },
}

#[derive(Clone, Debug)]
pub struct SleepTask {
  pub duration: Duration,
  pub callback: Option<TimeCallbackId>,
}

#[derive(Clone, Debug)]
pub enum TimeAsyncEvent {
  SleepFinished {
    task_id: TaskId,
    callback: Option<TimeCallbackId>,
  },
}

#[derive(Clone, Debug)]
pub enum EngineTask {
  Package(PackageTask),
  File(FileTask),
  Image(ImageTask),
  Network(NetworkTask),
  Sleep(SleepTask),
}

#[derive(Clone, Debug)]
pub enum EngineEvent {
  InputKey(KeyEvent),
  System(SystemEvent),
  Package(PackageAsyncEvent),
  File(FileEvent),
  Image(ImageEvent),
  Network(NetworkEvent),
  Time(TimeAsyncEvent),
  TaskFinished { id: TaskId },
  TaskFailed { id: TaskId, error: String },
}

enum WorkerMessage {
  Run(TaskId, EngineTask),
  Shutdown,
}

struct ManagedThread {
  stop: Arc<AtomicBool>,
  joinable: bool,
  handle: Option<JoinHandle<()>>,
}

pub struct AsyncRuntime {
  task_tx: Sender<WorkerMessage>,
  event_tx: Sender<EngineEvent>,
  event_rx: Receiver<EngineEvent>,
  workers: Vec<JoinHandle<()>>,
  task_states: Arc<Mutex<HashMap<TaskId, TaskState>>>,
  managed_threads: HashMap<ManagedThreadId, ManagedThread>,
  next_task_id: AtomicU64,
  next_thread_id: u64,
}

impl AsyncRuntime {
  pub fn new() -> Self {
    Self::with_worker_count(4)
  }

  pub fn with_worker_count(worker_count: usize) -> Self {
    let (task_tx, task_rx) = unbounded();
    let (event_tx, event_rx) = unbounded();
    let task_states = Arc::new(Mutex::new(HashMap::new()));
    let mut workers = Vec::new();

    for _ in 0..worker_count.max(1) {
      let task_rx = task_rx.clone();
      let event_tx = event_tx.clone();
      let task_states = task_states.clone();
      workers.push(thread::spawn(move || {
        worker_loop(task_rx, event_tx, task_states);
      }));
    }

    Self {
      task_tx,
      event_tx,
      event_rx,
      workers,
      task_states,
      managed_threads: HashMap::new(),
      next_task_id: AtomicU64::new(1),
      next_thread_id: 1,
    }
  }

  pub fn submit(&self, task: EngineTask) -> TaskId {
    let id = TaskId(self.next_task_id.fetch_add(1, Ordering::SeqCst));
    set_task_state(&self.task_states, id, TaskState::Pending);
    let _ = self.task_tx.send(WorkerMessage::Run(id, task));
    id
  }

  pub fn task_state(&self, id: TaskId) -> Option<TaskState> {
    self
      .task_states
      .lock()
      .ok()
      .and_then(|states| states.get(&id).copied())
  }

  pub fn poll_events(&self) -> Vec<EngineEvent> {
    self.event_rx.try_iter().collect()
  }

  pub fn event_sender(&self) -> Sender<EngineEvent> {
    self.event_tx.clone()
  }

  pub fn spawn_managed_listener<F>(&mut self, joinable: bool, start: F) -> ManagedThreadId
  where
    F: FnOnce(Sender<EngineEvent>, Arc<AtomicBool>) -> JoinHandle<()> + Send + 'static,
  {
    let id = ManagedThreadId(self.next_thread_id);
    self.next_thread_id += 1;

    let stop = Arc::new(AtomicBool::new(false));
    let handle = start(self.event_tx.clone(), stop.clone());
    let handle = joinable.then_some(handle);

    self.managed_threads.insert(
      id,
      ManagedThread {
        stop,
        joinable,
        handle,
      },
    );

    id
  }

  pub fn stop_managed_thread(&mut self, id: ManagedThreadId) -> bool {
    let Some(mut thread) = self.managed_threads.remove(&id) else {
      return false;
    };

    thread.stop.store(true, Ordering::SeqCst);
    if thread.joinable {
      if let Some(handle) = thread.handle.take() {
        let _ = handle.join();
      }
    }
    true
  }
}

impl Default for AsyncRuntime {
  fn default() -> Self {
    Self::new()
  }
}

impl Drop for AsyncRuntime {
  fn drop(&mut self) {
    for _ in &self.workers {
      let _ = self.task_tx.send(WorkerMessage::Shutdown);
    }

    while let Some(worker) = self.workers.pop() {
      let _ = worker.join();
    }

    for thread in self.managed_threads.values_mut() {
      thread.stop.store(true, Ordering::SeqCst);
      if thread.joinable {
        if let Some(handle) = thread.handle.take() {
          let _ = handle.join();
        }
      }
    }
  }
}

fn worker_loop(
  task_rx: Receiver<WorkerMessage>,
  event_tx: Sender<EngineEvent>,
  task_states: Arc<Mutex<HashMap<TaskId, TaskState>>>,
) {
  while let Ok(message) = task_rx.recv() {
    match message {
      WorkerMessage::Run(id, task) => {
        set_task_state(&task_states, id, TaskState::Running);
        match run_task(id, task, &event_tx) {
          Ok(()) => {
            set_task_state(&task_states, id, TaskState::Finished);
            let _ = event_tx.send(EngineEvent::TaskFinished { id });
          }
          Err(error) => {
            set_task_state(&task_states, id, TaskState::Failed);
            let _ = event_tx.send(EngineEvent::TaskFailed { id, error });
          }
        }
      }
      WorkerMessage::Shutdown => break,
    }
  }
}

fn run_task(id: TaskId, task: EngineTask, event_tx: &Sender<EngineEvent>) -> Result<(), String> {
  match task {
    EngineTask::Package(task) => package::run_package_task(id, task, event_tx),
    EngineTask::File(task) => run_file_task(id, task, event_tx),
    EngineTask::Image(task) => run_image_task(id, task, event_tx),
    EngineTask::Network(task) => run_network_task(id, task, event_tx),
    EngineTask::Sleep(task) => {
      thread::sleep(task.duration);
      let _ = event_tx.send(EngineEvent::Time(TimeAsyncEvent::SleepFinished {
        task_id: id,
        callback: task.callback,
      }));
      Ok(())
    }
  }
}

fn run_file_task(
  task_id: TaskId,
  task: FileTask,
  event_tx: &Sender<EngineEvent>,
) -> Result<(), String> {
  match task {
    FileTask::ReadText { path } => match fs::read_to_string(&path) {
      Ok(text) => {
        let _ = event_tx.send(EngineEvent::File(FileEvent::ReadTextFinished {
          task_id,
          path,
          text,
        }));
        Ok(())
      }
      Err(error) => {
        send_file_error(event_tx, task_id, path, error.to_string());
        Err(error.to_string())
      }
    },
    FileTask::WriteText { path, text } => match fs::write(&path, text) {
      Ok(()) => {
        let _ = event_tx.send(EngineEvent::File(FileEvent::WriteTextFinished {
          task_id,
          path,
        }));
        Ok(())
      }
      Err(error) => {
        send_file_error(event_tx, task_id, path, error.to_string());
        Err(error.to_string())
      }
    },
    FileTask::ReadBytes { path } => match fs::read(&path) {
      Ok(bytes) => {
        let _ = event_tx.send(EngineEvent::File(FileEvent::ReadBytesFinished {
          task_id,
          path,
          bytes,
        }));
        Ok(())
      }
      Err(error) => {
        send_file_error(event_tx, task_id, path, error.to_string());
        Err(error.to_string())
      }
    },
    FileTask::WriteBytes { path, bytes } => match fs::write(&path, bytes) {
      Ok(()) => {
        let _ = event_tx.send(EngineEvent::File(FileEvent::WriteBytesFinished {
          task_id,
          path,
        }));
        Ok(())
      }
      Err(error) => {
        send_file_error(event_tx, task_id, path, error.to_string());
        Err(error.to_string())
      }
    },
  }
}

fn send_file_error(event_tx: &Sender<EngineEvent>, task_id: TaskId, path: PathBuf, error: String) {
  let _ = event_tx.send(EngineEvent::File(FileEvent::Failed {
    task_id,
    path,
    error,
  }));
}

fn run_image_task(
  task_id: TaskId,
  task: ImageTask,
  event_tx: &Sender<EngineEvent>,
) -> Result<(), String> {
  match task {
    ImageTask::Convert { params, cache_dir } => {
      match ImageService::new(cache_dir).convert(params) {
        Ok(output) => {
          let _ = event_tx.send(EngineEvent::Image(ImageEvent::ConvertFinished {
            task_id,
            output,
          }));
          Ok(())
        }
        Err(error) => {
          let _ = event_tx.send(EngineEvent::Image(ImageEvent::Failed {
            task_id,
            error: error.clone(),
          }));
          Err(error)
        }
      }
    }
  }
}

fn run_network_task(
  task_id: TaskId,
  task: NetworkTask,
  event_tx: &Sender<EngineEvent>,
) -> Result<(), String> {
  match task {
    NetworkTask::Get { url } => match reqwest::blocking::get(&url) {
      Ok(response) => {
        let status = response.status().as_u16();
        match response.text() {
          Ok(body) => {
            let _ = event_tx.send(EngineEvent::Network(NetworkEvent::GetFinished {
              task_id,
              url,
              status,
              body,
            }));
            Ok(())
          }
          Err(error) => {
            send_network_error(event_tx, task_id, url, error.to_string());
            Err(error.to_string())
          }
        }
      }
      Err(error) => {
        send_network_error(event_tx, task_id, url, error.to_string());
        Err(error.to_string())
      }
    },
  }
}

fn send_network_error(event_tx: &Sender<EngineEvent>, task_id: TaskId, url: String, error: String) {
  let _ = event_tx.send(EngineEvent::Network(NetworkEvent::Failed {
    task_id,
    url,
    error,
  }));
}

fn set_task_state(
  task_states: &Arc<Mutex<HashMap<TaskId, TaskState>>>,
  id: TaskId,
  state: TaskState,
) {
  if let Ok(mut states) = task_states.lock() {
    states.insert(id, state);
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::time::Duration;

  #[test]
  fn async_runtime_assigns_unique_task_ids() {
    let runtime = AsyncRuntime::with_worker_count(1);
    let first = runtime.submit(EngineTask::Sleep(SleepTask {
      duration: Duration::ZERO,
      callback: None,
    }));
    let second = runtime.submit(EngineTask::Sleep(SleepTask {
      duration: Duration::ZERO,
      callback: None,
    }));

    assert_ne!(first, second);
  }

  #[test]
  fn sleep_task_returns_time_event() {
    let runtime = AsyncRuntime::with_worker_count(1);
    let task_id = runtime.submit(EngineTask::Sleep(SleepTask {
      duration: Duration::from_millis(1),
      callback: None,
    }));

    let mut found = false;
    for _ in 0..50 {
      if runtime.poll_events().into_iter().any(|event| {
        matches!(
            event,
            EngineEvent::Time(TimeAsyncEvent::SleepFinished { task_id: id, .. })
                if id == task_id
        )
      }) {
        found = true;
        break;
      }
      thread::sleep(Duration::from_millis(2));
    }

    assert!(found);
  }

  #[test]
  fn failed_file_task_emits_task_failed() {
    let runtime = AsyncRuntime::with_worker_count(1);
    let task_id = runtime.submit(EngineTask::File(FileTask::ReadText {
      path: PathBuf::from("missing-file-for-async-runtime-test.txt"),
    }));

    let mut found = false;
    for _ in 0..50 {
      if runtime
        .poll_events()
        .into_iter()
        .any(|event| matches!(event, EngineEvent::TaskFailed { id, .. } if id == task_id))
      {
        found = true;
        break;
      }
      thread::sleep(Duration::from_millis(2));
    }

    assert!(found);
  }
}
