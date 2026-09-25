//! Minimal entry: hit-tests one point against a rectangle.

use tg_core_geometry::{Rect, Size};

fn main() {
  let area = Rect { x: 1, y: 1, width: 3, height: 2 };
  assert!(area.contains(3, 2));
  assert!(!area.contains(4, 2));
  let size = Size { width: area.width, height: area.height };
  println!("geometry ok: {}x{} rect at ({},{})", size.width, size.height, area.x, area.y);
}
