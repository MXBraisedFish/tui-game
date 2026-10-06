//! Identifiers, configuration, states, and events shared by this module.

use tg_core_style::TextStyle;
use tg_service_code_highlight::CodeHighlightTheme;

/// The identity of markdown view within its owning pool or session.
///
/// # Fields
///
/// * `0` - The wrapped u64 value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MarkdownViewId(
  /// The wrapped u64 value.
  pub u64,
);

/// The markdown theme representation used by this module.
///
/// # Fields
///
/// * `h1` - The style applied to h1 content.
/// * `h2` - The style applied to h2 content.
/// * `h3` - The style applied to h3 content.
/// * `h4_to_h6` - The style applied to h4 to h6 content.
/// * `paragraph` - The style applied to paragraph content.
/// * `bold` - The style applied to bold content.
/// * `italic` - The style applied to italic content.
/// * `strike` - The style applied to strike content.
/// * `inline_code` - The style applied to inline code content.
/// * `code_block` - The style applied to code block content.
/// * `code_border` - The style applied to code border content.
/// * `quote` - The style applied to quote content.
/// * `quote_marker` - The quote marker.
/// * `link` - The style applied to link content.
/// * `table_border` - The style applied to table border content.
/// * `table_header` - The style applied to table header content.
/// * `task_checked` - The style applied to task checked content.
/// * `task_unchecked` - The style applied to task unchecked content.
/// * `horizontal_rule` - The style applied to horizontal rule content.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MarkdownTheme {
  /// The style applied to h1 content.
  pub h1: TextStyle,
  /// The style applied to h2 content.
  pub h2: TextStyle,
  /// The style applied to h3 content.
  pub h3: TextStyle,
  /// The style applied to h4 to h6 content.
  pub h4_to_h6: TextStyle,
  /// The style applied to paragraph content.
  pub paragraph: TextStyle,
  /// The style applied to bold content.
  pub bold: TextStyle,
  /// The style applied to italic content.
  pub italic: TextStyle,
  /// The style applied to strike content.
  pub strike: TextStyle,
  /// The style applied to inline code content.
  pub inline_code: TextStyle,
  /// The style applied to code block content.
  pub code_block: TextStyle,
  /// The style applied to code border content.
  pub code_border: TextStyle,
  /// The style applied to quote content.
  pub quote: TextStyle,
  /// The quote marker.
  pub quote_marker: String,
  /// The style applied to link content.
  pub link: TextStyle,
  /// The style applied to table border content.
  pub table_border: TextStyle,
  /// The style applied to table header content.
  pub table_header: TextStyle,
  /// The style applied to task checked content.
  pub task_checked: TextStyle,
  /// The style applied to task unchecked content.
  pub task_unchecked: TextStyle,
  /// The style applied to horizontal rule content.
  pub horizontal_rule: TextStyle,
}

/// Configuration values controlling markdown view behavior.
///
/// # Fields
///
/// * `markdown` - The markdown.
/// * `theme` - The markdown theme carried by this markdown view options.
/// * `code_theme` - The code theme.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MarkdownViewOptions {
  /// The markdown.
  pub markdown: String,
  /// The markdown theme carried by this markdown view options.
  pub theme: MarkdownTheme,
  /// The code theme.
  pub code_theme: CodeHighlightTheme,
}

/// Configuration values controlling markdown render behavior.
///
/// # Fields
///
/// * `x` - The horizontal coordinate in terminal cells.
/// * `y` - The vertical coordinate in terminal cells.
/// * `width` - The width in terminal columns.
/// * `max_height` - The max height in terminal rows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MarkdownRenderParams {
  /// The horizontal coordinate in terminal cells.
  pub x: u16,
  /// The vertical coordinate in terminal cells.
  pub y: u16,
  /// The width in terminal columns.
  pub width: u16,
  /// The max height in terminal rows.
  pub max_height: Option<u16>,
}

/// A markdown event payload queued for its owning consumer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MarkdownEvent {
  /// A link clicked notification delivered to the owning consumer.
  LinkClicked {
    /// The identifier of the owned object.
    id: MarkdownViewId,
    /// The href.
    href: String,
    /// The text to process or display.
    text: String,
  },
}

impl MarkdownViewOptions {
  /// Create a markdown view options initialized from `markdown`.
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
    use tg_core_style::TextColor;
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
  tg_core_style::TextStyle {
    foreground: Some(tg_core_style::TextColor::Rgb { r, g, b }),
    bold,
    ..Default::default()
  }
}
