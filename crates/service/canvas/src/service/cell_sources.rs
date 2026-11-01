//! Per-frame source retention and projection of shader results back to their owning surfaces.

use super::*;
use crate::{
  CellEffectCell, CellEffectChange, CellEffectRegion, CellFrameSnapshot, CellSource, CellSurface,
};

impl CanvasService {
  /// Retain independent source contributions while this prepared frame is drawn and composed.
  ///
  /// Call once after preparing the frame and before replaying drawing submissions.
  pub fn begin_cell_source_tracking(&mut self) {
    self.track_cell_sources = true;
    self.source_backgrounds_resolved = false;
    self.base.track_surface(CellSurface::Base, (0, 0));
    for (id, slice) in &mut self.slices {
      if slice.visible {
        slice.buffer.track_surface(
          CellSurface::Slice(*id),
          (i32::from(slice.source_x), i32::from(slice.source_y)),
        );
      }
    }
  }

  /// Project one atomic shader batch to its source surfaces without exposing occluded cells.
  ///
  /// Clear all original source positions before writing results in processing order. Clip
  /// projected writes to the original source surface, even when the canvas result remains visible.
  /// Set `final_canvas` for post-composition shaders so a cleared background stays cleared.
  pub fn project_cell_changes(&mut self, changes: &[CellEffectChange], final_canvas: bool) {
    if final_canvas {
      self.resolve_source_backgrounds();
    }
    for change in changes {
      if let Some(source) = change.original.source
        && let Some(owned) = self.source_cells_mut(source.surface)
        && let (Ok(x), Ok(y)) = (
          u16::try_from(source.x - owned.offset.0),
          u16::try_from(source.y - owned.offset.1),
        )
      {
        owned.buffer.erase_grapheme(x, y);
      }
    }
    for change in changes {
      if let Some(output) = &change.output {
        self.project_cell(output);
      }
    }
  }

  /// Freeze final composed cells and all independent source-surface contributions.
  ///
  /// Resolve visible cells' final composition styles back to their source views. Preserve
  /// covered contributions in their original styles and discard mutable tracing metadata.
  pub fn snapshot_cells(&mut self, canvas: &CanvasBuffer) -> CellFrameSnapshot {
    self.resolve_source_backgrounds();
    for cell in canvas.select_region(CellEffectRegion {
      x: 0,
      y: 0,
      width: canvas.width(),
      height: canvas.height(),
      include_continuation: false,
    }) {
      self.project_cell(&cell);
    }
    let mut surfaces = HashMap::new();
    if let Some(owned) = self.base.owned_cells() {
      surfaces.insert(CellSurface::Base, owned.clone());
    }
    for (id, slice) in &self.slices {
      if let Some(owned) = slice.buffer.owned_cells() {
        surfaces.insert(CellSurface::Slice(*id), owned.clone());
      }
    }
    CellFrameSnapshot {
      canvas: canvas.without_cell_sources(),
      surfaces,
    }
  }

  fn source_cells_mut(
    &mut self,
    surface: CellSurface,
  ) -> Option<&mut crate::cell_source::OwnedCellBuffer> {
    match surface {
      CellSurface::Base => self.base.owned_cells_mut(),
      CellSurface::Slice(id) => self.slices.get_mut(&id)?.buffer.owned_cells_mut(),
    }
  }

  fn resolve_source_backgrounds(&mut self) {
    if self.source_backgrounds_resolved {
      return;
    }
    for slice in self.slices.values_mut() {
      if let Some(owned) = slice.buffer.owned_cells_mut() {
        owned
          .buffer
          .resolve_default_background(slice.background.clone());
      }
    }
    self.source_backgrounds_resolved = true;
  }

  fn project_cell(&mut self, cell: &CellEffectCell) {
    let Some(CellSource { surface, x, y }) = cell.source else {
      return;
    };
    let Some(owned) = self.source_cells_mut(surface) else {
      return;
    };
    owned.buffer.write_effect_cell(CellEffectCell {
      x: x.saturating_sub(owned.offset.0),
      y: y.saturating_sub(owned.offset.1),
      cell: cell.cell.clone(),
      written: cell.written,
      source: None,
    });
  }
}
