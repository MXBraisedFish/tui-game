use super::cell::CanvasCell;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanvasBuffer {
  width: u16,
  height: u16,
  cells: Vec<CanvasCell>,
}

impl CanvasBuffer {
  pub fn new(width: u16, height: u16) -> Self {
    let size = width as usize * height as usize;
    Self {
      width,
      height,
      cells: vec![CanvasCell::blank(); size],
    }
  }

  // 获得宽
  pub fn width(&self) -> u16 {
    self.width
  }

  // 获得高
  pub fn height(&self) -> u16 {
    self.height
  }

  // 调整画布大小
  pub fn resize(&mut self, width: u16, height: u16) {
    self.width = width;
    self.height = height;
    let size = width as usize * height as usize;
    self.cells = vec![CanvasCell::blank(); size];
  }

  // 清理画布
  pub fn clear(&mut self) {
    for cell in &mut self.cells {
      *cell = CanvasCell::blank();
    }
  }

  // 设置单元格内容
  pub fn set(&mut self, x: u16, y: u16, cell: CanvasCell) {
    let Some(index) = self.index(x, y) else {
      return;
    };
    if let Some(target) = self.cells.get_mut(index) {
      *target = cell;
    }
  }

  // 获取单元格内容
  pub fn get(&self, x: u16, y: u16) -> Option<&CanvasCell> {
    let index = self.index(x, y)?;
    self.cells.get(index)
  }

  // 获取整行文本
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

  // 计算单元格索引
  fn index(&self, x: u16, y: u16) -> Option<usize> {
    if x >= self.width || y >= self.height {
      return None;
    }
    Some(y as usize * self.width as usize + x as usize)
  }
}
