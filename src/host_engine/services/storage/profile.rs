use std::fs;

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
}
