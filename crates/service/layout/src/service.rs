//! Service support for the layout service.

use super::types::{Position, Rect, Size};
use super::{measure, position};
use tg_service_rich_text::RichTextParams;
use tg_service_text_layout::DrawTextParams;

/// The public entry point for layout operations.
pub struct LayoutService {
  physical: Size,
  viewport_request: Option<Rect>,
  viewport: Rect,
}

impl Default for LayoutService {
  fn default() -> Self {
    Self::new()
  }
}

impl LayoutService {
  /// Create a layout service with its initial state.
  pub fn new() -> Self {
    let physical = measure::get_terminal_size();
    Self {
      physical,
      viewport_request: None,
      viewport: Rect {
        x: 0,
        y: 0,
        width: physical.width,
        height: physical.height,
      },
    }
  }

  /// Measure visible text as terminal columns and rows.
  pub fn get_text_size(&self, text: &str, params: Option<&RichTextParams>) -> Size {
    measure::get_text_size(text, params)
  }

  /// Return the maximum visible line width in terminal columns.
  pub fn get_text_width(&self, text: &str, params: Option<&RichTextParams>) -> u16 {
    measure::get_text_width(text, params)
  }

  /// Return the visible text height in terminal rows.
  pub fn get_text_height(&self, text: &str, params: Option<&RichTextParams>) -> u16 {
    measure::get_text_height(text, params)
  }

  /// Measure the terminal-cell footprint after applying draw parameters.
  pub fn get_draw_text_size(&self, params: &DrawTextParams) -> Size {
    measure::get_draw_text_size(params)
  }

  /// Return the column width after wrapping and draw-parameter resolution.
  pub fn get_draw_text_width(&self, params: &DrawTextParams) -> u16 {
    measure::get_draw_text_width(params)
  }

  /// Return the row height after wrapping and draw-parameter resolution.
  pub fn get_draw_text_height(&self, params: &DrawTextParams) -> u16 {
    measure::get_draw_text_height(params)
  }

  /// Return the current physical size.
  pub fn physical_size(&self) -> Size {
    self.physical
  }

  /// Return the current physical width.
  pub fn physical_width(&self) -> u16 {
    self.physical.width
  }

  /// Return the current physical height.
  pub fn physical_height(&self) -> u16 {
    self.physical.height
  }

  /// Return the current developer size.
  pub fn developer_size(&self) -> Size {
    Size {
      width: self.viewport.width,
      height: self.viewport.height,
    }
  }

  /// Return the current developer width.
  pub fn developer_width(&self) -> u16 {
    self.viewport.width
  }

  /// Return the current developer height.
  pub fn developer_height(&self) -> u16 {
    self.viewport.height
  }

  /// Return the resolved developer viewport clipped to physical terminal bounds.
  pub fn developer_viewport_rect(&self) -> Rect {
    self.viewport
  }

  /// Update the physical terminal dimensions used by subsequent layout resolution.
  pub fn resize_physical(&mut self, width: u16, height: u16) {
    self.physical = Size { width, height };
    self.resolve_viewport();
  }

  /// Apply the requested developer viewport within the physical terminal.
  pub fn set_developer_viewport(&mut self, rect: Rect) {
    self.viewport_request = Some(rect);
    self.resolve_viewport();
  }

  /// Restore the developer viewport to the full physical terminal.
  #[cfg(test)]
  pub(crate) fn reset_developer_viewport(&mut self) {
    self.viewport_request = None;
    self.resolve_viewport();
  }

  /// Resolve a symbolic horizontal position within the available terminal columns.
  ///
  /// # Arguments
  ///
  /// * `x_anchor` - The x anchor.
  /// * `content_width` - The content width in terminal columns.
  /// * `offset_x` - The offset x.
  pub fn resolve_x(&self, x_anchor: &str, content_width: u16, offset_x: u16) -> u16 {
    position::resolve_x(self.developer_size(), x_anchor, content_width, offset_x)
  }

  /// Resolve a horizontal position relative to the base/developer viewport.
  ///
  /// # Arguments
  ///
  /// * `x_anchor` - The x anchor.
  /// * `content_width` - The content width in terminal columns.
  /// * `offset_x` - The offset x.
  pub fn resolve_base_x(&self, x_anchor: &str, content_width: u16, offset_x: u16) -> u16 {
    self.resolve_x(x_anchor, content_width, offset_x)
  }

  /// Resolve a symbolic vertical position within the available terminal rows.
  ///
  /// # Arguments
  ///
  /// * `y_anchor` - The y anchor.
  /// * `content_height` - The content height in terminal rows.
  /// * `offset_y` - The offset y.
  pub fn resolve_y(&self, y_anchor: &str, content_height: u16, offset_y: u16) -> u16 {
    position::resolve_y(self.developer_size(), y_anchor, content_height, offset_y)
  }

  /// Resolve a vertical position relative to the base/developer viewport.
  ///
  /// # Arguments
  ///
  /// * `y_anchor` - The y anchor.
  /// * `content_height` - The content height in terminal rows.
  /// * `offset_y` - The offset y.
  pub fn resolve_base_y(&self, y_anchor: &str, content_height: u16, offset_y: u16) -> u16 {
    self.resolve_y(y_anchor, content_height, offset_y)
  }

