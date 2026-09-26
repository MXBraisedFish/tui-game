//! Basic terminal geometry: [`Size`], [`Position`] and [`Rect`].

/// Size of an area (width x height).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Size {
  pub width: u16,
  pub height: u16,
}

/// Position in two-dimensional coordinates.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Position {
  pub x: u16,
  pub y: u16,
}

/// Rectangular area.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rect {
  pub x: u16,
  pub y: u16,
  pub width: u16,
  pub height: u16,
}

impl Rect {
  /// Returns whether the point (`px`, `py`) lies inside the rectangle; the right and bottom
  /// edges are excluded.
  pub fn contains(&self, px: u16, py: u16) -> bool {
    px >= self.x
      && px < self.x.saturating_add(self.width)
      && py >= self.y
      && py < self.y.saturating_add(self.height)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn contains_is_half_open_and_saturates_at_the_edge() {
    let rect = Rect { x: 2, y: 3, width: 4, height: 2 };
    assert!(rect.contains(2, 3));
    assert!(rect.contains(5, 4));
    assert!(!rect.contains(6, 4));
    assert!(!rect.contains(5, 5));
    assert!(!rect.contains(1, 3));
    let edge = Rect { x: u16::MAX - 1, y: 0, width: 10, height: 1 };
    // The right edge saturates at u16::MAX, which itself stays outside the rect.
    assert!(edge.contains(u16::MAX - 1, 0));
    assert!(!edge.contains(u16::MAX, 0));
    assert!(!Rect::default().contains(0, 0));
  }
}
