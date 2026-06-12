//! 图片协议推荐器。
//!
//! 只做无副作用的环境变量推测。
//! 不读取 stdin，不发送查询序列。
//! 最终结果必须由 TerminalCheck 可视确认。
//!
//! TODO: 等终端方案稳定后重命名为 TerminalCapabilityRecommender。

use super::terminal_capabilities::ImageProtocol;

/// 推荐结果（供终端检测向导预选选项）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DetectionResult {
  /// 推荐的图片协议
  pub image_protocol: ImageProtocol,
}

impl Default for DetectionResult {
  fn default() -> Self {
    Self {
      image_protocol: ImageProtocol::None,
    }
  }
}

/// 图片协议推荐器（仅环境变量，无副作用）。
pub struct TerminalDetector;

impl TerminalDetector {
  /// 执行环境变量推测，不占用 stdin。
  pub fn detect() -> DetectionResult {
    DetectionResult {
      image_protocol: Self::detect_from_env().unwrap_or(ImageProtocol::None),
    }
  }

  /// 兼容旧签名：忽略 stdout，等价于 `detect()`。
  pub fn detect_in_terminal(_stdout: &mut impl std::io::Write) -> DetectionResult {
    Self::detect()
  }

  // ── 环境变量推测 ──

  fn detect_from_env() -> Option<ImageProtocol> {
    // Kitty 终端
    if std::env::var("KITTY_WINDOW_ID").is_ok_and(|s| !s.is_empty()) {
      return Some(ImageProtocol::Kitty);
    }

    // Sixel 终端
    if let Ok(term) = std::env::var("TERM") {
      if term.to_lowercase().contains("sixel") {
        return Some(ImageProtocol::Sixel);
      }
    }
    if std::env::var("SIXEL_SUPPORTED").is_ok_and(|s| s == "1") {
      return Some(ImageProtocol::Sixel);
    }

    // iTerm2（仅匹配真正的 iTerm2，其余终端由用户手动确认）
    if let Ok(term_program) = std::env::var("TERM_PROGRAM") {
      if term_program.to_lowercase().contains("iterm") {
        return Some(ImageProtocol::ITerm2);
      }
    }
    if std::env::var("LC_TERMINAL").is_ok_and(|v| v.to_lowercase().contains("iterm")) {
      return Some(ImageProtocol::ITerm2);
    }

    None
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn detect_from_env_does_not_panic() {
    let _ = TerminalDetector::detect_from_env();
  }

  #[test]
  fn detect_returns_result_without_panic() {
    let result = TerminalDetector::detect();
    // 在任何环境中都不应 panic
    let _ = result.image_protocol;
  }

  #[test]
  fn detect_in_terminal_ignores_stdout() {
    // 验证兼容签名不 panic，且结果与 detect() 一致
    let mut dummy = Vec::new();
    let result = TerminalDetector::detect_in_terminal(&mut dummy);
    let _ = result.image_protocol;
    // stdout 不应被写入任何内容
    assert!(dummy.is_empty());
  }
}
