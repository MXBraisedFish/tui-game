//! Composed terminal-cell grids and blank-cell initialization.

use crate::CanvasCell;

/// An empty composition slot or a styled terminal text cell.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ComposedCell {
  /// No text contribution at this composition slot.
  Empty,
  /// A styled text cell or wide-cell continuation marker.
  Text(CanvasCell),
}

/// A rectangular grid of cells ready for terminal presentation or structured capture.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComposedFrame {
  width: u16,
  height: u16,
  cells: Vec<ComposedCell>,
}

impl ComposedFrame {
  /// Create a terminal-cell grid with every composition slot initially empty.
  pub fn new(width: u16, height: u16) -> Self {
    let len = width as usize * height as usize;
    Self {
      width,
      height,
      cells: vec![ComposedCell::Empty; len],
    }
  }

  /// Return the frame width in terminal columns.
  pub fn width(&self) -> u16 {
    self.width
  }

  /// Return the frame height in terminal rows.
  pub fn height(&self) -> u16 {
    self.height
  }

  /// Return the composed cell at the coordinates, or `None` outside the frame.
  pub fn get(&self, x: u16, y: u16) -> Option<&ComposedCell> {
    let index = self.index(x, y)?;
    self.cells.get(index)
  }

  /// Write a cell only when its coordinates lie within the buffer bounds.
  ///
  /// # Arguments
  ///
  /// * `x` - The horizontal coordinate in terminal cells.
  /// * `y` - The vertical coordinate in terminal cells.
  /// * `cell` - The styled terminal cell to write.
  pub fn set(&mut self, x: u16, y: u16, cell: ComposedCell) {
    let Some(index) = self.index(x, y) else {
      return;
    };
    if let Some(target) = self.cells.get_mut(index) {
      *target = cell;
    }
  }

  /// Return the default styled space used to initialize a composed frame.
  pub fn blank_text_cell() -> CanvasCell {
    CanvasCell::blank()
  }

  fn index(&self, x: u16, y: u16) -> Option<usize> {
    if x >= self.width || y >= self.height {
      return None;
    }
    Some(y as usize * self.width as usize + x as usize)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn out_of_bounds_access_is_ignored_and_cells_start_empty() {
    let mut frame = ComposedFrame::new(3, 2);
    assert_eq!(frame.get(2, 1), Some(&ComposedCell::Empty));
    frame.set(2, 1, ComposedCell::Text(CanvasCell::new("x")));
    assert_eq!(
      frame.get(2, 1),
      Some(&ComposedCell::Text(CanvasCell::new("x")))
    );
    frame.set(3, 0, ComposedCell::Text(CanvasCell::new("y")));
    assert_eq!(frame.get(3, 0), None);
    assert_eq!(frame.get(0, 2), None);
    assert_eq!(ComposedFrame::blank_text_cell(), CanvasCell::blank());
  }
}
