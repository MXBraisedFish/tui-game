use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use super::state::TextInputState;
use super::types::TextInputMode;

#[derive(Clone)]
pub(super) struct VisualGlyph {
  pub start: usize,
  pub end: usize,
  pub text: String,
  pub line: usize,
  pub x: usize,
  pub width: usize,
}

#[derive(Clone, Copy)]
pub(super) struct VisualLine {
  pub start: usize,
  pub end: usize,
}

pub(super) struct VisualLayout {
  pub glyphs: Vec<VisualGlyph>,
  pub lines: Vec<VisualLine>,
}

impl VisualLayout {
  pub(super) fn new(text: &str, width: usize) -> Self {
    let width = width.max(1);
    let mut glyphs = Vec::new();
    let mut lines = Vec::new();
    let (mut line_start, mut line, mut x) = (0, 0, 0);
    for (start, grapheme) in text.grapheme_indices(true) {
      let end = start + grapheme.len();
      if grapheme == "\n" {
        lines.push(VisualLine {
          start: line_start,
          end: start,
        });
        line += 1;
        x = 0;
        line_start = end;
        continue;
      }
      let glyph_width = UnicodeWidthStr::width(grapheme);
      if x > 0 && x + glyph_width > width {
        lines.push(VisualLine {
          start: line_start,
          end: start,
        });
        line += 1;
        x = 0;
        line_start = start;
      }
      if glyph_width <= width {
        glyphs.push(VisualGlyph {
          start,
          end,
          text: grapheme.to_string(),
          line,
          x,
          width: glyph_width,
        });
        x += glyph_width;
      }
    }
    lines.push(VisualLine {
      start: line_start,
      end: text.len(),
    });
    Self { glyphs, lines }
  }

  pub(super) fn position(&self, cursor: usize, hint: Option<usize>) -> (usize, usize) {
    let line = hint
      .filter(|line| {
        self
          .lines
          .get(*line)
          .is_some_and(|row| (row.start..=row.end).contains(&cursor))
      })
      .or_else(|| {
        self
          .lines
          .iter()
          .enumerate()
          .rev()
          .find(|(_, row)| (row.start..=row.end).contains(&cursor))
          .map(|(line, _)| line)
      })
      .unwrap_or(0);
    let x = self
      .glyphs
      .iter()
      .filter(|glyph| glyph.line == line && glyph.end <= cursor)
      .map(|glyph| glyph.width)
      .sum();
    (line, x)
  }

  pub(super) fn boundary_at(&self, line: usize, x: usize) -> usize {
    let Some(row) = self.lines.get(line) else {
      return self.lines.last().map(|line| line.end).unwrap_or(0);
    };
    for glyph in self.glyphs.iter().filter(|glyph| glyph.line == line) {
      if x <= glyph.x {
        return glyph.start;
      }
      if x < glyph.x + glyph.width {
        return glyph.end;
      }
    }
    row.end
  }
}

pub(super) fn move_vertical(state: &mut TextInputState, width: usize, delta: isize, extend: bool) {
  if !extend {
    if let Some(range) = state.buffer.selection() {
      state
        .buffer
        .move_to(if delta < 0 { range.start } else { range.end }, false);
      state.visual_line = None;
      return;
    }
  }
  if state.mode == TextInputMode::SingleLine {
    return;
  }
  let layout = VisualLayout::new(state.buffer.text(), width);
  let (line, x) = layout.position(state.buffer.cursor(), state.visual_line);
  let preferred = state.buffer.preferred_column().unwrap_or(x);
  let target =
    (line as isize + delta).clamp(0, layout.lines.len().saturating_sub(1) as isize) as usize;
  if target == line {
    return;
  }
  state
    .buffer
    .set_cursor(layout.boundary_at(target, preferred), extend);
  state.buffer.set_preferred_column(Some(preferred));
  state.visual_line = Some(target);
}

pub(super) fn move_line_edge(state: &mut TextInputState, width: usize, end: bool, extend: bool) {
  let layout = VisualLayout::new(state.buffer.text(), width);
  let (line, _) = layout.position(state.buffer.cursor(), state.visual_line);
  let row = layout.lines[line];
  state
    .buffer
    .move_to(if end { row.end } else { row.start }, extend);
  state.visual_line = Some(line);
}

pub(super) fn cursor_from_point(state: &TextInputState, x: u16, y: u16) -> (usize, usize) {
  let hit = state.hit.unwrap();
  let layout = VisualLayout::new(state.buffer.text(), hit.width);
  let line = if state.mode == TextInputMode::SingleLine {
    0
  } else {
    hit.first_line + y.saturating_sub(hit.rect.y) as usize
  }
  .min(layout.lines.len().saturating_sub(1));
  let local_x = x.saturating_sub(hit.rect.x) as usize;
  let cursor = if state.mode == TextInputMode::SingleLine {
    let start_x = layout.position(hit.single_start, Some(0)).1;
    layout.boundary_at(0, start_x + local_x)
  } else {
    layout.boundary_at(line, local_x)
  };
  (cursor, line)
}
