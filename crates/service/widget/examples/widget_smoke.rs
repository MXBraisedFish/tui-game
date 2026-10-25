//! Independent widget smoke entry exercising the public API and checking its results.

use tg_service_widget::{HitAreaOptions, HitAreaService, SliceOptions, SliceService, UiObjectPool};

fn main() {
  let slices = SliceService::new();
  let mut pool = UiObjectPool::new();

  let slice = slices
    .create(&mut pool, SliceOptions::default())
    .expect("create slice");
  assert!(slices.exists(&pool, slice));
  assert!(slices.set_position(&mut pool, slice, 3, 2));
  let rect = slices.configured_rect(&pool, slice).expect("slice rect");
  assert_eq!((rect.x, rect.y), (3, 2));

  assert!(slices.remove(&mut pool, slice));
  assert!(!slices.exists(&pool, slice));
  let hitboxes = HitAreaService::new();
  let id = hitboxes.create(&mut pool, HitAreaOptions::default());
  assert!(hitboxes.set_options(
    &mut pool,
    id,
    HitAreaOptions {
      hover_move: true,
      drag: true
    }
  ));
  let mut canvas = tg_service_canvas::CanvasService::new();
  assert!(hitboxes.render_at(
    &mut pool,
    id,
    (-1, 0),
    tg_service_layout::Size {
      width: 3,
      height: 2
    },
    &canvas
  ));
  let mut input = tg_service_widget::TextInputService::new();
  assert!(hitboxes.route_mouse_event(
    &mut pool,
    &mut input,
    &canvas,
    tg_core_input::MouseEvent {
      kind: tg_core_input::MouseEventKind::Press,
      button: Some(tg_core_input::MouseButton::Left),
      scroll: None,
      x: 0,
      y: 0,
    }
  ));
  assert!(hitboxes.is_pressed(&pool, id, tg_core_input::MouseButton::Left));
  assert!(hitboxes.remove(&mut pool, id));
  assert!(!hitboxes.is_pressed(&pool, id, tg_core_input::MouseButton::Left));
  let overlay = slices.create(&mut pool, SliceOptions::default()).unwrap();
  let lower = hitboxes.create(&mut pool, HitAreaOptions::default());
  let layout = tg_service_layout::LayoutService::new();
  for passthrough in [false, true] {
    assert!(slices.set_event_passthrough(&mut pool, overlay, passthrough));
    assert_eq!(slices.event_passthrough(&pool, overlay), Some(passthrough));
    pool.begin_render();
    pool.prepare_canvas(&mut canvas, &layout);
    hitboxes.render_at(
      &mut pool,
      lower,
      (0, 0),
      tg_service_layout::Size {
        width: 3,
        height: 2,
      },
      &canvas,
    );
    hitboxes.route_mouse_event(
      &mut pool,
      &mut input,
      &canvas,
      tg_core_input::MouseEvent {
        kind: tg_core_input::MouseEventKind::Press,
        button: Some(tg_core_input::MouseButton::Left),
        scroll: None,
        x: 0,
        y: 0,
      },
    );
    assert_eq!(
      hitboxes.is_pressed(&pool, lower, tg_core_input::MouseButton::Left),
      passthrough
    );
  }
  assert!(slices.remove(&mut pool, overlay));
  assert!(hitboxes.remove(&mut pool, lower));

  let edit = input.create(
    &mut pool,
    tg_service_widget::TextInputOptions {
      initial_text: "ab".into(),
      mode: tg_service_widget::TextInputMode::MultiLine,
      ..Default::default()
    },
  );
  assert!(input.focus(&mut pool, edit));
  assert!(input.configure(
    &mut pool,
    edit,
    tg_service_widget::TextInputOptions {
      initial_text: "ab".into(),
      mode: tg_service_widget::TextInputMode::MultiLine,
      max_chars: Some(4),
      ..Default::default()
    },
    false
  ));
  input.insert_text(&mut pool, "\nc");
  assert_eq!(input.get_text(&pool, edit), Some("ab\nc"));
  pool.prepare_canvas(&mut canvas, &layout);
  let params = tg_service_widget::TextInputRenderParams {
    rect: tg_service_layout::Rect {
      x: 0,
      y: 0,
      width: 4,
      height: 2,
    },
    cursor_character: Some("|".into()),
    cursor_blink: false,
    ..Default::default()
  };
  input.render_at(&mut pool, edit, None, (-1, 0), &params, &mut canvas);
  assert_eq!(canvas.base_buffer().get(0, 0).unwrap().text, "b");
  assert!(input.blur(&mut pool));
  assert!(input.remove(&mut pool, edit));
  println!("widget ok: slice lifecycle, signed hit region, event options, press and cleanup");
}
