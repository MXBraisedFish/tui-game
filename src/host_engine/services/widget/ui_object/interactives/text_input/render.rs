use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use super::layout::VisualLayout;
use super::state::{HitSnapshot, TextInputState};
use super::types::{
  TextAlign, TextInputCursorShape, TextInputRenderParams, TextSurface, VerticalAlign,
};
use crate::host_engine::services::{CanvasService, TextStyle};

fn align_offset(align: TextAlign, container: u16, content: u16) -> u16 {
  let clamped = content.min(container);
  match align {
    TextAlign::Left => 0,
    TextAlign::Center => (container - clamped) / 2,
    TextAlign::Right => container - clamped,
  }
}

pub(super) fn render_single_line(
  state: &mut TextInputState,
  active: bool,
  cursor_visible: bool,
  params: &TextInputRenderParams,
  canvas: &mut CanvasService,
  surface: TextSurface,
  order: u64,
) -> Option<(u16, u16)> {
  let y = match params.vertical_align {
    VerticalAlign::Top => params.rect.y,
    VerticalAlign::Center => params.rect.y + (params.rect.height - 1) / 2,
    VerticalAlign::Bottom => params.rect.y + params.rect.height - 1,
  };
  if state.buffer.text().is_empty() {
    draw_placeholder(canvas, surface, y, params);
  }
  if state.buffer.text().is_empty() && !active {
    state.hit = Some(HitSnapshot {
      rect: params.rect,
      origin: (0, 0),
      surface_rank: 0,
      width: params.rect.width as usize,
      first_line: 0,
      single_start: 0,
      order,
    });
    return None;
  }
  let layout = VisualLayout::new(state.buffer.text(), usize::MAX / 2);
  let (_, cursor_x_full) = layout.position(state.buffer.cursor(), Some(0));
  let cursor_glyph = layout
    .glyphs
    .iter()
    .find(|glyph| glyph.start == state.buffer.cursor());
  let cursor_width = cursor_glyph.map(|glyph| glyph.width).unwrap_or_else(|| {
    cursor_marker(params.cursor_shape.unwrap_or_default())
      .map(UnicodeWidthStr::width)
      .unwrap_or(0)
  });
  let mut start = state.buffer.cursor();
  let mut used = cursor_width;
  for glyph in layout
    .glyphs
    .iter()
    .rev()
    .filter(|glyph| glyph.end <= state.buffer.cursor())
  {
    if used + glyph.width > params.rect.width as usize {
      break;
    }
    used += glyph.width;
    start = glyph.start;
  }
  let offset_x = align_offset(params.text_align, params.rect.width, used as u16);
  let start_x = layout.position(start, Some(0)).1;
  let mut x = 0;
  let selection = active.then(|| state.buffer.selection()).flatten();
  for glyph in layout.glyphs.iter().filter(|glyph| glyph.end > start) {
    if x + glyph.width > params.rect.width as usize {
      break;
    }
    let at_cursor = active && glyph.start == state.buffer.cursor();
    let selected = selection
      .as_ref()
      .is_some_and(|range| range.start < glyph.end && glyph.start < range.end);
    let style = if (at_cursor && cursor_visible) || selected {
      reversed_text_style(params)
    } else {
      input_text_style(params)
    };
    draw_styled(
      canvas,
      surface,
      params.rect.x + offset_x + x as u16,
      y,
      &glyph.text,
      style,
    );
    x += glyph.width;
  }
  let cursor_x = cursor_x_full.saturating_sub(start_x);
  if active && cursor_glyph.is_none() && cursor_visible {
    if let Some(marker) = cursor_marker(params.cursor_shape.unwrap_or_default()) {
      draw_styled(
        canvas,
        surface,
        params.rect.x + offset_x + cursor_x as u16,
        y,
        marker,
        input_cursor_style(params),
      );
    }
  }
  state.hit = Some(HitSnapshot {
    rect: params.rect,
    origin: (0, 0),
    surface_rank: 0,
    width: params.rect.width as usize,
    first_line: 0,
    single_start: start,
    order,
  });
  active.then_some((params.rect.x + offset_x + cursor_x as u16, y))
}

