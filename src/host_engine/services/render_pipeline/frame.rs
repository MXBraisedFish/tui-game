use crate::host_engine::services::CanvasCell;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ComposedCell {
  Empty,
  Text(CanvasCell),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComposedFrame {
  width: u16,
  height: u16,
  cells: Vec<ComposedCell>,
}

impl ComposedFrame {
  pub fn new(width: u16, height: u16) -> Self {
    let len = width as usize * height as usize;
    Self {
      width,
      height,
      cells: vec![ComposedCell::Empty; len],
    }
  }

  pub fn width(&self) -> u16 {
    self.width
  }

  pub fn height(&self) -> u16 {
    self.height
  }

  pub fn get(&self, x: u16, y: u16) -> Option<&ComposedCell> {
    let index = self.index(x, y)?;
    self.cells.get(index)
  }

  pub fn set(&mut self, x: u16, y: u16, cell: ComposedCell) {
    let Some(index) = self.index(x, y) else {
      return;
    };
    if let Some(target) = self.cells.get_mut(index) {
      *target = cell;
    }
  }

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
