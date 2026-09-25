use std::fs;
use std::io::ErrorKind;

use super::layout;
use super::service::StorageService;
use tg_service_log::{LogService, LogSource};

/// 确保存储目录和默认文件存在，缺失时自动创建。
pub fn ensure_storage_layout(storage: &StorageService, log: &mut LogService) {
  ensure_required_directories(storage, log);
  ensure_default_files(storage, log);
}

fn ensure_required_directories(storage: &StorageService, log: &mut LogService) {
  for relative_dir in layout::REQUIRED_DIRECTORIES {
    let path = storage.path(relative_dir);
    if let Err(error) = fs::create_dir_all(&path) {
      log.fatal_message(
        LogSource::Boot,
        tg_service_log::HostLogMessage::new(
          "log_info.operation.failed",
          "Host operation {operation} failed for {target}: {error}",
        )
        .param("operation", "create_required_directory")
        .param("target", path.display().to_string())
        .param("error", error.to_string()),
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
          log.warn_operation_failed(
            LogSource::Storage,
            "inspect_default_file",
            path.display().to_string(),
            error.to_string(),
          );
        }
      }
    }
    if let Some(parent) = path.parent() {
      if let Err(error) = fs::create_dir_all(parent) {
        log.error_operation_failed(
          LogSource::Storage,
          "create_default_parent",
          parent.display().to_string(),
          error.to_string(),
        );

        continue;
      }
    }
    if let Err(error) = fs::write(&path, default_content) {
      log.error_operation_failed(
        LogSource::Storage,
        "create_default_file",
        path.display().to_string(),
        error.to_string(),
      );
    }
  }
}
