use std::path::{Path, PathBuf};

use super::bootstrap::ensure_storage_layout;
use super::layout;
use crate::host_engine::services::LogService;

pub struct StorageService {
  root_dir: PathBuf,
}

impl StorageService {
  pub fn new(log: &mut LogService) -> Self {
    // 根目录
    let root_dir = resolve_root_dir();

    let service = Self { root_dir };

    ensure_storage_layout(&service, log);

    service
  }

  // 获取根目录
  pub fn root_dir(&self) -> &Path {
    &self.root_dir
  }

  // 拼接绝对路径
  pub fn path(&self, relative_path: &str) -> PathBuf {
    self.root_dir.join(relative_path)
  }

  // 配置文件路径
  pub fn profile_language_path(&self) -> PathBuf {
    self.path(layout::PROFILE_LANGUAGE_FILE)
  }

  pub fn profile_terminal_path(&self) -> PathBuf {
    self.path(layout::PROFILE_TERMINAL_FILE)
  }

  // 语言资源路径
  pub fn language_assets_root_path(&self) -> PathBuf {
    self.path(layout::ASSETS_LANGUAGE_DIR)
  }

  // 语言注册表路径
  pub fn language_registry_path(&self) -> PathBuf {
    self.path(layout::LANGUAGE_REGISTRY_FILE)
  }

  // 语言包路径
  pub fn language_package_path(&self, language_code: &str) -> PathBuf {
    self.language_assets_root_path().join(language_code)
  }

  // 语言包路径
  pub fn language_runtime_path(&self, language_code: &str) -> PathBuf {
    self.language_package_path(language_code).join("runtime")
  }

  // 命名空间文件
  pub fn language_runtime_namespace_path(&self, language_code: &str, namespace: &str) -> PathBuf {
    self
      .language_runtime_path(language_code)
      .join(format!("{}.json", namespace))
  }

  // 语言信息文件
  pub fn language_info_path(&self, language_code: &str) -> PathBuf {
    self
      .language_package_path(language_code)
      .join("language.json")
  }
}

// 获取根目录
fn resolve_root_dir() -> PathBuf {
  // 开发模式
  if let Ok(current_dir) = std::env::current_dir() {
    if current_dir.join("assets").exists() || current_dir.join("Cargo.toml").exists() {
      return current_dir;
    }
  }

  // 运行目录
  if let Ok(exe_path) = std::env::current_exe() {
    if let Some(exe_dir) = exe_path.parent() {
      return exe_dir.to_path_buf();
    }
  }

  // 最后保底
  PathBuf::from(".")
}
