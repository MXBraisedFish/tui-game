//! 差异化画布渲染模块
//!
//! 通过对比前后两帧画布缓冲区的差异，仅重绘发生变化的区域，
//! 从而减少终端 I/O 操作，提升渲染性能。

use std::io::{self, Stdout, Write};

use crossterm::QueueableCommand;
use crossterm::cursor::MoveTo;
use crossterm::style::Print;

use super::{
  CanvasBuffer, CanvasCellContent, CanvasStyle, DirtySpan, apply_canvas_style,
  reset_canvas_style,
};

/// 差异渲染块
///
/// 表示画布上一段连续的需要重绘的区域。
/// 同一行的多个相邻差异单元格会被合并为一个 DiffSpan。
struct DiffSpan {
  x: u16,             // 该差异块在行内的起始列位置
  text: String,       // 该差异块包含的文本内容
  style: CanvasStyle, // 该差异块使用的样式
}

/// 将当前累积的文本缓冲刷新为一个差异块
///
/// 当遇到样式变化、单元格内容相同、或行结束时，
/// 需要将之前累积的文本打包为一个 DiffSpan。
/// 如果当前文本为空、起始位置未确定、或样式未设置，则跳过本次刷新。
fn flush_diff_span(
  spans: &mut Vec<DiffSpan>,
  current_x: Option<u16>,
  current_text: &mut String,
  current_style: &mut Option<CanvasStyle>,
) {
  // 如果当前文本为空，无需刷新
  if current_text.is_empty() {
    return;
  }

  // 确保起始列位置已确定
  let Some(x) = current_x else {
    return;
  };

  // 确保当前样式已设置
  let Some(style) = current_style.clone() else {
    return;
  };

  // 入队一个差异渲染块，并将累积文本清空
  spans.push(DiffSpan {
    x,
    text: std::mem::take(current_text),
    style,
  });

  // 重置当前样式，准备下一轮的差异收集
  *current_style = None;
}

/// 收集脏区间内的所有差异渲染块
///
/// 仅在脏区间的列范围内逐列对比前后两帧缓冲区，找出内容或样式发生变化的单元格，
/// 将连续的差异单元格合并为一个 DiffSpan 以提高渲染效率。
fn collect_diff_spans_for_dirty_span(
  front_buffer: &CanvasBuffer,
  back_buffer: &CanvasBuffer,
  dirty: DirtySpan,
) -> Vec<DiffSpan> {
  let y = dirty.y;
  let x_start = dirty.start_x;
  let x_end = dirty.end_x.min(back_buffer.width());

  // 差异块列表
  let mut spans = Vec::new();

  // 当前差异块的起始列位置
  let mut current_x: Option<u16> = None;
  // 当前差异块累积的文本
  let mut current_text = String::new();
  // 当前差异块的样式
  let mut current_style: Option<CanvasStyle> = None;

  // 仅遍历脏区间范围内的列
  for x in x_start..x_end {
    let Some(front_cell) = front_buffer.get(x, y) else {
      continue;
    };
    let Some(back_cell) = back_buffer.get(x, y) else {
      continue;
    };

    // 前后帧单元格完全相同，结束当前差异块
    if front_cell == back_cell {
      flush_diff_span(&mut spans, current_x, &mut current_text, &mut current_style);
      current_x = None;
      continue;
    }

    // 单元格内容不同，开始构建差异块
    match back_cell.content {
      // 普通字符：累积到当前差异块的文本中
      CanvasCellContent::Character(ch) => {
        let same_style = current_style.as_ref() == Some(&back_cell.style);

        // 样式相同则继续累积，样式不同则刷新当前块并开始新的差异块
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
      // 宽字符占位符：不直接打印，但会打断当前可打印区域
      CanvasCellContent::WideContinuation => {
        flush_diff_span(&mut spans, current_x, &mut current_text, &mut current_style);
        current_x = None;
      }
    }
  }

  // 区间遍历结束，刷新最后累积的差异块
  flush_diff_span(&mut spans, current_x, &mut current_text, &mut current_style);

  spans
}

/// 检查是否需要清理旧帧中的宽字符占位符
///
/// 当前帧是宽字符占位符但后帧不再是占位符时，
/// 需要清理该位置以避免终端上残留半个宽字符的显示。
fn needs_wide_cleanup(
  front_buffer: &CanvasBuffer,
  back_buffer: &CanvasBuffer,
  x: u16,
  y: u16,
) -> bool {
  // 获取前后帧对应位置的单元格
  let Some(front_cell) = front_buffer.get(x, y) else {
    return false;
  };
  let Some(back_cell) = back_buffer.get(x, y) else {
    return false;
  };

  // 前后相同则无需清理
  if front_cell == back_cell {
    return false;
  }

  // 前帧是宽字符占位符，且前后帧不匹配时才需要清理
  match front_cell.content {
    CanvasCellContent::WideContinuation => !back_cell.is_wide_continuation(),
    _ => false,
  }
}

/// 对比前后两帧画布，仅将脏区间的差异区域渲染到终端
///
/// 对比 front_buffer（前一帧）和 back_buffer（当前帧），仅在 `dirty_spans`
/// 标记的行区间中找出发生变化的内容和样式，对这些差异区域执行终端 I/O 操作。
/// 这是 diff 渲染的入口函数，相比全量渲染能大幅减少终端输出量。
///
/// - `dirty_spans` — 本帧中被修改的行区间列表，跳过未修改的行和列区间
/// - `truecolor` — 是否使用真彩色；false 时降级为 ANSI256
pub fn present_buffer_diff(
  front_buffer: &CanvasBuffer,
  back_buffer: &CanvasBuffer,
  dirty_spans: &[DirtySpan],
  stdout: &mut Stdout,
  truecolor: bool,
) -> io::Result<()> {
  // 仅遍历被标记为脏的行区间，跳过未修改的行
  for span in dirty_spans {
    let y = span.y;

    // 防御性检查：行号超出缓冲区范围则跳过
    if y >= back_buffer.height() {
      continue;
    }

    // 第一遍：在脏区间范围内清理宽字符占位符残留
    // 当前帧的宽字符被移除后，需要在其占位符位置输出空格进行清理
    for x in span.start_x..span.end_x.min(back_buffer.width()) {
      if needs_wide_cleanup(front_buffer, back_buffer, x, y) {
        stdout.queue(MoveTo(x, y))?;
        reset_canvas_style(stdout)?;
        stdout.queue(Print(' '))?;
      }
    }

    // 第二遍：渲染差异化的内容片段
    // 仅收集脏区间范围内的差异块，无需额外过滤
    let spans = collect_diff_spans_for_dirty_span(front_buffer, back_buffer, *span);

    for diff_span in spans {
      // 光标移动到差异块的起始位置
      stdout.queue(MoveTo(diff_span.x, y))?;
      // 应用该差异块的样式
      apply_canvas_style(stdout, &diff_span.style, truecolor)?;
      // 输出差异文本
      stdout.queue(Print(&diff_span.text))?;
    }
  }

  // 渲染完成后重置样式并刷新输出缓冲区
  reset_canvas_style(stdout)?;
  stdout.flush()?;

  Ok(())
}
