use crate::host_engine::services::{TextColor, TextStyle};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct HyperlinkId(pub u64);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HyperlinkOptions {
  pub link: String,
  pub text: String,
  pub style: TextStyle,
}

impl HyperlinkOptions {
  pub fn new(link: impl Into<String>, text: impl Into<String>) -> Self {
    Self {
      link: link.into(),
      text: text.into(),
      style: default_hyperlink_style(),
    }
  }

  pub fn style(mut self, style: TextStyle) -> Self {
    self.style = style;
    self
  }

  pub fn fg(mut self, color: TextColor) -> Self {
    self.style.foreground = Some(color);
    self
  }

  pub fn bg(mut self, color: TextColor) -> Self {
    self.style.background = Some(color);
    self
  }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HyperlinkEvent {
  Clicked { id: HyperlinkId, link: String },
}

fn default_hyperlink_style() -> TextStyle {
  use crate::host_engine::services::{TerminalColor, TextColor};

  TextStyle {
    foreground: Some(TextColor::Terminal(TerminalColor::BrightBlue)),
    underline: true,
    ..Default::default()
  }
}
