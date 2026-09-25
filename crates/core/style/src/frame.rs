use crate::CanvasCell;

/// 合成后的单元：要么为空，要么包含一个已着色的 CanvasCell。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ComposedCell {
  Empty,
  Text(CanvasCell),
}

/// 合成后的帧缓冲区，用于最终输出到终端。
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

  /// 返回一个标准空白文本单元格，用于初始化帧。
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
    assert_eq!(frame.get(2, 1), Some(&ComposedCell::Text(CanvasCell::new("x"))));
    frame.set(3, 0, ComposedCell::Text(CanvasCell::new("y")));
    assert_eq!(frame.get(3, 0), None);
    assert_eq!(frame.get(0, 2), None);
    assert_eq!(ComposedFrame::blank_text_cell(), CanvasCell::blank());
  }
}
