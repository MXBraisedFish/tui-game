//! Storage and mutation of the module's buffered data.

use tg_core_style::CanvasCell;

/// A terminal-cell buffer retaining explicit writes and wide-grapheme continuation state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanvasBuffer {
  width: u16,
  height: u16,
  cells: Vec<CanvasCell>,
  written: Vec<bool>,
}

impl CanvasBuffer {
  /// Create a canvas buffer initialized from `width`, `height`.
  pub fn new(width: u16, height: u16) -> Self {
    let size = width as usize * height as usize;
    Self {
      width,
      height,
      cells: vec![CanvasCell::blank(); size],
      written: vec![false; size],
    }
  }
  /// Return the current width.
  pub fn width(&self) -> u16 {
    self.width
  }
  /// Return the current height.
  pub fn height(&self) -> u16 {
    self.height
  }

  /// Resize the base terminal-cell buffer and invalidate its previous contents.
  pub fn resize(&mut self, width: u16, height: u16) {
    self.width = width;
    self.height = height;
    let size = width as usize * height as usize;
    self.cells = vec![CanvasCell::blank(); size];
    self.written = vec![false; size];
  }

  /// Discard the previously retained canvas contents.
  pub fn clear(&mut self) {
    for (cell, written) in self.cells.iter_mut().zip(&mut self.written) {
      *cell = CanvasCell::blank();
      *written = false;
    }
  }

  /// Write a cell only when its coordinates lie within the buffer bounds.
  ///
  /// # Arguments
  ///
  /// * `x` - The horizontal coordinate in terminal cells.
  /// * `y` - The vertical coordinate in terminal cells.
  /// * `cell` - The styled terminal cell to write.
  pub fn set(&mut self, x: u16, y: u16, cell: CanvasCell) {
    let Some(index) = self.index(x, y) else {
      return;
    };
    if let Some(target) = self.cells.get_mut(index) {
      *target = cell;
      self.written[index] = true;
    }
  }

  /// Remove the written cell contribution at the supplied buffer coordinates.
  pub fn erase(&mut self, x: u16, y: u16) {
    let Some(index) = self.index(x, y) else {
      return;
    };
    self.cells[index] = CanvasCell::blank();
    self.written[index] = false;
  }
  /// Return access to the requested canvas buffer value when it exists.
  pub fn get(&self, x: u16, y: u16) -> Option<&CanvasCell> {
    let index = self.index(x, y)?;
    self.cells.get(index)
  }

  /// Report whether the addressed object is written.
  pub fn is_written(&self, x: u16, y: u16) -> bool {
    self
      .index(x, y)
      .and_then(|index| self.written.get(index))
      .copied()
      .unwrap_or(false)
  }

  /// Return the visible text stored in the requested buffer row.
  pub fn row_text(&self, y: u16) -> String {
    if y >= self.height {
      return String::new();
    }
    let mut text = String::new();
    for x in 0..self.width {
      if let Some(cell) = self.get(x, y) {
        text.push_str(&cell.text);
      } else {
        text.push(' ');
      }
    }
    text
  }

  fn index(&self, x: u16, y: u16) -> Option<usize> {
    if x >= self.width || y >= self.height {
      return None;
    }
    Some(y as usize * self.width as usize + x as usize)
  }
}
