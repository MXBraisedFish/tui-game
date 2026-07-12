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
  pub table_header: TextStyle,
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
    use crate::host_engine::services::TextColor;
    Self {
      h1: style(170, 105, 225, true),
      h2: style(184, 122, 232, true),
      h3: style(198, 140, 238, true),
      h4_to_h6: style(234, 198, 250, true),
      paragraph: style(220, 223, 218, false),
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
        foreground: Some(TextColor::Rgb {
          r: 249,
          g: 232,
          b: 147,
        }),
        background: Some(TextColor::Rgb {
          r: 45,
          g: 47,
          b: 45,
        }),
        ..Default::default()
      },
      code_block: TextStyle {
        foreground: Some(TextColor::Rgb {
          r: 220,
          g: 223,
          b: 218,
        }),
        background: Some(TextColor::Rgb { r: 0, g: 0, b: 0 }),
        ..Default::default()
      },
      code_border: style(85, 87, 83, false),
      quote: style(255, 164, 209, false),
      quote_marker: "▌ ".to_string(),
      link: TextStyle {
        foreground: Some(TextColor::Rgb {
          r: 80,
          g: 165,
          b: 255,
        }),
        underline: true,
        ..Default::default()
      },
      table_border: style(255, 255, 255, false),
      table_header: style(86, 182, 194, true),
      task_checked: style(95, 215, 105, false),
      task_unchecked: style(85, 87, 83, false),
      horizontal_rule: style(85, 87, 83, false),
    }
  }
}

fn style(r: u8, g: u8, b: u8, bold: bool) -> TextStyle {
  crate::host_engine::services::TextStyle {
    foreground: Some(crate::host_engine::services::TextColor::Rgb { r, g, b }),
    bold,
    ..Default::default()
  }
}
