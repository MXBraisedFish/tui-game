//! Reusable storage for content that appears above composed application surfaces.

use super::buffer::CanvasBuffer;

/// A reusable terminal-cell buffer drawn above the other canvas surfaces.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TopLayer {
  buffer: CanvasBuffer,
}

impl TopLayer {
  /// Create a top layer initialized from `width`, `height`.
  pub fn new(width: u16, height: u16) -> Self {
    Self {
      buffer: CanvasBuffer::new(width, height),
    }
  }

  /// Resize the top-layer buffer when necessary, otherwise clear its previous-frame writes.
  pub fn resize_or_clear(&mut self, width: u16, height: u16) -> bool {
    if self.buffer.width() == width && self.buffer.height() == height {
      self.buffer.clear();
      false
    } else {
      self.buffer.resize(width, height);
      true
    }
  }

  /// Return the current buffer.
  pub fn buffer(&self) -> &CanvasBuffer {
    &self.buffer
  }

  /// Return mutable access to the owned buffer.
  pub fn buffer_mut(&mut self) -> &mut CanvasBuffer {
    &mut self.buffer
  }
}
