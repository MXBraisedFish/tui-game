use std::path::{Path, PathBuf};

use super::bootstrap::ensure_storage_layout;
use super::layout;
use crate::host_engine::services::LogService;

/// 存储服务：管理应用根目录，提供各子路径的构建方法，并在初始化时确保目录结构存在。
pub struct StorageService {
  root_dir: PathBuf,
}

impl StorageService {
  pub fn new(log: &mut LogService) -> Self {
    let root_dir = resolve_root_dir();

    let service = Self { root_dir };

    ensure_storage_layout(&service, log);

    service
  }

  pub fn root_dir(&self) -> &Path {
    &self.root_dir
  }

  /// 拼装根目录下的相对路径为完整路径。
  pub fn path(&self, relative_path: &str) -> PathBuf {
    self.root_dir.join(relative_path)
  }

  pub fn profile_language_path(&self) -> PathBuf {
    self.path(layout::PROFILE_LANGUAGE_FILE)
  }

  pub fn profile_terminal_path(&self) -> PathBuf {
    self.path(layout::PROFILE_TERMINAL_FILE)
  }

  pub fn language_assets_root_path(&self) -> PathBuf {
    self.path(layout::ASSETS_LANGUAGE_DIR)
  }

  pub fn language_registry_path(&self) -> PathBuf {
    self.path(layout::LANGUAGE_REGISTRY_FILE)
  }

  pub fn language_package_path(&self, language_code: &str) -> PathBuf {
    self.language_assets_root_path().join(language_code)
  }

  pub fn language_runtime_path(&self, language_code: &str) -> PathBuf {
    self.language_package_path(language_code).join("runtime")
  }

  pub fn language_runtime_namespace_path(&self, language_code: &str, namespace: &str) -> PathBuf {
    self
      .language_runtime_path(language_code)
      .join(format!("{}.json", namespace))
  }

  pub fn language_info_path(&self, language_code: &str) -> PathBuf {
    self
      .language_package_path(language_code)
      .join("language.json")
  }
}

// 自动探测应用根目录：依次尝试当前目录、可执行文件目录。
fn resolve_root_dir() -> PathBuf {
  if let Ok(current_dir) = std::env::current_dir() {
    if current_dir.join("assets").exists() || current_dir.join("Cargo.toml").exists() {
      return current_dir;
    }
  }
  if let Ok(exe_path) = std::env::current_exe() {
    if let Some(exe_dir) = exe_path.parent() {
      return exe_dir.to_path_buf();
    }
  }
  PathBuf::from(".")
}
