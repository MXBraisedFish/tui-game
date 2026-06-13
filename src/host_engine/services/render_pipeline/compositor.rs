use super::{ComposedCell, ComposedFrame};
use crate::host_engine::services::{CanvasService, ImageLayerFrame};

pub struct FrameCompositor;

impl FrameCompositor {
  pub fn new() -> Self {
    Self
  }

  pub fn compose(&self, text: &CanvasService, image_layer: &ImageLayerFrame) -> ComposedFrame {
    let mut frame = ComposedFrame::new(text.width(), text.height());

    for y in 0..text.height() {
      for x in 0..text.width() {
        if let Some(cell) = text.cell_at(x, y) {
          frame.set(x, y, ComposedCell::Text(cell.clone()));
        }
      }
    }

    frame.set_removed_regions(image_layer.removed_regions.clone());
    frame.set_image_dirty(image_layer.dirty);

    for image in &image_layer.images {
      frame.add_image(image.clone());
      for y in image.rect.y..image.rect.y.saturating_add(image.rect.height) {
        for x in image.rect.x..image.rect.x.saturating_add(image.rect.width) {
          if x == image.rect.x && y == image.rect.y {
            frame.set(x, y, ComposedCell::ImageAnchor(image.id));
          } else {
            frame.set(x, y, ComposedCell::ImageBody(image.id));
          }
        }
      }
    }

    frame
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::host_engine::services::{
    CellPixelSize, DrawTextParams, ImageCellRect, ImageProtocol, ImageSignature,
  };
  use crate::host_engine::services::{LayerImage, TextStyle};
  use std::path::PathBuf;

  #[test]
  fn compose_copies_text_without_images() {
    let mut canvas = CanvasService::new();
    canvas.styled_text(1, 2, "a", TextStyle::default());
    let layer = ImageLayerFrame {
      images: Vec::new(),
      dirty: false,
      removed_regions: Vec::new(),
    };

    let frame = FrameCompositor::new().compose(&canvas, &layer);

    assert!(matches!(
      frame.get(1, 2),
      Some(ComposedCell::Text(cell)) if cell.ch == 'a'
    ));
  }

  #[test]
  fn compose_image_overrides_text_rect() {
    let mut canvas = CanvasService::new();
    canvas.text(&DrawTextParams::new(0, 0, "abcdef"));
    let rect = ImageCellRect {
      x: 1,
      y: 0,
      width: 3,
      height: 2,
    };
    let sig = ImageSignature {
      protocol: ImageProtocol::Kitty,
      path: PathBuf::from("image.png"),
      rect,
      cell: CellPixelSize {
        width: 8,
        height: 16,
      },
      preserve_aspect_ratio: false,
    };
    let layer = ImageLayerFrame {
      images: vec![LayerImage {
        id: 1,
        protocol: ImageProtocol::Kitty,
        rect,
        signature: sig,
        sequence: "seq".to_string(),
      }],
      dirty: true,
      removed_regions: Vec::new(),
    };

    let frame = FrameCompositor::new().compose(&canvas, &layer);

    assert_eq!(frame.get(1, 0), Some(&ComposedCell::ImageAnchor(1)));
    assert_eq!(frame.get(2, 0), Some(&ComposedCell::ImageBody(1)));
    assert_eq!(frame.get(1, 1), Some(&ComposedCell::ImageBody(1)));
  }
}
