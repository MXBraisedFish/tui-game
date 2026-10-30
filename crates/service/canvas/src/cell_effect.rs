//! Atomic, row-ordered cell transformations over clipped drawing buffers.

use tg_core_style::CanvasCell;
use tg_core_unicode::graphemes;

use crate::buffer::CanvasBuffer;

/// A signed rectangular selection in buffer coordinates.
///
/// # Fields
///
/// * `x`, `y` - The signed selection origin.
/// * `width`, `height` - The selection dimensions in terminal cells.
/// * `include_continuation` - Whether a continuation selects its original grapheme.
#[derive(Clone, Copy, Debug)]
pub struct CellEffectRegion {
  /// The first selected column.
  pub x: i32,
  /// The first selected row.
  pub y: i32,
  /// The number of selected columns.
  pub width: u16,
  /// The number of selected rows.
  pub height: u16,
  /// Whether selecting a continuation also selects its original wide grapheme.
  pub include_continuation: bool,
}

/// An original cell or a transformed write in signed buffer coordinates.
///
/// # Fields
///
/// * `x`, `y` - The original or destination coordinates.
/// * `cell` - One grapheme and its style, or empty text for an unwritten cell.
/// * `written` - Whether the cell contributes to composition.
#[derive(Clone, Debug)]
pub struct CellEffectCell {
  /// The original or destination column.
  pub x: i32,
  /// The original or destination row.
  pub y: i32,
  /// The grapheme and its independent style flags.
  pub cell: CanvasCell,
  /// Whether this cell contributes to surface composition.
  pub written: bool,
}

impl CanvasBuffer {
  /// Transform a snapshot of selected graphemes, then clear and rewrite them in original order.
  ///
  /// Skip continuation markers unless they select their original grapheme. Unwritten cells
  /// carry empty text. Writes may leave the selection but remain clipped to this buffer.
  /// Preserve the original buffer if processing any cell fails.
  ///
  /// # Errors
  ///
  /// Propagate the processing callback's error without applying any writes.
  pub fn transform_region<E>(
    &mut self,
    region: CellEffectRegion,
    mut process: impl FnMut(CellEffectCell) -> Result<CellEffectCell, E>,
  ) -> Result<(), E> {
    let start_x = i64::from(region.x).clamp(0, i64::from(self.width())) as u16;
    let start_y = i64::from(region.y).clamp(0, i64::from(self.height())) as u16;
    let end_x =
      (i64::from(region.x) + i64::from(region.width)).clamp(0, i64::from(self.width())) as u16;
    let end_y =
      (i64::from(region.y) + i64::from(region.height)).clamp(0, i64::from(self.height())) as u16;
    let mut originals = Vec::new();
    for y in start_y..end_y {
      let mut previous = None;
      for x in start_x..end_x {
        let mut head = x;
        if self.get(x, y).is_some_and(CanvasCell::is_continuation) {
          if !region.include_continuation {
            continue;
          }
          while head > 0 && self.get(head, y).is_some_and(CanvasCell::is_continuation) {
            head -= 1;
          }
          let original = self.get(head, y).unwrap();
          if original.is_continuation() || usize::from(x - head) >= cell_width(&original.text) {
            continue;
          }
        }
        if previous == Some(head) {
          continue;
        }
        previous = Some(head);
        let written = self.is_written(head, y);
        let mut cell = self.get(head, y).unwrap().clone();
        if !written {
          cell.text.clear();
        }
        originals.push(CellEffectCell {
          x: i32::from(head),
          y: i32::from(y),
          cell,
          written,
        });
      }
    }
    let mut writes = Vec::with_capacity(originals.len());
    for original in &originals {
      writes.push(process(original.clone())?);
    }
    for original in &originals {
      self.erase_grapheme(original.x as u16, original.y as u16);
    }
    for write in writes {
      self.write_effect_cell(write);
    }
    Ok(())
  }

  fn erase_grapheme(&mut self, x: u16, y: u16) {
    let Some(cell) = self.get(x, y) else {
      return;
    };
    let mut head = x;
    if cell.is_continuation() {
      while head > 0 && self.get(head, y).is_some_and(CanvasCell::is_continuation) {
        head -= 1;
      }
    }
    let width = self.get(head, y).map_or(1, |cell| cell_width(&cell.text));
    for column in u32::from(head)..(u32::from(head) + width as u32).min(u32::from(self.width())) {
      self.erase(column as u16, y);
    }
  }

