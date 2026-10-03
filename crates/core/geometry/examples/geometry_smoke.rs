//! Independent geometry smoke entry exercising the public API and checking its results.

use tg_core_geometry::{Rect, Size};

fn main() {
  let area = Rect {
    x: 1,
    y: 1,
    width: 3,
    height: 2,
  };
  assert!(area.contains(3, 2));
  assert!(!area.contains(4, 2));
  let size = Size {
    width: area.width,
    height: area.height,
  };
  println!(
    "geometry ok: {}x{} rect at ({},{})",
    size.width, size.height, area.x, area.y
  );
}
