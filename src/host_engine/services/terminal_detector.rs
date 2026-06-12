//! 图片协议自动检测器。
//!
//! 参考 ratatui-image 的 Picker 设计，两层检测：
//! 1. stdin/stdout 应答查询（Kitty / Sixel 支持）
//! 2. 环境变量推测（iTerm2、WezTerm 等不支持 stdin 应答的终端）
//!
//! 必须在 crossterm 事件监听线程启动前调用，否则 stdin 响应会被抢占。

use std::io::{self, Read, Write};
use std::time::Duration;

use super::terminal_capabilities::ImageProtocol;

/// 自动检测结果。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DetectionResult {
  /// 检测到的图片协议
  pub image_protocol: ImageProtocol,
}

impl Default for DetectionResult {
  fn default() -> Self {
    Self {
      image_protocol: ImageProtocol::None,
    }
  }
}

/// 图片协议检测器。
pub struct TerminalDetector;

impl TerminalDetector {
  /// 在已经进入 alt screen/raw mode 的终端内检测。
  /// 必须在 crossterm 事件监听线程启动前调用，
  /// 否则 `ct_event::read()` 会消耗 stdin 查询应答。
  pub fn detect_in_terminal(stdout: &mut impl Write) -> DetectionResult {
    Self::detect_with_writer(stdout)
  }

  fn detect_with_writer(stdout: &mut impl Write) -> DetectionResult {
    let mut result = DetectionResult::default();

    // stdin 响应是权威结果；环境变量只用于无响应时兜底。
    let stdin_proto = Self::query_stdio(stdout);
    let env_proto = Self::detect_from_env();
    result.image_protocol = choose_image_protocol(stdin_proto, env_proto);

    result
  }

  // ── 环境变量图片协议检测 ──

  fn detect_from_env() -> Option<ImageProtocol> {
    // Kitty 终端
    if std::env::var("KITTY_WINDOW_ID").is_ok_and(|s| !s.is_empty()) {
      return Some(ImageProtocol::Kitty);
    }

    // Sixel 终端
    if let Ok(term) = std::env::var("TERM") {
      if term.contains("sixel") || term.contains("SIXEL") {
        return Some(ImageProtocol::Sixel);
      }
    }
    if std::env::var("SIXEL_SUPPORTED").is_ok_and(|s| s == "1") {
      return Some(ImageProtocol::Sixel);
    }

    // iTerm2 系终端
    if let Ok(term_program) = std::env::var("TERM_PROGRAM") {
      let term_program = term_program.to_lowercase();
      if term_program.contains("iterm")
        || term_program.contains("wezterm")
        || term_program.contains("mintty")
        || term_program.contains("vscode")
        || term_program.contains("tabby")
        || term_program.contains("hyper")
        || term_program.contains("rio")
        || term_program.contains("warp")
      {
        return Some(ImageProtocol::ITerm2);
      }
    }

    if std::env::var("LC_TERMINAL").is_ok_and(|v| v.contains("iTerm")) {
      return Some(ImageProtocol::ITerm2);
    }

    None
  }

  // ── stdin 应答查询 ──

  fn query_stdio(stdout: &mut impl Write) -> Option<ImageProtocol> {
    Self::query_stdio_with_timeout(stdout, Duration::from_millis(500))
  }

  fn query_stdio_with_timeout(
    stdout: &mut impl Write,
    timeout: Duration,
  ) -> Option<ImageProtocol> {
    use std::sync::mpsc;
    use std::thread;

    // 1. Kitty 图形协议询问: ESC _Gi=31,s=1,v=1,a=q,t=d,f=24;AAAA ESC \
    // 2. Sixel 支持询问: ESC [c (Device Attributes)
    // 3. Cell pixel size: ESC [16t
    // 4. DSR 兜底: ESC [5n (确保有应答，避免无限等待)
    let query = query_sequence();
    if stdout
      .write_all(query.as_bytes())
      .and_then(|_| stdout.flush())
      .is_err()
    {
      return None;
    }

    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
      let result = Self::read_query_response(Duration::from_millis(250));
      let _ = tx.send(result);
    });

    match rx.recv_timeout(timeout) {
      Ok(result) => result,
      Err(_) => None, // 超时
    }
  }

  fn read_query_response(timeout: Duration) -> Option<ImageProtocol> {
    let mut buf = [0u8; 256];
    let mut response = String::new();
    let mut stdin = io::stdin();
    let mut has_dsr = false;

    let deadline = std::time::Instant::now() + timeout;

    while !has_dsr && std::time::Instant::now() < deadline {
      match stdin.read(&mut buf) {
        Ok(0) => break,
        Ok(n) => {
          response.push_str(&String::from_utf8_lossy(&buf[..n]));
          // DSR 应答是 ESC [0n
          if response.contains("\x1b[0n") {
            has_dsr = true;
          }
        }
        Err(_) => break,
      }
    }

    parse_terminal_response(&response)
  }
}

