use super::CanvasCell;

// 画布缓冲区
#[derive(Clone, Debug, PartialEq)]
pub struct CanvasBuffer {
  width: u16,             // 宽
  height: u16,            // 高
  cells: Vec<CanvasCell>, // 网格
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

  // 获取宽
  pub fn width(&self) -> u16 {
    self.width
  }

  // 获取高
  pub fn height(&self) -> u16 {
    self.height
  }

  // 尺寸变化
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

  // 读取单元格
  pub fn get(&self, x: u16, y: u16) -> Option<&CanvasCell> {
    let index = self.index(x, y)?;
    self.cells.get(index)
  }

  // 写入网格
  pub fn set(&mut self, x: u16, y: u16, cell: CanvasCell) {
    if let Some(index) = self.index(x, y) {
      if let Some(target) = self.cells.get_mut(index) {
        *target = cell;
      }
    }
  }

  // 二维转一维索引
  fn index(&self, x: u16, y: u16) -> Option<usize> {
    if x >= self.width || y >= self.height {
      return None;
    }
    Some(y as usize * self.width as usize + x as usize)
  }

  // 临时行转字符串
  pub fn line_as_string(&self, y: u16) -> String {
    // 边界检查
    if y >= self.height {
      return String::new();
    }

    // 创建空字符串
    let mut line = String::new();

    // 遍历改行的每一列
    for x in 0..self.width {
      if let Some(cell) = self.get(x, y) {
        if !cell.is_wide_continuation() {
          line.push(cell.display_char());
        }
      }
    }

    line
  }
}
