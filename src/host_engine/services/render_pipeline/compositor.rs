use super::{ComposedCell, ComposedFrame};
use crate::host_engine::services::canvas::buffer::CanvasBuffer;
use crate::host_engine::services::{CanvasCell, CanvasService, TextColor};

pub struct FrameCompositor;

impl FrameCompositor {
  pub fn new() -> Self {
    Self
  }

  pub fn compose(&self, canvas: &CanvasService) -> ComposedFrame {
    let host = canvas.host_buffer();
    let mut frame = ComposedFrame::new(host.width(), host.height());
    for y in 0..host.height() {
      for x in 0..host.width() {
        frame.set(x, y, ComposedCell::Text(CanvasCell::blank()));
      }
    }

    let viewport = canvas.viewport();
    overlay(
      &mut frame,
      canvas.base_buffer(),
      viewport.x,
      viewport.y,
      true,
    );
    for (_, slice) in canvas.prepared_slices().filter(|(_, slice)| slice.visible) {
      overlay(
        &mut frame,
        &slice.buffer,
        viewport.x.saturating_add(slice.rect.x),
        viewport.y.saturating_add(slice.rect.y),
        slice.opaque,
      );
    }
    overlay(&mut frame, host, 0, 0, false);

    frame
  }
}

fn overlay(frame: &mut ComposedFrame, buffer: &CanvasBuffer, ox: u16, oy: u16, opaque: bool) {
  for y in 0..buffer.height() {
    for x in 0..buffer.width() {
      if !opaque && !buffer.is_written(x, y) {
        continue;
      }
      let Some(source) = buffer.get(x, y) else {
        continue;
      };
      let px = ox.saturating_add(x);
      let py = oy.saturating_add(y);
      let Some(lower) = frame.get(px, py) else {
        continue;
      };
      let mut cell = source.clone();
      if cell.style.background == Some(TextColor::Transparent) {
        cell.style.background = match lower {
          ComposedCell::Text(lower) => lower.style.background.clone(),
          ComposedCell::Empty => None,
        };
      }
      frame.set(px, py, ComposedCell::Text(cell));
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::host_engine::services::{
    LayoutService, SliceLength, SliceOptions, SliceRect, SliceService, TerminalColor, TextColor,
    TextStyle, UiObjectPool,
  };

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

  fn text(frame: &ComposedFrame, x: u16, y: u16) -> &str {
    match frame.get(x, y).unwrap() {
      ComposedCell::Text(cell) => &cell.text,
      ComposedCell::Empty => "",
    }
  }

  #[test]
  fn viewport_slices_and_host_compose_in_order() {
    let mut layout = LayoutService::new();
    layout.resize_physical(12, 6);
    layout.set_developer_viewport(crate::host_engine::services::Rect {
      x: 2,
      y: 1,
      width: 8,
      height: 4,
    });
    let slices = SliceService::new();
    let mut pool = UiObjectPool::new();
    let opaque = slices
      .create(
        &mut pool,
        SliceOptions {
          rect: SliceRect {
            x: 2,
            y: 1,
            width: SliceLength::Fixed(3),
            height: SliceLength::Fixed(2),
          },
          ..Default::default()
        },
      )
      .unwrap();
    let transparent = slices
      .create(
        &mut pool,
        SliceOptions {
          rect: SliceRect {
            x: 2,
            y: 1,
            width: SliceLength::Fixed(3),
            height: SliceLength::Fixed(2),
          },
          opaque: false,
          ..Default::default()
        },
      )
      .unwrap();
    let mut canvas = CanvasService::new();
    canvas.begin_frame(&layout);
    canvas.prepare(&pool, &layout);
    canvas.styled_text(2, 1, "B", TextStyle::default());
    canvas.styled_text_on(transparent, 1, 0, "T", TextStyle::default());
    canvas.host_styled_text(5, 2, "H", TextStyle::default());

    let frame = FrameCompositor::new().compose(&canvas);
    assert_eq!(text(&frame, 4, 2), " ");
    assert_eq!(text(&frame, 5, 2), "H");
    assert_eq!(text(&frame, 6, 2), " ");
    assert!(canvas.slice_rect(opaque).is_some());
  }

  #[test]
  fn later_slice_wins_and_wide_grapheme_is_not_split() {
    let mut layout = LayoutService::new();
    layout.resize_physical(5, 2);
    let service = SliceService::new();
    let mut pool = UiObjectPool::new();
    let options = SliceOptions {
      rect: SliceRect {
        x: 0,
        y: 0,
        width: SliceLength::Fixed(1),
        height: SliceLength::Fixed(1),
      },
      opaque: false,
      ..Default::default()
    };
    let a = service.create(&mut pool, options).unwrap();
    let b = service.create(&mut pool, options).unwrap();
    let mut canvas = CanvasService::new();
    canvas.begin_frame(&layout);
    canvas.prepare(&pool, &layout);
    canvas.styled_text_on(a, 0, 0, "A", TextStyle::default());
    canvas.styled_text_on(b, 0, 0, "B", TextStyle::default());
    canvas.styled_text_on(b, 0, 0, "我", TextStyle::default());
    let frame = FrameCompositor::new().compose(&canvas);
    assert_eq!(text(&frame, 0, 0), "B");
  }

  #[test]
  fn transparent_slice_explicit_space_inherits_lower_background() {
    let mut layout = LayoutService::new();
    layout.resize_physical(3, 1);
    let slices = SliceService::new();
    let mut pool = UiObjectPool::new();
    let slice = slices
      .create(
        &mut pool,
        SliceOptions {
          opaque: false,
          ..Default::default()
        },
      )
      .unwrap();
    let mut canvas = CanvasService::new();
    canvas.begin_frame(&layout);
    canvas.prepare(&pool, &layout);
    let background = TextColor::Terminal(TerminalColor::Blue);
    canvas.styled_text(
      0,
      0,
      "B",
      TextStyle {
        background: Some(background.clone()),
        ..Default::default()
      },
    );
    canvas.styled_text_on(
      slice,
      0,
      0,
      " ",
      TextStyle {
        background: Some(TextColor::Transparent),
        ..Default::default()
      },
    );

    let frame = FrameCompositor::new().compose(&canvas);
    let ComposedCell::Text(cell) = frame.get(0, 0).unwrap() else {
      panic!("expected text cell")
    };
    assert_eq!(cell.text, " ");
    assert_eq!(cell.style.background, Some(background));
  }

  #[test]
  fn slice_state_changes_apply_on_next_prepare() {
    let mut layout = LayoutService::new();
    layout.resize_physical(3, 1);
    let slices = SliceService::new();
    let mut pool = UiObjectPool::new();
    let slice = slices.create(&mut pool, SliceOptions::default()).unwrap();
    let mut canvas = CanvasService::new();
    canvas.begin_frame(&layout);
    canvas.prepare(&pool, &layout);
    slices.set_visible(&mut pool, slice, false);
    assert!(canvas.styled_text_on(slice, 0, 0, "A", TextStyle::default()));
    assert_eq!(text(&FrameCompositor::new().compose(&canvas), 0, 0), "A");

    canvas.begin_frame(&layout);
    canvas.prepare(&pool, &layout);
    assert!(!canvas.styled_text_on(slice, 0, 0, "B", TextStyle::default()));
    assert_eq!(text(&FrameCompositor::new().compose(&canvas), 0, 0), " ");
  }
}
