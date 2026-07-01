use super::types::{Position, Rect, Size};
use super::{measure, position};
use crate::host_engine::services::DrawTextParams;
use crate::host_engine::services::RichTextParams;
use crate::host_engine::services::ui::UiObjectPool;
use crate::host_engine::services::widget::scroll_box::effective_viewport;
use crate::host_engine::services::widget::slice::resolve_rect as resolve_slice_rect;
use crate::host_engine::services::widget::{ScrollBoxId, SliceId};

/// 布局服务，管理终端尺寸、视口和坐标计算
pub struct LayoutService {
  physical: Size,
  viewport_request: Option<Rect>,
  viewport: Rect,
}

impl LayoutService {
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

  pub fn get_text_size(&self, text: &str, params: Option<&RichTextParams>) -> Size {
    measure::get_text_size(text, params)
  }

  pub fn get_text_width(&self, text: &str, params: Option<&RichTextParams>) -> u16 {
    measure::get_text_width(text, params)
  }

  pub fn get_text_height(&self, text: &str, params: Option<&RichTextParams>) -> u16 {
    measure::get_text_height(text, params)
  }

  pub fn get_draw_text_size(&self, params: &DrawTextParams) -> Size {
    measure::get_draw_text_size(params)
  }

  pub fn get_draw_text_width(&self, params: &DrawTextParams) -> u16 {
    measure::get_draw_text_width(params)
  }

  pub fn get_draw_text_height(&self, params: &DrawTextParams) -> u16 {
    measure::get_draw_text_height(params)
  }

  pub fn physical_size(&self) -> Size {
    self.physical
  }

  pub fn physical_width(&self) -> u16 {
    self.physical.width
  }

  pub fn physical_height(&self) -> u16 {
    self.physical.height
  }

  pub fn developer_size(&self) -> Size {
    Size {
      width: self.viewport.width,
      height: self.viewport.height,
    }
  }

  pub fn developer_width(&self) -> u16 {
    self.viewport.width
  }

  pub fn developer_height(&self) -> u16 {
    self.viewport.height
  }

  pub fn developer_viewport_rect(&self) -> Rect {
    self.viewport
  }

  pub(crate) fn resize_physical(&mut self, width: u16, height: u16) {
    self.physical = Size { width, height };
    self.resolve_viewport();
  }

  pub(crate) fn set_developer_viewport(&mut self, rect: Rect) {
    self.viewport_request = Some(rect);
    self.resolve_viewport();
  }

  pub(crate) fn reset_developer_viewport(&mut self) {
    self.viewport_request = None;
    self.resolve_viewport();
  }

  /// 在视口内根据水平锚点和内容宽度计算 X 坐标
  pub fn resolve_x(&self, x_anchor: &str, content_width: u16, offset_x: u16) -> u16 {
    position::resolve_x(self.developer_size(), x_anchor, content_width, offset_x)
  }

  pub fn resolve_base_x(&self, x_anchor: &str, content_width: u16, offset_x: u16) -> u16 {
    self.resolve_x(x_anchor, content_width, offset_x)
  }

  /// 在视口内根据垂直锚点和内容高度计算 Y 坐标
  pub fn resolve_y(&self, y_anchor: &str, content_height: u16, offset_y: u16) -> u16 {
    position::resolve_y(self.developer_size(), y_anchor, content_height, offset_y)
  }

  pub fn resolve_base_y(&self, y_anchor: &str, content_height: u16, offset_y: u16) -> u16 {
    self.resolve_y(y_anchor, content_height, offset_y)
  }

  /// 在视口内根据锚点和内容尺寸计算位置
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

  pub fn resolve_slice_x(
    &self,
    pool: &UiObjectPool,
    id: SliceId,
    x_anchor: &str,
    content_width: u16,
    offset_x: u16,
  ) -> Option<u16> {
    Some(position::resolve_x(
      self.slice_size(pool, id)?,
      x_anchor,
      content_width,
      offset_x,
    ))
  }

  pub fn resolve_slice_y(
    &self,
    pool: &UiObjectPool,
    id: SliceId,
    y_anchor: &str,
    content_height: u16,
    offset_y: u16,
  ) -> Option<u16> {
    Some(position::resolve_y(
      self.slice_size(pool, id)?,
      y_anchor,
      content_height,
      offset_y,
    ))
  }

  pub fn resolve_slice_rect(
    &self,
    pool: &UiObjectPool,
    id: SliceId,
    x_anchor: &str,
    y_anchor: &str,
    content_width: u16,
    content_height: u16,
    offset_x: u16,
    offset_y: u16,
  ) -> Option<Position> {
    Some(position::resolve_rect(
      self.slice_size(pool, id)?,
      x_anchor,
      y_anchor,
      content_width,
      content_height,
      offset_x,
      offset_y,
    ))
  }

  pub fn resolve_scroll_box_x(
    &self,
    pool: &UiObjectPool,
    id: ScrollBoxId,
    x_anchor: &str,
    content_width: u16,
    offset_x: u16,
  ) -> Option<u16> {
    Some(position::resolve_x(
      self.scroll_box_visible_size(pool, id)?,
      x_anchor,
      content_width,
      offset_x,
    ))
  }