pub(super) fn render_multi_line(
  state: &mut TextInputState,
  active: bool,
  cursor_visible: bool,
  params: &TextInputRenderParams,
  canvas: &mut CanvasService,
  surface: TextSurface,
  order: u64,
) -> Option<(u16, u16)> {
  if state.buffer.text().is_empty() {
    draw_placeholder(canvas, surface, params.rect.y, params);
  }
  if state.buffer.text().is_empty() && !active {
    state.hit = Some(HitSnapshot {
      rect: params.rect,
      origin: (0, 0),
      surface_rank: 0,
      width: params.rect.width as usize,
      first_line: 0,
      single_start: 0,
      order,
    });
    return None;
  }
  let layout = VisualLayout::new(state.buffer.text(), params.rect.width as usize);
  let (mut cursor_line, mut cursor_x) = layout.position(state.buffer.cursor(), state.visual_line);
  if active
    && !layout
      .glyphs
      .iter()
      .any(|glyph| glyph.start == state.buffer.cursor())
    && cursor_x >= params.rect.width as usize
  {
    cursor_line += 1;
    cursor_x = 0;
  }
  let first_line = if active {
    cursor_line.saturating_sub(params.rect.height as usize - 1)
  } else {
    0
  };

  // 预计算每行的最大宽度（用于水平对齐）
  let mut line_widths: Vec<usize> = Vec::new();
  for glyph in &layout.glyphs {
    let line_end = glyph.x + glyph.width;
    while line_widths.len() <= glyph.line {
      line_widths.push(0);
    }
    if line_end > line_widths[glyph.line] {
      line_widths[glyph.line] = line_end;
    }
  }

  let selection = active.then(|| state.buffer.selection()).flatten();
  for glyph in layout
    .glyphs
    .iter()
    .filter(|glyph| (first_line..first_line + params.rect.height as usize).contains(&glyph.line))
  {
    let line_w = line_widths.get(glyph.line).copied().unwrap_or(0) as u16;
    let offset_x = align_offset(params.text_align, params.rect.width, line_w);
    let at_cursor = active && glyph.start == state.buffer.cursor();
    let selected = selection
      .as_ref()
      .is_some_and(|range| range.start < glyph.end && glyph.start < range.end);
    let style = if (at_cursor && cursor_visible) || selected {
      reversed_text_style(params)
    } else {
      input_text_style(params)
    };
    draw_styled(
      canvas,
      surface,
      params.rect.x + offset_x + glyph.x as u16,
      params.rect.y + (glyph.line - first_line) as u16,
      &glyph.text,
      style,
    );
  }
  if active
    && !layout
      .glyphs
      .iter()
      .any(|glyph| glyph.start == state.buffer.cursor())
    && cursor_visible
  {
    if let Some(marker) = cursor_marker(params.cursor_shape.unwrap_or_default()) {
      let line_w = line_widths.get(cursor_line).copied().unwrap_or(0) as u16;
      let offset_x = align_offset(params.text_align, params.rect.width, line_w);
      draw_styled(
        canvas,
        surface,
        params.rect.x + offset_x + cursor_x as u16,
        params.rect.y + (cursor_line - first_line) as u16,
        marker,
        input_cursor_style(params),
      );
    }
  }
  state.hit = Some(HitSnapshot {
    rect: params.rect,
    origin: (0, 0),
    surface_rank: 0,
    width: params.rect.width as usize,
    first_line,
    single_start: 0,
    order,
  });
  let cursor_line_w = line_widths.get(cursor_line).copied().unwrap_or(0) as u16;
  let cursor_offset_x = align_offset(params.text_align, params.rect.width, cursor_line_w);
  active.then_some((
    params.rect.x + cursor_offset_x + cursor_x as u16,
    params.rect.y + (cursor_line - first_line) as u16,
  ))
}

fn cursor_marker(shape: TextInputCursorShape) -> Option<&'static str> {
  match shape {
    TextInputCursorShape::Block => Some("█"),
    TextInputCursorShape::Underline => Some("_"),
    TextInputCursorShape::None => None,
    TextInputCursorShape::Line => Some("▏"),
  }
}

pub(super) fn fill_input_background(
  canvas: &mut CanvasService,
  surface: TextSurface,
  params: &TextInputRenderParams,
) {
  let line = " ".repeat(params.rect.width as usize);
  let style = TextStyle {
    background: params.bg.clone(),
    ..Default::default()
  };
  for y in 0..params.rect.height {
    draw_styled(
      canvas,
      surface,
      params.rect.x,
      params.rect.y + y,
      &line,
      style.clone(),
    );
  }
}

fn input_text_style(params: &TextInputRenderParams) -> TextStyle {
  TextStyle {
    foreground: params.fg.clone(),
    background: params.bg.clone(),
    ..params.text_style.clone()
  }
}

fn input_placeholder_style(params: &TextInputRenderParams) -> TextStyle {
  TextStyle {
    foreground: params.placeholder_fg.clone(),
    background: params.bg.clone(),
    ..params.placeholder_style.clone()
  }
}

fn input_cursor_style(params: &TextInputRenderParams) -> TextStyle {
  TextStyle {
    background: params.bg.clone(),
    ..params.cursor_style.clone()
  }
}

fn reversed_text_style(params: &TextInputRenderParams) -> TextStyle {
  let mut style = input_text_style(params);
  style.reverse = !style.reverse;
  style
}

fn draw_prefix(
  canvas: &mut CanvasService,
  surface: TextSurface,
  x: u16,
  y: u16,
  text: &str,
  width: u16,
  style: TextStyle,
) {
  let mut used = 0;
  let text: String = text
    .graphemes(true)
    .take_while(|grapheme| {
      let next = used + UnicodeWidthStr::width(*grapheme);
      if next > width as usize {
        false
      } else {
        used = next;
        true
      }
    })
    .collect();
  draw_styled(canvas, surface, x, y, &text, style);
}

fn draw_placeholder(
  canvas: &mut CanvasService,
  surface: TextSurface,
  y: u16,
  params: &TextInputRenderParams,
) {
  let max_w = params.rect.width.saturating_sub(1) as usize;
  let placeholder_w = params.placeholder.graphemes(true).fold(0usize, |acc, g| {
    let w = UnicodeWidthStr::width(g);
    if acc + w > max_w {
      acc
    } else {
      acc + w
    }
  }) as u16;
  let offset = align_offset(params.text_align, params.rect.width, placeholder_w + 1);
  draw_prefix(
    canvas,
    surface,
    params.rect.x + offset + 1,
    y,
    &params.placeholder,
    params.rect.width.saturating_sub(offset + 1),
    input_placeholder_style(params),
  );
}

fn draw_styled(
  canvas: &mut CanvasService,
  surface: TextSurface,
  x: u16,
  y: u16,
  text: &str,
  style: TextStyle,
) {
  match surface {
    TextSurface::Base => canvas.styled_text(x, y, text, style),
    TextSurface::Slice(id) => {
      canvas.styled_text_on(id, x, y, text, style);
    }
    TextSurface::Host => canvas.host_styled_text(x, y, text, style),
  }
}
