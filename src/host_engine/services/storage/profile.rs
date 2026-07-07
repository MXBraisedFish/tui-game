use std::{collections::HashMap, fs};

use serde::{Deserialize, Serialize};

use super::layout;
use super::service::StorageService;

/// 终端配置文件：存储 Unicode 支持、颜色模式和鼠标支持的用户偏好。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TerminalProfile {
  pub unicode: Option<bool>,

  pub color: Option<String>,

  pub mouse: Option<bool>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PackageStateProfile {
  #[serde(default)]
  pub games: HashMap<String, GamePackageState>,

  #[serde(default)]
  pub screensavers: HashMap<String, ScreensaverPackageState>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct GamePackageState {
  #[serde(default = "default_enabled")]
  pub enabled: bool,

  #[serde(default)]
  pub debug: bool,

  #[serde(default = "default_safe_mode")]
  pub safe_mode: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScreensaverPackageState {
  #[serde(default = "default_enabled")]
  pub enabled: bool,

  #[serde(default)]
  pub debug: bool,
}

impl Default for GamePackageState {
  fn default() -> Self {
    Self {
      enabled: true,
      debug: false,
      safe_mode: true,
    }
  }
}

impl Default for ScreensaverPackageState {
  fn default() -> Self {
    Self {
      enabled: true,
      debug: false,
    }
  }
}

fn default_enabled() -> bool {
  true
}

fn default_safe_mode() -> bool {
  true
}

impl Default for TerminalProfile {
  fn default() -> Self {
    Self {
      unicode: None,
      color: None,
      mouse: None,
    }
  }
}

impl TerminalProfile {
  /// 检查三项配置是否已全部填写完毕。
  pub fn is_complete(&self) -> bool {
    self.unicode.is_some()
      && self
        .color
        .as_deref()
        .map_or(false, |c| c == "truecolor" || c == "256")
      && self.mouse.is_some()
  }
}

impl StorageService {
  /// 读取保存的语言代码。
  pub fn read_language_code(&self) -> Option<String> {
    let content = fs::read_to_string(self.profile_language_path()).ok()?;
    let code = content.trim();
    if code.is_empty() {
      None
    } else {
      Some(code.to_string())
    }
  }

  /// 写入语言代码到配置文件。
  pub fn write_language_code(&self, language_code: &str) -> std::io::Result<()> {
    fs::write(self.profile_language_path(), language_code.trim())
  }

  /// 返回默认语言代码。
  pub fn default_language_code(&self) -> &'static str {
    layout::DEFAULT_LANGUAGE_CODE
  }

  /// 从文件读取终端配置。
  pub fn read_terminal_profile(&self) -> Option<TerminalProfile> {
    let content = fs::read_to_string(self.profile_terminal_path()).ok()?;
    serde_json::from_str(&content).ok()
  }

  /// 读取终端配置，缺失时返回默认值。
  pub fn read_terminal_profile_or_default(&self) -> TerminalProfile {
    self.read_terminal_profile().unwrap_or_default()
  }

  /// 读取并修改终端配置后写回。
  pub fn update_terminal_profile(
    &self,
    f: impl FnOnce(&mut TerminalProfile),
  ) -> std::io::Result<()> {
    let mut profile = self.read_terminal_profile_or_default();
    f(&mut profile);
    self.write_terminal_profile(&profile)
  }

  /// 将终端配置序列化后写入文件。
  pub fn write_terminal_profile(&self, profile: &TerminalProfile) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(profile).unwrap_or_default();
    fs::write(self.profile_terminal_path(), json)
  }

  /// 检查终端配置文件是否已填写完整。
  pub fn is_terminal_profile_complete(&self) -> bool {
    self
      .read_terminal_profile()
      .map_or(false, |p| p.is_complete())
  }

  pub fn read_package_state(&self) -> Option<PackageStateProfile> {
    let content = fs::read_to_string(self.profile_package_state_path()).ok()?;
    serde_json::from_str(&content).ok()
  }

  pub fn read_package_state_or_default(&self) -> PackageStateProfile {
    self.read_package_state().unwrap_or_default()
  }

  pub fn write_package_state(&self, profile: &PackageStateProfile) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(profile).unwrap_or_default();
    fs::write(self.profile_package_state_path(), json)
  }

  pub fn update_game_package_state(
    &self,
    mod_id: &str,
    f: impl FnOnce(&mut GamePackageState),
  ) -> std::io::Result<()> {
    let mut profile = self.read_package_state_or_default();
    f(profile.games.entry(mod_id.to_string()).or_default());
    self.write_package_state(&profile)
  }

  pub fn update_screensaver_package_state(
    &self,
    mod_id: &str,
    f: impl FnOnce(&mut ScreensaverPackageState),
  ) -> std::io::Result<()> {
    let mut profile = self.read_package_state_or_default();
    f(profile.screensavers.entry(mod_id.to_string()).or_default());
    self.write_package_state(&profile)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn temp_storage(name: &str) -> StorageService {
    let root = std::env::temp_dir().join(format!("tg_storage_{name}_{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("data/profiles")).unwrap();
    StorageService::from_root_for_test(root)
  }

  #[test]
  fn missing_package_state_returns_default() {
    let storage = temp_storage("missing_package_state");
    assert_eq!(
      storage.read_package_state_or_default(),
      PackageStateProfile::default()
    );
  }

  #[test]
  fn package_state_persists_game_and_screensaver_independently() {
    let storage = temp_storage("package_state_persists");

    storage
      .update_game_package_state("same_id", |state| {
        state.enabled = false;
        state.debug = true;
        state.safe_mode = false;
      })
      .unwrap();
    storage
      .update_screensaver_package_state("same_id", |state| {
        state.enabled = false;
        state.debug = true;
      })
      .unwrap();

    let profile = storage.read_package_state_or_default();
    assert_eq!(
      profile.games.get("same_id"),
      Some(&GamePackageState {
        enabled: false,
        debug: true,
        safe_mode: false,
      })
    );
    assert_eq!(
      profile.screensavers.get("same_id"),
      Some(&ScreensaverPackageState {
        enabled: false,
        debug: true,
      })
    );
  }

  #[test]
  fn invalid_package_state_json_falls_back_to_default() {
    let storage = temp_storage("invalid_package_state");
    fs::write(storage.profile_package_state_path(), "{").unwrap();
    assert_eq!(
      storage.read_package_state_or_default(),
      PackageStateProfile::default()
    );
  }
}