  pub fn resolve_scroll_box_y(
    &self,
    pool: &UiObjectPool,
    id: ScrollBoxId,
    y_anchor: &str,
    content_height: u16,
    offset_y: u16,
  ) -> Option<u16> {
    Some(position::resolve_y(
      self.scroll_box_visible_size(pool, id)?,
      y_anchor,
      content_height,
      offset_y,
    ))
  }

  pub fn resolve_scroll_box_rect(
    &self,
    pool: &UiObjectPool,
    id: ScrollBoxId,
    x_anchor: &str,
    y_anchor: &str,
    content_width: u16,
    content_height: u16,
    offset_x: u16,
    offset_y: u16,
  ) -> Option<Position> {
    Some(position::resolve_rect(
      self.scroll_box_visible_size(pool, id)?,
      x_anchor,
      y_anchor,
      content_width,
      content_height,
      offset_x,
      offset_y,
    ))
  }

  pub(crate) fn resolve_host_x(&self, x_anchor: &str, content_width: u16, offset_x: u16) -> u16 {
    position::resolve_x(self.physical, x_anchor, content_width, offset_x)
  }

  pub(crate) fn resolve_host_y(&self, y_anchor: &str, content_height: u16, offset_y: u16) -> u16 {
    position::resolve_y(self.physical, y_anchor, content_height, offset_y)
  }

  fn slice_size(&self, pool: &UiObjectPool, id: SliceId) -> Option<Size> {
    let rect = resolve_slice_rect(pool.slices.slices.get(&id)?.rect, self);
    Some(Size {
      width: rect.width,
      height: rect.height,
    })
  }

  fn scroll_box_visible_size(&self, pool: &UiObjectPool, id: ScrollBoxId) -> Option<Size> {
    Some(effective_viewport(
      pool.scroll_boxes.boxes.get(&id)?,
      self.developer_size(),
    ))
  }

  // 根据物理尺寸和开发者视口请求计算最终视口，并裁剪到物理边界内
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

  pub const ALIGN_LEFT: &'static str = position::ALIGN_LEFT;
  pub const ALIGN_CENTER: &'static str = position::ALIGN_CENTER;
  pub const ALIGN_RIGHT: &'static str = position::ALIGN_RIGHT;
  pub const ALIGN_TOP: &'static str = position::ALIGN_TOP;
  pub const ALIGN_MIDDLE: &'static str = position::ALIGN_MIDDLE;
  pub const ALIGN_BOTTOM: &'static str = position::ALIGN_BOTTOM;
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::host_engine::services::{
    Overflow, ScrollBoxId, ScrollBoxOptions, ScrollBoxService, ScrollbarPolicy,
    ScrollbarVisibility, SliceId, SliceLength, SliceOptions, SliceRect, SliceService, UiObjectPool,
  };

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

  #[test]
  fn slice_layout_uses_slice_local_size() {
    let mut layout = LayoutService::new();
    layout.resize_physical(100, 40);
    let mut pool = UiObjectPool::new();
    let slice = SliceService::new()
      .create(
        &mut pool,
        SliceOptions {
          rect: SliceRect {
            x: 5,
            y: 5,
            width: SliceLength::Fixed(30),
            height: SliceLength::Fixed(10),
          },
          ..Default::default()
        },
      )
      .unwrap();

    assert_eq!(
      layout.resolve_slice_rect(
        &pool,
        slice,
        LayoutService::ALIGN_CENTER,
        LayoutService::ALIGN_MIDDLE,
        10,
        4,
        0,
        0,
      ),
      Some(Position { x: 10, y: 3 })
    );
    assert_eq!(
      layout.resolve_slice_x(&pool, SliceId(999), LayoutService::ALIGN_CENTER, 10, 0),
      None
    );
  }

  #[test]
  fn scroll_box_layout_uses_visible_content_size() {
    let mut layout = LayoutService::new();
    layout.resize_physical(100, 40);
    let mut pool = UiObjectPool::new();
    let scroll_box = ScrollBoxService::new()
      .create(
        &mut pool,
        ScrollBoxOptions {
          rect: Rect {
            x: 0,
            y: 0,
            width: 20,
            height: 5,
          },
          content_width: 20,
          content_height: 10,
          overflow_y: Overflow::Auto,
          scrollbar: ScrollbarPolicy {
            vertical: ScrollbarVisibility::Auto,
            horizontal: ScrollbarVisibility::Never,
          },
          ..Default::default()
        },
      )
      .unwrap();

    assert_eq!(
      layout.resolve_scroll_box_rect(
        &pool,
        scroll_box,
        LayoutService::ALIGN_CENTER,
        LayoutService::ALIGN_MIDDLE,
        9,
        1,
        0,
        0,
      ),
      Some(Position { x: 5, y: 2 })
    );
    assert_eq!(
      layout.resolve_scroll_box_y(&pool, ScrollBoxId(999), LayoutService::ALIGN_MIDDLE, 1, 0,),
      None
    );
  }
}
