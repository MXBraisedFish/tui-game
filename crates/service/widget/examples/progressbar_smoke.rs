//! Exercise signed progress bars and style updates without the host.
//!
//! Run `cargo run -p tg-service-widget --example progressbar_smoke --locked`.
//! No terminal or external devices are required.
use tg_service_canvas::CanvasService;
use tg_service_layout::{LayoutService, Rect};
use tg_service_widget::{
  ProgressBarFillOrigin, ProgressBarOptions, ProgressBarService, SliceLength, SliceOptions,
  SliceRect, SliceService, UiObjectPool,
};
fn main() {
  let service = ProgressBarService::new();
  let mut pool = UiObjectPool::new();
  let mut options = ProgressBarOptions::default();
  options.completed.ch = 'C';
  options.preview.ch = 'P';
  options.remaining.ch = 'R';
  let id = service.create(&mut pool, options.clone()).unwrap();
  assert!(service.set_progress(&mut pool, id, 0.3, 0.6));
  let slice = SliceService::new()
    .create(
      &mut pool,
      SliceOptions {
        rect: SliceRect {
          x: 1,
          y: 0,
          width: SliceLength::Fixed(4),
          height: SliceLength::Fixed(1),
        },
        ..Default::default()
      },
    )
    .unwrap();
  let mut layout = LayoutService::new();
  layout.resize_physical(12, 6);
  layout.set_developer_viewport(Rect {
    x: 0,
    y: 2,
    width: 12,
    height: 4,
  });
  let mut canvas = CanvasService::new();
  canvas.begin_frame(&layout);
  pool.prepare_canvas(&mut canvas, &layout);
  assert!(service.render_at(&pool, id, None, (-2, 0), 10, &mut canvas));
  let text = (0..8)
    .map(|x| canvas.cell_at(x, 0).unwrap().text.as_str())
    .collect::<String>();
  assert_eq!(text, "CPPPRRRR");
  assert!(service.render_at(&pool, id, Some(slice), (-2, 0), 10, &mut canvas));
  let hit = canvas
    .slice_hit_rect(
      slice,
      Rect {
        x: 0,
        y: 0,
        width: 4,
        height: 1,
      },
    )
    .unwrap();
  assert_eq!(hit.0.y, 2);
  assert!(!service.render_at(&pool, id, None, (i32::MIN, 0), 10, &mut canvas));
  assert!(!service.render_at(&pool, id, None, (i32::MAX, 0), 10, &mut canvas));
  let mut invalid = options.clone();
  invalid.remaining.ch = '界';
  assert!(!service.set_options(&mut pool, id, invalid));
  options.origin = ProgressBarFillOrigin::Right;
  assert!(service.set_options(&mut pool, id, options));
  assert!(service.render_at(&pool, id, None, (0, 1), 10, &mut canvas));
  assert_eq!(canvas.cell_at(0, 1).unwrap().text, "R");
  assert_eq!(canvas.cell_at(9, 1).unwrap().text, "C");
  assert!(service.render_at(&pool, id, None, (-65530, 2), 65535, &mut canvas));
  assert_eq!(canvas.cell_at(4, 2).unwrap().text, "C");
  assert!(service.remove(&mut pool, id));
  assert!(!service.render_at(&pool, id, None, (0, 0), 10, &mut canvas));
}
