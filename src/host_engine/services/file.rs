use std::{
  fs::{self, OpenOptions},
  io::{self, Write},
  path::{Path, PathBuf},
};

use super::async_runtime::{AsyncRuntime, EngineTask, FileTask, TaskId};

pub struct FileService;

impl FileService {
  pub fn new() -> Self {
    Self
  }

  pub fn read_text(&self, async_runtime: &AsyncRuntime, path: PathBuf) -> TaskId {
    async_runtime.submit(EngineTask::File(FileTask::ReadText { path }))
  }

  pub fn write_text(&self, async_runtime: &AsyncRuntime, path: PathBuf, text: String) -> TaskId {
    async_runtime.submit(EngineTask::File(FileTask::WriteText { path, text }))
  }

  pub fn read_bytes(&self, async_runtime: &AsyncRuntime, path: PathBuf) -> TaskId {
    async_runtime.submit(EngineTask::File(FileTask::ReadBytes { path }))
  }

  pub fn write_bytes(&self, async_runtime: &AsyncRuntime, path: PathBuf, bytes: Vec<u8>) -> TaskId {
    async_runtime.submit(EngineTask::File(FileTask::WriteBytes { path, bytes }))
  }

  pub fn append_text_to(path: &Path, text: &str) -> io::Result<()> {
    if let Some(parent) = path.parent() {
      fs::create_dir_all(parent)?;
    }

    OpenOptions::new()
      .create(true)
      .append(true)
      .open(path)?
      .write_all(text.as_bytes())
  }
}

impl Default for FileService {
  fn default() -> Self {
    Self::new()
  }
}
