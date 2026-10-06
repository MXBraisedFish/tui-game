//! Independent canvas smoke entry exercising the public API and checking its results.

use tg_core_geometry::Rect;
use tg_service_canvas::{CanvasService, SliceFrame, SliceId, SurfaceFrame};
use tg_service_layout::LayoutService;
use tg_service_text_layout::{DrawTextParams, TextAlign, measure_draw_text};

fn main() {
  let mut layout = LayoutService::new();
  layout.resize_physical(20, 5);
  let mut canvas = CanvasService::new();
  canvas.begin_frame(&layout);
  let slice = SliceId(1);
  canvas.prepare(
    1,
    vec![SurfaceFrame::Slice(SliceFrame {
      id: slice,
      rect: Rect {
        x: 2,
        y: 1,
        width: 4,
        height: 2,
      },
      source_x: 0,
      source_y: 0,
      visible: true,
      opaque: false,
      background: None,
    })],
    &layout,
  );
  let text = DrawTextParams {
    text: "hello".to_string(),
    ..Default::default()
  };
  assert!(canvas.text_at_on(slice, 0, 0, &text));
  canvas.text_at(0, 0, &text);
  assert_eq!(
    canvas.cell_at(1, 0).map(|cell| cell.text.as_str()),
    Some("e")
  );
  assert_eq!(canvas.prepared_slice_width(slice), Some(4));
  canvas.begin_frame(&layout);
  let text = DrawTextParams {
    text: "a\nabcdef".to_string(),
    line_align: TextAlign::Center,
    max_width: Some(20),
    ..Default::default()
  };
  let (width, height) = measure_draw_text(&text);
  assert_eq!((width, height), (6, 2));
  let x = layout.resolve_x(LayoutService::ALIGN_CENTER, width, 0);
  let y = layout.resolve_y(LayoutService::ALIGN_MIDDLE, height, 0);
  canvas.text_at(i32::from(x), i32::from(y), &text);
  assert_eq!(canvas.cell_at(x + 2, y).unwrap().text, "a");
  assert_eq!(canvas.cell_at(x, y + 1).unwrap().text, "a");
  assert_eq!(canvas.cell_at(x + width - 1, y + 1).unwrap().text, "f");
  println!("canvas ok: slice {:?}", canvas.prepared_slice_rect(slice));
}
