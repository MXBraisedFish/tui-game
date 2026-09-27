use tg_core_style::{TextColor, TextStyle};

/// The character and style configuration of a single border position.
#[derive(Clone, Debug, Default)]
pub struct BorderCharacter {
  pub char: Option<char>,
  pub fg: Option<TextColor>,
  pub bg: Option<TextColor>,
  pub style: Option<TextStyle>,
}

impl BorderCharacter {
  /// Merges the position's style with the defaults into the final [`TextStyle`] used for rendering.
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

/// The characters and styles of the eight positions of a custom border.
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

/// A border style: none, single line, bold, double line, rounded corners, or custom.
// reason: boxing the large `Custom` variant would change the public `BorderStyle::Custom` type.
#[allow(clippy::large_enum_variant)]
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
  /// Expands the style into a concrete [`CustomBorder`]; returns `None` for [`BorderStyle::None`].
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
