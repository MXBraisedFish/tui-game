use std::fs;
use std::io::ErrorKind;

use super::layout;
use super::service::StorageService;
use crate::host_engine::services::{LogService, LogSource};

// 确保文件存在
pub fn ensure_storage_layout(storage: &StorageService, log: &mut LogService) {
  ensure_required_directories(storage, log);
  ensure_default_files(storage, log);
}

// 确保必须的目录存在
fn ensure_required_directories(storage: &StorageService, log: &mut LogService) {
  for relative_dir in layout::REQUIRED_DIRECTORIES {
    // 获取绝对路径
    let path = storage.path(relative_dir);

    // 创建目录
    if let Err(error) = fs::create_dir_all(&path) {
      log.error(
        LogSource::Storage,
        format!("Failed to create directory {}: {}", path.display(), error),
      );
    }
  }
}

// 确保所需的文件存在
fn ensure_default_files(storage: &StorageService, log: &mut LogService) {
  for (relative_file, default_content) in layout::DEFAULT_FILES {
    // 获取绝对路径
    let path = storage.path(relative_file);

    // 确保文件存在且有应由的内容
    match fs::metadata(&path) {
        Ok(metadata) => {
          // 如果是文件且有内容，跳过
          if metadata.is_file() && metadata.len() > 0 {
            continue;
          }
        }
        Err(error) => {
          // 如果不是没找到就跳过
          if error.kind() != ErrorKind::NotFound {
            log.warn(
              LogSource::Storage,
              format!("Cannot access file {}: {}", path.display(), error),
            );
          }
        }
    }

    // 创建父目录
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

    // 创建并写入
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
