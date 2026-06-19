use crate::host_engine::services::{TextColor, TextStyle};

// ── 边框样式类型 ──

/// 每个位置的边框字符配置。
/// 所有字段均为可选，`None` 表示继承 API 级默认值或使用固定样式默认字符。
#[derive(Clone, Debug, Default)]
pub struct BorderCharacter {
  pub char: Option<char>,
  pub fg: Option<TextColor>,
  pub bg: Option<TextColor>,
  pub style: Option<TextStyle>,
}

impl BorderCharacter {
  /// 按优先级解析最终字符与样式：本位置覆盖 > API 参数 > 固定样式默认。
  pub fn resolve(
    &self,
    default_fg: Option<&TextColor>,
    default_bg: Option<&TextColor>,
    default_style: Option<&TextStyle>,
  ) -> TextStyle {
    let fg = self.fg.as_ref().or(default_fg);
    let bg = self.bg.as_ref().or(default_bg);

    let base = if let Some(ref s) = self.style {
      s.clone()
    } else if let Some(s) = default_style {
      s.clone()
    } else {
      TextStyle::default()
    };

    TextStyle {
      foreground: fg.cloned(),
      background: bg.cloned(),
      ..base
    }
  }
}

/// 8 位置自定义边框（上排 l_t / top / r_t，中排 left / right，下排 l_b / bottom / r_b）。
#[derive(Clone, Debug, Default)]
pub struct CustomBorder {
  pub left_top: BorderCharacter,
  pub top: BorderCharacter,
  pub right_top: BorderCharacter,
  pub right: BorderCharacter,
  pub right_bottom: BorderCharacter,
  pub bottom: BorderCharacter,
  pub left_bottom: BorderCharacter,
  pub left: BorderCharacter,
}

// ── 固定边框样式 → CustomBorder 映射 ──

/// 边框样式枚举。
#[derive(Clone, Debug)]
pub enum BorderStyle {
  None,
  Line,
  Bold,
  Double,
  Circle,
  Custom(CustomBorder),
}

impl BorderStyle {
  /// 将枚举转为具体的 `CustomBorder`。`None` 返回 `None`（不绘制）。
  pub fn to_custom(&self) -> Option<CustomBorder> {
    match self {
      Self::None => None,
      Self::Line => Some(CustomBorder {
        left_top: BorderCharacter {
          char: Some('┌'),
          ..Default::default()
        },
        top: BorderCharacter {
          char: Some('─'),
          ..Default::default()
        },
        right_top: BorderCharacter {
          char: Some('┐'),
          ..Default::default()
        },
        right: BorderCharacter {
          char: Some('│'),
          ..Default::default()
        },
        right_bottom: BorderCharacter {
          char: Some('┘'),
          ..Default::default()
        },
        bottom: BorderCharacter {
          char: Some('─'),
          ..Default::default()
        },
        left_bottom: BorderCharacter {
          char: Some('└'),
          ..Default::default()
        },
        left: BorderCharacter {
          char: Some('│'),
          ..Default::default()
        },
      }),
      Self::Bold => Some(CustomBorder {
        left_top: BorderCharacter {
          char: Some('┏'),
          ..Default::default()
        },
        top: BorderCharacter {
          char: Some('━'),
          ..Default::default()
        },
        right_top: BorderCharacter {
          char: Some('┓'),
          ..Default::default()
        },
        right: BorderCharacter {
          char: Some('┃'),
          ..Default::default()
        },
        right_bottom: BorderCharacter {
          char: Some('┛'),
          ..Default::default()
        },
        bottom: BorderCharacter {
          char: Some('━'),
          ..Default::default()
        },
        left_bottom: BorderCharacter {
          char: Some('┗'),
          ..Default::default()
        },
        left: BorderCharacter {
          char: Some('┃'),
          ..Default::default()
        },
      }),
      Self::Double => Some(CustomBorder {
        left_top: BorderCharacter {
          char: Some('╔'),
          ..Default::default()
        },
        top: BorderCharacter {
          char: Some('═'),
          ..Default::default()
        },
        right_top: BorderCharacter {
          char: Some('╗'),
          ..Default::default()
        },
        right: BorderCharacter {
          char: Some('║'),
          ..Default::default()
        },
        right_bottom: BorderCharacter {
          char: Some('╝'),
          ..Default::default()
        },
        bottom: BorderCharacter {
          char: Some('═'),
          ..Default::default()
        },
        left_bottom: BorderCharacter {
          char: Some('╚'),
          ..Default::default()
        },
        left: BorderCharacter {
          char: Some('║'),
          ..Default::default()
        },
      }),
      Self::Circle => Some(CustomBorder {
        left_top: BorderCharacter {
          char: Some('╭'),
          ..Default::default()
        },
        top: BorderCharacter {
          char: Some('─'),
          ..Default::default()
        },
        right_top: BorderCharacter {
          char: Some('╮'),
          ..Default::default()
        },
        right: BorderCharacter {
          char: Some('│'),
          ..Default::default()
        },
        right_bottom: BorderCharacter {
          char: Some('╯'),
          ..Default::default()
        },
        bottom: BorderCharacter {
          char: Some('─'),
          ..Default::default()
        },
        left_bottom: BorderCharacter {
          char: Some('╰'),
          ..Default::default()
        },
        left: BorderCharacter {
          char: Some('│'),
          ..Default::default()
        },
      }),
      Self::Custom(c) => Some(c.clone()),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn line_border_has_all_eight_positions() {
    let c = BorderStyle::Line.to_custom().unwrap();
    assert_eq!(c.left_top.char, Some('┌'));
    assert_eq!(c.top.char, Some('─'));
    assert_eq!(c.right_top.char, Some('┐'));
    assert_eq!(c.right.char, Some('│'));
    assert_eq!(c.right_bottom.char, Some('┘'));
    assert_eq!(c.bottom.char, Some('─'));
    assert_eq!(c.left_bottom.char, Some('└'));
    assert_eq!(c.left.char, Some('│'));
  }

  #[test]
  fn none_returns_none() {
    assert!(BorderStyle::None.to_custom().is_none());
  }

  #[test]
  fn resolve_uses_position_over_api() {
    let pos = BorderCharacter {
      fg: Some(TextColor::Terminal(
        crate::host_engine::services::TerminalColor::Red,
      )),
      ..Default::default()
    };
    let style = pos.resolve(
      Some(&TextColor::Terminal(
        crate::host_engine::services::TerminalColor::Blue,
      )),
      None,
      None,
    );
    assert_eq!(
      style.foreground,
      Some(TextColor::Terminal(
        crate::host_engine::services::TerminalColor::Red
      ))
    );
  }

  #[test]
  fn resolve_falls_back_to_api_when_position_none() {
    let pos = BorderCharacter::default();
    let style = pos.resolve(
      Some(&TextColor::Terminal(
        crate::host_engine::services::TerminalColor::Green,
      )),
      None,
      None,
    );
    assert_eq!(
      style.foreground,
      Some(TextColor::Terminal(
        crate::host_engine::services::TerminalColor::Green
      ))
    );
  }
}
