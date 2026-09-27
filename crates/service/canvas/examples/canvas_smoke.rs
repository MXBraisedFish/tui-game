//! Minimal entry: prepares one slice, draws clipped text into it and reads the base layer.

use tg_core_geometry::Rect;
use tg_service_canvas::{CanvasService, SliceFrame, SliceId, SurfaceFrame};
use tg_service_layout::LayoutService;
use tg_service_text_layout::DrawTextParams;

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
      rect: Rect { x: 2, y: 1, width: 4, height: 2 },
      visible: true,
      opaque: false,
      background: None,
    })],
    &layout,
  );
  let text = DrawTextParams { text: "hello".to_string(), ..Default::default() };
  assert!(canvas.text_at_on(slice, 0, 0, &text));
  canvas.text_at(0, 0, &text);
  assert_eq!(canvas.cell_at(1, 0).map(|cell| cell.text.as_str()), Some("e"));
  assert_eq!(canvas.prepared_slice_width(slice), Some(4));
  println!("canvas ok: slice {:?}", canvas.prepared_slice_rect(slice));
}
