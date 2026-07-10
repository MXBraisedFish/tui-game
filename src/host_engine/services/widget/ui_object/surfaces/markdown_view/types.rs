use crate::host_engine::services::{CodeHighlightTheme, TextStyle};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MarkdownViewId(pub u64);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MarkdownTheme {
  pub h1: TextStyle,
  pub h2: TextStyle,
  pub h3: TextStyle,
  pub h4_to_h6: TextStyle,
  pub paragraph: TextStyle,
  pub bold: TextStyle,
  pub italic: TextStyle,
  pub strike: TextStyle,
  pub inline_code: TextStyle,
  pub code_block: TextStyle,
  pub code_border: TextStyle,
  pub quote: TextStyle,
  pub quote_marker: String,
  pub link: TextStyle,
  pub table_border: TextStyle,
  pub task_checked: TextStyle,
  pub task_unchecked: TextStyle,
  pub horizontal_rule: TextStyle,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MarkdownViewOptions {
  pub markdown: String,
  pub theme: MarkdownTheme,
  pub code_theme: CodeHighlightTheme,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MarkdownRenderParams {
  pub x: u16,
  pub y: u16,
  pub width: u16,
  pub max_height: Option<u16>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MarkdownEvent {
  LinkClicked {
    id: MarkdownViewId,
    href: String,
    text: String,
  },
}

impl MarkdownViewOptions {
  pub fn new(markdown: impl Into<String>) -> Self {
    Self {
      markdown: markdown.into(),
      theme: MarkdownTheme::default(),
      code_theme: CodeHighlightTheme::default(),
    }
  }
}

impl Default for MarkdownTheme {
  fn default() -> Self {
    use crate::host_engine::services::{TerminalColor, TextColor};
    Self {
      h1: style(TerminalColor::Yellow, true),
      h2: style(TerminalColor::BrightYellow, true),
      h3: style(TerminalColor::Cyan, true),
      h4_to_h6: style(TerminalColor::Magenta, true),
      paragraph: TextStyle::default(),
      bold: TextStyle {
        bold: true,
        ..Default::default()
      },
      italic: TextStyle {
        italic: true,
        ..Default::default()
      },
      strike: TextStyle {
        strike: true,
        ..Default::default()
      },
      inline_code: TextStyle {
        foreground: Some(TextColor::Terminal(TerminalColor::Yellow)),
        background: Some(TextColor::Terminal(TerminalColor::BrightBlack)),
        ..Default::default()
      },
      code_block: TextStyle {
        foreground: Some(TextColor::Terminal(TerminalColor::White)),
        background: Some(TextColor::Terminal(TerminalColor::Black)),
        ..Default::default()
      },
      code_border: style(TerminalColor::BrightBlack, false),
      quote: style(TerminalColor::BrightBlack, false),
      quote_marker: "▌ ".to_string(),
      link: TextStyle {
        foreground: Some(TextColor::Terminal(TerminalColor::BrightBlue)),
        underline: true,
        ..Default::default()
      },
      table_border: style(TerminalColor::BrightBlack, false),
      task_checked: style(TerminalColor::Green, false),
      task_unchecked: style(TerminalColor::BrightBlack, false),
      horizontal_rule: style(TerminalColor::BrightBlack, false),
    }
  }
}

fn style(color: crate::host_engine::services::TerminalColor, bold: bool) -> TextStyle {
  crate::host_engine::services::TextStyle {
    foreground: Some(crate::host_engine::services::TextColor::Terminal(color)),
    bold,
    ..Default::default()
  }
}
