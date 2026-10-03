//! Text decoration flags and terminal/default/RGB color values.

/// Foreground/background colors and independent terminal text-decoration flags.
///
/// # Fields
///
/// * `foreground` - The foreground color override, or `None` to inherit the default.
/// * `background` - The background color override, or `None` to inherit the default.
/// * `bold` - Whether the bold text style is enabled.
/// * `italic` - Whether the italic text style is enabled.
/// * `underline` - Whether the underline text style is enabled.
/// * `strike` - Whether the strike text style is enabled.
/// * `blink` - Whether the blink text style is enabled.
/// * `reverse` - Whether the reverse text style is enabled.
/// * `hidden` - Whether the hidden text style is enabled.
/// * `dim` - Whether the dim text style is enabled.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TextStyle {
  /// The foreground color override, or `None` to inherit the default.
  pub foreground: Option<TextColor>,
  /// The background color override, or `None` to inherit the default.
  pub background: Option<TextColor>,
  /// Whether the bold text style is enabled.
  pub bold: bool,
  /// Whether the italic text style is enabled.
  pub italic: bool,
  /// Whether the underline text style is enabled.
  pub underline: bool,
  /// Whether the strike text style is enabled.
  pub strike: bool,
  /// Whether the blink text style is enabled.
  pub blink: bool,
  /// Whether the reverse text style is enabled.
  pub reverse: bool,
  /// Whether the hidden text style is enabled.
  pub hidden: bool,
  /// Whether the dim text style is enabled.
  pub dim: bool,
}

/// A named terminal color, explicit RGB color, or terminal-default color.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TextColor {
  /// A named ANSI palette color.
  Terminal(TerminalColor),
  /// RGB components subject to the terminal color-capability policy.
  Rgb {
    /// The red color component from 0 to 255.
    r: u8,
    /// The green color component from 0 to 255.
    g: u8,
    /// The blue color component from 0 to 255.
    b: u8,
  },
  /// RGB components explicitly requesting true-color output.
  ForceRgb {
    /// The red color component from 0 to 255.
    r: u8,
    /// The green color component from 0 to 255.
    g: u8,
    /// The blue color component from 0 to 255.
    b: u8,
  },

  /// The inherited or terminal-default color rather than an explicit RGB value.
  Transparent,
}

/// One of the sixteen named ANSI colors resolved by the terminal or export palette.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TerminalColor {
  /// The black entry in the ANSI palette.
  Black,
  /// The red entry in the ANSI palette.
  Red,
  /// The green entry in the ANSI palette.
  Green,
  /// The yellow entry in the ANSI palette.
  Yellow,
  /// The blue entry in the ANSI palette.
  Blue,
  /// The magenta entry in the ANSI palette.
  Magenta,
  /// The cyan entry in the ANSI palette.
  Cyan,
  /// The white entry in the ANSI palette.
  White,
  /// The bright black entry in the ANSI palette.
  BrightBlack,
  /// The bright red entry in the ANSI palette.
  BrightRed,
  /// The bright green entry in the ANSI palette.
  BrightGreen,
  /// The bright yellow entry in the ANSI palette.
  BrightYellow,
  /// The bright blue entry in the ANSI palette.
  BrightBlue,
  /// The bright magenta entry in the ANSI palette.
  BrightMagenta,
  /// The bright cyan entry in the ANSI palette.
  BrightCyan,
  /// The bright white entry in the ANSI palette.
  BrightWhite,
}

impl TextStyle {
  /// Enable a recognized style tag and return whether the tag is supported.
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

  /// Disable a recognized style tag and return whether the tag is supported.
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

  /// Assign the text foreground color.
  pub fn set_foreground(&mut self, color: TextColor) {
    self.foreground = Some(color);
  }

  /// Reset the foreground to the transparent/default color.
  pub fn clear_foreground(&mut self) {
    self.foreground = None;
  }

  /// Invert explicit RGB foreground components while leaving named terminal colors unchanged.
  pub fn reverse_foreground(&mut self) {
    if let Some(color) = self.foreground.as_mut() {
      color.reverse_rgb();
    }
  }

  /// Assign the cell background color.
  pub fn set_background(&mut self, color: TextColor) {
    self.background = Some(color);
  }

  /// Reset the background to the transparent/default color.
  pub fn clear_background(&mut self) {
    self.background = None;
  }

  /// Invert explicit RGB background components while leaving named terminal colors unchanged.
  pub fn reverse_background(&mut self) {
    if let Some(color) = self.background.as_mut() {
      color.reverse_rgb();
    }
  }

  /// Reset the text style state addressed by this operation.
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
