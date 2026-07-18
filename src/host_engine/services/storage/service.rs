use std::{
  io,
  path::{Path, PathBuf},
};

use super::bootstrap::ensure_storage_layout;
use super::layout;
use super::profile::DisplaySettingsProfile;
use crate::host_engine::services::{LogService, LogSource};

/// 存储服务：管理应用根目录，提供各子路径的构建方法，并在初始化时确保目录结构存在。
pub struct StorageService {
  root_dir: PathBuf,
  pub(super) display_settings: DisplaySettingsProfile,
}

impl StorageService {
  pub fn new(log: &mut LogService) -> Self {
    let root_dir = resolve_root_dir(log);

    let mut service = Self {
      root_dir,
      display_settings: DisplaySettingsProfile::default(),
    };

    ensure_storage_layout(&service, log);
    service.reload_display_settings_profile(log);

    service
  }

  pub fn root_dir(&self) -> &Path {
    &self.root_dir
  }

  pub fn data_dir_path(&self) -> PathBuf {
    self.path(layout::DATA_DIR)
  }

  pub fn cache_dir_path(&self) -> PathBuf {
    self.path(layout::DATA_CACHE_DIR)
  }

  pub fn log_dir_path(&self) -> PathBuf {
    self.path(layout::DATA_LOG_DIR)
  }

  pub fn screenshot_dir_path(&self) -> PathBuf {
    self.path(layout::DATA_SCREENSHOT_DIR)
  }

  pub fn screenshot_cache_dir_path(&self) -> PathBuf {
    self.path(layout::SCREENSHOT_CACHE_DIR)
  }

  pub fn recording_dir_path(&self) -> PathBuf {
    self.path(layout::DATA_RECORDING_DIR)
  }

  pub fn tui_log_path(&self) -> PathBuf {
    self.path(layout::TUI_LOG_FILE)
  }

  pub fn mod_dir_path(&self) -> PathBuf {
    self.path(layout::DATA_MOD_DIR)
  }

  pub fn profiles_dir_path(&self) -> PathBuf {
    self.path(layout::DATA_PROFILES_DIR)
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

  pub fn profile_package_state_path(&self) -> PathBuf {
    self.path(layout::PROFILE_PACKAGE_STATE_FILE)
  }

  pub fn profile_screenshot_path(&self) -> PathBuf {
    self.path(layout::PROFILE_SCREENSHOT_FILE)
  }

  pub fn profile_display_settings_path(&self) -> PathBuf {
    self.path(layout::PROFILE_DISPLAY_SETTINGS_FILE)
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

  pub fn clear_data(&self, log: &mut LogService) -> io::Result<()> {
    self.remove_recreate(self.data_dir_path(), log)
  }

  pub fn clear_cache(&self, log: &mut LogService) -> io::Result<()> {
    self.remove_recreate(self.cache_dir_path(), log)
  }

  pub fn clear_log(&self, log: &mut LogService) -> io::Result<()> {
    self.remove_recreate(self.log_dir_path(), log)
  }

  pub fn clear_screenshot(&self, log: &mut LogService) -> io::Result<()> {
    self.remove_recreate(self.screenshot_dir_path(), log)
  }

  pub fn clear_recording(&self, log: &mut LogService) -> io::Result<()> {
    self.remove_recreate(self.recording_dir_path(), log)
  }

  pub fn clear_mod(&self, log: &mut LogService) -> io::Result<()> {
    self.remove_recreate(self.mod_dir_path(), log)
  }

  pub fn clear_profiles(&self, log: &mut LogService) -> io::Result<()> {
    self.remove_recreate(self.profiles_dir_path(), log)
  }

  fn remove_recreate(&self, path: PathBuf, log: &mut LogService) -> io::Result<()> {
    if path.exists() {
      std::fs::remove_dir_all(path)?;
    }
    ensure_storage_layout(self, log);
    Ok(())
  }
}

#[cfg(test)]
impl StorageService {
  pub(crate) fn from_root_for_test(root_dir: PathBuf) -> Self {
    Self {
      root_dir,
      display_settings: DisplaySettingsProfile::default(),
    }
  }
}

// 自动探测应用根目录：依次尝试当前目录、可执行文件目录。
fn resolve_root_dir(log: &mut LogService) -> PathBuf {
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
  log.warn(
    LogSource::Boot,
    "Could not resolve root directory, falling back to '.'",
  );
  PathBuf::from(".")
}
