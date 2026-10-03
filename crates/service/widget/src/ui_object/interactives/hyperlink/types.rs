//! Identifiers, configuration, states, and events shared by this module.

use tg_core_style::{TextColor, TextStyle};

/// The identity of hyperlink within its owning pool or session.
///
/// # Fields
///
/// * `0` - The wrapped u64 value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct HyperlinkId(
  /// The wrapped u64 value.
  pub u64,
);

/// Configuration values controlling hyperlink behavior.
///
/// # Fields
///
/// * `link` - The link.
/// * `text` - The text to process or display.
/// * `style` - The text style applied to the rendered content.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HyperlinkOptions {
  /// The link.
  pub link: String,
  /// The text to process or display.
  pub text: String,
  /// The text style applied to the rendered content.
  pub style: TextStyle,
}

impl HyperlinkOptions {
  /// Create a hyperlink options initialized from `link`, `text`.
  pub fn new(link: impl Into<String>, text: impl Into<String>) -> Self {
    Self {
      link: link.into(),
      text: text.into(),
      style: default_hyperlink_style(),
    }
  }

  /// Apply the requested style to the hyperlink configuration and return the updated options.
  pub fn style(mut self, style: TextStyle) -> Self {
    self.style = style;
    self
  }

  /// Set the hyperlink foreground color and return the updated options.
  pub fn fg(mut self, color: TextColor) -> Self {
    self.style.foreground = Some(color);
    self
  }

  /// Set the hyperlink background color and return the updated options.
  pub fn bg(mut self, color: TextColor) -> Self {
    self.style.background = Some(color);
    self
  }
}

/// A hyperlink event payload queued for its owning consumer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HyperlinkEvent {
  /// A clicked notification delivered to the owning consumer.
  Clicked {
    /// The identifier of the owned object.
    id: HyperlinkId,
    /// The link.
    link: String,
  },
}

fn default_hyperlink_style() -> TextStyle {
  use tg_core_style::{TerminalColor, TextColor};

  TextStyle {
    foreground: Some(TextColor::Terminal(TerminalColor::BrightBlue)),
    underline: true,
    ..Default::default()
  }
}