fn choose_image_protocol(
  stdin_proto: Option<ImageProtocol>,
  env_proto: Option<ImageProtocol>,
) -> ImageProtocol {
  stdin_proto.or(env_proto).unwrap_or(ImageProtocol::None)
}

fn query_sequence() -> &'static str {
  "\x1b_Gi=31,s=1,v=1,a=q,t=d,f=24;AAAA\x1b\\\x1b[c\x1b[16t\x1b[5n"
}

/// 解析终端应答字符串。
fn parse_terminal_response(response: &str) -> Option<ImageProtocol> {
  let mut protocol: Option<ImageProtocol> = None;

  // Kitty: _Gi=31;OK
  if response.contains("_Gi=31;OK") {
    protocol = Some(ImageProtocol::Kitty);
  }

  // Sixel: Device Attributes 回复格式 ESC [?...;4c
  // 参数含 "4" 表示支持 Sixel
  let mut rest = response;
  while let Some(da_start) = rest.find("\x1b[?") {
    let after_escape = &rest[da_start + 1..]; // skip ESC
    if let Some(c_end) = after_escape.find('c') {
      let params = &after_escape[1..c_end]; // skip [
      if params.split(';').any(|p| p == "4") {
        // 仅在未检测到 Kitty 时使用 Sixel
        if protocol.is_none() {
          protocol = Some(ImageProtocol::Sixel);
        }
      }
      rest = &after_escape[c_end + 1..];
    } else {
      break;
    }
  }

  protocol
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_detect_from_env_does_not_panic() {
    let _ = TerminalDetector::detect_from_env();
  }

  #[test]
  fn test_parse_kitty_response() {
    let response = "\x1b_Gi=31;OK\x1b\\\x1b[0n";
    let proto = parse_terminal_response(response);
    assert_eq!(proto, Some(ImageProtocol::Kitty));
  }

  #[test]
  fn test_parse_sixel_response() {
    let response = "\x1b[?64;4c\x1b[0n";
    let proto = parse_terminal_response(response);
    assert_eq!(proto, Some(ImageProtocol::Sixel));
  }

  #[test]
  fn test_parse_no_protocol() {
    let response = "\x1b[0n";
    let proto = parse_terminal_response(response);
    assert_eq!(proto, None);
  }

  #[test]
  fn test_parse_sixel_after_cell_size_response() {
    let response = "\x1b[6;20;10t\x1b[?64;4c\x1b[0n";
    let proto = parse_terminal_response(response);
    assert_eq!(proto, Some(ImageProtocol::Sixel));
  }

  #[test]
  fn test_parse_kitty_takes_priority_over_sixel() {
    let response = "\x1b[?64;4c\x1b_Gi=31;OK\x1b\\\x1b[0n";
    let proto = parse_terminal_response(response);
    assert_eq!(proto, Some(ImageProtocol::Kitty));
  }

  #[test]
  fn test_stdin_protocol_takes_priority_over_env_iterm2() {
    assert_eq!(
      choose_image_protocol(Some(ImageProtocol::Kitty), Some(ImageProtocol::ITerm2)),
      ImageProtocol::Kitty
    );
    assert_eq!(
      choose_image_protocol(Some(ImageProtocol::Sixel), Some(ImageProtocol::ITerm2)),
      ImageProtocol::Sixel
    );
  }

  #[test]
  fn test_env_iterm2_is_only_fallback() {
    assert_eq!(
      choose_image_protocol(None, Some(ImageProtocol::ITerm2)),
      ImageProtocol::ITerm2
    );
    assert_eq!(choose_image_protocol(None, None), ImageProtocol::None);
  }

  #[test]
  fn test_query_sequence_contains_all_capability_queries() {
    let query = query_sequence();
    assert!(query.contains("_Gi=31"));
    assert!(query.contains("\x1b[c"));
    assert!(query.contains("\x1b[16t"));
    assert!(query.ends_with("\x1b[5n"));
  }
}
