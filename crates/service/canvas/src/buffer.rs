//! Storage and mutation of the module's buffered data.

use crate::cell_source::{BufferCellSources, OwnedCellBuffer};
use crate::{CellSource, CellSurface};
use tg_core_style::CanvasCell;

/// A terminal-cell buffer retaining explicit writes and wide-grapheme continuation state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanvasBuffer {
  width: u16,
  height: u16,
  cells: Vec<CanvasCell>,
  written: Vec<bool>,
  sources: Option<BufferCellSources>,
}

impl CanvasBuffer {
  /// Create a canvas buffer initialized from `width`, `height`.
  pub fn new(width: u16, height: u16) -> Self {
    let size = width as usize * height as usize;
    Self {
      width,
      height,
      cells: vec![CanvasCell::blank(); size],
      written: vec![false; size],
      sources: None,
    }
  }
  /// Return the current width.
  pub fn width(&self) -> u16 {
    self.width
  }
  /// Return the current height.
  pub fn height(&self) -> u16 {
    self.height
  }

  /// Resize the base terminal-cell buffer and invalidate its previous contents.
  pub fn resize(&mut self, width: u16, height: u16) {
    self.width = width;
    self.height = height;
    let size = width as usize * height as usize;
    self.cells = vec![CanvasCell::blank(); size];
    self.written = vec![false; size];
    self.sources = None;
  }

  /// Discard the previously retained canvas contents.
  pub fn clear(&mut self) {
    self.sources = None;
    for (cell, written) in self.cells.iter_mut().zip(&mut self.written) {
      *cell = CanvasCell::blank();
      *written = false;
    }
  }

  /// Write a cell only when its coordinates lie within the buffer bounds.
  ///
  /// # Arguments
  ///
  /// * `x` - The horizontal coordinate in terminal cells.
  /// * `y` - The vertical coordinate in terminal cells.
  /// * `cell` - The styled terminal cell to write.
  pub fn set(&mut self, x: u16, y: u16, cell: CanvasCell) {
    let Some(index) = self.index(x, y) else {
      return;
    };
    if let Some(sources) = &mut self.sources {
      if let Some(owned) = &mut sources.owned {
        owned.buffer.set(x, y, cell.clone());
        sources.sources[index] = Some(CellSource {
          surface: owned.surface,
          x: i32::from(x) + owned.offset.0,
          y: i32::from(y) + owned.offset.1,
        });
      } else {
        sources.sources[index] = None;
      }
    }
    if let Some(target) = self.cells.get_mut(index) {
      *target = cell;
      self.written[index] = true;
    }
  }

  /// Remove the written cell contribution at the supplied buffer coordinates.
  pub fn erase(&mut self, x: u16, y: u16) {
    if let Some(owned) = self
      .sources
      .as_mut()
      .and_then(|sources| sources.owned.as_mut())
    {
      owned.buffer.erase(x, y);
    }
    self.erase_visual(x, y);
  }

  pub(crate) fn erase_visual(&mut self, x: u16, y: u16) {
    let Some(index) = self.index(x, y) else {
      return;
    };
    self.cells[index] = CanvasCell::blank();
    self.written[index] = false;
    if let Some(sources) = &mut self.sources {
      sources.sources[index] = None;
    }
  }

  /// Start retaining source identities, assigning unwritten positions to the fallback surface.
  pub fn track_cell_sources(&mut self, surface: CellSurface) {
    let sources = (0..self.height)
      .flat_map(|y| {
        (0..self.width).map(move |x| {
          Some(CellSource {
            surface,
            x: i32::from(x),
            y: i32::from(y),
          })
        })
      })
      .collect();
    self.sources = Some(BufferCellSources {
      sources,
      owned: None,
    });
  }

  pub(crate) fn track_surface(&mut self, surface: CellSurface, offset: (i32, i32)) {
    let original = self.without_cell_sources();
    let sources = (0..self.height)
      .flat_map(|y| {
        (0..self.width).map(move |x| {
          Some(CellSource {
            surface,
            x: i32::from(x) + offset.0,
            y: i32::from(y) + offset.1,
          })
        })
      })
      .collect();
    self.sources = Some(BufferCellSources {
      sources,
      owned: Some(OwnedCellBuffer {
        surface,
        offset,
        buffer: Box::new(original),
      }),
    });
  }

  /// Return the source identity attached to this composition slot, when tracking is enabled.
  pub fn cell_source(&self, x: u16, y: u16) -> Option<CellSource> {
    self
      .sources
      .as_ref()?
      .sources
      .get(self.index(x, y)?)
      .copied()
      .flatten()
  }

  /// Write a composed cell while preserving its original owning surface.
  ///
  /// # Arguments
  ///
  /// * `x` - The destination buffer column.
  /// * `y` - The destination buffer row.
  /// * `cell` - The cell contribution to compose.
  /// * `source` - The original surface identity, independent of the destination surface.
  pub fn set_composited(&mut self, x: u16, y: u16, cell: CanvasCell, source: Option<CellSource>) {
    let Some(index) = self.index(x, y) else {
      return;
    };
    self.cells[index] = cell;
    self.written[index] = true;
    if let Some(sources) = &mut self.sources {
      sources.sources[index] = source;
    }
  }

  pub(crate) fn owned_cells(&self) -> Option<&OwnedCellBuffer> {
    self.sources.as_ref()?.owned.as_ref()
  }
  pub(crate) fn owned_cells_mut(&mut self) -> Option<&mut OwnedCellBuffer> {
    self.sources.as_mut()?.owned.as_mut()
  }

  pub(crate) fn without_cell_sources(&self) -> Self {
    Self {
      width: self.width,
      height: self.height,
      cells: self.cells.clone(),
      written: self.written.clone(),
      sources: None,
    }
  }

  pub(crate) fn resolve_default_background(
    &mut self,
    background: Option<tg_core_style::TextColor>,
  ) {
    for cell in &mut self.cells {
      if cell.style.background.is_none() {
        cell.style.background.clone_from(&background);
      }
    }
  }
  /// Return access to the requested canvas buffer value when it exists.
  pub fn get(&self, x: u16, y: u16) -> Option<&CanvasCell> {
    let index = self.index(x, y)?;
    self.cells.get(index)
  }

  /// Report whether the addressed object is written.
  pub fn is_written(&self, x: u16, y: u16) -> bool {
    self
      .index(x, y)
      .and_then(|index| self.written.get(index))
      .copied()
      .unwrap_or(false)
  }

  /// Return the visible text stored in the requested buffer row.
  pub fn row_text(&self, y: u16) -> String {
    if y >= self.height {
      return String::new();
    }
    let mut text = String::new();
    for x in 0..self.width {
      if let Some(cell) = self.get(x, y) {
        text.push_str(&cell.text);
      } else {
        text.push(' ');
      }
    }
    text
  }

  fn index(&self, x: u16, y: u16) -> Option<usize> {
    if x >= self.width || y >= self.height {
      return None;
    }
    Some(y as usize * self.width as usize + x as usize)
  }
}
