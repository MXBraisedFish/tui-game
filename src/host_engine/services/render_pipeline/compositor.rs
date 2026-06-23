use super::{ComposedCell, ComposedFrame};
use crate::host_engine::services::CanvasService;

pub struct FrameCompositor;

impl FrameCompositor {
  pub fn new() -> Self {
    Self
  }

  pub fn compose(&self, text: &CanvasService) -> ComposedFrame {
    let mut frame = ComposedFrame::new(text.width(), text.height());

    for y in 0..text.height() {
      for x in 0..text.width() {
        if let Some(cell) = text.cell_at(x, y) {
          frame.set(x, y, ComposedCell::Text(cell.clone()));
        }
      }
    }

    frame
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::host_engine::services::TextStyle;

  #[test]
  fn compose_copies_text() {
    let mut canvas = CanvasService::new();
    canvas.styled_text(1, 2, "a", TextStyle::default());

    let frame = FrameCompositor::new().compose(&canvas);

    assert!(matches!(
      frame.get(1, 2),
      Some(ComposedCell::Text(cell)) if cell.text == "a"
    ));
  }
}
