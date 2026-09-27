//! Minimal entry: creates a slice in a UI object pool, moves it and removes it.

use tg_service_widget::{SliceOptions, SliceService, UiObjectPool};

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
  println!("widget ok: slice created, moved and removed");
}
