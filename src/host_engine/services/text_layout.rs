use crate::host_engine::services::rich_text::{
  RichTextParams, RichTextService, TextColor, TextStyle,
};
use crate::host_engine::services::unicode::graphemes;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextAlign {
  Left,
  Center,
  Right,
}

impl Default for TextAlign {
  fn default() -> Self {
    Self::Left
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextWrapMode {
  /// 拒绝任何换行，包括文本中的 `\n`。
  None,
  /// 根据 `max_width` 自动换行，同时保留文本中的 `\n`。
  Auto,
  /// 只使用文本中的 `\n` 换行。
  Normal,
}

impl Default for TextWrapMode {
  fn default() -> Self {
    Self::Normal
  }
}

/// 单次 `draw_text()` 调用的全部参数。
///
/// `x`、`y`、`text` 是必填语义，推荐使用 [`DrawTextParams::new`] 构造。
/// 其余字段都是可选排版或样式参数，默认保持关闭。
#[derive(Clone, Debug, Default)]
pub struct DrawTextParams {
  pub x: u16,
  pub y: u16,
  pub text: String,

  /// 富文本参数替换表（`{value:xxx}` / `{key:xxx}`）。
  pub params: Option<RichTextParams>,

  pub fg: Option<TextColor>,
  pub bg: Option<TextColor>,

  pub line_align: TextAlign,
  pub wrap_mode: TextWrapMode,
  pub max_width: Option<u16>,
  pub max_height: Option<u16>,
  pub overflow_marker: Option<String>,

  pub bold: bool,
  pub italic: bool,
  pub underline: bool,
  pub strike: bool,
  pub blink: bool,
  pub reverse: bool,
  pub hidden: bool,
  pub dim: bool,
}

impl DrawTextParams {
  pub fn new(x: u16, y: u16, text: impl Into<String>) -> Self {
    Self {
      x,
      y,
      text: text.into(),
      ..Default::default()
    }
  }

  pub(crate) fn to_text_style(&self) -> TextStyle {
    TextStyle {
      foreground: self.fg.clone(),
      background: self.bg.clone(),
      bold: self.bold,
      italic: self.italic,
      underline: self.underline,
      strike: self.strike,
      blink: self.blink,
      reverse: self.reverse,
      hidden: self.hidden,
      dim: self.dim,
    }
  }
}

#[derive(Clone, Debug)]
pub(crate) struct StyledGrapheme {
  pub(crate) text: String,
  pub(crate) width: usize,
  pub(crate) style: TextStyle,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct LayoutLine {
  pub(crate) items: Vec<StyledGrapheme>,
  pub(crate) width: usize,
}

impl LayoutLine {
  fn push(&mut self, item: StyledGrapheme) {
    self.width += item.width;
    self.items.push(item);
  }
}

#[derive(Clone, Debug)]
enum TextToken {
  Grapheme(StyledGrapheme),
  Newline,
}

pub(crate) fn layout_text_lines(params: &DrawTextParams) -> Vec<LayoutLine> {
  let default_style = params.to_text_style();
  let tokens = build_text_tokens(params, &default_style);
  layout_tokens(&tokens, params, &default_style)
}

pub(crate) fn measure_draw_text(params: &DrawTextParams) -> (u16, u16) {
  let lines = layout_text_lines(params);
  let width = lines
    .iter()
    .map(|line| line.width)
    .max()
    .unwrap_or(0)
    .min(u16::MAX as usize) as u16;
  let height = if lines.iter().any(|line| !line.items.is_empty()) {
    lines.len().min(u16::MAX as usize) as u16
  } else {
    0
  };

  (width, height)
}

fn build_text_tokens(params: &DrawTextParams, style: &TextStyle) -> Vec<TextToken> {
  let rich_text = RichTextService::new().parse(&params.text, params.params.as_ref());
  let mut tokens = Vec::new();

  for segment in &rich_text.segments {
    let merged = merge_style(style, &segment.style);
    for g in graphemes(&segment.text) {
      if g.text == "\n" {
        tokens.push(TextToken::Newline);
      } else {
        tokens.push(TextToken::Grapheme(StyledGrapheme {
          text: g.text,
          width: g.display_width,
          style: merged.clone(),
        }));
      }
    }
  }

  tokens
}

fn layout_tokens(
  tokens: &[TextToken],
  params: &DrawTextParams,
  default_style: &TextStyle,
) -> Vec<LayoutLine> {
  let max_lines = params
    .max_height
    .map(|height| height as usize)
    .unwrap_or(usize::MAX);
  if max_lines == 0 {
    return Vec::new();
  }

  let max_width = params.max_width.map(|width| width as usize);
  let mut lines = Vec::new();
  let mut current = LayoutLine::default();
  let mut overflow = false;
  let mut overflow_style = None;

  for (index, token) in tokens.iter().enumerate() {
    match token {
      TextToken::Newline => {
        if params.wrap_mode == TextWrapMode::None {
          continue;
        }
        lines.push(std::mem::take(&mut current));
        if lines.len() >= max_lines {
          overflow = tokens[index + 1..]
            .iter()
            .any(|token| matches!(token, TextToken::Grapheme(_)));
          overflow_style = first_grapheme_style(&tokens[index + 1..]);
          break;
        }
      }
      TextToken::Grapheme(item) => {
        if let Some(limit) = max_width {
          if item.width > limit {
            overflow = true;
            overflow_style = Some(item.style.clone());
            break;
          }

          if current.width + item.width > limit {
            if params.wrap_mode == TextWrapMode::Auto && current.width > 0 {
              lines.push(std::mem::take(&mut current));
              if lines.len() >= max_lines {
                overflow = true;
                overflow_style = Some(item.style.clone());
                break;
              }
            } else {
              overflow = true;
              overflow_style = Some(item.style.clone());
              break;
            }
          }
        }

        current.push(item.clone());
      }
    }
  }

  if lines.len() < max_lines {
    lines.push(current);
  }

  if overflow {
    if let Some(line) = lines.last_mut() {
      apply_overflow_marker(
        line,
        params.overflow_marker.as_deref(),
        max_width,
        overflow_style.as_ref().unwrap_or(default_style),
      );
    }
  }

  lines
}

fn first_grapheme_style(tokens: &[TextToken]) -> Option<TextStyle> {
  tokens.iter().find_map(|token| match token {
    TextToken::Grapheme(item) => Some(item.style.clone()),
    TextToken::Newline => None,
  })
}

fn apply_overflow_marker(
  line: &mut LayoutLine,
  marker: Option<&str>,
  max_width: Option<usize>,
  style: &TextStyle,
) {
  let Some(marker) = marker else {
    return;
  };
  if marker.is_empty() {
    return;
  }

  let available = max_width.unwrap_or(usize::MAX);
  if available == 0 {
    line.items.clear();
    line.width = 0;
    return;
  }

  let mut marker_items = marker_graphemes(marker, available, style);
  let marker_width: usize = marker_items.iter().map(|item| item.width).sum();
  if marker_width == 0 && marker_items.is_empty() {
    return;
  }

  while line.width.saturating_add(marker_width) > available {
    let Some(removed) = line.items.pop() else {
      break;
    };
    line.width = line.width.saturating_sub(removed.width);
  }

  if line.width.saturating_add(marker_width) > available {
    let remaining = available.saturating_sub(line.width);
    marker_items = marker_graphemes(marker, remaining, style);
  }

  for item in marker_items {
    line.push(item);
  }
}

fn marker_graphemes(marker: &str, max_width: usize, style: &TextStyle) -> Vec<StyledGrapheme> {
  let mut result = Vec::new();
  let mut width = 0usize;
  for g in graphemes(marker) {
    if width + g.display_width > max_width {
      break;
    }
    width += g.display_width;
    result.push(StyledGrapheme {
      text: g.text,
      width: g.display_width,
      style: style.clone(),
    });
  }
  result
}

fn merge_style(base: &TextStyle, overrides: &TextStyle) -> TextStyle {
  let mut merged = base.clone();

  if overrides.foreground.is_some() {
    merged.foreground = overrides.foreground.clone();
  }
  if overrides.background.is_some() {
    merged.background = overrides.background.clone();
  }

  if overrides.bold {
    merged.bold = true;
  }
  if overrides.italic {
    merged.italic = true;
  }
  if overrides.underline {
    merged.underline = true;
  }
  if overrides.strike {
    merged.strike = true;
  }
  if overrides.blink {
    merged.blink = true;
  }
  if overrides.reverse {
    merged.reverse = true;
  }
  if overrides.hidden {
    merged.hidden = true;
  }
  if overrides.dim {
    merged.dim = true;
  }

  merged
}
