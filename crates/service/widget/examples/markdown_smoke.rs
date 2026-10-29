//! Exercise Markdown layout, embedded clipping, links and rich text without the host.
//!
//! Run `cargo run -p tg-service-widget --example markdown_smoke --locked`.
//! No terminal or external applications are required; pointer events are injected.
use tg_core_input::{MouseButton, MouseEvent, MouseEventKind};
use tg_service_canvas::CanvasService;
use tg_service_code_highlight::CodeHighlightService;
use tg_service_layout::{LayoutService, Size};
use tg_service_widget::{
  MarkdownEvent, MarkdownService, MarkdownViewOptions, ScrollBoxWidget, ScrollBoxWidgetOptions,
  TextInputService, UiEvent, UiObjectPool,
};
fn main() {
  let service = MarkdownService::new();
  let code = CodeHighlightService::new();
  let mut pool = UiObjectPool::new();
  let options = MarkdownViewOptions::new(
    "[Open](https://example.com)\n\n- a long list that wraps\n\n```rust\n    println!(\"hello\");\n```",
  );
  let id = service.create(&mut pool, options.clone()).unwrap();
  let layout = service.layout(&options, 11, false, &code);
  assert!(layout.size().height > 5);
  assert!(service.layout(&options, 11, true, &code).size().width > 11);
  let mut scroll = ScrollBoxWidget::new(
    &mut pool,
    ScrollBoxWidgetOptions {
      width: 12,
      height: 5,
      horizontal: false,
      ..Default::default()
    },
  )
  .unwrap();
  scroll.set_extent(layout.size());
  let mut geometry = LayoutService::new();
  geometry.resize_physical(20, 10);
  let mut canvas = CanvasService::new();
  canvas.begin_frame(&geometry);
  pool.begin_render();
  pool.prepare_canvas(&mut canvas, &geometry);
  let view = scroll
    .begin_draw(&mut pool, &mut canvas, None, (1, 1))
    .unwrap();
  let surface = pool.allocate_embedded_slice();
  canvas.prepare_embedded_slice(surface, view.clip, view.origin, view.rank, None, false);
  assert!(service.render_layout_on(&mut pool, id, surface, &layout, &mut canvas));
  canvas.composite_embedded_slice(surface, None);
  pool.end_pointer_scope();
  assert_eq!(canvas.cell_at(1, 1).unwrap().text, "O");
  let input = TextInputService::new();
  let event = |kind| MouseEvent {
    kind,
    x: 2,
    y: 1,
    button: Some(MouseButton::Left),
    scroll: None,
  };
  assert!(service.route_mouse_event(&mut pool, &input, event(MouseEventKind::Press)));
  assert!(service.route_mouse_event(&mut pool, &input, event(MouseEventKind::Release)));
  assert!(
    matches!(pool.pop_event(),Some(UiEvent::Markdown(MarkdownEvent::LinkClicked{href,..})) if href=="https://example.com")
  );
  scroll.scroll_to(0, 65535);
  scroll.set_extent(Size {
    width: 1,
    height: 1,
  });
  assert_eq!(scroll.offset(), (0, 0));
  assert!(service.remove(&mut pool, id));
  assert!(!service.exists(&pool, id));
  let rich = service.markdown_to_rich_text("**bold**");
  canvas.text(&tg_service_text_layout::DrawTextParams::new(0, 8, rich));
  assert!(canvas.cell_at(0, 8).unwrap().style.bold);
}
