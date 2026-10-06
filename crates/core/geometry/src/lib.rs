//! Positions, sizes, and half-open rectangles measured in terminal cells.
//!
//! # Examples
//!
//! ```rust
//! use tg_core_geometry::Rect;
//!
//! let rect = Rect { x: 2, y: 3, width: 4, height: 2 };
//! assert!(rect.contains(2, 3));
//! assert!(!rect.contains(6, 3));
//! ```

/// A two-dimensional size measured in terminal columns and rows.
///
/// # Fields
///
/// * `width` - The width in terminal columns.
/// * `height` - The height in terminal rows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Size {
  /// The width in terminal columns.
  pub width: u16,
  /// The height in terminal rows.
  pub height: u16,
}

/// A position measured in terminal-cell coordinates.
///
/// # Fields
///
/// * `x` - The horizontal coordinate in terminal cells.
/// * `y` - The vertical coordinate in terminal cells.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Position {
  /// The horizontal coordinate in terminal cells.
  pub x: u16,
  /// The vertical coordinate in terminal cells.
  pub y: u16,
}

/// A terminal-cell rectangle with exclusive right and bottom edges.
///
/// # Fields
///
/// * `x` - The horizontal coordinate in terminal cells.
/// * `y` - The vertical coordinate in terminal cells.
/// * `width` - The width in terminal columns.
/// * `height` - The height in terminal rows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rect {
  /// The horizontal coordinate in terminal cells.
  pub x: u16,
  /// The vertical coordinate in terminal cells.
  pub y: u16,
  /// The width in terminal columns.
  pub width: u16,
  /// The height in terminal rows.
  pub height: u16,
}

impl Rect {
  /// Report whether the point lies within the rectangle, excluding the right and bottom edges.
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
    let rect = Rect {
      x: 2,
      y: 3,
      width: 4,
      height: 2,
    };
    assert!(rect.contains(2, 3));
    assert!(rect.contains(5, 4));
    assert!(!rect.contains(6, 4));
    assert!(!rect.contains(5, 5));
    assert!(!rect.contains(1, 3));
    let edge = Rect {
      x: u16::MAX - 1,
      y: 0,
      width: 10,
      height: 1,
    };
    // Saturation keeps u16::MAX outside the half-open right edge.

    assert!(edge.contains(u16::MAX - 1, 0));
    assert!(!edge.contains(u16::MAX, 0));
    assert!(!Rect::default().contains(0, 0));
  }
}
