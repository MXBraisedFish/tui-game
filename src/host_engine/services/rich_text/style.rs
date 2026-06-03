// 文本样式
#[derive(Clone, Debug, Default)]
pub struct TextStyle {
  pub foreground: Option<TextColor>, // 前景颜色
  pub background: Option<TextColor>, // 背景颜色
  pub bold: bool,                    // 加粗
  pub italic: bool,                  // 斜体
  pub underline: bool,               // 下划线
  pub strike: bool,                  // 删除线
  pub blink: bool,                   // 闪烁
  pub reverse: bool,                 // 反转
  pub hidden: bool,                  // 隐藏
  pub dim: bool,                     // 变暗
}

// 颜色
#[derive(Clone, Debug, PartialEq)]
pub enum TextColor {
  Terminal(TerminalColor),
  Rgb { r: u8, g: u8, b: u8 },
}

// 终端默认16色
#[derive(Clone, Debug, PartialEq)]
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
  // 将样式标记为启用
  pub fn enable_style(&mut self, tag: &str) -> bool {
    match tag {
      "bold" => self.bold = true,
      "italic" => self.italic = true,
      "underline" => self.underline = true,
      "strike" => self.strike = true,
      "blink" => self.blink = true,
      "reverse" => self.reverse = true,
      "hidden" => self.hidden = true,
      "dim" => self.dim = true,
      _ => return false,
    }
    true
  }

  // 将样式标记为关闭
  pub fn disable_style(&mut self, tag: &str) -> bool {
    match tag {
      "bold" => self.bold = false,
      "italic" => self.italic = false,
      "underline" => self.underline = false,
      "strike" => self.strike = false,
      "blink" => self.blink = false,
      "reverse" => self.reverse = false,
      "hidden" => self.hidden = false,
      "dim" => self.dim = false,
      _ => return false,
    }
    true
  }

  // 设置前景色
  pub fn set_foreground(&mut self, color: TextColor) {
    self.foreground = Some(color);
  }

  // 清理前景色
  pub fn clear_foreground(&mut self) {
    self.foreground = None;
  }

  // 设置背景色
  pub fn set_background(&mut self, color: TextColor) {
    self.background = Some(color);
  }

  // 清理背景色
  pub fn clear_background(&mut self) {
    self.background = None;
  }

  // 重置
  pub fn reset(&mut self) {
    *self = Self::default();
  }
}
