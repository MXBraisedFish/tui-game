use std::io::{self, Write};

use crossterm::{
  QueueableCommand,
  cursor::MoveTo,
  style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};

use super::{buffer::CanvasBuffer, cell::CanvasCell};
use crate::host_engine::services::unicode::graphemes;
use crate::host_engine::services::{
  RichTextParams, RichTextService, TerminalColor, TerminalService, TextColor, TextStyle,
};

// ── 绘制参数 ──

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

/// 单次 `canvas.text()` 调用的全部参数。
/// 所有样式字段可选，默认关闭。
#[derive(Clone, Debug, Default)]
pub struct DrawTextParams {
  pub x: u16,
  pub y: u16,
  pub text: String,

  /// 富文本参数替换表（`{value:xxx}` / `{key:xxx}`）
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
  fn to_text_style(&self) -> TextStyle {
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

// ── 画布服务 ──

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanvasService {
  current: CanvasBuffer,
  previous: CanvasBuffer,
  force_full_redraw: bool,
}

impl CanvasService {
  pub fn new() -> Self {
    let (width, height) = crossterm::terminal::size().unwrap_or((80, 24));
    Self {
      current: CanvasBuffer::new(width, height),
      previous: CanvasBuffer::new(width, height),
      force_full_redraw: true,
    }
  }

  pub fn width(&self) -> u16 {
    self.current.width()
  }

  pub fn height(&self) -> u16 {
    self.current.height()
  }

  pub fn size(&self) -> (u16, u16) {
    (self.width(), self.height())
  }

  pub fn begin_frame(&mut self) {
    // 为运行生命周期保留。
  }

  pub fn clear(&mut self) {
    self.current.clear();
  }

  // ── 文本绘制 ──

  /// 文本绘制路由入口。
  /// 检查 `f%` 前缀决定走富文本流还是普通文本流，
  /// 最终都汇聚到 `styled_text()` 写入画布单元格。
  pub fn text(&mut self, params: &DrawTextParams) {
    let default_style = params.to_text_style();
    let tokens = self.build_text_tokens(params, &default_style);
    let lines = layout_text_lines(&tokens, params, &default_style);
    self.draw_layout_lines(params.x, params.y, params.line_align, &lines);
  }

  /// 画布底层基元：在 (x, y) 处以指定样式绘制文本。
  /// 不做任何前缀检查，调用方给什么就画什么。
  ///
  /// 正确处理 Unicode 显示宽度：
  /// - 零宽字符（ZWJ、ZWS、组合标记）写入单元格但不推进光标
  /// - 普通字符（ASCII、拉丁）推进 1 格
  /// - 宽字符（CJK、emoji、全角）推进 2 格，并标记右侧格为 WIDE_CONTINUATION
  pub fn styled_text(&mut self, x: u16, y: u16, text: &str, style: TextStyle) {
    let gs = graphemes(text);
    let mut cursor_x = x;

    for g in &gs {
      if cursor_x >= self.current.width() || y >= self.current.height() {
        break;
      }

      // 取 grapheme 的首个 char 作为单元格内容
      let ch = g.text.chars().next().unwrap_or(' ');

      if g.display_width == 0 {
        // 零宽字符：写入当前格但不推进光标（与前一字符合并于同一 Print 输出）
        self
          .current
          .set(cursor_x, y, CanvasCell::styled(ch, style.clone()));
        // cursor_x 不变
        continue;
      }

      // 宽字符 ≥1：写入首格
      self
        .current
        .set(cursor_x, y, CanvasCell::styled(ch, style.clone()));

      // 宽字符 ≥2：标记右侧连续格为 CONTINUATION
      for offset in 1..g.display_width {
        let cont_x = cursor_x.saturating_add(offset as u16);
        if cont_x < self.current.width() {
          self.current.set(cont_x, y, CanvasCell::continuation());
        }
      }

      cursor_x = cursor_x.saturating_add(g.display_width as u16);
    }
  }

  // ── 尺寸 ──

  /// 仅更新画布缓冲尺寸，不触发强制重绘。
  /// 需要重绘时由调用方显式调用 `request_render()`。
  pub fn resize(&mut self, width: u16, height: u16) {
    self.current.resize(width, height);
    self.previous.resize(width, height);
  }

  /// 标记下一帧为强制全屏重绘。收到 resize / focus 等系统事件时调用。
  pub fn request_render(&mut self) {
    self.force_full_redraw = true;
  }

  pub fn present(&mut self, terminal: &mut TerminalService) -> io::Result<()> {
    let Some(stdout) = terminal.writer_mut() else {
      return Ok(());
    };

    for y in 0..self.current.height() {
      // (起始 x, 参考样式) —— 样式变了就切新 run
      let mut run: Option<(u16, &TextStyle)> = None;

      for x in 0..self.current.width() {
        let current_cell = self.current.get(x, y).unwrap_or(&BLANK_CELL);

        // 跳过宽字符延续格 —— 它们属于左侧的宽字符，
        // 不应打断当前 run，也不应启动新 run。
        if current_cell.is_continuation() {
          continue;
        }

        let previous_cell = self.previous.get(x, y).unwrap_or(&BLANK_CELL);

        let cell_changed = self.force_full_redraw || current_cell != previous_cell;

        match &run {
          // 当前不在 run 内，新变化启动 run
          None if cell_changed => {
            run = Some((x, &current_cell.style));
          }

          // 在 run 内，样式相同 → 继续
          Some((_, run_style)) if cell_changed && *run_style == &current_cell.style => {
            // 继续累积
          }

          // 在 run 内，但样式变了 或 单元格未变化 → 输出当前 run
          _ => {
            if let Some((start, style)) = run {
              let run_text: String = (start..x)
                .filter_map(|rx| {
                  self.current.get(rx, y).and_then(|cell| {
                    if cell.is_continuation() {
                      None
                    } else {
                      Some(cell.ch)
                    }
                  })
                })
                .collect();

              stdout.queue(MoveTo(start, y))?;
              queue_style(stdout, style)?;
              stdout.queue(Print(run_text))?;
            }

            // 如果当前单元格变了但样式不同，启动新 run
            if cell_changed {
              run = Some((x, &current_cell.style));
            } else {
              run = None;
            }
          }
        }
      }

      // 行尾残留 run
      if let Some((start, style)) = run {
        let run_text: String = (start..self.current.width())
          .filter_map(|rx| {
            self.current.get(rx, y).and_then(|cell| {
              if cell.is_continuation() {
                None
              } else {
                Some(cell.ch)
              }
            })
          })
          .collect();

        stdout.queue(MoveTo(start, y))?;
        queue_style(stdout, style)?;
        stdout.queue(Print(run_text))?;
      }
    }

    stdout.queue(ResetColor)?;
    stdout.flush()?;

    self.previous = self.current.clone();
    self.force_full_redraw = false;

    Ok(())
  }

  // ── 内部：文本流 ──

  fn build_text_tokens(&mut self, params: &DrawTextParams, style: &TextStyle) -> Vec<TextToken> {
    let rich_text_service = RichTextService::new();
    let rich_text = rich_text_service.parse(&params.text, params.params.as_ref());
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

  fn draw_layout_lines(&mut self, x: u16, y: u16, align: TextAlign, lines: &[LayoutLine]) {
    let base_width = lines.first().map(|line| line.width).unwrap_or(0);

    for (line_index, line) in lines.iter().enumerate() {
      let offset = match align {
        TextAlign::Left => 0,
        TextAlign::Center => base_width.saturating_sub(line.width) / 2,
        TextAlign::Right => base_width.saturating_sub(line.width),
      } as u16;
      let mut cursor_x = x.saturating_add(offset);
      let cursor_y = y.saturating_add(line_index as u16);
      let mut run_text = String::new();
      let mut run_style: Option<&TextStyle> = None;
      let mut run_width = 0usize;

      for item in &line.items {
        match run_style {
          Some(style) if style == &item.style => {}
          Some(style) => {
            self.styled_text(cursor_x, cursor_y, &run_text, style.clone());
            cursor_x = cursor_x.saturating_add(run_width as u16);
            run_text.clear();
            run_width = 0;
            run_style = Some(&item.style);
          }
          None => run_style = Some(&item.style),
        }
        run_text.push_str(&item.text);
        run_width += item.width;
      }

      if let Some(style) = run_style {
        self.styled_text(cursor_x, cursor_y, &run_text, style.clone());
      }
    }
  }
}

#[derive(Clone, Debug)]
enum TextToken {
  Grapheme(StyledGrapheme),
  Newline,
}

#[derive(Clone, Debug)]
struct StyledGrapheme {
  text: String,
  width: usize,
  style: TextStyle,
}

#[derive(Clone, Debug, Default)]
struct LayoutLine {
  items: Vec<StyledGrapheme>,
  width: usize,
}

impl LayoutLine {
  fn push(&mut self, item: StyledGrapheme) {
    self.width += item.width;
    self.items.push(item);
  }
}

fn layout_text_lines(
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

// ── 样式合并 ──

/// 将富文本段样式合并到默认样式上。
/// 段显式设置的字段覆盖默认值，未设置的保持默认。
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

// ── 终端颜色转换 ──

static BLANK_CELL: CanvasCell = CanvasCell {
  ch: ' ',
  style: TextStyle {
    foreground: None,
    background: None,
    bold: false,
    italic: false,
    underline: false,
    strike: false,
    blink: false,
    reverse: false,
    hidden: false,
    dim: false,
  },
};

fn terminal_color_to_crossterm(color: &TerminalColor) -> Color {
  match color {
    TerminalColor::Black => Color::Black,
    TerminalColor::Red => Color::DarkRed,
    TerminalColor::Green => Color::DarkGreen,
    TerminalColor::Yellow => Color::DarkYellow,
    TerminalColor::Blue => Color::DarkBlue,
    TerminalColor::Magenta => Color::DarkMagenta,
    TerminalColor::Cyan => Color::DarkCyan,
    TerminalColor::White => Color::White,
    TerminalColor::BrightBlack => Color::Grey,
    TerminalColor::BrightRed => Color::Red,
    TerminalColor::BrightGreen => Color::Green,
    TerminalColor::BrightYellow => Color::Yellow,
    TerminalColor::BrightBlue => Color::Blue,
    TerminalColor::BrightMagenta => Color::Magenta,
    TerminalColor::BrightCyan => Color::Cyan,
    TerminalColor::BrightWhite => Color::White,
  }
}

fn text_color_to_crossterm(color: &TextColor) -> Color {
  match color {
    TextColor::Terminal(color) => terminal_color_to_crossterm(color),
    TextColor::Rgb { r, g, b } => Color::Rgb {
      r: *r,
      g: *g,
      b: *b,
    },
  }
}

fn queue_style(stdout: &mut impl Write, style: &TextStyle) -> io::Result<()> {
  stdout.queue(ResetColor)?;

  if let Some(foreground) = &style.foreground {
    stdout.queue(SetForegroundColor(text_color_to_crossterm(foreground)))?;
  }

  if let Some(background) = &style.background {
    stdout.queue(SetBackgroundColor(text_color_to_crossterm(background)))?;
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::collections::HashMap;

  fn visible_row(canvas: &CanvasService, y: u16) -> String {
    (0..canvas.width())
      .filter_map(|x| {
        canvas.current.get(x, y).and_then(|cell| {
          if cell.is_continuation() || cell.ch == ' ' {
            None
          } else {
            Some(cell.ch)
          }
        })
      })
      .collect()
  }

  fn raw_row_prefix(canvas: &CanvasService, y: u16, width: u16) -> String {
    (0..width)
      .map(|x| canvas.current.get(x, y).map(|cell| cell.ch).unwrap_or(' '))
      .collect()
  }

  /// 模拟 home 界面的 action 提示渲染：{key:} + CJK 尾随文本。
  /// 验证富文本解析后的所有字符均被写入画布，不会被截断。
  #[test]
  fn rich_text_key_with_cjk_tail() {
    let mut canvas = CanvasService::new();

    // 构建参数：模拟 home.confirm → [Enter]
    let mut key_actions = HashMap::new();
    key_actions.insert("home.confirm".to_string(), vec![vec!["enter".to_string()]]);
    let params = RichTextParams {
      values: HashMap::new(),
      key_actions,
    };

    let text = "f%<fg:bright_black>{key:home.confirm} 确认</fg>";
    canvas.text(&DrawTextParams {
      x: 0,
      y: 0,
      text: text.to_string(),
      params: Some(params),
      ..Default::default()
    });

    // 读取第 0 行的全部非空白格，拼接为可见字符串
    // 预期：{key:home.confirm} → [Enter]，后面跟 " 确认"
    assert_eq!(
      visible_row(&canvas, 0),
      "[Enter]确认",
      "full text including CJK tail must be present"
    );
  }

  /// 验证纯 CJK 文本（不含 {key:}）也能完整写入。
  #[test]
  fn styled_text_cjk_full() {
    let mut canvas = CanvasService::new();
    let style = TextStyle::default();
    canvas.styled_text(0, 0, "确认", style);

    assert_eq!(
      visible_row(&canvas, 0),
      "确认",
      "CJK characters must all be written"
    );
  }

  #[test]
  fn normal_wrap_truncates_with_marker_by_grapheme_width() {
    let mut canvas = CanvasService::new();
    canvas.text(&DrawTextParams {
      x: 0,
      y: 0,
      text: "我爱你xxxxoooo".to_string(),
      max_width: Some(10),
      overflow_marker: Some("...".to_string()),
      ..Default::default()
    });

    assert_eq!(visible_row(&canvas, 0), "我爱你x...");
  }

  #[test]
  fn none_wrap_ignores_explicit_newlines() {
    let mut canvas = CanvasService::new();
    canvas.text(&DrawTextParams {
      x: 0,
      y: 0,
      text: "ab\ncd".to_string(),
      wrap_mode: TextWrapMode::None,
      ..Default::default()
    });

    assert_eq!(visible_row(&canvas, 0), "abcd");
    assert_eq!(visible_row(&canvas, 1), "");
  }

  #[test]
  fn auto_wrap_respects_width_and_explicit_newlines() {
    let mut canvas = CanvasService::new();
    canvas.text(&DrawTextParams {
      x: 0,
      y: 0,
      text: "abcd\nefgh".to_string(),
      wrap_mode: TextWrapMode::Auto,
      max_width: Some(3),
      ..Default::default()
    });

    assert_eq!(visible_row(&canvas, 0), "abc");
    assert_eq!(visible_row(&canvas, 1), "d");
    assert_eq!(visible_row(&canvas, 2), "efg");
    assert_eq!(visible_row(&canvas, 3), "h");
  }

  #[test]
  fn max_height_marks_hidden_text() {
    let mut canvas = CanvasService::new();
    canvas.text(&DrawTextParams {
      x: 0,
      y: 0,
      text: "abcd".to_string(),
      wrap_mode: TextWrapMode::Auto,
      max_width: Some(2),
      max_height: Some(1),
      overflow_marker: Some(".".to_string()),
      ..Default::default()
    });

    assert_eq!(visible_row(&canvas, 0), "a.");
    assert_eq!(visible_row(&canvas, 1), "");
  }

  #[test]
  fn multiline_alignment_is_relative_to_first_line() {
    let mut canvas = CanvasService::new();
    canvas.text(&DrawTextParams {
      x: 0,
      y: 0,
      text: "abcd\nef".to_string(),
      line_align: TextAlign::Right,
      ..Default::default()
    });

    assert_eq!(raw_row_prefix(&canvas, 0, 4), "abcd");
    assert_eq!(raw_row_prefix(&canvas, 1, 4), "  ef");
  }

  #[test]
  fn rich_text_wrapping_preserves_segment_style() {
    let mut canvas = CanvasService::new();
    canvas.text(&DrawTextParams {
      x: 0,
      y: 0,
      text: "f%<fg:red>ab</fg><fg:blue>cd</fg>".to_string(),
      wrap_mode: TextWrapMode::Auto,
      max_width: Some(3),
      ..Default::default()
    });

    let c = canvas.current.get(2, 0).expect("c cell");
    let d = canvas.current.get(0, 1).expect("d cell");
    assert_eq!(c.ch, 'c');
    assert_eq!(d.ch, 'd');
    assert_eq!(
      c.style.foreground,
      Some(TextColor::Terminal(TerminalColor::Blue))
    );
    assert_eq!(
      d.style.foreground,
      Some(TextColor::Terminal(TerminalColor::Blue))
    );
  }
}
