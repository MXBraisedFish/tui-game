use std::{
  collections::{HashMap, HashSet},
  fs,
  path::{Path, PathBuf},
  sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicU64, Ordering},
  },
  thread::{self, JoinHandle},
  time::Duration,
};

use chardetng::{Iso2022JpDetection, Utf8Detection};
use encoding_rs::Encoding;

use crossbeam_channel::{Receiver, Sender, unbounded};
use tg_core_atomic_fs::atomic_write;

use super::{
  WriteBarrier,
  audio::AudioAsyncEvent,
  export::{self, ExportAsyncEvent, ExportTask},
  image::{ImageConvertParams, ImageService},
  input::{KeyEvent, SystemEvent},
  log::LogSource,
  lua::path::{SafeRelativePath, SandboxPathError, SandboxPathKind, resolve_sandbox_path},
  network::{NetworkEvent, NetworkTask},
  package::{self, PackageAsyncEvent, PackageTask},
  recording::{self, RecordingAsyncEvent, RecordingTask},
  screenshot::{self, ScreenshotAsyncEvent, ScreenshotTask},
  video::{self, VideoAsyncEvent, VideoExportTask},
  time::TimeCallbackId,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TaskId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ManagedThreadId(pub u64);

#[derive(Clone)]
pub(crate) struct TaskCancellation {
  task_id: TaskId,
  cancelled: Arc<Mutex<HashSet<TaskId>>>,
}

impl TaskCancellation {
  #[cfg(test)]
  pub(crate) fn new(task_id: TaskId) -> Self {
    Self {
      task_id,
      cancelled: Arc::new(Mutex::new(HashSet::new())),
    }
  }

  pub(crate) fn is_cancelled(&self) -> bool {
    is_cancelled(&self.cancelled, self.task_id)
  }

  #[cfg(test)]
  pub(crate) fn cancel(&self) {
    self
      .cancelled
      .lock()
      .unwrap_or_else(|poison| poison.into_inner())
      .insert(self.task_id);
  }
}

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
  ReadText {
    path: PathBuf,
  },
  WriteText {
    path: PathBuf,
    text: String,
  },
  ReadBytes {
    path: PathBuf,
  },
  WriteBytes {
    path: PathBuf,
    bytes: Vec<u8>,
  },
  LuaReadText {
    path: PathBuf,
    encoding: String,
  },
  LuaReadBytes {
    path: PathBuf,
  },
  LuaWriteText {
    path: PathBuf,
    text: String,
    encoding: String,
    end_of_line: String,
  },
  LuaWriteBytes {
    path: PathBuf,
    bytes: Vec<u8>,
  },
  LuaListDir {
    path: PathBuf,
    recursive: bool,
    file_type: Option<String>,
  },
  LuaCreateDir {
    root: PathBuf,
    path: PathBuf,
    virtual_path: String,
  },
  LuaRemove {
    root: PathBuf,
    path: PathBuf,
    virtual_path: String,
    recursive: bool,
  },
  LuaLoadI18n {
    assets_root: PathBuf,
    language_code: String,
    callback_language_code: String,
  },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileListEntry {
  pub path: String,
  pub file_type: String,
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
  LuaReadTextFinished {
    task_id: TaskId,
    path: PathBuf,
    text: String,
  },
  LuaWriteTextFinished {
    task_id: TaskId,
    path: PathBuf,
  },
  LuaListDirFinished {
    task_id: TaskId,
    path: PathBuf,
    entries: Vec<FileListEntry>,
  },
  LuaCreateDirFinished {
    task_id: TaskId,
    path: PathBuf,
  },
  LuaRemoveFinished {
    task_id: TaskId,
    path: PathBuf,
  },
  LuaI18nFinished {
    task_id: TaskId,
    language_code: String,
    callback_language_code: String,
    namespaces: HashMap<String, HashMap<String, String>>,
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
  Export(ExportTask),
  Screenshot(ScreenshotTask),
  Recording(RecordingTask),
  Video(VideoExportTask),
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
  Export(ExportAsyncEvent),
  Screenshot(ScreenshotAsyncEvent),
  Recording(RecordingAsyncEvent),
  Video(VideoAsyncEvent),
  File(FileEvent),
  Image(ImageEvent),
  Network(NetworkEvent),
  Audio(AudioAsyncEvent),
  Time(TimeAsyncEvent),
  TaskFinished { id: TaskId },
  TaskFailed { id: TaskId, error: String },
  Log { source: LogSource, message: String },
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
  cancelled_tasks: Arc<Mutex<HashSet<TaskId>>>,
  write_barrier: WriteBarrier,
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
    let cancelled_tasks = Arc::new(Mutex::new(HashSet::new()));
    let write_barrier = WriteBarrier::new();
    let mut workers = Vec::new();

    for _ in 0..worker_count.max(1) {
      let task_rx = task_rx.clone();
      let event_tx = event_tx.clone();
      let task_states = task_states.clone();
      let cancelled_tasks = cancelled_tasks.clone();
      let write_barrier = write_barrier.clone();
      // Note: thread::spawn failure (e.g. OOM) is a process-level abort in std;
      // no recoverable error to log here.
      workers.push(thread::spawn(move || {
        worker_loop(
          task_rx,
          event_tx,
          task_states,
          cancelled_tasks,
          write_barrier,
        );
      }));
    }

    Self {
      task_tx,
      event_tx,
      event_rx,
      workers,
      task_states,
      cancelled_tasks,
      write_barrier,
      managed_threads: HashMap::new(),
      next_task_id: AtomicU64::new(1),
      next_thread_id: 1,
    }
  }

  pub fn submit(&self, task: EngineTask) -> TaskId {
    let id = TaskId(self.next_task_id.fetch_add(1, Ordering::SeqCst));
    if let Some(target) = write_target(&task) {
      let temporary = temporary_target(&task, &target, id);
      if !self.write_barrier.register(id, target, Some(temporary)) {
        set_task_state(&self.task_states, id, TaskState::Cancelled);
        return id;
      }
    }
    set_task_state(&self.task_states, id, TaskState::Pending);
    if self.task_tx.send(WorkerMessage::Run(id, task)).is_err() {
      let error = "asynchronous worker queue is closed".to_string();
      set_task_state(&self.task_states, id, TaskState::Failed);
      self.write_barrier.fail(id, error.clone());
      let _ = self.event_tx.send(EngineEvent::TaskFailed { id, error });
    }
    id
  }

  pub fn write_barrier(&self) -> WriteBarrier {
    self.write_barrier.clone()
  }

  pub fn task_state(&self, id: TaskId) -> Option<TaskState> {
    let states = self.task_states.lock().unwrap_or_else(|poison| {
      // Mutex poisoned — a previous task panicked. Recover the guard.
      poison.into_inner()
    });
    states.get(&id).copied()
  }

  /// 请求取消任务。尚未开始的任务不会执行；运行中的任务会在任务边界停止提交结果。
  pub fn cancel_task(&self, id: TaskId) {
    self
      .cancelled_tasks
      .lock()
      .unwrap_or_else(|poison| poison.into_inner())
      .insert(id);
    if self.task_state(id) == Some(TaskState::Pending) {
      set_task_state(&self.task_states, id, TaskState::Cancelled);
    }
  }

  pub fn cancel_tasks(&self, ids: impl IntoIterator<Item = TaskId>) {
    for id in ids {
      self.cancel_task(id);
    }
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

  pub fn stop_all_managed_threads(&mut self) {
    let ids = self.managed_threads.keys().copied().collect::<Vec<_>>();
    for id in ids {
      let _ = self.stop_managed_thread(id);
    }
  }

  /// 停止任务执行器并等待所有工作线程结束。
  ///
  /// Shutdown 在销毁 Lua 与其它宿主服务前显式调用，Drop 仅作为异常路径兜底。
  pub fn shutdown(&mut self) {
    self.stop_all_managed_threads();
    for _ in &self.workers {
      if self.task_tx.send(WorkerMessage::Shutdown).is_err() {
        break;
      }
    }
    while let Some(worker) = self.workers.pop() {
      let _ = worker.join();
    }
  }
}

impl Default for AsyncRuntime {
  fn default() -> Self {
    Self::new()
  }
}

impl Drop for AsyncRuntime {
  fn drop(&mut self) {
    self.shutdown();
  }
}

fn worker_loop(
  task_rx: Receiver<WorkerMessage>,
  event_tx: Sender<EngineEvent>,
  task_states: Arc<Mutex<HashMap<TaskId, TaskState>>>,
  cancelled_tasks: Arc<Mutex<HashSet<TaskId>>>,
  write_barrier: WriteBarrier,
) {
  while let Ok(message) = task_rx.recv() {
    match message {
      WorkerMessage::Run(id, task) => {
        if is_cancelled(&cancelled_tasks, id) {
          if let EngineTask::Network(task) = &task {
            super::network::emit_cancelled(id, task, &event_tx);
          }
          set_task_state(&task_states, id, TaskState::Cancelled);
          clear_cancelled(&cancelled_tasks, id);
          write_barrier.finish(id);
          continue;
        }
        set_task_state(&task_states, id, TaskState::Running);
        write_barrier.start(id);
        let cancellation = TaskCancellation {
          task_id: id,
          cancelled: cancelled_tasks.clone(),
        };
        let is_network = matches!(&task, EngineTask::Network(_));
        let result = run_task(id, task, &event_tx, &cancellation);
        let was_cancelled = is_cancelled(&cancelled_tasks, id);
        clear_cancelled(&cancelled_tasks, id);
        match result {
          Ok(()) => {
            if !is_network && was_cancelled {
              set_task_state(&task_states, id, TaskState::Cancelled);
            } else {
              set_task_state(&task_states, id, TaskState::Finished);
              let _ = event_tx.send(EngineEvent::TaskFinished { id });
            }
            write_barrier.finish(id);
          }
          Err(error) => {
            if was_cancelled {
              set_task_state(&task_states, id, TaskState::Cancelled);
              write_barrier.finish(id);
            } else {
              set_task_state(&task_states, id, TaskState::Failed);
              write_barrier.fail(id, error.clone());
              let _ = event_tx.send(EngineEvent::TaskFailed { id, error });
            }
          }
        }
      }
      WorkerMessage::Shutdown => break,
    }
  }
}

fn write_target(task: &EngineTask) -> Option<PathBuf> {
  match task {
    EngineTask::Package(_)
    | EngineTask::Image(_)
    | EngineTask::Network(_)
    | EngineTask::Sleep(_) => None,
    EngineTask::Export(task) => Some(task.output_dir.join(format!(
      "{}.{}",
      task.file_stem,
      task.format.extension()
    ))),
    EngineTask::Screenshot(task) => Some(task.png_path.clone()),
    EngineTask::Recording(task) => Some(task.path().to_path_buf()),
    EngineTask::Video(task) => Some(task.output_path.clone()),
    EngineTask::File(FileTask::WriteText { path, .. })
    | EngineTask::File(FileTask::WriteBytes { path, .. })
    | EngineTask::File(FileTask::LuaWriteText { path, .. })
    | EngineTask::File(FileTask::LuaWriteBytes { path, .. })
    | EngineTask::File(FileTask::LuaCreateDir { path, .. })
    | EngineTask::File(FileTask::LuaRemove { path, .. }) => Some(path.clone()),
    EngineTask::File(
      FileTask::ReadText { .. }
      | FileTask::ReadBytes { .. }
      | FileTask::LuaReadText { .. }
      | FileTask::LuaReadBytes { .. }
      | FileTask::LuaListDir { .. }
      | FileTask::LuaLoadI18n { .. },
    ) => None,
  }
}

fn temporary_target(task: &EngineTask, target: &std::path::Path, task_id: TaskId) -> PathBuf {
  if matches!(task, EngineTask::Video(_)) {
    return target.with_extension(format!("mp4.task-{}.part", task_id.0));
  }
  let extension = target
    .extension()
    .and_then(|value| value.to_str())
    .map(|value| format!("{value}.tmp"))
    .unwrap_or_else(|| "tmp".to_string());
  target.with_extension(extension)
}

fn is_cancelled(cancelled_tasks: &Arc<Mutex<HashSet<TaskId>>>, id: TaskId) -> bool {
  cancelled_tasks
    .lock()
    .unwrap_or_else(|poison| poison.into_inner())
    .contains(&id)
}

fn clear_cancelled(cancelled_tasks: &Arc<Mutex<HashSet<TaskId>>>, id: TaskId) {
  cancelled_tasks
    .lock()
    .unwrap_or_else(|poison| poison.into_inner())
    .remove(&id);
}

fn run_task(
  id: TaskId,
  task: EngineTask,
  event_tx: &Sender<EngineEvent>,
  cancellation: &TaskCancellation,
) -> Result<(), String> {
  match task {
    EngineTask::Package(task) => package::run_package_task(id, task, event_tx),
    EngineTask::Export(task) => export::run_export_task(id, task, event_tx, cancellation),
    EngineTask::Screenshot(task) => {
      screenshot::run_screenshot_task(id, task, event_tx, cancellation)
    }
    EngineTask::Recording(task) => recording::run_recording_task(id, task, event_tx),
    EngineTask::Video(task) => video::run_video_task(id, task, event_tx, cancellation),
    EngineTask::File(task) => run_file_task(id, task, event_tx),
    EngineTask::Image(task) => run_image_task(id, task, event_tx),
    EngineTask::Network(task) => super::network::run_network_task(id, task, event_tx, cancellation),
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
    FileTask::WriteText { path, text } => match atomic_write(&path, text.as_bytes(), true) {
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
    FileTask::WriteBytes { path, bytes } => match atomic_write(&path, &bytes, true) {
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
    FileTask::LuaReadText { path, encoding } => match read_lua_text(&path, &encoding) {
      Ok(text) => {
        let _ = event_tx.send(EngineEvent::File(FileEvent::LuaReadTextFinished {
          task_id,
          path,
          text,
        }));
        Ok(())
      }
      Err(error) => {
        send_file_error(event_tx, task_id, path, error.clone());
        Err(error)
      }
    },
    FileTask::LuaReadBytes { path } => match read_lua_bytes(&path) {
      Ok(bytes) => {
        let _ = event_tx.send(EngineEvent::File(FileEvent::ReadBytesFinished {
          task_id,
          path,
          bytes,
        }));
        Ok(())
      }
      Err(error) => {
        send_file_error(event_tx, task_id, path, error.clone());
        Err(error)
      }
    },
    FileTask::LuaWriteText {
      path,
      text,
      encoding,
      end_of_line,
    } => match write_lua_text(&path, &text, &encoding, &end_of_line) {
      Ok(()) => {
        let _ = event_tx.send(EngineEvent::File(FileEvent::LuaWriteTextFinished {
          task_id,
          path,
        }));
        Ok(())
      }
      Err(error) => {
        send_file_error(event_tx, task_id, path, error.clone());
        Err(error)
      }
    },
    FileTask::LuaWriteBytes { path, bytes } => {
      let result = if bytes.len() > LUA_FILE_LIMIT {
        Err("file exceeds 1 MiB".to_string())
      } else {
        atomic_write(&path, &bytes, true).map_err(|error| error.to_string())
      };
      match result {
        Ok(()) => {
          let _ = event_tx.send(EngineEvent::File(FileEvent::WriteBytesFinished {
            task_id,
            path,
          }));
          Ok(())
        }
        Err(error) => {
          send_file_error(event_tx, task_id, path, error.clone());
          Err(error)
        }
      }
    }
    FileTask::LuaListDir {
      path,
      recursive,
      file_type,
    } => match list_lua_files(&path, recursive, file_type.as_deref()) {
      Ok(entries) => {
        let _ = event_tx.send(EngineEvent::File(FileEvent::LuaListDirFinished {
          task_id,
          path,
          entries,
        }));
        Ok(())
      }
      Err(error) => {
        send_file_error(event_tx, task_id, path, error.clone());
        Err(error)
      }
    },
    FileTask::LuaCreateDir {
      root,
      path,
      virtual_path,
    } => match revalidate_lua_path(
      &root,
      &path,
      &virtual_path,
      SandboxPathKind::WritableDirectory,
    )
    .and_then(|path| {
      create_lua_directory_tree(&root, &path)?;
      Ok(path)
    }) {
      Ok(path) => {
        let _ = event_tx.send(EngineEvent::File(FileEvent::LuaCreateDirFinished {
          task_id,
          path,
        }));
        Ok(())
      }
      Err(error) => {
        send_file_error(event_tx, task_id, path, error.clone());
        Err(error)
      }
    },
    FileTask::LuaRemove {
      root,
      path,
      virtual_path,
      recursive,
    } => match revalidate_lua_path(&root, &path, &virtual_path, SandboxPathKind::Removable)
      .and_then(|path| remove_lua_path(&path, recursive).map(|_| path))
    {
      Ok(path) => {
        let _ = event_tx.send(EngineEvent::File(FileEvent::LuaRemoveFinished {
          task_id,
          path,
        }));
        Ok(())
      }
      Err(error) => {
        send_file_error(event_tx, task_id, path, error.clone());
        Err(error)
      }
    },
    FileTask::LuaLoadI18n {
      assets_root,
      language_code,
      callback_language_code,
    } => match load_lua_i18n(&assets_root, &language_code, &callback_language_code) {
      Ok((language_code, namespaces)) => {
        let _ = event_tx.send(EngineEvent::File(FileEvent::LuaI18nFinished {
          task_id,
          language_code,
          callback_language_code,
          namespaces,
        }));
        Ok(())
      }
      Err(error) => {
        send_file_error(event_tx, task_id, assets_root, error.clone());
        Err(error)
      }
    },
  }
}

const LUA_FILE_LIMIT: usize = 1024 * 1024;
const LUA_DIRECTORY_LIMIT: usize = 4096;
const LUA_DIRECTORY_DEPTH_LIMIT: usize = 32;
const LUA_I18N_NAMESPACE_LIMIT: usize = 256;
const LUA_I18N_TOTAL_LIMIT: usize = 4 * 1024 * 1024;

fn load_lua_i18n(
  assets_root: &Path,
  language_code: &str,
  callback_language_code: &str,
) -> Result<(String, HashMap<String, HashMap<String, String>>), String> {
  validate_lua_language_code(language_code)?;
  validate_lua_language_code(callback_language_code)?;

  let primary = load_lua_language(assets_root, language_code)?;
  let fallback = if callback_language_code == language_code {
    None
  } else {
    load_lua_language(assets_root, callback_language_code)?
  };

  let (actual_language, mut namespaces) = match (primary, fallback.as_ref()) {
    (Some(primary), _) => (language_code.to_string(), primary),
    (None, Some(fallback)) => (callback_language_code.to_string(), fallback.clone()),
    (None, None) => return Err("no valid i18n language files were found".to_string()),
  };

  if actual_language == language_code {
    if let Some(fallback) = fallback {
      for (namespace, values) in fallback {
        let target = namespaces.entry(namespace).or_default();
        for (key, value) in values {
          target.entry(key).or_insert(value);
        }
      }
    }
  }
  Ok((actual_language, namespaces))
}

fn validate_lua_language_code(language_code: &str) -> Result<(), String> {
  if language_code.is_empty()
    || language_code.len() > 64
    || !language_code
      .bytes()
      .all(|value| value.is_ascii_alphanumeric() || matches!(value, b'.' | b'_' | b'-'))
  {
    return Err("invalid i18n language code".to_string());
  }
  Ok(())
}

fn load_lua_language(
  assets_root: &Path,
  language_code: &str,
) -> Result<Option<HashMap<String, HashMap<String, String>>>, String> {
  let relative = SafeRelativePath::parse(&format!("language/{language_code}"))
    .map_err(|_| "invalid i18n language path".to_string())?;
  let directory = match resolve_sandbox_path(assets_root, &relative, SandboxPathKind::Directory) {
    Ok(path) => path,
    Err(SandboxPathError::NotFound) => return Ok(None),
    Err(error) => return Err(format!("unsafe i18n language path: {error}")),
  };
  let canonical_directory = directory
    .canonicalize()
    .map_err(|_| "i18n language directory is unavailable".to_string())?;
  let mut paths = Vec::new();
  for entry in fs::read_dir(&canonical_directory)
    .map_err(|_| "i18n language directory could not be read".to_string())?
  {
    let entry = entry.map_err(|_| "i18n language entry could not be read".to_string())?;
    let path = entry.path();
    let metadata = fs::symlink_metadata(&path)
      .map_err(|_| "i18n language entry metadata could not be read".to_string())?;
    if metadata.is_dir() {
      continue;
    }
    let resolved = path
      .canonicalize()
      .map_err(|_| "i18n language file could not be resolved".to_string())?;
    if !resolved.starts_with(&canonical_directory) || !resolved.is_file() {
      return Err("i18n language file escapes its language directory".to_string());
    }
    if resolved
      .extension()
      .and_then(|value| value.to_str())
      .is_some_and(|value| value.eq_ignore_ascii_case("json"))
    {
      paths.push(resolved);
    }
  }
  paths.sort();
  if paths.len() > LUA_I18N_NAMESPACE_LIMIT {
    return Err("i18n language exceeds 256 namespaces".to_string());
  }

  let mut total_bytes = 0usize;
  let mut namespaces = HashMap::new();
  for path in paths {
    let namespace = path
      .file_stem()
      .and_then(|value| value.to_str())
      .ok_or_else(|| "i18n namespace is not valid UTF-8".to_string())?;
    if namespace.is_empty()
      || namespace.len() > 128
      || !namespace
        .bytes()
        .all(|value| value.is_ascii_alphanumeric() || matches!(value, b'.' | b'_' | b'-'))
    {
      return Err("invalid i18n namespace name".to_string());
    }
    let size = fs::metadata(&path)
      .map_err(|_| "i18n language file metadata could not be read".to_string())?
      .len() as usize;
    if size > LUA_FILE_LIMIT {
      return Err("i18n namespace exceeds 1 MiB".to_string());
    }
    total_bytes = total_bytes.saturating_add(size);
    if total_bytes > LUA_I18N_TOTAL_LIMIT {
      return Err("i18n language exceeds 4 MiB".to_string());
    }
    let bytes = fs::read(&path).map_err(|_| "i18n language file could not be read".to_string())?;
    let _: &str = std::str::from_utf8(&bytes)
      .map_err(|_| "i18n language file is not valid UTF-8".to_string())?;
    let values: HashMap<String, String> = serde_json::from_slice(&bytes)
      .map_err(|_| "i18n namespace must be a flat string object".to_string())?;
    namespaces.insert(namespace.to_string(), values);
  }
  Ok((!namespaces.is_empty()).then_some(namespaces))
}

fn read_lua_bytes(path: &Path) -> Result<Vec<u8>, String> {
  let size = fs::metadata(path).map_err(|error| error.to_string())?.len();
  if size > LUA_FILE_LIMIT as u64 {
    return Err("file exceeds 1 MiB".to_string());
  }
  let bytes = fs::read(path).map_err(|error| error.to_string())?;
  if bytes.len() > LUA_FILE_LIMIT {
    return Err("file exceeds 1 MiB".to_string());
  }
  Ok(bytes)
}

fn revalidate_lua_path(
  root: &Path,
  submitted_path: &Path,
  virtual_path: &str,
  kind: SandboxPathKind,
) -> Result<PathBuf, String> {
  let relative = SafeRelativePath::parse(virtual_path).map_err(|error| error.to_string())?;
  if relative.is_root() && matches!(kind, SandboxPathKind::Removable) {
    return Err("cannot remove the assets root".to_string());
  }
  let resolved = resolve_sandbox_path(root, &relative, kind).map_err(|error| error.to_string())?;
  if resolved != submitted_path {
    return Err("safe path changed before the operation started".to_string());
  }
  Ok(resolved)
}

fn create_lua_directory_tree(root: &Path, target: &Path) -> Result<(), String> {
  let canonical_root = root.canonicalize().map_err(|error| error.to_string())?;
  let relative = target
    .strip_prefix(&canonical_root)
    .map_err(|_| "directory path escapes assets root".to_string())?;
  if relative.components().count() > LUA_DIRECTORY_DEPTH_LIMIT {
    return Err("directory creation exceeds 32 levels".to_string());
  }

  let mut current = canonical_root.clone();
  for component in relative.components() {
    current.push(component.as_os_str());
    match fs::symlink_metadata(&current) {
      Ok(_) => {
        let resolved = current.canonicalize().map_err(|error| error.to_string())?;
        if !resolved.starts_with(&canonical_root) {
          return Err("symbolic link escapes assets root".to_string());
        }
        if !resolved.is_dir() {
          return Err("directory path contains a non-directory entry".to_string());
        }
        current = resolved;
      }
      Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
        fs::create_dir(&current).map_err(|error| error.to_string())?;
        let resolved = current.canonicalize().map_err(|error| error.to_string())?;
        if !resolved.starts_with(&canonical_root) {
          return Err("created directory escapes assets root".to_string());
        }
        current = resolved;
      }
      Err(error) => return Err(error.to_string()),
    }
  }
  Ok(())
}

fn remove_lua_path(path: &Path, recursive: bool) -> Result<(), String> {
  let metadata = fs::symlink_metadata(path).map_err(|error| error.to_string())?;
  if metadata.file_type().is_symlink() {
    return if fs::metadata(path).is_ok_and(|target| target.is_dir()) {
      fs::remove_dir(path).map_err(|error| error.to_string())
    } else {
      fs::remove_file(path).map_err(|error| error.to_string())
    };
  }
  if metadata.is_file() {
    return fs::remove_file(path).map_err(|error| error.to_string());
  }
  if !metadata.is_dir() {
    return Err("path is not a file or directory".to_string());
  }
  if !recursive {
    return fs::remove_dir(path).map_err(|error| error.to_string());
  }

  let mut entries = 0usize;
  validate_lua_remove_tree(path, 0, &mut entries)?;
  fs::remove_dir_all(path).map_err(|error| error.to_string())
}

fn validate_lua_remove_tree(
  directory: &Path,
  depth: usize,
  entries: &mut usize,
) -> Result<(), String> {
  if depth > LUA_DIRECTORY_DEPTH_LIMIT {
    return Err("directory recursion exceeds 32 levels".to_string());
  }
  for entry in fs::read_dir(directory).map_err(|error| error.to_string())? {
    let entry = entry.map_err(|error| error.to_string())?;
    *entries = entries.saturating_add(1);
    if *entries > LUA_DIRECTORY_LIMIT {
      return Err("directory operation exceeds 4096 entries".to_string());
    }
    let metadata = fs::symlink_metadata(entry.path()).map_err(|error| error.to_string())?;
    if metadata.is_dir() && !metadata.file_type().is_symlink() {
      validate_lua_remove_tree(&entry.path(), depth + 1, entries)?;
    }
  }
  Ok(())
}

fn read_lua_text(path: &Path, encoding: &str) -> Result<String, String> {
  let bytes = fs::read(path).map_err(|error| error.to_string())?;
  if bytes.len() > LUA_FILE_LIMIT {
    return Err("file exceeds 1 MiB".to_string());
  }
  let text = decode_lua_text(&bytes, encoding)?;
  validate_lua_text(&text)?;
  Ok(normalize_lua_newlines(&text))
}

fn write_lua_text(
  path: &Path,
  text: &str,
  encoding: &str,
  end_of_line: &str,
) -> Result<(), String> {
  validate_lua_text(text)?;
  let normalized = normalize_lua_newlines(text);
  let eol = match end_of_line {
    "cr" => "\r",
    "lf" => "\n",
    "crlf" => "\r\n",
    "auto" if path.is_file() => {
      let old = fs::read(path).map_err(|error| error.to_string())?;
      let old = decode_lua_text(&old, "auto")?;
      if old.contains("\r\n") {
        "\r\n"
      } else if old.contains('\r') {
        "\r"
      } else {
        "\n"
      }
    }
    "auto" => {
      if cfg!(windows) {
        "\r\n"
      } else {
        "\n"
      }
    }
    _ => return Err("unsupported end-of-line mode".to_string()),
  };
  let converted = if eol == "\n" {
    normalized
  } else {
    normalized.replace('\n', eol)
  };
  let selected_encoding = if encoding == "auto" && path.is_file() {
    detect_encoding_name(&fs::read(path).map_err(|error| error.to_string())?)
  } else if encoding == "auto" {
    "utf-8".to_string()
  } else {
    encoding.to_string()
  };
  let bytes = encode_lua_text(&converted, &selected_encoding)?;
  if bytes.len() > LUA_FILE_LIMIT {
    return Err("encoded file exceeds 1 MiB".to_string());
  }
  atomic_write(path, &bytes, true).map_err(|error| error.to_string())
}

fn decode_lua_text(bytes: &[u8], encoding: &str) -> Result<String, String> {
  if encoding.eq_ignore_ascii_case("auto") {
    if let Some(bytes) = bytes.strip_prefix(&[0xef, 0xbb, 0xbf]) {
      return String::from_utf8(bytes.to_vec()).map_err(|_| "invalid UTF-8 text".to_string());
    }
    if bytes.starts_with(&[0xff, 0xfe]) {
      return decode_utf16(&bytes[2..], true);
    }
    if bytes.starts_with(&[0xfe, 0xff]) {
      return decode_utf16(&bytes[2..], false);
    }
    if let Ok(text) = std::str::from_utf8(bytes) {
      return Ok(text.to_string());
    }
    let mut detector = chardetng::EncodingDetector::new(Iso2022JpDetection::Allow);
    detector.feed(bytes, true);
    let guessed = detector.guess(None, Utf8Detection::Allow);
    return guessed
      .decode_without_bom_handling_and_without_replacement(bytes)
      .map(|text| text.into_owned())
      .ok_or_else(|| "text cannot be decoded without replacement".to_string());
  }
  match encoding.to_ascii_lowercase().as_str() {
    "utf-16le" => decode_utf16(bytes.strip_prefix(&[0xff, 0xfe]).unwrap_or(bytes), true),
    "utf-16be" => decode_utf16(bytes.strip_prefix(&[0xfe, 0xff]).unwrap_or(bytes), false),
    name => {
      let encoding = Encoding::for_label(name.as_bytes())
        .ok_or_else(|| "unsupported text encoding".to_string())?;
      encoding
        .decode_without_bom_handling_and_without_replacement(bytes)
        .map(|text| text.into_owned())
        .ok_or_else(|| "text cannot be decoded without replacement".to_string())
    }
  }
}

fn encode_lua_text(text: &str, encoding: &str) -> Result<Vec<u8>, String> {
  match encoding.to_ascii_lowercase().as_str() {
    "utf-16le" => Ok(
      text
        .encode_utf16()
        .flat_map(u16::to_le_bytes)
        .collect::<Vec<_>>(),
    ),
    "utf-16be" => Ok(
      text
        .encode_utf16()
        .flat_map(u16::to_be_bytes)
        .collect::<Vec<_>>(),
    ),
    name => {
      let encoding = Encoding::for_label(name.as_bytes())
        .ok_or_else(|| "unsupported text encoding".to_string())?;
      let (bytes, _, had_errors) = encoding.encode(text);
      if had_errors {
        Err("text cannot be encoded without replacement".to_string())
      } else {
        Ok(bytes.into_owned())
      }
    }
  }
}

fn decode_utf16(bytes: &[u8], little_endian: bool) -> Result<String, String> {
  if !bytes.len().is_multiple_of(2) {
    return Err("invalid UTF-16 text".to_string());
  }
  let units = bytes.chunks_exact(2).map(|bytes| {
    let pair = [bytes[0], bytes[1]];
    if little_endian {
      u16::from_le_bytes(pair)
    } else {
      u16::from_be_bytes(pair)
    }
  });
  std::char::decode_utf16(units)
    .collect::<Result<String, _>>()
    .map_err(|_| "invalid UTF-16 text".to_string())
}

fn detect_encoding_name(bytes: &[u8]) -> String {
  if bytes.starts_with(&[0xff, 0xfe]) {
    return "utf-16le".to_string();
  }
  if bytes.starts_with(&[0xfe, 0xff]) {
    return "utf-16be".to_string();
  }
  if bytes.starts_with(&[0xef, 0xbb, 0xbf]) || std::str::from_utf8(bytes).is_ok() {
    return "utf-8".to_string();
  }
  let mut detector = chardetng::EncodingDetector::new(Iso2022JpDetection::Allow);
  detector.feed(bytes, true);
  detector
    .guess(None, Utf8Detection::Allow)
    .name()
    .to_ascii_lowercase()
}

fn validate_lua_text(text: &str) -> Result<(), String> {
  if text.contains('\0') {
    return Err("NUL is not allowed in text files".to_string());
  }
  let suspicious = text
    .chars()
    .filter(|value| value.is_control() && !matches!(*value, '\n' | '\r' | '\t'))
    .count();
  if suspicious > text.chars().count().saturating_div(20).max(8) {
    return Err("file appears to contain binary data".to_string());
  }
  Ok(())
}

fn normalize_lua_newlines(text: &str) -> String {
  text.replace("\r\n", "\n").replace('\r', "\n")
}

fn list_lua_files(
  path: &Path,
  recursive: bool,
  file_type: Option<&str>,
) -> Result<Vec<FileListEntry>, String> {
  let root = path.canonicalize().map_err(|error| error.to_string())?;
  if !root.is_dir() {
    return Err("path is not a directory".to_string());
  }
  let mut entries = Vec::new();
  collect_lua_files(&root, &root, recursive, file_type, 0, &mut entries)?;
  entries.sort_by(|left, right| left.path.cmp(&right.path));
  Ok(entries)
}

fn collect_lua_files(
  root: &Path,
  directory: &Path,
  recursive: bool,
  file_type: Option<&str>,
  depth: usize,
  output: &mut Vec<FileListEntry>,
) -> Result<(), String> {
  if depth > 32 {
    return Err("directory recursion exceeds 32 levels".to_string());
  }
  for entry in fs::read_dir(directory).map_err(|error| error.to_string())? {
    let entry = entry.map_err(|error| error.to_string())?;
    let canonical = entry
      .path()
      .canonicalize()
      .map_err(|error| error.to_string())?;
    if !canonical.starts_with(root) {
      return Err("symbolic link escapes assets root".to_string());
    }
    let metadata = canonical.metadata().map_err(|error| error.to_string())?;
    if metadata.is_dir() {
      if recursive {
        collect_lua_files(root, &canonical, true, file_type, depth + 1, output)?;
      }
      continue;
    }
    if !metadata.is_file() {
      continue;
    }
    let extension = canonical
      .extension()
      .and_then(|value| value.to_str())
      .unwrap_or_default();
    if file_type.is_some_and(|filter| !extension.eq_ignore_ascii_case(filter)) {
      continue;
    }
    if output.len() >= LUA_DIRECTORY_LIMIT {
      return Err("directory result exceeds 4096 files".to_string());
    }
    let relative = canonical
      .strip_prefix(root)
      .map_err(|_| "invalid directory entry".to_string())?
      .to_string_lossy()
      .replace('\\', "/");
    output.push(FileListEntry {
      path: relative,
      file_type: extension.to_ascii_lowercase(),
    });
  }
  Ok(())
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

fn set_task_state(
  task_states: &Arc<Mutex<HashMap<TaskId, TaskState>>>,
  id: TaskId,
  state: TaskState,
) {
  let mut states = task_states.lock().unwrap_or_else(|poison| {
    // Mutex poisoned — a previous task panicked. Recover the guard.
    poison.into_inner()
  });
  states.insert(id, state);
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::sync::atomic::{AtomicU64, Ordering};
  use std::time::Duration;

  static FILE_TEST_ID: AtomicU64 = AtomicU64::new(1);

  fn file_test_directory() -> PathBuf {
    let id = FILE_TEST_ID.fetch_add(1, Ordering::Relaxed);
    let path =
      std::env::temp_dir().join(format!("tui_game_lua_file_{}_{}", std::process::id(), id));
    fs::create_dir_all(&path).unwrap();
    path
  }

  #[cfg(unix)]
  fn create_directory_link(target: &Path, link: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(target, link)
  }

  #[cfg(windows)]
  fn create_directory_link(target: &Path, link: &Path) -> std::io::Result<()> {
    std::os::windows::fs::symlink_dir(target, link)
  }

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
  fn lua_i18n_merges_fallback_keys_without_overwriting_primary_values() {
    let root = file_test_directory();
    let primary = root.join("language/zh_cn");
    let fallback = root.join("language/en_us");
    fs::create_dir_all(&primary).unwrap();
    fs::create_dir_all(&fallback).unwrap();
    fs::write(
      primary.join("menu.json"),
      r#"{"title":"标题","primary":"主语言"}"#,
    )
    .unwrap();
    fs::write(
      fallback.join("menu.json"),
      r#"{"title":"Title","fallback":"Fallback"}"#,
    )
    .unwrap();
    fs::write(fallback.join("extra.json"), r#"{"value":"Extra"}"#).unwrap();

    let (actual, namespaces) = load_lua_i18n(&root, "zh_cn", "en_us").unwrap();
    assert_eq!(actual, "zh_cn");
    assert_eq!(namespaces["menu"]["title"], "标题");
    assert_eq!(namespaces["menu"]["fallback"], "Fallback");
    assert_eq!(namespaces["extra"]["value"], "Extra");
    fs::remove_dir_all(root).unwrap();
  }

  #[test]
  fn lua_i18n_uses_fallback_language_when_primary_directory_is_missing() {
    let root = file_test_directory();
    let fallback = root.join("language/en_us");
    fs::create_dir_all(&fallback).unwrap();
    fs::write(fallback.join("menu.json"), r#"{"title":"Title"}"#).unwrap();

    let (actual, namespaces) = load_lua_i18n(&root, "missing", "en_us").unwrap();
    assert_eq!(actual, "en_us");
    assert_eq!(namespaces["menu"]["title"], "Title");
    fs::remove_dir_all(root).unwrap();
  }

  #[test]
  fn lua_i18n_rejects_nested_values_and_unsafe_language_codes() {
    let root = file_test_directory();
    let language = root.join("language/en_us");
    fs::create_dir_all(&language).unwrap();
    fs::write(language.join("bad.json"), r#"{"nested":{"value":"no"}}"#).unwrap();

    assert!(
      load_lua_i18n(&root, "en_us", "en_us")
        .unwrap_err()
        .contains("flat string object")
    );
    assert!(load_lua_i18n(&root, "../secret", "en_us").is_err());
    fs::remove_dir_all(root).unwrap();
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

  #[test]
  fn lua_text_read_normalizes_newlines_and_rejects_non_text() {
    let directory = file_test_directory();
    let mixed = directory.join("mixed.txt");
    fs::write(&mixed, b"one\r\ntwo\rthree\nfour").unwrap();
    assert_eq!(
      read_lua_text(&mixed, "utf-8").unwrap(),
      "one\ntwo\nthree\nfour"
    );

    let nul = directory.join("nul.txt");
    fs::write(&nul, b"text\0data").unwrap();
    assert!(read_lua_text(&nul, "utf-8").is_err());

    let invalid = directory.join("invalid.txt");
    fs::write(&invalid, [0xff, 0xfe, 0xfd]).unwrap();
    assert!(read_lua_text(&invalid, "utf-8").is_err());
    fs::remove_dir_all(directory).unwrap();
  }

  #[test]
  fn lua_byte_tasks_preserve_binary_data_and_enforce_the_file_limit() {
    let directory = file_test_directory();
    let input = directory.join("input.bin");
    let output = directory.join("output.bin");
    let oversized_input = directory.join("oversized-input.bin");
    let protected_output = directory.join("protected-output.bin");
    let bytes = vec![0x00, 0xff, b'\r', b'\n', 0x7f];
    fs::write(&input, &bytes).unwrap();
    fs::write(&oversized_input, vec![0u8; LUA_FILE_LIMIT + 1]).unwrap();
    fs::write(&protected_output, b"keep").unwrap();
    let (event_tx, event_rx) = unbounded();

    run_file_task(
      TaskId(1),
      FileTask::LuaReadBytes {
        path: input.clone(),
      },
      &event_tx,
    )
    .unwrap();
    assert!(matches!(
      event_rx.recv().unwrap(),
      EngineEvent::File(FileEvent::ReadBytesFinished {
        task_id: TaskId(1),
        path,
        bytes: found,
      }) if path == input && found == bytes
    ));

    run_file_task(
      TaskId(2),
      FileTask::LuaWriteBytes {
        path: output.clone(),
        bytes: bytes.clone(),
      },
      &event_tx,
    )
    .unwrap();
    assert_eq!(fs::read(&output).unwrap(), bytes);
    assert!(matches!(
      event_rx.recv().unwrap(),
      EngineEvent::File(FileEvent::WriteBytesFinished {
        task_id: TaskId(2),
        path,
      }) if path == output
    ));

    assert!(
      run_file_task(
        TaskId(3),
        FileTask::LuaReadBytes {
          path: oversized_input.clone(),
        },
        &event_tx,
      )
      .is_err()
    );
    assert!(matches!(
      event_rx.recv().unwrap(),
      EngineEvent::File(FileEvent::Failed {
        task_id: TaskId(3),
        path,
        ..
      }) if path == oversized_input
    ));

    assert!(
      run_file_task(
        TaskId(4),
        FileTask::LuaWriteBytes {
          path: protected_output.clone(),
          bytes: vec![0u8; LUA_FILE_LIMIT + 1],
        },
        &event_tx,
      )
      .is_err()
    );
    assert_eq!(fs::read(&protected_output).unwrap(), b"keep");
    assert!(matches!(
      event_rx.recv().unwrap(),
      EngineEvent::File(FileEvent::Failed {
        task_id: TaskId(4),
        path,
        ..
      }) if path == protected_output
    ));

    fs::remove_dir_all(directory).unwrap();
  }

  #[test]
  fn lua_directory_listing_returns_only_matching_files() {
    let directory = file_test_directory();
    let nested = directory.join("nested");
    fs::create_dir_all(&nested).unwrap();
    fs::write(directory.join("first.RS"), "one").unwrap();
    fs::write(directory.join("other.txt"), "two").unwrap();
    fs::write(nested.join("second.rs"), "three").unwrap();

    assert_eq!(
      list_lua_files(&directory, false, Some("rs")).unwrap(),
      vec![FileListEntry {
        path: "first.RS".to_string(),
        file_type: "rs".to_string(),
      }]
    );
    assert_eq!(
      list_lua_files(&directory, true, Some("rs")).unwrap(),
      vec![
        FileListEntry {
          path: "first.RS".to_string(),
          file_type: "rs".to_string(),
        },
        FileListEntry {
          path: "nested/second.rs".to_string(),
          file_type: "rs".to_string(),
        },
      ]
    );
    fs::remove_dir_all(directory).unwrap();
  }

  #[test]
  fn lua_directory_recursion_is_limited_to_32_levels_below_the_root() {
    let directory = file_test_directory();
    let mut nested = directory.clone();
    for _ in 0..32 {
      nested.push("d");
      fs::create_dir(&nested).unwrap();
    }
    fs::write(nested.join("allowed.txt"), "allowed").unwrap();
    assert!(list_lua_files(&directory, true, None).is_ok());

    nested.push("d");
    fs::create_dir(&nested).unwrap();
    fs::write(nested.join("too-deep.txt"), "too deep").unwrap();
    assert_eq!(
      list_lua_files(&directory, true, None).unwrap_err(),
      "directory recursion exceeds 32 levels"
    );
    fs::remove_dir_all(directory).unwrap();
  }

  #[test]
  fn lua_directory_creation_and_removal_are_revalidated_and_reported() {
    let directory = file_test_directory().canonicalize().unwrap();
    let created = directory.join("created/nested/leaf");
    let (event_tx, event_rx) = unbounded();

    run_file_task(
      TaskId(1),
      FileTask::LuaCreateDir {
        root: directory.clone(),
        path: created.clone(),
        virtual_path: "created/nested/leaf".to_string(),
      },
      &event_tx,
    )
    .unwrap();
    assert!(created.is_dir());
    assert!(directory.join("created").is_dir());
    assert!(directory.join("created/nested").is_dir());
    create_lua_directory_tree(&directory, &created).unwrap();
    assert!(matches!(
      event_rx.recv().unwrap(),
      EngineEvent::File(FileEvent::LuaCreateDirFinished { task_id: TaskId(1), path })
        if path == created
    ));

    fs::write(created.join("child.txt"), "child").unwrap();
    assert!(
      run_file_task(
        TaskId(2),
        FileTask::LuaRemove {
          root: directory.clone(),
          path: created.clone(),
          virtual_path: "created/nested/leaf".to_string(),
          recursive: false,
        },
        &event_tx,
      )
      .is_err()
    );
    assert!(created.is_dir());
    assert!(matches!(
      event_rx.recv().unwrap(),
      EngineEvent::File(FileEvent::Failed { task_id: TaskId(2), path, error })
        if path == created && !error.is_empty()
    ));

    run_file_task(
      TaskId(3),
      FileTask::LuaRemove {
        root: directory.clone(),
        path: created.clone(),
        virtual_path: "created/nested/leaf".to_string(),
        recursive: true,
      },
      &event_tx,
    )
    .unwrap();
    assert!(!created.exists());
    assert!(matches!(
      event_rx.recv().unwrap(),
      EngineEvent::File(FileEvent::LuaRemoveFinished { task_id: TaskId(3), path })
        if path == created
    ));

    let root_removal = run_file_task(
      TaskId(4),
      FileTask::LuaRemove {
        root: directory.clone(),
        path: directory.clone(),
        virtual_path: ".".to_string(),
        recursive: true,
      },
      &event_tx,
    );
    assert_eq!(root_removal.unwrap_err(), "cannot remove the assets root");
    assert!(directory.is_dir());
    fs::remove_dir_all(directory).unwrap();
  }

  #[test]
  fn lua_directory_creation_checks_depth_before_creating_any_level() {
    let directory = file_test_directory().canonicalize().unwrap();
    let mut target = directory.clone();
    for _ in 0..33 {
      target.push("d");
    }

    assert_eq!(
      create_lua_directory_tree(&directory, &target).unwrap_err(),
      "directory creation exceeds 32 levels"
    );
    assert!(!directory.join("d").exists());
    fs::remove_dir_all(directory).unwrap();
  }

  #[test]
  fn lua_recursive_remove_preflights_depth_before_deleting() {
    let directory = file_test_directory();
    let target = directory.join("target");
    fs::create_dir(&target).unwrap();
    let mut nested = target.clone();
    for _ in 0..33 {
      nested.push("d");
      fs::create_dir(&nested).unwrap();
    }
    fs::write(nested.join("kept.txt"), "kept").unwrap();

    assert_eq!(
      remove_lua_path(&target, true).unwrap_err(),
      "directory recursion exceeds 32 levels"
    );
    assert!(target.is_dir());
    assert!(nested.join("kept.txt").is_file());
    fs::remove_dir_all(directory).unwrap();
  }

  #[test]
  fn lua_remove_deletes_a_symbolic_link_without_following_it() {
    let directory = file_test_directory().canonicalize().unwrap();
    let target = directory.join("target");
    let link = directory.join("link");
    fs::create_dir(&target).unwrap();
    fs::write(target.join("kept.txt"), "kept").unwrap();
    if create_directory_link(&target, &link).is_err() {
      fs::remove_dir_all(directory).unwrap();
      return;
    }

    let relative = SafeRelativePath::parse("link").unwrap();
    let removable =
      resolve_sandbox_path(&directory, &relative, SandboxPathKind::Removable).unwrap();
    assert_eq!(removable, link);
    remove_lua_path(&removable, true).unwrap();
    assert!(fs::symlink_metadata(&link).is_err());
    assert!(target.join("kept.txt").is_file());
    fs::remove_dir_all(directory).unwrap();
  }

  #[test]
  fn lua_text_write_is_strict_and_preserves_requested_eol() {
    let directory = file_test_directory();
    let path = directory.join("output.txt");
    write_lua_text(&path, "one\r\ntwo\rthree", "utf-8", "crlf").unwrap();
    assert_eq!(fs::read(&path).unwrap(), b"one\r\ntwo\r\nthree");
    assert!(write_lua_text(&path, "bad\0text", "utf-8", "lf").is_err());
    fs::remove_dir_all(directory).unwrap();
  }

  #[test]
  fn documented_file_encodings_are_supported() {
    for encoding in [
      "utf-8",
      "gbk",
      "gb18030",
      "big5",
      "shift_jis",
      "euc-jp",
      "iso-2022-jp",
      "euc-kr",
      "windows-874",
      "windows-1250",
      "windows-1251",
      "windows-1252",
      "windows-1253",
      "windows-1254",
      "windows-1255",
      "windows-1256",
      "windows-1257",
      "windows-1258",
      "iso-8859-2",
      "iso-8859-3",
      "iso-8859-4",
      "iso-8859-5",
      "iso-8859-6",
      "iso-8859-7",
      "iso-8859-8",
      "iso-8859-8-i",
      "iso-8859-10",
      "iso-8859-13",
      "iso-8859-14",
      "iso-8859-15",
      "iso-8859-16",
      "koi8-r",
      "koi8-u",
      "ibm866",
      "macintosh",
      "x-mac-cyrillic",
    ] {
      assert!(
        Encoding::for_label(encoding.as_bytes()).is_some(),
        "{encoding}"
      );
    }
  }
}
