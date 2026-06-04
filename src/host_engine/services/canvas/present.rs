use crossterm::style::Attribute;
use std::io::{self, Stdout, Write};

use crossterm::QueueableCommand;
use crossterm::cursor::MoveTo;
use crossterm::style::{Print, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor};

use super::{
  CanvasBuffer, CanvasCellContent, CanvasStyle, style_attributes, text_color_to_crossterm_color,
};

// 差异块
struct DiffSpan {
  x: u16,             // x轴
  text: String,       // 文本
  style: CanvasStyle, // 样式
}

// 差异块缓冲区
fn flush_diff_span(
  spans: &mut Vec<DiffSpan>,
  current_x: Option<u16>,
  current_text: &mut String,
  current_style: &mut Option<CanvasStyle>,
) {
  // 如果当前文本是空就跳过
  if current_text.is_empty() {
    return;
  }

  // 判断x轴
  let Some(x) = current_x else {
    return;
  };

  // 判断样式
  let Some(style) = current_style.clone() else {
    return;
  };

  // 入队一个差异缓冲区块
  spans.push(DiffSpan {
    x,
    text: std::mem::take(current_text),
    style,
  });

  *current_style = None;
}

// 计算两帧之间的差异区域
fn collect_diff_spans_for_row(
  front_buffer: &CanvasBuffer,
  back_buffer: &CanvasBuffer,
  y: u16,
) -> Vec<DiffSpan> {
  // 内容块
  let mut spans = Vec::new();

  // 初始化当前x轴
  let mut current_x: Option<u16> = None;
  // 初始化当前文本
  let mut current_text = String::new();
  // 初始化当前样式
  let mut current_style: Option<CanvasStyle> = None;

  // 遍历当前行的每一列
  for x in 0..back_buffer.width() {
    let Some(front_cell) = front_buffer.get(x, y) else {
      continue;
    };
    let Some(back_cell) = back_buffer.get(x, y) else {
      continue;
    };

    // 如果相同就结束当前内容快
    if front_cell == back_cell {
      flush_diff_span(&mut spans, current_x, &mut current_text, &mut current_style);
      current_x = None;
      continue;
    }

    // 字符类型
    match back_cell.content {
      // 普通字符处理
      CanvasCellContent::Character(ch) => {
        let same_style = current_style.as_ref() == Some(&back_cell.style);

        if current_x.is_none() {
          current_x = Some(x);
          current_style = Some(back_cell.style.clone());
        } else if !same_style {
          flush_diff_span(&mut spans, current_x, &mut current_text, &mut current_style);
          current_x = Some(x);
          current_style = Some(back_cell.style.clone());
        }

        current_text.push(ch);
      }
      // 宽字符
      CanvasCellContent::WideContinuation => {
        // WideContinuation 不直接打印，也会打断当前可打印的区域
        flush_diff_span(&mut spans, current_x, &mut current_text, &mut current_style);
        current_x = None;
      }
    }
  }

  flush_diff_span(&mut spans, current_x, &mut current_text, &mut current_style);

  spans
}

// 应用画布样式
fn apply_canvas_style(stdout: &mut Stdout, style: &CanvasStyle) -> io::Result<()> {
  stdout.queue(ResetColor)?;

  if let Some(foreground) = &style.foreground {
    stdout.queue(SetForegroundColor(text_color_to_crossterm_color(
      foreground,
    )))?;
  }

  if let Some(background) = &style.background {
    stdout.queue(SetBackgroundColor(text_color_to_crossterm_color(
      background,
    )))?;
  }

  for attribute in style_attributes(style) {
    stdout.queue(SetAttribute(attribute))?;
  }

  Ok(())
}

// 重置画布样式
fn reset_canvas_style(stdout: &mut Stdout) -> io::Result<()> {
  stdout.queue(ResetColor)?;
  stdout.queue(SetAttribute(Attribute::Reset))?;
  Ok(())
}

// 将缓冲区会知到终端上
pub fn present_buffer(buffer: &CanvasBuffer, stdout: &mut Stdout) -> io::Result<()> {
  // 遍历缓冲区每一行
  for y in 0..buffer.height() {
    // 光标移动到当前行开头
    stdout.queue(MoveTo(0, y))?;

    // 遍历每一列
    for x in 0..buffer.width() {
      // 单元格不存在就跳过（防御）
      let Some(cell) = buffer.get(x, y) else {
        continue;
      };

      // 根据类型输出
      match cell.content {
        // 正常字符直接输出
        CanvasCellContent::Character(ch) => {
          apply_canvas_style(stdout, &cell.style)?;
          stdout.queue(Print(ch))?;
        }
        CanvasCellContent::WideContinuation => {
          // 已经被前一个宽字符占据，这里不打印任何东西
        }
      }
    }
  }

  // 刷新缓冲区
  reset_canvas_style(stdout)?;
  stdout.flush()?;
  Ok(())
}

// 对比提交画布到终端
pub fn present_buffer_diff(
  front_buffer: &CanvasBuffer,
  back_buffer: &CanvasBuffer,
  stdout: &mut Stdout,
) -> io::Result<()> {
  // TODO(canvas):
  // Wide character replacement needs stricter cleanup rules.
  // Example: replacing "中" with "A" must clear the second occupied cell.

  for y in 0..back_buffer.height() {
    let spans = collect_diff_spans_for_row(front_buffer, back_buffer, y);

    for span in spans {
      stdout.queue(MoveTo(span.x, y))?;
      apply_canvas_style(stdout, &span.style)?;
      stdout.queue(Print(span.text))?;
    }
  }

  reset_canvas_style(stdout)?;
  stdout.flush()?;

  Ok(())
}
