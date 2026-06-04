use crossterm::style::{Attribute, Color};

use crate::host_engine::services::{TerminalColor, TextColor};

use super::{rgb_to_ansi256, CanvasStyle};

// 转换终端16色
pub fn terminal_color_to_crossterm_color(color: &TerminalColor) -> Color {
  match color {
    TerminalColor::Black => Color::Black,
    TerminalColor::Red => Color::DarkRed,
    TerminalColor::Green => Color::DarkGreen,
    TerminalColor::Yellow => Color::DarkYellow,
    TerminalColor::Blue => Color::DarkBlue,
    TerminalColor::Magenta => Color::DarkMagenta,
    TerminalColor::Cyan => Color::DarkCyan,
    TerminalColor::White => Color::Grey,

    TerminalColor::BrightBlack => Color::DarkGrey,
    TerminalColor::BrightRed => Color::Red,
    TerminalColor::BrightGreen => Color::Green,
    TerminalColor::BrightYellow => Color::Yellow,
    TerminalColor::BrightBlue => Color::Blue,
    TerminalColor::BrightMagenta => Color::Magenta,
    TerminalColor::BrightCyan => Color::Cyan,
    TerminalColor::BrightWhite => Color::White,
  }
}

// 转换文本颜色
//
// 当 `truecolor` 为 true 时直接使用 RGB 真彩色输出；
// 为 false 时将 RGB 降级为 ANSI256 调色板中最接近的颜色。
pub fn text_color_to_crossterm_color(color: &TextColor, truecolor: bool) -> Color {
  match color {
    TextColor::Terminal(color) => terminal_color_to_crossterm_color(color),
    TextColor::Rgb { r, g, b } => {
      if truecolor {
        Color::Rgb {
          r: *r,
          g: *g,
          b: *b,
        }
      } else {
        Color::AnsiValue(rgb_to_ansi256(*r, *g, *b))
      }
    }
  }
}

// 样式属性列表
pub fn style_attributes(style: &CanvasStyle) -> Vec<Attribute> {
  let mut attributes = Vec::new();

  if style.bold {
    attributes.push(Attribute::Bold);
  }
  if style.italic {
    attributes.push(Attribute::Italic);
  }
  if style.underline {
    attributes.push(Attribute::Underlined);
  }
  if style.strike {
    attributes.push(Attribute::CrossedOut);
  }
  if style.blink {
    attributes.push(Attribute::SlowBlink);
  }
  if style.reverse {
    attributes.push(Attribute::Reverse);
  }
  if style.hidden {
    attributes.push(Attribute::Hidden);
  }
  if style.dim {
    attributes.push(Attribute::Dim);
  }

  attributes
}
