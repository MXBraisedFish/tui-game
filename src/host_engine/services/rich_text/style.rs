/// 终端文本样式：包含前景色、背景色及各种文本修饰属性。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TextStyle {
  pub foreground: Option<TextColor>,
  pub background: Option<TextColor>,
  pub bold: bool,
  pub italic: bool,
  pub underline: bool,
  pub strike: bool,
  pub blink: bool,
  pub reverse: bool,
  pub hidden: bool,
  pub dim: bool,
}

/// 文本颜色：终端色、RGB 真彩色或透明。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TextColor {
  Terminal(TerminalColor),
  Rgb { r: u8, g: u8, b: u8 },
  ForceRgb { r: u8, g: u8, b: u8 },

  Transparent,
}

/// ANSI 16 色终端颜色枚举。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TerminalColor {
  Black,
  Red,
  Green,
  Yellow,
  Blue,
  Magenta,
  Cyan,
  White,
  BrightBlack,
  BrightRed,
  BrightGreen,
  BrightYellow,
  BrightBlue,
  BrightMagenta,
  BrightCyan,
  BrightWhite,
}

impl TextStyle {
  /// 按标签名启用一种文本修饰（如 "bold"、"italic" 等），返回是否识别成功。
  pub fn enable_style(&mut self, tag: &str) -> bool {
    match tag {
      "bold" | "b" => self.bold = true,
      "italic" | "i" => self.italic = true,
      "underline" | "u" => self.underline = true,
      "strike" | "s" => self.strike = true,
      "blink" | "l" => self.blink = true,
      "reverse" | "r" => self.reverse = true,
      "hidden" | "h" => self.hidden = true,
      "dim" | "d" => self.dim = true,
      _ => return false,
    }
    true
  }

  /// 按标签名禁用一个文本修饰，返回是否识别成功。
  pub fn disable_style(&mut self, tag: &str) -> bool {
    match tag {
      "bold" | "b" => self.bold = false,
      "italic" | "i" => self.italic = false,
      "underline" | "u" => self.underline = false,
      "strike" | "s" => self.strike = false,
      "blink" | "l" => self.blink = false,
      "reverse" | "r" => self.reverse = false,
      "hidden" | "h" => self.hidden = false,
      "dim" | "d" => self.dim = false,
      _ => return false,
    }
    true
  }

  pub fn set_foreground(&mut self, color: TextColor) {
    self.foreground = Some(color);
  }

  pub fn clear_foreground(&mut self) {
    self.foreground = None;
  }

  pub fn set_background(&mut self, color: TextColor) {
    self.background = Some(color);
  }

  pub fn clear_background(&mut self) {
    self.background = None;
  }

  /// 将样式重置为默认值。
  pub fn reset(&mut self) {
    *self = Self::default();
  }
}