  fn write_effect_cell(&mut self, mut write: CellEffectCell) {
    let (Ok(x), Ok(y)) = (u16::try_from(write.x), u16::try_from(write.y)) else {
      return;
    };
    let width = cell_width(&write.cell.text);
    if y >= self.height() || u32::from(x) + width as u32 > u32::from(self.width()) {
      return;
    }
    for offset in 0..width {
      self.erase_grapheme(x + offset as u16, y);
    }
    if !write.written {
      return;
    }
    // A written empty result needs a physical space to erase previous terminal output.
    if write.cell.text.is_empty() {
      write.cell.text.push(' ');
    }
    self.set(x, y, write.cell);
    for offset in 1..width {
      self.set(x + offset as u16, y, CanvasCell::continuation());
    }
  }
}

fn cell_width(text: &str) -> usize {
  graphemes(text)
    .first()
    .map_or(1, |grapheme| grapheme.display_width.max(1))
}

#[cfg(test)]
mod tests {
  use super::*;

  fn region(x: i32, width: u16, include_continuation: bool) -> CellEffectRegion {
    CellEffectRegion {
      x,
      y: 0,
      width,
      height: 1,
      include_continuation,
    }
  }

  #[test]
  fn snapshot_preserves_move_sources_and_row_order_and_failure_is_atomic() {
    let mut buffer = CanvasBuffer::new(5, 2);
    buffer.set(0, 0, CanvasCell::new("A"));
    buffer.set(1, 0, CanvasCell::new("B"));
    let before = buffer.clone();
    let result = buffer.transform_region(region(0, 2, false), |cell| {
      if cell.x == 1 { Err("failed") } else { Ok(cell) }
    });
    assert_eq!(result, Err("failed"));
    assert_eq!(buffer, before);
    buffer
      .transform_region(region(0, 2, false), |mut cell| {
        cell.x = 3;
        Ok::<_, ()>(cell)
      })
      .unwrap();
    assert!(!buffer.is_written(0, 0));
    assert!(!buffer.is_written(1, 0));
    assert_eq!(buffer.get(3, 0).unwrap().text, "B");
    buffer
      .transform_region(region(0, 5, false), |mut cell| {
        cell.y += 1;
        Ok::<_, ()>(cell)
      })
      .unwrap();
    assert_eq!(buffer.get(3, 1).unwrap().text, "B");
  }

  #[test]
  fn wide_graphemes_are_selected_once_and_overwrites_remove_the_entire_old_grapheme() {
    let mut buffer = CanvasBuffer::new(5, 1);
    buffer.set(0, 0, CanvasCell::new("我\u{301}"));
    buffer.set(1, 0, CanvasCell::continuation());
    let mut seen = Vec::new();
    buffer
      .transform_region(region(1, 1, false), |cell| {
        seen.push(cell.x);
        Ok::<_, ()>(cell)
      })
      .unwrap();
    assert!(seen.is_empty());
    buffer
      .transform_region(region(1, 1, true), |mut cell| {
        seen.push(cell.x);
        assert_eq!(cell.cell.text, "我\u{301}");
        cell.x = 2;
        Ok::<_, ()>(cell)
      })
      .unwrap();
    assert_eq!(seen, [0]);
    assert_eq!(buffer.get(2, 0).unwrap().text, "我\u{301}");
    assert!(buffer.get(3, 0).unwrap().is_continuation());
    buffer
      .transform_region(region(4, 1, false), |mut cell| {
        assert_eq!(cell.cell.text, "");
        cell.x = 3;
        cell.cell.text = "X".into();
        cell.written = true;
        Ok::<_, ()>(cell)
      })
      .unwrap();
    assert!(!buffer.is_written(2, 0));
    assert_eq!(buffer.get(3, 0).unwrap().text, "X");
  }

  #[test]
  fn extreme_coordinates_clip_without_overflow_or_region_sized_allocation() {
    let mut buffer = CanvasBuffer::new(3, 1);
    for x in [i32::MIN, i32::MAX] {
      buffer
        .transform_region(region(x, u16::MAX, true), |_| -> Result<_, ()> {
          panic!("outside buffer")
        })
        .unwrap();
    }
    buffer
      .transform_region(region(-1, 3, false), |mut cell| {
        cell.x = i32::MAX;
        Ok::<_, ()>(cell)
      })
      .unwrap();
    assert!(!buffer.is_written(0, 0));
  }

  #[test]
  fn orphaned_continuations_never_reach_the_processing_callback() {
    let mut buffer = CanvasBuffer::new(3, 1);
    buffer.set(0, 0, CanvasCell::continuation());
    buffer.set(1, 0, CanvasCell::new("A"));
    buffer.set(2, 0, CanvasCell::continuation());
    let mut seen = Vec::new();
    buffer
      .transform_region(region(0, 3, true), |cell| {
        assert!(!cell.cell.is_continuation());
        seen.push(cell.x);
        Ok::<_, ()>(cell)
      })
      .unwrap();
    assert_eq!(seen, [1]);
  }
}
