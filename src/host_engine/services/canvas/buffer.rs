use super::cell::CanvasCell;

/// 画布缓冲区：以二维网格存储字符单元，并跟踪已写入区域。
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

  /// 以新尺寸重建缓冲区，原有内容将被丢弃。
  pub fn resize(&mut self, width: u16, height: u16) {
    self.width = width;
    self.height = height;
    let size = width as usize * height as usize;
    self.cells = vec![CanvasCell::blank(); size];
    self.written = vec![false; size];
  }

  /// 清空缓冲区，所有单元格重置为空白。
  pub fn clear(&mut self) {
    for (cell, written) in self.cells.iter_mut().zip(&mut self.written) {
      *cell = CanvasCell::blank();
      *written = false;
    }
  }

  /// 在指定坐标写入字符单元，超出范围则忽略。
  pub fn set(&mut self, x: u16, y: u16, cell: CanvasCell) {
    let Some(index) = self.index(x, y) else {
      return;
    };
    if let Some(target) = self.cells.get_mut(index) {
      *target = cell;
      self.written[index] = true;
    }
  }
  pub fn get(&self, x: u16, y: u16) -> Option<&CanvasCell> {
    let index = self.index(x, y)?;
    self.cells.get(index)
  }

  /// 检查指定坐标是否已被写入过。
  pub fn is_written(&self, x: u16, y: u16) -> bool {
    self
      .index(x, y)
      .and_then(|index| self.written.get(index))
      .copied()
      .unwrap_or(false)
  }

  /// 获取指定行的纯文本内容。
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
