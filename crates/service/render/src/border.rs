//! Border support for the render service.

use tg_core_style::{TextColor, TextStyle};

/// A styled character occupying one position in a rectangle border.
///
/// # Fields
///
/// * `char` - The char.
/// * `fg` - The foreground color override, or `None` to inherit the default.
/// * `bg` - The background color override, or `None` to inherit the default.
/// * `style` - The text style applied to the rendered content.
#[derive(Clone, Debug, Default)]
pub struct BorderCharacter {
  /// The char.
  pub char: Option<char>,
  /// The foreground color override, or `None` to inherit the default.
  pub fg: Option<TextColor>,
  /// The background color override, or `None` to inherit the default.
  pub bg: Option<TextColor>,
  /// The text style applied to the rendered content.
  pub style: Option<TextStyle>,
}

impl BorderCharacter {
  /// Resolve a built-in or custom border into the characters used to draw its edges and corners.
  ///
  /// # Arguments
  ///
  /// * `default_fg` - The default fg.
  /// * `default_bg` - The default bg.
  /// * `default_style` - The default style.
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

/// Eight styled positions describing the edges and corners of a custom border.
///
/// # Fields
///
/// * `left_top` - The left top.
/// * `top` - The top.
/// * `right_top` - The right top.
/// * `right` - The right.
/// * `right_bottom` - The right bottom.
/// * `bottom` - The bottom.
/// * `left_bottom` - The left bottom.
/// * `left` - The left.
#[derive(Clone, Debug, Default)]
pub struct CustomBorder {
  /// The left top.
  pub left_top: BorderCharacter,
  /// The top.
  pub top: BorderCharacter,
  /// The right top.
  pub right_top: BorderCharacter,
  /// The right.
  pub right: BorderCharacter,
  /// The right bottom.
  pub right_bottom: BorderCharacter,
  /// The bottom.
  pub bottom: BorderCharacter,
  /// The left bottom.
  pub left_bottom: BorderCharacter,
  /// The left.
  pub left: BorderCharacter,
}

// Keep Custom unboxed to preserve the public border configuration type.

/// The built-in or custom character arrangement used for a rectangle border.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug)]
pub enum BorderStyle {
  /// The none setting for border style.
  None,
  /// The line setting for border style.
  Line,
  /// The bold setting for border style.
  Bold,
  /// The double setting for border style.
  Double,
  /// The circle setting for border style.
  Circle,
  /// The custom setting for border style.
  Custom(CustomBorder),
}

impl BorderStyle {
  /// Return the current to custom.
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
      fg: Some(TextColor::Terminal(tg_core_style::TerminalColor::Red)),
      ..Default::default()
    };
    let style = pos.resolve(
      Some(&TextColor::Terminal(tg_core_style::TerminalColor::Blue)),
      None,
      None,
    );
    assert_eq!(
      style.foreground,
      Some(TextColor::Terminal(tg_core_style::TerminalColor::Red))
    );
  }

  #[test]
  fn resolve_falls_back_to_api_when_position_none() {
    let pos = BorderCharacter::default();
    let style = pos.resolve(
      Some(&TextColor::Terminal(tg_core_style::TerminalColor::Green)),
      None,
      None,
    );
    assert_eq!(
      style.foreground,
      Some(TextColor::Terminal(tg_core_style::TerminalColor::Green))
    );
  }
}
