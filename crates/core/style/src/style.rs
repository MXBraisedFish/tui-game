/// Terminal text style: foreground and background colors plus text decoration flags.
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

/// Text color: a terminal color, an RGB true color or transparent.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TextColor {
  Terminal(TerminalColor),
  Rgb { r: u8, g: u8, b: u8 },
  ForceRgb { r: u8, g: u8, b: u8 },

  Transparent,
}

/// One of the 16 ANSI terminal colors.
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
  /// Enables the text decoration named by `tag` (such as "bold" or "italic") and returns
  /// whether the tag was recognized.
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

  /// Disables the text decoration named by `tag` and returns whether the tag was recognized.
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

  /// Inverts the explicit RGB foreground color. Named terminal colors depend on the terminal
  /// theme and stay unchanged.
  pub fn reverse_foreground(&mut self) {
    if let Some(color) = self.foreground.as_mut() {
      color.reverse_rgb();
    }
  }

  pub fn set_background(&mut self, color: TextColor) {
    self.background = Some(color);
  }

  pub fn clear_background(&mut self) {
    self.background = None;
  }

  /// Inverts the explicit RGB background color. Named terminal colors depend on the terminal
  /// theme and stay unchanged.
  pub fn reverse_background(&mut self) {
    if let Some(color) = self.background.as_mut() {
      color.reverse_rgb();
    }
  }

  /// Resets the style to its default value.
  pub fn reset(&mut self) {
    *self = Self::default();
  }
}

impl TextColor {
  fn reverse_rgb(&mut self) {
    match self {
      Self::Rgb { r, g, b } | Self::ForceRgb { r, g, b } => {
        *r = 255 - *r;
        *g = 255 - *g;
        *b = 255 - *b;
      }
      Self::Terminal(_) | Self::Transparent => {}
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn style_tags_toggle_flags_and_unknown_tags_are_rejected() {
    let mut style = TextStyle::default();
    assert!(style.enable_style("b"));
    assert!(style.enable_style("underline"));
    assert!(!style.enable_style("wobble"));
    assert!(style.bold && style.underline);
    assert!(style.disable_style("bold"));
    assert!(!style.bold);
    style.reset();
    assert_eq!(style, TextStyle::default());
  }

  #[test]
  fn reversing_only_changes_explicit_rgb_colors() {
    let mut style = TextStyle::default();
    style.set_foreground(TextColor::Rgb {
      r: 0,
      g: 100,
      b: 255,
    });
    style.set_background(TextColor::Terminal(TerminalColor::Red));
    style.reverse_foreground();
    style.reverse_background();
    assert_eq!(
      style.foreground,
      Some(TextColor::Rgb {
        r: 255,
        g: 155,
        b: 0
      })
    );
    assert_eq!(
      style.background,
      Some(TextColor::Terminal(TerminalColor::Red))
    );
    style.clear_foreground();
    assert_eq!(style.foreground, None);
  }
}