  /// Resolve a rectangular request against its coordinate space and available bounds.
  ///
  /// # Arguments
  ///
  /// * `x_anchor` - The x anchor.
  /// * `y_anchor` - The y anchor.
  /// * `content_width` - The content width in terminal columns.
  /// * `content_height` - The content height in terminal rows.
  /// * `offset_x` - The offset x.
  /// * `offset_y` - The offset y.
  pub fn resolve_rect(
    &self,
    x_anchor: &str,
    y_anchor: &str,
    content_width: u16,
    content_height: u16,
    offset_x: u16,
    offset_y: u16,
  ) -> Position {
    position::resolve_rect(
      self.developer_size(),
      x_anchor,
      y_anchor,
      content_width,
      content_height,
      offset_x,
      offset_y,
    )
  }

  /// Resolve a rectangle relative to the base/developer viewport.
  ///
  /// # Arguments
  ///
  /// * `x_anchor` - The x anchor.
  /// * `y_anchor` - The y anchor.
  /// * `content_width` - The content width in terminal columns.
  /// * `content_height` - The content height in terminal rows.
  /// * `offset_x` - The offset x.
  /// * `offset_y` - The offset y.
  pub fn resolve_base_rect(
    &self,
    x_anchor: &str,
    y_anchor: &str,
    content_width: u16,
    content_height: u16,
    offset_x: u16,
    offset_y: u16,
  ) -> Position {
    self.resolve_rect(
      x_anchor,
      y_anchor,
      content_width,
      content_height,
      offset_x,
      offset_y,
    )
  }

  /// Resolve a horizontal position relative to the physical terminal.
  ///
  /// # Arguments
  ///
  /// * `x_anchor` - The x anchor.
  /// * `content_width` - The content width in terminal columns.
  /// * `offset_x` - The offset x.
  pub fn resolve_host_x(&self, x_anchor: &str, content_width: u16, offset_x: u16) -> u16 {
    position::resolve_x(self.physical, x_anchor, content_width, offset_x)
  }

  fn resolve_viewport(&mut self) {
    let requested = self.viewport_request.unwrap_or(Rect {
      x: 0,
      y: 0,
      width: self.physical.width,
      height: self.physical.height,
    });
    self.viewport = Rect {
      x: requested.x.min(self.physical.width),
      y: requested.y.min(self.physical.height),
      width: requested
        .width
        .min(self.physical.width.saturating_sub(requested.x)),
      height: requested
        .height
        .min(self.physical.height.saturating_sub(requested.y)),
    };
  }

  /// The align left used by this module.
  pub const ALIGN_LEFT: &'static str = position::ALIGN_LEFT;
  /// The align center used by this module.
  pub const ALIGN_CENTER: &'static str = position::ALIGN_CENTER;
  /// The align right used by this module.
  pub const ALIGN_RIGHT: &'static str = position::ALIGN_RIGHT;
  /// The align top used by this module.
  pub const ALIGN_TOP: &'static str = position::ALIGN_TOP;
  /// The align middle used by this module.
  pub const ALIGN_MIDDLE: &'static str = position::ALIGN_MIDDLE;
  /// The align bottom used by this module.
  pub const ALIGN_BOTTOM: &'static str = position::ALIGN_BOTTOM;
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn viewport_clips_resizes_and_resets() {
    let mut layout = LayoutService::new();
    layout.resize_physical(100, 40);
    assert_eq!(layout.physical_width(), 100);
    assert_eq!(layout.physical_height(), 40);
    assert_eq!(
      layout.developer_size(),
      Size {
        width: 100,
        height: 40
      }
    );
    layout.set_developer_viewport(Rect {
      x: 80,
      y: 30,
      width: 50,
      height: 20,
    });
    assert_eq!(
      layout.developer_viewport_rect(),
      Rect {
        x: 80,
        y: 30,
        width: 20,
        height: 10
      }
    );
    assert_eq!(layout.developer_width(), 20);
    assert_eq!(layout.developer_height(), 10);
    layout.resize_physical(90, 35);
    assert_eq!(
      layout.developer_viewport_rect(),
      Rect {
        x: 80,
        y: 30,
        width: 10,
        height: 5
      }
    );
    layout.reset_developer_viewport();
    assert_eq!(
      layout.developer_viewport_rect(),
      Rect {
        x: 0,
        y: 0,
        width: 90,
        height: 35
      }
    );
  }

  #[test]
  fn base_layout_aliases_match_developer_layout() {
    let mut layout = LayoutService::new();
    layout.resize_physical(100, 40);

    assert_eq!(
      layout.resolve_base_x(LayoutService::ALIGN_CENTER, 20, 0),
      layout.resolve_x(LayoutService::ALIGN_CENTER, 20, 0)
    );
    assert_eq!(
      layout.resolve_base_y(LayoutService::ALIGN_MIDDLE, 10, 0),
      layout.resolve_y(LayoutService::ALIGN_MIDDLE, 10, 0)
    );
    assert_eq!(
      layout.resolve_base_rect(
        LayoutService::ALIGN_CENTER,
        LayoutService::ALIGN_MIDDLE,
        20,
        10,
        0,
        0,
      ),
      layout.resolve_rect(
        LayoutService::ALIGN_CENTER,
        LayoutService::ALIGN_MIDDLE,
        20,
        10,
        0,
        0,
      )
    );
  }
}
