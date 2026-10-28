//! Exercise embedded scrollbox geometry, clipping and wheel routing without the host.
//!
//! Run `cargo run -p tg-service-widget --example scrollbox_smoke --locked`.
//! No terminal or external devices are required; pointer events are injected.

use tg_core_input::{MouseEvent, MouseEventKind, ScrollDirection};
use tg_service_canvas::CanvasService;
use tg_service_layout::{LayoutService, Rect, Size};
use tg_service_widget::{ScrollBoxWidget, ScrollBoxWidgetOptions, UiObjectPool};

pub(crate) fn main() {
  let mut pool = UiObjectPool::new();
  let mut widget = ScrollBoxWidget::new(
    &mut pool,
    ScrollBoxWidgetOptions {
      width: 6,
      height: 4,
      ..Default::default()
    },
  )
  .unwrap();
  assert_eq!(
    widget.layout().content,
    Rect {
      x: 0,
      y: 0,
      width: 6,
      height: 4
    }
  );
  widget.scroll_to_axes(Some(65535), Some(65535));
  widget.set_extent(Size {
    width: 20,
    height: 10,
  });
  assert_eq!(widget.offset(), (15, 7));
  assert_eq!(
    widget.layout().content,
    Rect {
      x: 0,
      y: 0,
      width: 5,
      height: 3
    }
  );
  let mut options = widget.options().clone();
  options.include_scrollbar = false;
  options.scrollbar_swap_vertical = true;
  options.scrollbar_swap_horizontal = true;
  assert!(widget.set_options(options));
  assert_eq!(
    widget.layout().outer,
    Size {
      width: 7,
      height: 5
    }
  );
  assert_eq!(
    widget.layout().content,
    Rect {
      x: 1,
      y: 1,
      width: 6,
      height: 4
    }
  );
  widget.scroll_to(0, 0);
  let mut layout = LayoutService::new();
  layout.resize_physical(12, 8);
  layout.set_developer_viewport(Rect {
    x: 0,
    y: 2,
    width: 12,
    height: 6,
  });
  let mut canvas = CanvasService::new();
  canvas.begin_frame(&layout);
  pool.begin_render();
  pool.prepare_canvas(&mut canvas, &layout);
  let view = widget
    .begin_draw(&mut pool, &mut canvas, None, (-2, -1))
    .unwrap();
  assert_eq!(
    view.clip,
    Rect {
      x: 0,
      y: 2,
      width: 5,
      height: 4
    }
  );
  let surface = pool.allocate_embedded_slice();
  canvas.prepare_embedded_slice(surface, view.clip, view.origin, view.rank, None, false);
  assert!(canvas.text_at_on(
    surface,
    1,
    0,
    &tg_service_text_layout::DrawTextParams::new(0, 0, "OK")
  ));
  canvas.composite_embedded_slice(surface, None);
  pool.end_pointer_scope();
  assert_eq!(canvas.cell_at(0, 0).unwrap().text, "O");
  let wheel = MouseEvent {
    kind: MouseEventKind::Scroll,
    button: None,
    scroll: Some(ScrollDirection::Down),
    x: 1,
    y: 3,
  };
  assert!(widget.route_mouse(&pool, wheel));
  assert_eq!(widget.offset().1, 2);
  assert_eq!(widget.take_scroll_event(), Some((0, 2)));
  widget.set_extent(Size {
    width: 1,
    height: 1,
  });
  assert_eq!(widget.offset(), (0, 0));
  assert_eq!(
    widget.layout().outer,
    Size {
      width: 6,
      height: 4
    }
  );
  assert!(widget.layout().vertical.is_none() && widget.layout().horizontal.is_none());
  let mut options = widget.options().clone();
  options.width = 0;
  assert!(!widget.set_options(options));
  let mut options = widget.options().clone();
  options.style.track_char = '界';
  assert!(!widget.set_options(options));
  widget.deactivate();
  assert!(!widget.route_mouse(&pool, wheel));
}
