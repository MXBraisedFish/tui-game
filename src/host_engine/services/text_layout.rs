use crate::host_engine::services::rich_text::{
  RichTextParams, RichTextService, TextColor, TextStyle,
};
use crate::host_engine::services::unicode::graphemes;
use std::collections::{HashMap, HashSet};
use unicode_linebreak::BreakOpportunity;

/// 文本对齐方式
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

/// 文本换行模式
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextWrapMode {
  None,

  Auto,

  Normal,
}

impl Default for TextWrapMode {
  fn default() -> Self {
    Self::Normal
  }
}

/// 绘制文本的参数
#[derive(Clone, Debug)]
pub struct DrawTextParams {
  pub x: u16,
  pub y: u16,
  pub text: String,

  pub params: Option<RichTextParams>,
  pub fg: Option<TextColor>,
  pub bg: Option<TextColor>,
  pub line_align: TextAlign,
  pub wrap_mode: TextWrapMode,
  pub non_truncate_word_wrap: bool,
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

impl Default for DrawTextParams {
  fn default() -> Self {
    Self {
      x: 0,
      y: 0,
      text: String::new(),
      params: None,
      fg: None,
      bg: None,
      line_align: TextAlign::default(),
      wrap_mode: TextWrapMode::default(),
      non_truncate_word_wrap: true,
      max_width: None,
      max_height: None,
      overflow_marker: None,
      bold: false,
      italic: false,
      underline: false,
      strike: false,
      blink: false,
      reverse: false,
      hidden: false,
      dim: false,
    }
  }
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

// 将绘制文本参数转换为已排版好的文本行列表
pub(crate) fn layout_text_lines(params: &DrawTextParams) -> Vec<LayoutLine> {
  let default_style = params.to_text_style();
  let tokens = build_text_tokens(params, &default_style);
  layout_tokens(&tokens, params, &default_style)
}

// 测量绘制文本所需的尺寸（宽度 x 高度）
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

// 将富文本解析为字素 token 流，每个 token 携带样式信息
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

// 将 token 流按最大宽度/高度/换行模式排版为文本行
fn layout_tokens(
  tokens: &[TextToken],
  params: &DrawTextParams,
  default_style: &TextStyle,
) -> Vec<LayoutLine> {
  if params.wrap_mode == TextWrapMode::Auto && params.non_truncate_word_wrap {
    return layout_tokens_auto(tokens, params, default_style);
  }

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

fn layout_tokens_auto(
  tokens: &[TextToken],
  params: &DrawTextParams,
  default_style: &TextStyle,
) -> Vec<LayoutLine> {
  let mut lines = Vec::new();
  let mut paragraph = Vec::new();

  for token in tokens {
    match token {
      TextToken::Grapheme(item) => paragraph.push(item.clone()),
      TextToken::Newline => {
        lines.extend(wrap_paragraph(
          &paragraph,
          params.max_width.map(|width| width as usize),
        ));
        paragraph.clear();
      }
    }
  }
  lines.extend(wrap_paragraph(
    &paragraph,
    params.max_width.map(|width| width as usize),
  ));

  limit_lines(lines, params, default_style)
}

fn wrap_paragraph(items: &[StyledGrapheme], max_width: Option<usize>) -> Vec<LayoutLine> {
  let Some(max_width) = max_width else {
    return vec![line_from_items(items)];
  };

  if items.is_empty() {
    return vec![LayoutLine::default()];
  }

  let mut lines = Vec::new();
  let breaks = line_breaks(items);
  let mut start = 0usize;

  while start < items.len() {
    start = skip_leading_whitespace(items, start);
    if start >= items.len() {
      lines.push(LayoutLine::default());
      break;
    }

    if item_width(&items[start..]) <= max_width {
      lines.push(line_from_items(&items[start..]));
      break;
    }

    if let Some((break_index, trim_end)) = best_break(items, &breaks, start, max_width) {
      lines.push(line_from_items(&items[start..trim_end]));
      start = break_index;
      continue;
    }

    if let Some((line, next_start)) = hyphenate_english_word(items, start, max_width) {
      lines.push(line);
      start = next_start;
      continue;
    }

    let next_start = hard_break(items, start, max_width);
    lines.push(line_from_items(&items[start..next_start]));
    start = next_start;
  }

  lines
}

fn line_breaks(items: &[StyledGrapheme]) -> HashSet<usize> {
  let plain = items
    .iter()
    .map(|item| item.text.as_str())
    .collect::<String>();
  let boundaries = byte_boundaries(items);
  let mut breaks = unicode_linebreak::linebreaks(&plain)
    .filter_map(|(byte, opportunity)| match opportunity {
      BreakOpportunity::Mandatory | BreakOpportunity::Allowed => boundaries.get(&byte).copied(),
    })
    .collect::<HashSet<_>>();
  breaks.insert(items.len());
  breaks
}

fn byte_boundaries(items: &[StyledGrapheme]) -> HashMap<usize, usize> {
  let mut map = HashMap::new();
  let mut byte = 0usize;
  map.insert(byte, 0);
  for (index, item) in items.iter().enumerate() {
    byte += item.text.len();
    map.insert(byte, index + 1);
  }
  map
}

fn best_break(
  items: &[StyledGrapheme],
  breaks: &HashSet<usize>,
  start: usize,
  max_width: usize,
) -> Option<(usize, usize)> {
  let mut width = 0usize;
  let mut best = None;

  for index in start + 1..=items.len() {
    width += items[index - 1].width;
    if breaks.contains(&index) {
      if is_inside_english_word_break(items, index) {
        continue;
      }
      let trim_end = trim_trailing_whitespace(items, start, index);
      let trimmed_width = item_width(&items[start..trim_end]);
      if trim_end > start && trimmed_width <= max_width {
        best = Some((index, trim_end));
      }
    }
    if width > max_width && !is_whitespace(&items[index - 1]) {
      break;
    }
    if width > max_width && is_whitespace(&items[index - 1]) {
      break;
    }
  }

  best
}

fn is_inside_english_word_break(items: &[StyledGrapheme], index: usize) -> bool {
  index > 0
    && index < items.len()
    && is_english_word_grapheme(&items[index - 1])
    && is_english_word_grapheme(&items[index])
}

fn hyphenate_english_word(
  items: &[StyledGrapheme],
  start: usize,
  max_width: usize,
) -> Option<(LayoutLine, usize)> {
  if max_width <= 1 {
    return None;
  }

  let end = english_word_end(items, start);
  let word_len = end.saturating_sub(start);
  if word_len < 8 {
    return None;
  }

  let available = max_width - 1;
  let mut best = None;
  let mut best_score = isize::MIN;
  let mut width = 0usize;

  for index in start..end {
    width += items[index].width;
    let prefix_len = index + 1 - start;
    let suffix_len = end - index - 1;
    if width > available {
      break;
    }
    if prefix_len < 3 || suffix_len < 3 {
      continue;
    }

    let score = hyphen_score(items, index, width, available);
    if score > best_score {
      best_score = score;
      best = Some(index + 1);
    }
  }

  let split = best?;
  let mut line = line_from_items(&items[start..split]);
  line.push(StyledGrapheme {
    text: "-".to_string(),
    width: 1,
    style: items[split - 1].style.clone(),
  });
  Some((line, split))
}

fn hyphen_score(items: &[StyledGrapheme], index: usize, width: usize, available: usize) -> isize {
  let mut score = (available.saturating_sub(width) as isize) * -10;
  let current = items[index].text.as_str();
  let next = items
    .get(index + 1)
    .map(|item| item.text.as_str())
    .unwrap_or("");

  if current == "-" {
    score += 100;
  }
  if is_lower_ascii(current) && is_upper_ascii(next) {
    score += 80;
  }
  if current.chars().any(|ch| "aeiouAEIOU".contains(ch)) {
    score += 30;
  }

  score
}

fn hard_break(items: &[StyledGrapheme], start: usize, max_width: usize) -> usize {
  let mut width = 0usize;
  let mut end = start;
  while end < items.len() {
    let next_width = width + items[end].width;
    if end > start && next_width > max_width {
      break;
    }
    width = next_width;
    end += 1;
    if width >= max_width {
      break;
    }
  }
  end.max(start + 1).min(items.len())
}

fn limit_lines(
  mut lines: Vec<LayoutLine>,
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

  let overflow = lines.len() > max_lines;
  if overflow {
    lines.truncate(max_lines);
    if let Some(line) = lines.last_mut() {
      let style = line
        .items
        .last()
        .map(|item| item.style.clone())
        .unwrap_or_else(|| default_style.clone());
      apply_overflow_marker(
        line,
        params.overflow_marker.as_deref(),
        params.max_width.map(|width| width as usize),
        &style,
      );
    }
  }

  lines
}

fn line_from_items(items: &[StyledGrapheme]) -> LayoutLine {
  let mut line = LayoutLine::default();
  for item in items {
    line.push(item.clone());
  }
  line
}

fn item_width(items: &[StyledGrapheme]) -> usize {
  items.iter().map(|item| item.width).sum()
}

fn skip_leading_whitespace(items: &[StyledGrapheme], mut index: usize) -> usize {
  while index < items.len() && is_whitespace(&items[index]) {
    index += 1;
  }
  index
}

fn trim_trailing_whitespace(items: &[StyledGrapheme], start: usize, mut end: usize) -> usize {
  while end > start && is_whitespace(&items[end - 1]) {
    end -= 1;
  }
  end
}

fn is_whitespace(item: &StyledGrapheme) -> bool {
  item.text.chars().all(char::is_whitespace)
}

fn english_word_end(items: &[StyledGrapheme], start: usize) -> usize {
  let mut end = start;
  while end < items.len() && is_english_word_grapheme(&items[end]) {
    end += 1;
  }
  end
}

fn is_english_word_grapheme(item: &StyledGrapheme) -> bool {
  item
    .text
    .chars()
    .all(|ch| ch.is_ascii_alphabetic() || ch == '\'' || ch == '-')
}

fn is_lower_ascii(text: &str) -> bool {
  text.chars().all(|ch| ch.is_ascii_lowercase())
}

fn is_upper_ascii(text: &str) -> bool {
  text.chars().all(|ch| ch.is_ascii_uppercase())
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::host_engine::services::rich_text::{TerminalColor, TextColor};

  fn auto_lines(text: &str, width: u16) -> Vec<String> {
    let lines = layout_text_lines(&DrawTextParams {
      text: text.to_string(),
      wrap_mode: TextWrapMode::Auto,
      max_width: Some(width),
      ..Default::default()
    });
    lines
      .iter()
      .map(|line| line.items.iter().map(|item| item.text.as_str()).collect())
      .collect()
  }

  fn line_width(text: &str) -> usize {
    graphemes(text).iter().map(|g| g.display_width).sum()
  }

  #[test]
  fn auto_wrap_keeps_normal_english_words() {
    assert_eq!(
      auto_lines("Hello world, this is a test.", 12),
      vec!["Hello world,", "this is a", "test."]
    );
  }

  #[test]
  fn auto_wrap_cjk_without_losing_text() {
    let input = "这是一个中文文本换行测试。";
    let lines = auto_lines(input, 10);

    assert!(lines.iter().all(|line| line_width(line) <= 10));
    assert_eq!(lines.join(""), input);
  }

  #[test]
  fn auto_wrap_mixed_text_keeps_ascii_words() {
    let lines = auto_lines("你好World，这是mixed text测试。", 12);

    assert!(lines.iter().all(|line| line_width(line) <= 12));
    assert!(lines.iter().any(|line| line.contains("World")));
    assert!(!lines.iter().any(|line| line == "Wor" || line == "ld"));
    assert!(!lines.iter().any(|line| line == "mix" || line == "ed"));
  }

  #[test]
  fn auto_wrap_respects_explicit_newlines() {
    assert_eq!(
      auto_lines("第一行\nSecond line mixed 中文", 12),
      vec!["第一行", "Second line", "mixed 中文"]
    );
  }

  #[test]
  fn auto_wrap_hyphenates_overlong_english_words() {
    let input = "SuperLongEnglishWordWithoutSpaces";
    let lines = auto_lines(input, 12);

    assert!(lines.iter().all(|line| line_width(line) <= 12));
    assert!(
      lines
        .iter()
        .take(lines.len() - 1)
        .all(|line| line.ends_with('-'))
    );
    assert_eq!(
      lines
        .iter()
        .map(|line| line.trim_end_matches('-'))
        .collect::<String>(),
      input
    );
  }

  #[test]
  fn auto_wrap_does_not_split_emoji_graphemes() {
    let input = "hello 👨‍👩‍👧‍👦 world";
    let lines = auto_lines(input, 8);

    assert!(lines.iter().all(|line| line_width(line) <= 8));
    assert!(lines.iter().any(|line| line.contains("👨‍👩‍👧‍👦")));
  }

  #[test]
  fn auto_wrap_preserves_rich_text_styles() {
    let lines = layout_text_lines(&DrawTextParams {
      text: "f%<fg:red>Hello</fg> <fg:blue>world</fg>".to_string(),
      wrap_mode: TextWrapMode::Auto,
      max_width: Some(6),
      ..Default::default()
    });

    assert_eq!(
      lines[0].items[0].style.foreground,
      Some(TextColor::Terminal(TerminalColor::Red))
    );
    assert_eq!(
      lines[1].items[0].style.foreground,
      Some(TextColor::Terminal(TerminalColor::Blue))
    );
  }

  #[test]
  fn draw_text_params_default_enables_non_truncate_word_wrap() {
    assert!(DrawTextParams::default().non_truncate_word_wrap);
  }

  #[test]
  fn auto_wrap_can_use_legacy_grapheme_wrapping() {
    let lines = layout_text_lines(&DrawTextParams {
      text: "Hello".to_string(),
      wrap_mode: TextWrapMode::Auto,
      non_truncate_word_wrap: false,
      max_width: Some(3),
      ..Default::default()
    });
    let text: Vec<String> = lines
      .iter()
      .map(|line| line.items.iter().map(|item| item.text.as_str()).collect())
      .collect();

    assert_eq!(text, vec!["Hel", "lo"]);
  }

  #[test]
  fn auto_wrap_keeps_rich_text_style_after_word_wrap() {
    let lines = layout_text_lines(&DrawTextParams {
      text: "f%If you need a backup via <fg:blue>[Settings -> Storage Management -> Export Data]</fg> first.".to_string(),
      wrap_mode: TextWrapMode::Auto,
      max_width: Some(48),
      ..Default::default()
    });
    let visible = lines
      .iter()
      .flat_map(|line| line.items.iter().map(|item| item.text.as_str()))
      .collect::<String>();

    assert!(visible.contains("Export Data"));
    assert!(
      lines
        .iter()
        .flat_map(|line| &line.items)
        .any(|item| item.text == "E"
          && item.style.foreground == Some(TextColor::Terminal(TerminalColor::Blue)))
    );
  }

  #[test]
  fn auto_wrap_does_not_break_inside_ascii_words() {
    let input = "Clearing this directory will permanently remove all contents. Missing files and directories will be recreated automatically.";
    let lines = auto_lines(input, 86);
    let joined_without_spaces = lines.join("").replace(' ', "");
    let input_without_spaces = input.replace(' ', "");

    assert_eq!(joined_without_spaces, input_without_spaces);
    assert!(!lines.iter().any(|line| line.ends_with(" wi")));
    assert!(!lines.iter().any(|line| line.starts_with("ll ")));
  }

  #[test]
  fn auto_wrap_keeps_rich_text_style_across_line_breaks() {
    let blue_text = "[Settings -> Storage Management -> Export Data]";
    let lines = layout_text_lines(&DrawTextParams {
      text: format!(
        "f%If you need to proceed, create a backup via <fg:blue>{}</fg> first.",
        blue_text
      ),
      wrap_mode: TextWrapMode::Auto,
      max_width: Some(76),
      ..Default::default()
    });
    let visible = lines
      .iter()
      .flat_map(|line| line.items.iter().map(|item| item.text.as_str()))
      .collect::<String>();

    let visible_no_spaces = visible.replace(' ', "");
    let blue_text_no_spaces = blue_text.replace(' ', "");
    assert!(visible_no_spaces.contains(&blue_text_no_spaces));
    let blue_styled_text = lines
      .iter()
      .flat_map(|line| &line.items)
      .filter(|item| item.style.foreground == Some(TextColor::Terminal(TerminalColor::Blue)))
      .map(|item| item.text.as_str())
      .collect::<String>();
    assert!(
      blue_styled_text
        .replace(' ', "")
        .contains(&blue_text_no_spaces)
    );
  }

  #[test]
  fn auto_wrap_keeps_warning_text_complete_across_many_widths() {
    let blue_text = "[Settings -> Storage Management -> Export Data]";
    let input = format!(
      "f%The cache/ directory stores all cached assets.\n\
Clearing this directory will permanently remove all contents. Missing files and directories will be recreated automatically, but all cached assets will be lost.\n\
If you need to proceed, it is recommended to create a backup via <fg:bright_blue>{}</fg> first.\n\
<fg:red>This action cannot be undone!</fg>",
      blue_text
    );
    let plain = "The cache/ directory stores all cached assets.\n\
Clearing this directory will permanently remove all contents. Missing files and directories will be recreated automatically, but all cached assets will be lost.\n\
If you need to proceed, it is recommended to create a backup via [Settings -> Storage Management -> Export Data] first.\n\
This action cannot be undone!";

    for width in [36, 48, 60, 76, 86, 100, 120] {
      let lines = layout_text_lines(&DrawTextParams {
        text: input.clone(),
        wrap_mode: TextWrapMode::Auto,
        max_width: Some(width),
        ..Default::default()
      });
      let visible = lines
        .iter()
        .flat_map(|line| line.items.iter().map(|item| item.text.as_str()))
        .collect::<String>();
      let blue_styled_text = lines
        .iter()
        .flat_map(|line| &line.items)
        .filter(|item| {
          item.style.foreground == Some(TextColor::Terminal(TerminalColor::BrightBlue))
        })
        .map(|item| item.text.as_str())
        .collect::<String>();

      assert_eq!(
        visible.replace([' ', '\n'], ""),
        plain.replace([' ', '\n'], "")
      );
      assert!(
        blue_styled_text
          .replace(' ', "")
          .contains(&blue_text.replace(' ', "")),
        "blue style lost at width {width}: {blue_styled_text:?}",
      );
      assert!(
        !lines.iter().any(|line| {
          let text = line
            .items
            .iter()
            .map(|item| item.text.as_str())
            .collect::<String>();
          text.ends_with(" wi")
            || text.starts_with("ll ")
            || text.ends_with("Manageme")
            || text.starts_with("nt ->")
            || text.ends_with("Manag")
            || text.starts_with("ement")
        }),
        "word was split at width {width}: {:?}",
        lines
          .iter()
          .map(|line| line
            .items
            .iter()
            .map(|item| item.text.as_str())
            .collect::<String>())
          .collect::<Vec<_>>()
      );
    }
  }
}

fn first_grapheme_style(tokens: &[TextToken]) -> Option<TextStyle> {
  tokens.iter().find_map(|token| match token {
    TextToken::Grapheme(item) => Some(item.style.clone()),
    TextToken::Newline => None,
  })
}

// 在行尾添加溢出标记（如 "..."），必要时裁剪字符以腾出空间
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

// 将溢出标记字符串按字素拆分，并限制总宽度
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

// 合并基础样式和覆盖样式，覆盖样式的非默认值优先
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
