//! Minimal entry: clips a developer viewport to the physical size and centers content in it.

use tg_core_geometry::Rect;
use tg_service_layout::LayoutService;

fn main() {
  let mut layout = LayoutService::new();
  layout.resize_physical(100, 40);
  layout.set_developer_viewport(Rect { x: 10, y: 5, width: 200, height: 20 });
  assert_eq!(layout.developer_width(), 90, "viewport is clipped to the physical width");
  let x = layout.resolve_x(LayoutService::ALIGN_CENTER, 10, 0);
  assert_eq!(x, 40);
  println!("layout ok: viewport {:?}, centered x {x}", layout.developer_viewport_rect());
}
