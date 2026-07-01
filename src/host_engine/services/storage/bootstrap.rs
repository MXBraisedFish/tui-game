use std::fs;
use std::io::ErrorKind;

use super::layout;
use super::service::StorageService;
use crate::host_engine::services::{LogService, LogSource};

/// 确保存储目录和默认文件存在，缺失时自动创建。
pub fn ensure_storage_layout(storage: &StorageService, log: &mut LogService) {
  ensure_required_directories(storage, log);
  ensure_default_files(storage, log);
}

fn ensure_required_directories(storage: &StorageService, log: &mut LogService) {
  for relative_dir in layout::REQUIRED_DIRECTORIES {
    let path = storage.path(relative_dir);
    if let Err(error) = fs::create_dir_all(&path) {
      log.error(
        LogSource::Storage,
        format!("Failed to create directory {}: {}", path.display(), error),
      );
    }
  }
}

fn ensure_default_files(storage: &StorageService, log: &mut LogService) {
  for (relative_file, default_content) in layout::DEFAULT_FILES {
    let path = storage.path(relative_file);
    match fs::metadata(&path) {
      Ok(metadata) => {
        if metadata.is_file() && metadata.len() > 0 {
          continue;
        }
      }
      Err(error) => {
        if error.kind() != ErrorKind::NotFound {
          log.warn(
            LogSource::Storage,
            format!("Cannot access file {}: {}", path.display(), error),
          );
        }
      }
    }
    if let Some(parent) = path.parent() {
      if let Err(error) = fs::create_dir_all(parent) {
        log.error(
          LogSource::Storage,
          format!(
            "Failed to create parent directory {}: {}",
            parent.display(),
            error
          ),
        );

        continue;
      }
    }
    if let Err(error) = fs::write(&path, default_content) {
      log.error(
        LogSource::Storage,
        format!(
          "Failed to create default file {}: {}",
          path.display(),
          error
        ),
      );
    }
  }
}
