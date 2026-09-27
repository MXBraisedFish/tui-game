use tg_core_style::CanvasCell;

/// A canvas buffer: a 2D grid of character cells that also tracks which cells were written.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanvasBuffer {
  width: u16,
  height: u16,
  cells: Vec<CanvasCell>,
  written: Vec<bool>,
}

impl CanvasBuffer {
  pub fn new(width: u16, height: u16) -> Self {
    let size = width as usize * height as usize;
    Self {
      width,
      height,
      cells: vec![CanvasCell::blank(); size],
      written: vec![false; size],
    }
  }
  pub fn width(&self) -> u16 {
    self.width
  }
  pub fn height(&self) -> u16 {
    self.height
  }

  /// Rebuilds the buffer with a new size, discarding the previous content.
  pub fn resize(&mut self, width: u16, height: u16) {
    self.width = width;
    self.height = height;
    let size = width as usize * height as usize;
    self.cells = vec![CanvasCell::blank(); size];
    self.written = vec![false; size];
  }

  /// Clears the buffer, resetting every cell to blank and unwritten.
  pub fn clear(&mut self) {
    for (cell, written) in self.cells.iter_mut().zip(&mut self.written) {
      *cell = CanvasCell::blank();
      *written = false;
    }
  }

  /// Writes `cell` at the given coordinates; out-of-range coordinates are ignored.
  pub fn set(&mut self, x: u16, y: u16, cell: CanvasCell) {
    let Some(index) = self.index(x, y) else {
      return;
    };
    if let Some(target) = self.cells.get_mut(index) {
      *target = cell;
      self.written[index] = true;
    }
  }

  /// Erases the cell at the given coordinates, restoring it to the unwritten state.
  pub fn erase(&mut self, x: u16, y: u16) {
    let Some(index) = self.index(x, y) else {
      return;
    };
    self.cells[index] = CanvasCell::blank();
    self.written[index] = false;
  }
  pub fn get(&self, x: u16, y: u16) -> Option<&CanvasCell> {
    let index = self.index(x, y)?;
    self.cells.get(index)
  }

  /// Returns whether the cell at the given coordinates has been written.
  pub fn is_written(&self, x: u16, y: u16) -> bool {
    self
      .index(x, y)
      .and_then(|index| self.written.get(index))
      .copied()
      .unwrap_or(false)
  }

  /// Returns the plain text of row `y`.
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
