use std::io::{self, Write};

use crossterm::{
  QueueableCommand,
  cursor::MoveTo,
  style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};

use super::{
  buffer::CanvasBuffer,
  cell::{CanvasCell, WIDE_CONTINUATION},
};
use crate::host_engine::services::{
  RichTextParams, RichTextSegment, RichTextService, TerminalColor, TerminalService, TextColor,
  TextStyle,
};
use crate::host_engine::services::unicode::{display_width, graphemes};

// ── 绘制参数 ──

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

    if params.text.starts_with("f%") {
      self.draw_rich_text(params, &default_style);
    } else {
      self.draw_plain_text(params, &default_style);
    }
  }

  /// 画布底层基元：在 (x, y) 处以指定样式绘制文本。
  /// 不做任何前缀检查，调用方给什么就画什么。
  ///
  /// 正确处理 Unicode 显示宽度：
  /// - 零宽字符（ZWJ、ZWS、组合标记）写入单元格但不推进光标
  /// - 普通字符（ASCII、拉丁）推进 1 格
  /// - 宽字符（CJK、emoji、全角）推进 2 格，并标记右侧格为 WIDE_CONTINUATION
  pub fn styled_text(
    &mut self,
    x: u16,
    y: u16,
    text: &str,
    style: TextStyle,
  ) {
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

  pub fn resize(&mut self, width: u16, height: u16) {
    self.current.resize(width, height);
    self.previous.resize(width, height);
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
        let current_cell = self
          .current
          .get(x, y)
          .unwrap_or(&BLANK_CELL);

        // 跳过宽字符延续格 —— 它们属于左侧的宽字符，
        // 不应打断当前 run，也不应启动新 run。
        if current_cell.is_continuation() {
          continue;
        }

        let previous_cell = self
          .previous
          .get(x, y)
          .unwrap_or(&BLANK_CELL);

        let cell_changed =
          self.force_full_redraw || current_cell != previous_cell;

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

  fn draw_plain_text(
    &mut self,
    params: &DrawTextParams,
    style: &TextStyle,
  ) {
    for (line_index, line) in params.text.lines().enumerate() {
      self.styled_text(
        params.x,
        params.y.saturating_add(line_index as u16),
        line,
        style.clone(),
      );
    }
  }

  fn draw_rich_text(
    &mut self,
    params: &DrawTextParams,
    default_style: &TextStyle,
  ) {
    let rich_text_service = RichTextService::new();
    let rich_text = rich_text_service.parse(&params.text, params.params.as_ref());

    let mut cursor_x = params.x;
    let mut cursor_y = params.y;

    for segment in &rich_text.segments {
      let merged = merge_style(default_style, &segment.style);
      let lines: Vec<&str> = segment.text.split('\n').collect();

      for (i, line) in lines.iter().enumerate() {
        if i > 0 {
          cursor_x = params.x;
          cursor_y = cursor_y.saturating_add(1);
        }
        if !line.is_empty() {
          self.styled_text(cursor_x, cursor_y, line, merged.clone());
          cursor_x = cursor_x.saturating_add(display_width(line) as u16);
        }
      }
    }
  }
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

  /// 模拟 home 界面的 action 提示渲染：{key:} + CJK 尾随文本。
  /// 验证富文本解析后的所有字符均被写入画布，不会被截断。
  #[test]
  fn rich_text_key_with_cjk_tail() {
    let mut canvas = CanvasService::new();

    // 构建参数：模拟 home.confirm → [Enter]
    let mut key_actions = HashMap::new();
    key_actions.insert(
      "home.confirm".to_string(),
      vec![vec!["enter".to_string()]],
    );
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
    let row: String = (0..canvas.width())
      .filter_map(|x| {
        canvas.current.get(x, 0).and_then(|cell| {
          if cell.is_continuation() || cell.ch == ' ' {
            None
          } else {
            Some(cell.ch)
          }
        })
      })
      .collect();

    // 预期：{key:home.confirm} → [Enter]，后面跟 " 确认"
    assert_eq!(row, "[Enter]确认", "full text including CJK tail must be present");
  }

  /// 验证纯 CJK 文本（不含 {key:}）也能完整写入。
  #[test]
  fn styled_text_cjk_full() {
    let mut canvas = CanvasService::new();
    let style = TextStyle::default();
    canvas.styled_text(0, 0, "确认", style);

    let row: String = (0..canvas.width())
      .filter_map(|x| {
        canvas.current.get(x, 0).and_then(|cell| {
          if cell.is_continuation() || cell.ch == ' ' {
            None
          } else {
            Some(cell.ch)
          }
        })
      })
      .collect();

    assert_eq!(row, "确认", "CJK characters must all be written");
  }
}
