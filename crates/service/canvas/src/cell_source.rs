//! Cell ownership and immutable completed-frame views of source surfaces.

use std::collections::HashMap;

use crate::{CellEffectCell, CellEffectRegion, SliceId, buffer::CanvasBuffer};

/// The original surface owning a rendered cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CellSurface {
  /// The developer base surface before other surfaces cover it.
  Base,
  /// A user slice or embedded component content surface.
  Slice(SliceId),
}

/// A rendered cell's current coordinates within its owning surface.
///
/// # Fields
///
/// * `surface` - The surface owning the character.
/// * `x`, `y` - The character's current logical surface coordinates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CellSource {
  /// The original owning surface.
  pub surface: CellSurface,
  /// The logical column within the owning surface.
  pub x: i32,
  /// The logical row within the owning surface.
  pub y: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BufferCellSources {
  pub sources: Vec<Option<CellSource>>,
  pub owned: Option<OwnedCellBuffer>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct OwnedCellBuffer {
  pub surface: CellSurface,
  pub offset: (i32, i32),
  pub buffer: Box<CanvasBuffer>,
}

/// A completed developer frame with independent, occlusion-preserving source-surface views.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CellFrameSnapshot {
  pub(crate) canvas: CanvasBuffer,
  pub(crate) surfaces: HashMap<CellSurface, OwnedCellBuffer>,
}

impl CellFrameSnapshot {
  /// Select final cells in row order, using `None` for the composed canvas.
  ///
  /// Use `Some(CellSurface::Base)` to inspect base contributions underneath other surfaces.
  /// Clip selection to the source surface's rendered bounds; returned positions remain logical
  /// surface coordinates. Undrawn surfaces return an empty collection.
  pub fn cells(
    &self,
    surface: Option<CellSurface>,
    mut region: CellEffectRegion,
  ) -> Vec<CellEffectCell> {
    let (buffer, offset) = match surface {
      None => (&self.canvas, (0, 0)),
      Some(surface) => match self.surfaces.get(&surface) {
        Some(owned) => (&*owned.buffer, owned.offset),
        None => return Vec::new(),
      },
    };
    region.x = region.x.saturating_sub(offset.0);
    region.y = region.y.saturating_sub(offset.1);
    let mut cells = buffer.select_region(region);
    for cell in &mut cells {
      cell.x += offset.0;
      cell.y += offset.1;
    }
    cells
  }
}
