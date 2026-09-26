//! File service: asynchronous file jobs (plain read/write plus the sandboxed Lua file and i18n
//! operations) that report through the async executor.

mod task;

use std::path::PathBuf;

use tg_service_async::{AsyncRuntime, TaskId, TaskStatusEvent};

pub use task::{FileEvent, FileListEntry, FileTask};

pub struct FileService;

impl FileService {
  pub fn new() -> Self {
    Self
  }

  pub fn read_text<E>(&self, async_runtime: &AsyncRuntime<E>, path: PathBuf) -> TaskId
  where
    E: From<FileEvent> + From<TaskStatusEvent> + Send + 'static,
  {
    async_runtime.submit(FileTask::ReadText { path })
  }

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

  pub fn read_bytes<E>(&self, async_runtime: &AsyncRuntime<E>, path: PathBuf) -> TaskId
  where
    E: From<FileEvent> + From<TaskStatusEvent> + Send + 'static,
  {
    async_runtime.submit(FileTask::ReadBytes { path })
  }

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
