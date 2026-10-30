//! Independent render pipeline smoke entry exercising the public API and checking its results.

use tg_core_style::{ComposedCell, TextStyle};
use tg_service_canvas::CanvasService;
use tg_service_render_pipeline::FrameCompositor;

fn main() {
  let mut canvas = CanvasService::new();
  canvas.styled_text(1, 2, "a", TextStyle::default());

  let frame = FrameCompositor::new().compose(&canvas);

  assert!(matches!(
    frame.get(1, 2),
    Some(ComposedCell::Text(cell)) if cell.text == "a"
  ));
  let compositor = FrameCompositor::new();
  let mut processed = compositor.compose_canvas(&canvas);
  processed.set(1, 2, tg_core_style::CanvasCell::new("b"));
  assert!(canvas.set_composed_canvas(processed));
  let frame = compositor.compose(&canvas);
  assert!(matches!(frame.get(1, 2), Some(ComposedCell::Text(cell)) if cell.text == "b"));
  assert_eq!(canvas.cell_at(1, 2).unwrap().text, "a");
  println!("render_pipeline ok: composed cell (1, 2) = \"a\"");
}
