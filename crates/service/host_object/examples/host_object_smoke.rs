//! Minimal entry: registers the top bar area and queries its size.

use tg_core_geometry::Rect;
use tg_service_host_object::{HostAreaKind, HostObjectPool};

fn main() {
  let mut pool = HostObjectPool::new();
  let top = pool.ensure_area(HostAreaKind::TopBar);
  let rect = Rect { x: 0, y: 0, width: 80, height: 1 };
  assert!(pool.update_area(top, rect, true));
  assert_eq!(pool.area_width(HostAreaKind::TopBar), Some(80));
  assert!(pool.is_visible(HostAreaKind::TopBar));
  println!("host_object ok: {:?}", pool.area_rect(HostAreaKind::TopBar));
}
