use super::buffer::CanvasBuffer;

/// 宿主最高优先级绘制层。只用于全局短提示等必须压过所有 UI/Overlay 的内容。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TopLayer {
  buffer: CanvasBuffer,
}

impl TopLayer {
  pub fn new(width: u16, height: u16) -> Self {
    Self {
      buffer: CanvasBuffer::new(width, height),
    }
  }

  pub fn resize_or_clear(&mut self, width: u16, height: u16) -> bool {
    if self.buffer.width() == width && self.buffer.height() == height {
      self.buffer.clear();
      false
    } else {
      self.buffer.resize(width, height);
      true
    }
  }

  pub fn buffer(&self) -> &CanvasBuffer {
    &self.buffer
  }

  pub fn buffer_mut(&mut self) -> &mut CanvasBuffer {
    &mut self.buffer
  }
}
