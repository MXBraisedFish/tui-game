use super::{TerminalColor, TextColor};

/// 解析颜色字符串为 TextColor，支持终端色名、十六进制和 rgb() 格式。
pub fn parse_text_color(value: &str) -> Option<TextColor> {
  let value = value.trim();

  if let Some(color) = parse_terminal_color(value) {
    return Some(TextColor::Terminal(color));
  }

  if let Some(color) = parse_hex_color(value) {
    return Some(color);
  }

  if let Some(color) = parse_rgb_color(value) {
    return Some(color);
  }

  None
}

fn parse_terminal_color(value: &str) -> Option<TerminalColor> {
  match value {
    "black" => Some(TerminalColor::Black),
    "red" => Some(TerminalColor::Red),
    "green" => Some(TerminalColor::Green),
    "yellow" => Some(TerminalColor::Yellow),
    "blue" => Some(TerminalColor::Blue),
    "magenta" => Some(TerminalColor::Magenta),
    "cyan" => Some(TerminalColor::Cyan),

    "bright_black" | "gray" | "grey" => Some(TerminalColor::BrightBlack),
    "dark_white" | "bright_gray" | "bright_grey" => Some(TerminalColor::White),
    "bright_red" => Some(TerminalColor::BrightRed),
    "bright_green" => Some(TerminalColor::BrightGreen),
    "bright_yellow" => Some(TerminalColor::BrightYellow),
    "bright_blue" => Some(TerminalColor::BrightBlue),
    "bright_magenta" => Some(TerminalColor::BrightMagenta),
    "bright_cyan" => Some(TerminalColor::BrightCyan),
    "bright_white" | "white" => Some(TerminalColor::BrightWhite),

    _ => None,
  }
}

fn parse_hex_color(value: &str) -> Option<TextColor> {
  let hex = value.strip_prefix('#')?;

  if hex.len() != 6 {
    return None;
  }

  let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
  let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
  let b = u8::from_str_radix(&hex[4..6], 16).ok()?;

  Some(TextColor::Rgb { r, g, b })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn semantic_gray_aliases_map_to_terminal_palette() {
    for name in ["bright_black", "gray", "grey"] {
      assert_eq!(
        parse_text_color(name),
        Some(TextColor::Terminal(TerminalColor::BrightBlack))
      );
    }

    for name in ["dark_white", "bright_gray", "bright_grey"] {
      assert_eq!(
        parse_text_color(name),
        Some(TextColor::Terminal(TerminalColor::White))
      );
    }
  }

  #[test]
  fn semantic_white_uses_bright_white_and_keeps_formal_alias() {
    for name in ["white", "bright_white"] {
      assert_eq!(
        parse_text_color(name),
        Some(TextColor::Terminal(TerminalColor::BrightWhite))
      );
    }
  }
}

fn parse_rgb_color(value: &str) -> Option<TextColor> {
  let inner = value.strip_prefix("rgb(")?.strip_suffix(')')?;
  let parts: Vec<&str> = inner.split(',').map(|part| part.trim()).collect();

  if parts.len() != 3 {
    return None;
  }

  let r = parts[0].parse::<u8>().ok()?;
  let g = parts[1].parse::<u8>().ok()?;
  let b = parts[2].parse::<u8>().ok()?;

  Some(TextColor::Rgb { r, g, b })
}
