//! Asynchronous text and byte-file requests submitted to the shared task executor.
//!
//! # Examples
//!
//! ```rust
//! use std::{
//!   path::PathBuf,
//!   time::{Duration, Instant},
//! };
//!
//! use tg_service_async::{AsyncRuntime, TaskStatusEvent};
//! use tg_service_file::{FileEvent, FileService};
//!
//! #[derive(Debug)]
//! enum Event {
//!   File(FileEvent),
//!     Status,
//! }
//!
//! impl From<FileEvent> for Event {
//!   fn from(event: FileEvent) -> Self {
//!     Self::File(event)
//!   }
//! }
//!
//! impl From<TaskStatusEvent> for Event {
//!   fn from(_: TaskStatusEvent) -> Self {
//!     Self::Status
//!   }
//! }
//!
//! fn next_file_event(runtime: &AsyncRuntime<Event>) -> FileEvent {
//!   let deadline = Instant::now() + Duration::from_secs(5);
//!   while Instant::now() < deadline {
//!     for event in runtime.poll_events() {
//!       if let Event::File(event) = event {
//!         return event;
//!       }
//!     }
//!     std::thread::sleep(Duration::from_millis(5));
//!   }
//!   panic!("no file event within 5 seconds");
//! }
//!
//! fn main() {
//!   let root = create_temp_dir("tg_file_smoke");
//!   let path = root.join("smoke.txt");
//!   let file = FileService::new();
//!   let runtime = AsyncRuntime::<Event>::with_worker_count(1);
//!
//!   let write = file.write_text(&runtime, path.clone(), "hello".to_string());
//!   match next_file_event(&runtime) {
//!     FileEvent::WriteTextFinished { task_id, .. } => assert_eq!(task_id, write),
//!     other => panic!("unexpected event after write: {other:?}"),
//!   }
//!
//!   let read = file.read_text(&runtime, path.clone());
//!   match next_file_event(&runtime) {
//!     FileEvent::ReadTextFinished { task_id, text, .. } => {
//!       assert_eq!(task_id, read);
//!       assert_eq!(text, "hello");
//!     }
//!     other => panic!("unexpected event after read: {other:?}"),
//!   }
//!
//!   std::fs::remove_dir_all(&root).expect("clean up temporary directory");
//!   println!("file ok: wrote and read back {}", path.display());
//! }
//!
//! fn create_temp_dir(prefix: &str) -> PathBuf {
//!   let nonce = std::time::SystemTime::now()
//!     .duration_since(std::time::UNIX_EPOCH)
//!     .unwrap_or_default()
//!     .as_nanos();
//!   let root = std::env::temp_dir().join(format!("{prefix}_{}_{nonce}", std::process::id()));
//!   std::fs::create_dir(&root)
//!     .unwrap_or_else(|error| panic!("create temporary directory {}: {error}", root.display()));
//!   root
//! }
//! ```

mod task;

use std::path::PathBuf;

use tg_service_async::{AsyncRuntime, TaskId, TaskStatusEvent};

pub use task::{FileEvent, FileListEntry, FileTask};

/// The public entry point for file operations.
pub struct FileService;

impl FileService {
  /// Create a file service with its initial state.
  pub fn new() -> Self {
    Self
  }

  /// Request or read the text contents of the specified file.
  pub fn read_text<E>(&self, async_runtime: &AsyncRuntime<E>, path: PathBuf) -> TaskId
  where
    E: From<FileEvent> + From<TaskStatusEvent> + Send + 'static,
  {
    async_runtime.submit(FileTask::ReadText { path })
  }

  /// Request or write the supplied text to the specified destination.
  ///
  /// # Arguments
  ///
  /// * `async_runtime` - The shared background-task executor.
  /// * `path` - The filesystem path to read, write, or resolve.
  /// * `text` - The text to process or display.
  pub fn write_text<E>(
    &self,
    async_runtime: &AsyncRuntime<E>,
    path: PathBuf,
    text: String,
  ) -> TaskId
  where
    E: From<FileEvent> + From<TaskStatusEvent> + Send + 'static,
  {
    async_runtime.submit(FileTask::WriteText { path, text })
  }

  /// Submit a binary-file read and return the request task identifier.
  pub fn read_bytes<E>(&self, async_runtime: &AsyncRuntime<E>, path: PathBuf) -> TaskId
  where
    E: From<FileEvent> + From<TaskStatusEvent> + Send + 'static,
  {
    async_runtime.submit(FileTask::ReadBytes { path })
  }

  /// Submit a binary-file write and return the request task identifier.
  ///
  /// # Arguments
  ///
  /// * `async_runtime` - The shared background-task executor.
  /// * `path` - The filesystem path to read, write, or resolve.
  /// * `bytes` - The binary payload.
  pub fn write_bytes<E>(
    &self,
    async_runtime: &AsyncRuntime<E>,
    path: PathBuf,
    bytes: Vec<u8>,
  ) -> TaskId
  where
    E: From<FileEvent> + From<TaskStatusEvent> + Send + 'static,
  {
    async_runtime.submit(FileTask::WriteBytes { path, bytes })
  }
}

impl Default for FileService {
  fn default() -> Self {
    Self::new()
  }
}
