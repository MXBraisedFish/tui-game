//! Layout support for the widget service.

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use super::state::TextInputState;
use super::types::TextInputMode;

/// The visual glyph representation used by this module.
///
/// # Fields
///
/// * `start` - The start.
/// * `end` - The end.
/// * `text` - The text to process or display.
/// * `line` - The line.
/// * `x` - The horizontal coordinate in terminal cells.
/// * `width` - The width in terminal columns.
#[derive(Clone)]
pub(super) struct VisualGlyph {
  /// The start.
  pub start: usize,
  /// The end.
  pub end: usize,
  /// The text to process or display.
  pub text: String,
  /// The line.
  pub line: usize,
  /// The horizontal coordinate in terminal cells.
  pub x: usize,
  /// The width in terminal columns.
  pub width: usize,
}

/// The visual line representation used by this module.
///
/// # Fields
///
/// * `start` - The start.
/// * `end` - The end.
#[derive(Clone, Copy)]
pub(super) struct VisualLine {
  /// The start.
  pub start: usize,
  /// The end.
  pub end: usize,
}

/// Resolved geometry and positions used to display visual.
///
/// # Fields
///
/// * `glyphs` - The ordered glyphs retained by this owner.
/// * `lines` - The ordered lines retained by this owner.
pub(super) struct VisualLayout {
  /// The ordered glyphs retained by this owner.
  pub glyphs: Vec<VisualGlyph>,
  /// The ordered lines retained by this owner.
  pub lines: Vec<VisualLine>,
}

impl VisualLayout {
  /// Create a visual layout initialized from `text`, `width`.
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

  /// Return the position for the addressed object.
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

  /// Return the grapheme boundary nearest to a text-layout column.
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

/// Move the cursor between rendered lines while retaining its preferred column.
///
/// # Arguments
///
/// * `state` - The state.
/// * `width` - The width in terminal columns.
/// * `delta` - The delta.
/// * `extend` - Whether movement extends the current selection.
pub(super) fn move_vertical(state: &mut TextInputState, width: usize, delta: isize, extend: bool) {
  if !extend && let Some(range) = state.buffer.selection() {
    state
      .buffer
      .move_to(if delta < 0 { range.start } else { range.end }, false);
    state.visual_line = None;
    return;
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

/// Move the cursor to the start or end of its rendered line.
///
/// # Arguments
///
/// * `state` - The state.
/// * `width` - The width in terminal columns.
/// * `end` - The end.
/// * `extend` - Whether movement extends the current selection.
pub(super) fn move_line_edge(state: &mut TextInputState, width: usize, end: bool, extend: bool) {
  let layout = VisualLayout::new(state.buffer.text(), width);
  let (line, _) = layout.position(state.buffer.cursor(), state.visual_line);
  let row = layout.lines[line];
  state
    .buffer
    .move_to(if end { row.end } else { row.start }, extend);
  state.visual_line = Some(line);
}

/// Resolve a terminal point to the nearest text cursor boundary.
///
/// # Arguments
///
/// * `state` - The state.
/// * `x` - The horizontal coordinate in terminal cells.
/// * `y` - The vertical coordinate in terminal cells.
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
