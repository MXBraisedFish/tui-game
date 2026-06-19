use std::fs;

use serde::{Deserialize, Serialize};

use super::layout;
use super::service::StorageService;

/// 终端能力配置。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TerminalProfile {
  /// `null` = 未检测
  pub unicode: Option<bool>,
  /// `"truecolor"` | `"256"`，`null` = 未检测
  pub color: Option<String>,
  /// `null` = 未检测
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
  /// 检查所有字段是否已填充（非 null 且值合法）。
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
  // 读取语言代码
  pub fn read_language_code(&self) -> Option<String> {
    let content = fs::read_to_string(self.profile_language_path()).ok()?;
    let code = content.trim();

    // 判断是否为空
    if code.is_empty() {
      None
    } else {
      Some(code.to_string())
    }
  }

  // 写入持久化语言代码
  pub fn write_language_code(&self, language_code: &str) -> std::io::Result<()> {
    fs::write(self.profile_language_path(), language_code.trim())
  }

  // 默认语言代码
  pub fn default_language_code(&self) -> &'static str {
    layout::DEFAULT_LANGUAGE_CODE
  }

  // ── 终端能力 profile ──

  /// 读取终端能力配置，文件不存在或解析失败返回 `None`。
  pub fn read_terminal_profile(&self) -> Option<TerminalProfile> {
    let content = fs::read_to_string(self.profile_terminal_path()).ok()?;
    serde_json::from_str(&content).ok()
  }

  /// 读取终端能力配置，文件不存在则返回默认（全 `None`）。
  pub fn read_terminal_profile_or_default(&self) -> TerminalProfile {
    self.read_terminal_profile().unwrap_or_default()
  }

  /// 读取 → 修改 → 写回。用于检测流程每一步完成后部分保存。
  pub fn update_terminal_profile(
    &self,
    f: impl FnOnce(&mut TerminalProfile),
  ) -> std::io::Result<()> {
    let mut profile = self.read_terminal_profile_or_default();
    f(&mut profile);
    self.write_terminal_profile(&profile)
  }

  /// 写入终端能力配置。
  pub fn write_terminal_profile(&self, profile: &TerminalProfile) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(profile).unwrap_or_default();
    fs::write(self.profile_terminal_path(), json)
  }

  /// 终端能力 profile 是否完整（所有字段已填充且合法）。
  pub fn is_terminal_profile_complete(&self) -> bool {
    self
      .read_terminal_profile()
      .map_or(false, |p| p.is_complete())
  }
}
