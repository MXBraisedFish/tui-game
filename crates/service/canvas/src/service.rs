use std::collections::HashMap;

use super::surface::{
  ResolvedScrollBoxLayout, ScrollBoxFrame, ScrollBoxId, ScrollbarStyle, SliceFrame, SliceId,
  SurfaceFrame, SurfaceId,
};
use super::{CanvasCell, buffer::CanvasBuffer, top_layer::TopLayer};
use tg_core_geometry::{Rect, Size};
use tg_core_style::{TextColor, TextStyle};
use tg_core_unicode::graphemes;
use tg_service_layout::LayoutService;
use tg_service_rich_text::RichTextSegment;
use tg_service_text_layout::{self as text_layout, DrawTextParams, LayoutLine, TextAlign};

/// The canvas service, owning the base layer, the host layer and the per-surface buffers; it
/// coordinates text drawing and area queries.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanvasService {
  base: CanvasBuffer,
  host: CanvasBuffer,
  top: TopLayer,
  slices: HashMap<SliceId, PreparedSlice>,
  scroll_boxes: HashMap<ScrollBoxId, PreparedScrollBox>,
  surface_order: Vec<SurfaceId>,
  viewport: Rect,
  active_pool: Option<u64>,
  force_full_redraw: bool,
}

/// A prepared slice: its own buffer plus metadata such as position and visibility.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreparedSlice {
  pub buffer: CanvasBuffer,
  pub rect: Rect,
  pub visible: bool,
  pub opaque: bool,
  pub background: Option<TextColor>,
  pub order: usize,
}

/// A prepared scroll box: its virtual content buffer plus viewport metadata.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreparedScrollBox {
  pub buffer: CanvasBuffer,
  pub layout: ResolvedScrollBoxLayout,
  pub content_size: Size,
  pub scroll_x: u16,
  pub scroll_y: u16,
  pub visible: bool,
  pub opaque: bool,
  pub order: usize,
  pub scrollbar_style: ScrollbarStyle,
}

/// A read-only reference to a prepared developer surface.
pub enum PreparedSurface<'a> {
  Slice(&'a PreparedSlice),
  ScrollBox(&'a PreparedScrollBox),
}

impl Default for CanvasService {
  fn default() -> Self {
    Self::new()
  }
}

impl CanvasService {
  pub fn new() -> Self {
    // TODO: log warn when terminal size query fails — fallback to (95, 24)
    let (width, height) = crossterm::terminal::size().unwrap_or((95, 24));
    Self {
      base: CanvasBuffer::new(width, height),
      host: CanvasBuffer::new(width, height),
      top: TopLayer::new(width, height),
      slices: HashMap::new(),
      scroll_boxes: HashMap::new(),
      surface_order: Vec::new(),
      viewport: Rect {
        x: 0,
        y: 0,
        width,
        height,
      },
      active_pool: None,
      force_full_redraw: true,
    }
  }

  pub fn base_width(&self) -> u16 {
    self.base.width()
  }

  pub fn base_height(&self) -> u16 {
    self.base.height()
  }

  pub fn base_size(&self) -> Size {
    Size {
      width: self.base.width(),
      height: self.base.height(),
    }
  }

  /// Starts a new frame: resizes the host buffers, requesting a full redraw when needed.
  pub fn begin_frame(&mut self, layout: &LayoutService) {
    let physical = layout.physical_size();
    if self.host.width() != physical.width || self.host.height() != physical.height {
      self.host.resize(physical.width, physical.height);
      self.force_full_redraw = true;
    } else {
      self.host.clear();
    }
    if self.top.resize_or_clear(physical.width, physical.height) {
      self.force_full_redraw = true;
    }
  }

  /// Prepares the buffers of all drawing surfaces of this frame in stacking order; when `pool_id`
  /// changes, the buffers of the previous object pool are dropped.
  pub fn prepare(&mut self, pool_id: u64, surfaces: Vec<SurfaceFrame>, layout: &LayoutService) {
    self.viewport = layout.developer_viewport_rect();
    let size = layout.developer_size();
    if self.base.width() != size.width || self.base.height() != size.height {
      self.base.resize(size.width, size.height);
      self.force_full_redraw = true;
    } else {
      self.base.clear();
    }
    if self.active_pool != Some(pool_id) {
      self.slices.clear();
      self.scroll_boxes.clear();
      self.force_full_redraw = true;
    }
    self.active_pool = Some(pool_id);
    self.surface_order = surfaces.iter().map(SurfaceFrame::id).collect();
    let order = &self.surface_order;
    self
      .slices
      .retain(|id, _| order.contains(&SurfaceId::Slice(*id)));
    self
      .scroll_boxes
      .retain(|id, _| order.contains(&SurfaceId::ScrollBox(*id)));
    for (order, surface) in surfaces.into_iter().enumerate() {
      match surface {
        SurfaceFrame::Slice(frame) => self.prepare_slice(order, frame),
        SurfaceFrame::ScrollBox(frame) => self.prepare_scroll_box(order, frame),
      }
    }
  }

  fn prepare_slice(&mut self, order: usize, frame: SliceFrame) {
    let rect = frame.rect;
    let prepared = self
      .slices
      .entry(frame.id)
      .or_insert_with(|| PreparedSlice {
        buffer: CanvasBuffer::new(rect.width, rect.height),
        rect,
        visible: frame.visible,
        opaque: frame.opaque,
        background: frame.background.clone(),
        order,
      });
    if prepared.buffer.width() != rect.width || prepared.buffer.height() != rect.height {
      prepared.buffer.resize(rect.width, rect.height);
      self.force_full_redraw = true;
    } else {
      prepared.buffer.clear();
    }
    prepared.rect = rect;
    prepared.visible = frame.visible;
    prepared.opaque = frame.opaque;
    prepared.background = frame.background;
    prepared.order = order;
  }

  fn prepare_scroll_box(&mut self, order: usize, frame: ScrollBoxFrame) {
    let content_size = frame.content_size;
    let prepared = self
      .scroll_boxes
      .entry(frame.id)
      .or_insert_with(|| PreparedScrollBox {
        buffer: CanvasBuffer::new(content_size.width, content_size.height),
        layout: frame.layout,
        content_size,
        scroll_x: 0,
        scroll_y: 0,
        visible: frame.visible,
        opaque: frame.opaque,
        order,
        scrollbar_style: frame.scrollbar_style.clone(),
      });
    if prepared.buffer.width() != content_size.width
      || prepared.buffer.height() != content_size.height
    {
      prepared
        .buffer
        .resize(content_size.width, content_size.height);
      self.force_full_redraw = true;
    } else {
      prepared.buffer.clear();
    }
    prepared.layout = frame.layout;
    prepared.content_size = content_size;
    prepared.scroll_x = frame.scroll_x;
    prepared.scroll_y = frame.scroll_y;
    prepared.visible = frame.visible;
    prepared.opaque = frame.opaque;
    prepared.order = order;
    prepared.scrollbar_style = frame.scrollbar_style;
  }

  pub fn clear(&mut self) {
    self.base.clear();
  }

  /// Erases a rectangle of the base layer so the content below shows through again.
  pub fn erase_rect(&mut self, x: i32, y: i32, width: u16, height: u16) {
    Self::erase_rect_from(&mut self.base, x, y, width, height);
  }

  /// Erases a rectangle in the given slice. Returns `false` when the slice is missing or hidden.
  pub fn erase_rect_on(&mut self, id: SliceId, x: i32, y: i32, width: u16, height: u16) -> bool {
    let Some(slice) = self.slices.get_mut(&id).filter(|slice| slice.visible) else {
      return false;
    };
    Self::erase_rect_from(&mut slice.buffer, x, y, width, height);
    true
  }

  /// Draws rich text on the base layer (supports style tags and layout parameters).
  pub fn text(&mut self, params: &DrawTextParams) {
    self.text_at(i32::from(params.x), i32::from(params.y), params);
  }

  /// Draws rich text at signed coordinates on the base layer; content outside the canvas is
  /// clipped.
  pub fn text_at(&mut self, x: i32, y: i32, params: &DrawTextParams) {
    let lines = text_layout::layout_text_lines(params);
    Self::draw_layout_lines(&mut self.base, x, y, params.line_align, &lines);
  }

  pub fn rich_text_segments(&mut self, segments: &[RichTextSegment], params: &DrawTextParams) {
    self.rich_text_segments_at(i32::from(params.x), i32::from(params.y), segments, params);
  }

  pub fn rich_text_segments_at(
    &mut self,
    x: i32,
    y: i32,
    segments: &[RichTextSegment],
    params: &DrawTextParams,
  ) {
    let lines = text_layout::layout_rich_text_segments(segments, params);
    Self::draw_layout_lines(&mut self.base, x, y, params.line_align, &lines);
  }

  /// Draws rich text into the buffer of the given slice. Returns whether it succeeded (`false`
  /// when the slice is not visible).
  pub fn text_on(&mut self, id: SliceId, params: &DrawTextParams) -> bool {
    self.text_at_on(id, i32::from(params.x), i32::from(params.y), params)
  }

  /// Draws text at signed local coordinates of the slice.
  pub fn text_at_on(&mut self, id: SliceId, x: i32, y: i32, params: &DrawTextParams) -> bool {
    let Some(slice) = self.slices.get_mut(&id).filter(|slice| slice.visible) else {
      return false;
    };
    let lines = text_layout::layout_text_lines(params);
    Self::draw_layout_lines(&mut slice.buffer, x, y, params.line_align, &lines);
    true
  }

  pub fn rich_text_segments_on(
    &mut self,
    id: SliceId,
    segments: &[RichTextSegment],
    params: &DrawTextParams,
  ) -> bool {
    self.rich_text_segments_at_on(
      id,
      i32::from(params.x),
      i32::from(params.y),
      segments,
      params,
    )
  }

  pub fn rich_text_segments_at_on(
    &mut self,
    id: SliceId,
    x: i32,
    y: i32,
    segments: &[RichTextSegment],
    params: &DrawTextParams,
  ) -> bool {
    let Some(slice) = self.slices.get_mut(&id).filter(|slice| slice.visible) else {
      return false;
    };
    let lines = text_layout::layout_rich_text_segments(segments, params);
    Self::draw_layout_lines(&mut slice.buffer, x, y, params.line_align, &lines);
    true
  }

  /// Draws rich text into the virtual content buffer of the given scroll box.
  pub fn text_in_scroll_box(&mut self, id: ScrollBoxId, params: &DrawTextParams) -> bool {
    self.text_at_in_scroll_box(id, i32::from(params.x), i32::from(params.y), params)
  }

  /// Draws text at signed coordinates of the scroll box's virtual content area.
  pub fn text_at_in_scroll_box(
    &mut self,
    id: ScrollBoxId,
    x: i32,
    y: i32,
    params: &DrawTextParams,
  ) -> bool {
    let Some(scroll_box) = self
      .scroll_boxes
      .get_mut(&id)
      .filter(|scroll_box| scroll_box.visible)
    else {
      return false;
    };
    let lines = text_layout::layout_text_lines(params);
    Self::draw_layout_lines(&mut scroll_box.buffer, x, y, params.line_align, &lines);
    true
  }

  pub fn rich_text_segments_in_scroll_box(
    &mut self,
    id: ScrollBoxId,
    segments: &[RichTextSegment],
    params: &DrawTextParams,
  ) -> bool {
    self.rich_text_segments_at_in_scroll_box(
      id,
      i32::from(params.x),
      i32::from(params.y),
      segments,
      params,
    )
  }

  pub fn rich_text_segments_at_in_scroll_box(
    &mut self,
    id: ScrollBoxId,
    x: i32,
    y: i32,
    segments: &[RichTextSegment],
    params: &DrawTextParams,
  ) -> bool {
    let Some(scroll_box) = self
      .scroll_boxes
      .get_mut(&id)
      .filter(|scroll_box| scroll_box.visible)
    else {
      return false;
    };
    let lines = text_layout::layout_rich_text_segments(segments, params);
    Self::draw_layout_lines(&mut scroll_box.buffer, x, y, params.line_align, &lines);
    true
  }

  /// Draws rich text on the host layer (used by overlays and the like).
  pub fn host_text(&mut self, params: &DrawTextParams) {
    self.host_text_at(i32::from(params.x), i32::from(params.y), params);
  }

  pub fn host_text_at(&mut self, x: i32, y: i32, params: &DrawTextParams) {
    let lines = text_layout::layout_text_lines(params);
    Self::draw_layout_lines(&mut self.host, x, y, params.line_align, &lines);
  }

  /// Draws rich text on the host's top layer.
  pub fn top_text(&mut self, params: &DrawTextParams) {
    self.top_text_at(i32::from(params.x), i32::from(params.y), params);
  }

  pub fn top_text_at(&mut self, x: i32, y: i32, params: &DrawTextParams) {
    let lines = text_layout::layout_text_lines(params);
    Self::draw_layout_lines(self.top.buffer_mut(), x, y, params.line_align, &lines);
  }

  pub fn host_rich_text_segments(&mut self, segments: &[RichTextSegment], params: &DrawTextParams) {
    self.host_rich_text_segments_at(i32::from(params.x), i32::from(params.y), segments, params);
  }

  pub(crate) fn host_rich_text_segments_at(
    &mut self,
    x: i32,
    y: i32,
    segments: &[RichTextSegment],
    params: &DrawTextParams,
  ) {
    let lines = text_layout::layout_rich_text_segments(segments, params);
    Self::draw_layout_lines(&mut self.host, x, y, params.line_align, &lines);
  }

  /// Draws plain text with the given style on the base layer.
  pub fn styled_text(
    &mut self,
    x: impl Into<i32>,
    y: impl Into<i32>,
    text: &str,
    style: TextStyle,
  ) {
    Self::styled_text_to(&mut self.base, x.into(), y.into(), text, style);
  }

  /// Draws plain text with the given style into the buffer of the given slice. Returns whether it
  /// succeeded.
  pub fn styled_text_on(
    &mut self,
    id: SliceId,
    x: impl Into<i32>,
    y: impl Into<i32>,
    text: &str,
    style: TextStyle,
  ) -> bool {
    let Some(slice) = self.slices.get_mut(&id).filter(|slice| slice.visible) else {
      return false;
    };
    Self::styled_text_to(&mut slice.buffer, x.into(), y.into(), text, style);
    true
  }

  /// Draws plain text with the given style into the virtual content buffer of the given scroll
  /// box.
  pub fn styled_text_in_scroll_box(
    &mut self,
    id: ScrollBoxId,
    x: impl Into<i32>,
    y: impl Into<i32>,
    text: &str,
    style: TextStyle,
  ) -> bool {
    let Some(scroll_box) = self
      .scroll_boxes
      .get_mut(&id)
      .filter(|scroll_box| scroll_box.visible)
    else {
      return false;
    };
    Self::styled_text_to(&mut scroll_box.buffer, x.into(), y.into(), text, style);
    true
  }

  /// Draws plain text with the given style on the host layer.
  pub fn host_styled_text(
    &mut self,
    x: impl Into<i32>,
    y: impl Into<i32>,
    text: &str,
    style: TextStyle,
  ) {
    Self::styled_text_to(&mut self.host, x.into(), y.into(), text, style);
  }

  pub fn host_cell(&mut self, x: impl Into<i32>, y: impl Into<i32>, cell: CanvasCell) {
    let (x, y) = (x.into(), y.into());
    if let (Ok(x), Ok(y)) = (u16::try_from(x), u16::try_from(y)) {
      self.host.set(x, y, cell);
    }
  }

  fn styled_text_to(buffer: &mut CanvasBuffer, x: i32, y: i32, text: &str, style: TextStyle) {
    let Ok(y) = u16::try_from(y) else {
      return;
    };
    if y >= buffer.height() {
      return;
    }

    let gs = graphemes(text);
    let mut cursor_x = x;
    let buffer_width = i32::from(buffer.width());

    for g in &gs {
      if cursor_x >= buffer_width {
        break;
      }

      if g.display_width == 0 {
        if let Ok(visible_x) = u16::try_from(cursor_x)
          && visible_x < buffer.width()
        {
          let final_style = resolve_background(style.clone(), buffer, visible_x, y);
          buffer.set(visible_x, y, CanvasCell::styled(&g.text, final_style));
        }

        continue;
      }

      let grapheme_width = i32::try_from(g.display_width).unwrap_or(i32::MAX);
      let next_x = cursor_x.saturating_add(grapheme_width);
      if next_x <= 0 {
        cursor_x = next_x;
        continue;
      }
      if cursor_x < 0 {
        cursor_x = next_x;
        continue;
      }
      if next_x > buffer_width {
        break;
      }
      let visible_x = cursor_x as u16;
      let final_style = resolve_background(style.clone(), buffer, visible_x, y);
      buffer.set(visible_x, y, CanvasCell::styled(&g.text, final_style));
      for offset in 1..g.display_width {
        let cont_x = visible_x.saturating_add(offset as u16);
        buffer.set(cont_x, y, CanvasCell::continuation());
      }

      cursor_x = next_x;
    }
  }

  /// Resizes the canvas and requests a full redraw.
  pub fn resize(&mut self, width: u16, height: u16) {
    self.host.resize(width, height);
    let _ = self.top.resize_or_clear(width, height);
    self.force_full_redraw = true;
  }

  /// Requests a full redraw (typically after a style or content change).
  pub fn request_render(&mut self) {
    self.force_full_redraw = true;
  }

  /// Takes and clears the "full redraw requested" flag, returning whether a redraw is needed.
  pub fn take_render_requested(&mut self) -> bool {
    let requested = self.force_full_redraw;
    self.force_full_redraw = false;
    requested
  }

  /// Returns the cell of the base layer at the given coordinates.
  pub fn cell_at(&self, x: u16, y: u16) -> Option<&CanvasCell> {
    self.base.get(x, y)
  }

  pub fn host_buffer(&self) -> &CanvasBuffer {
    &self.host
  }

  pub fn top_buffer(&self) -> &CanvasBuffer {
    self.top.buffer()
  }

  pub fn base_buffer(&self) -> &CanvasBuffer {
    &self.base
  }

  /// Iterates over all prepared developer surfaces in their shared stacking order.
  pub fn prepared_surfaces(&self) -> impl Iterator<Item = PreparedSurface<'_>> {
    self
      .surface_order
      .iter()
      .filter_map(|surface| match surface {
        SurfaceId::Slice(id) => self.slices.get(id).map(PreparedSurface::Slice),
        SurfaceId::ScrollBox(id) => self.scroll_boxes.get(id).map(PreparedSurface::ScrollBox),
      })
  }

  pub fn viewport(&self) -> Rect {
    self.viewport
  }

  /// Returns the rectangle of the given slice in viewport coordinates (`None` when the slice is
  /// not visible).
  pub fn prepared_slice_rect(&self, id: SliceId) -> Option<Rect> {
    let slice = self.slices.get(&id)?;
    slice.visible.then_some(slice.rect)
  }

  pub fn prepared_slice_size(&self, id: SliceId) -> Option<Size> {
    let rect = self.prepared_slice_rect(id)?;
    Some(Size {
      width: rect.width,
      height: rect.height,
    })
  }

  pub fn prepared_slice_width(&self, id: SliceId) -> Option<u16> {
    Some(self.prepared_slice_size(id)?.width)
  }

  pub fn prepared_slice_height(&self, id: SliceId) -> Option<u16> {
    Some(self.prepared_slice_size(id)?.height)
  }

  pub fn prepared_scroll_box_rect(&self, id: ScrollBoxId) -> Option<Rect> {
    let scroll_box = self.scroll_boxes.get(&id)?;
    scroll_box
      .visible
      .then_some(scroll_box.layout.viewport_rect)
  }

  pub fn prepared_scroll_box_size(&self, id: ScrollBoxId) -> Option<Size> {
    let rect = self.prepared_scroll_box_rect(id)?;
    Some(Size {
      width: rect.width,
      height: rect.height,
    })
  }

  /// Returns the content size of the prepared scroll box.
  pub fn prepared_scroll_box_content_size(&self, id: ScrollBoxId) -> Option<Size> {
    self
      .scroll_boxes
      .get(&id)
      .filter(|sb| sb.visible)
      .map(|sb| sb.content_size)
  }

  /// Returns the viewport size of the prepared scroll box.
  pub fn prepared_scroll_box_viewport_size(&self, id: ScrollBoxId) -> Option<Size> {
    let sb = self.scroll_boxes.get(&id)?;
    sb.visible.then_some(Size {
      width: sb.layout.viewport_rect.width,
      height: sb.layout.viewport_rect.height,
    })
  }

  /// Returns the scroll position of the prepared scroll box.
  pub fn prepared_scroll_box_scroll_position(&self, id: ScrollBoxId) -> Option<(u16, u16)> {
    let sb = self.scroll_boxes.get(&id)?;
    sb.visible.then_some((sb.scroll_x, sb.scroll_y))
  }

  /// Returns the surface stacking order as a read-only slice.
  pub fn surface_order(&self) -> &[SurfaceId] {
    &self.surface_order
  }

  pub fn top_scroll_box_at(&self, x: u16, y: u16) -> Option<ScrollBoxId> {
    self
      .surface_order
      .iter()
      .filter_map(|surface| match surface {
        SurfaceId::ScrollBox(id) => {
          let scroll_box = self.scroll_boxes.get(id)?;
          let rect = physical_rect(self.viewport, scroll_box.layout.occupied_rect);
          (scroll_box.visible && rect.width > 0 && rect.height > 0 && rect.contains(x, y))
            .then_some((*id, scroll_box.order))
        }
        SurfaceId::Slice(id) => {
          let slice = self.slices.get(id)?;
          let rect = physical_rect(self.viewport, slice.rect);
          (slice.visible && rect.width > 0 && rect.height > 0 && rect.contains(x, y))
            .then_some((ScrollBoxId(0), slice.order))
        }
      })
      .max_by_key(|(_, order)| *order)
      .and_then(|(id, _)| (id != ScrollBoxId(0)).then_some(id))
  }

  /// Converts physical coordinates into coordinates relative to the viewport; returns `None` when
  /// the point is outside the viewport.
  pub fn viewport_point(&self, x: u16, y: u16) -> Option<(u16, u16)> {
    self
      .viewport
      .contains(x, y)
      .then(|| (x - self.viewport.x, y - self.viewport.y))
  }

  /// Returns the hit-test result of a rectangle on the base layer.
  pub fn base_hit_rect(&self, rect: Rect) -> Option<(Rect, (u16, u16), usize)> {
    surface_hit_rect(
      rect,
      self.viewport.x,
      self.viewport.y,
      self.base.width(),
      self.base.height(),
      0,
    )
  }

  /// Returns the hit-test result of a rectangle on the given slice.
  pub fn slice_hit_rect(&self, id: SliceId, rect: Rect) -> Option<(Rect, (u16, u16), usize)> {
    let slice = self.slices.get(&id).filter(|slice| slice.visible)?;
    surface_hit_rect(
      rect,
      self.viewport.x.saturating_add(slice.rect.x),
      self.viewport.y.saturating_add(slice.rect.y),
      slice.buffer.width(),
      slice.buffer.height(),
      slice.order + 1,
    )
  }

  pub fn scroll_box_hit_rect(
    &self,
    id: ScrollBoxId,
    rect: Rect,
  ) -> Option<(Rect, (u16, u16), usize)> {
    let scroll_box = self
      .scroll_boxes
      .get(&id)
      .filter(|scroll_box| scroll_box.visible)?;
    let viewport = Rect {
      x: scroll_box.scroll_x,
      y: scroll_box.scroll_y,
      width: scroll_box.layout.content_viewport_rect.width,
      height: scroll_box.layout.content_viewport_rect.height,
    };
    let x1 = rect.x.max(viewport.x);
    let y1 = rect.y.max(viewport.y);
    let x2 = rect
      .x
      .saturating_add(rect.width)
      .min(viewport.x.saturating_add(viewport.width));
    let y2 = rect
      .y
      .saturating_add(rect.height)
      .min(viewport.y.saturating_add(viewport.height));
    if x2 <= x1 || y2 <= y1 {
      return None;
    }
    let visible = Rect {
      x: self
        .viewport
        .x
        .saturating_add(scroll_box.layout.content_viewport_rect.x)
        .saturating_add(x1.saturating_sub(scroll_box.scroll_x)),
      y: self
        .viewport
        .y
        .saturating_add(scroll_box.layout.content_viewport_rect.y)
        .saturating_add(y1.saturating_sub(scroll_box.scroll_y)),
      width: x2 - x1,
      height: y2 - y1,
    };
    Some((
      visible,
      (
        self
          .viewport
          .x
          .saturating_add(scroll_box.layout.content_viewport_rect.x),
        self
          .viewport
          .y
          .saturating_add(scroll_box.layout.content_viewport_rect.y),
      ),
      scroll_box.order + 1,
    ))
  }

  /// Returns the hit-test result of a rectangle on the host layer.
  pub fn host_hit_rect(&self, rect: Rect) -> Option<(Rect, (u16, u16), usize)> {
    surface_hit_rect(
      rect,
      0,
      0,
      self.host.width(),
      self.host.height(),
      usize::MAX,
    )
  }

  fn draw_layout_lines(
    buffer: &mut CanvasBuffer,
    x: i32,
    y: i32,
    align: TextAlign,
    lines: &[LayoutLine],
  ) {
    let base_width = lines.first().map(|line| line.width).unwrap_or(0);

    for (line_index, line) in lines.iter().enumerate() {
      let base_width = i32::try_from(base_width).unwrap_or(i32::MAX);
      let line_width = i32::try_from(line.width).unwrap_or(i32::MAX);
      let offset = match align {
        TextAlign::Left => 0,
        TextAlign::Center => base_width.saturating_sub(line_width) / 2,
        TextAlign::Right => base_width.saturating_sub(line_width),
      };
      let mut cursor_x = x.saturating_add(offset);
      let cursor_y = y.saturating_add(i32::try_from(line_index).unwrap_or(i32::MAX));
      let mut run_text = String::new();
      let mut run_style: Option<&TextStyle> = None;
      let mut run_width = 0usize;

      for item in &line.items {
        match run_style {
          Some(style) if style == &item.style => {}
          Some(style) => {
            Self::styled_text_to(buffer, cursor_x, cursor_y, &run_text, style.clone());
            cursor_x = cursor_x.saturating_add(i32::try_from(run_width).unwrap_or(i32::MAX));
            run_text.clear();
            run_width = 0;
            run_style = Some(&item.style);
          }
          None => run_style = Some(&item.style),
        }
        run_text.push_str(&item.text);
        run_width += item.width;
      }

      if let Some(style) = run_style {
        Self::styled_text_to(buffer, cursor_x, cursor_y, &run_text, style.clone());
      }
    }
  }

  fn erase_rect_from(buffer: &mut CanvasBuffer, x: i32, y: i32, width: u16, height: u16) {
    let left = x.max(0).min(i32::from(buffer.width()));
    let top = y.max(0).min(i32::from(buffer.height()));
    let right = x
      .saturating_add(i32::from(width))
      .max(0)
      .min(i32::from(buffer.width()));
    let bottom = y
      .saturating_add(i32::from(height))
      .max(0)
      .min(i32::from(buffer.height()));
    for row in top..bottom {
      for column in left..right {
        buffer.erase(column as u16, row as u16);
      }
    }
  }
}

// Computes the clipped hit area of a rectangle on the given surface.
fn surface_hit_rect(
  rect: Rect,
  ox: u16,
  oy: u16,
  width: u16,
  height: u16,
  rank: usize,
) -> Option<(Rect, (u16, u16), usize)> {
  let x = rect.x.min(width);
  let y = rect.y.min(height);
  let width = rect.width.min(width.saturating_sub(x));
  let height = rect.height.min(height.saturating_sub(y));
  (width > 0 && height > 0).then_some((
    Rect {
      x: ox.saturating_add(x),
      y: oy.saturating_add(y),
      width,
      height,
    },
    (ox, oy),
    rank,
  ))
}

fn physical_rect(viewport: Rect, rect: Rect) -> Rect {
  Rect {
    x: viewport.x.saturating_add(rect.x),
    y: viewport.y.saturating_add(rect.y),
    width: rect.width,
    height: rect.height,
  }
}

// Resolves the background color: a Transparent style background inherits the background of the
// already written cell.
fn resolve_background(mut style: TextStyle, buffer: &CanvasBuffer, x: u16, y: u16) -> TextStyle {
  if matches!(style.background, Some(TextColor::Transparent))
    && buffer.is_written(x, y)
    && let Some(existing) = buffer.get(x, y)
  {
    style.background = existing.style.background.clone();
  }
  style
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::collections::HashMap;
  use tg_service_rich_text::{RichTextParams, TerminalColor, TextMode};
  use tg_service_text_layout::TextWrapMode;

  /// Writes `height` rows of `ch` from (x, y) the way the render service fills a rectangle.
  fn fill_rect(
    canvas: &mut CanvasService,
    x: i32,
    y: i32,
    width: u16,
    height: u16,
    ch: char,
    bg: Option<TextColor>,
  ) {
    for row in 0..height {
      canvas.text_at(
        x,
        y + i32::from(row),
        &DrawTextParams {
          text: ch.to_string().repeat(usize::from(width)),
          bg: bg.clone(),
          ..Default::default()
        },
      );
    }
  }

  fn visible_row(canvas: &CanvasService, y: u16) -> String {
    (0..canvas.base_width())
      .filter_map(|x| {
        canvas.base.get(x, y).and_then(|cell| {
          if cell.is_continuation() || cell.text == " " {
            None
          } else {
            Some(cell.text.as_str())
          }
        })
      })
      .collect()
  }

  fn raw_row_prefix(canvas: &CanvasService, y: u16, width: u16) -> String {
    let mut text = String::new();
    for x in 0..width {
      text.push_str(
        canvas
          .base
          .get(x, y)
          .map(|cell| cell.text.as_str())
          .unwrap_or(" "),
      );
    }
    text
  }

  #[test]
  fn rich_text_key_with_cjk_tail() {
    let mut canvas = CanvasService::new();
    let mut key_actions = HashMap::new();
    key_actions.insert("home.confirm".to_string(), vec![vec!["enter".to_string()]]);
    let params = RichTextParams {
      values: HashMap::new(),
      key_default_actions: key_actions.clone(),
      key_actions,
    };

    let text = "f%<fg:bright_black>{key:home.confirm} 确认</fg>";
    canvas.text(&DrawTextParams {
      x: 0,
      y: 0,
      text: text.to_string(),
      params: Some(params),
      ..Default::default()
    });

    assert_eq!(
      visible_row(&canvas, 0),
      "[Enter]确认",
      "full text including CJK tail must be present"
    );
  }
  #[test]
  fn styled_text_cjk_full() {
    let mut canvas = CanvasService::new();
    let style = TextStyle::default();
    canvas.styled_text(0, 0, "确认", style);

    assert_eq!(
      visible_row(&canvas, 0),
      "确认",
      "CJK characters must all be written"
    );
  }

  #[test]
  fn signed_text_coordinates_are_clipped_to_the_base_canvas() {
    let mut canvas = CanvasService::new();
    canvas.base.resize(5, 2);

    canvas.text_at(
      -2,
      1,
      &DrawTextParams {
        text: "abcdef".to_string(),
        ..Default::default()
      },
    );
    canvas.text_at(
      0,
      -1,
      &DrawTextParams {
        text: "hidden\nshown".to_string(),
        ..Default::default()
      },
    );

    assert_eq!(raw_row_prefix(&canvas, 0, 5), "shown");
    assert_eq!(raw_row_prefix(&canvas, 1, 5), "cdef ");
  }

  #[test]
  fn signed_styled_text_skips_partially_visible_wide_graphemes() {
    let mut canvas = CanvasService::new();
    canvas.base.resize(5, 1);

    canvas.styled_text(-1, 0, "界ab", TextStyle::default());

    assert_eq!(raw_row_prefix(&canvas, 0, 5), " ab  ");
    assert!(
      (0..canvas.base_width()).all(|x| !canvas.base.get(x, 0).unwrap().is_continuation()),
      "a clipped wide grapheme must not leave a continuation cell"
    );
  }

  #[test]
  fn negative_filled_rect_is_clipped_by_canvas_text_writes() {
    let mut canvas = CanvasService::new();
    canvas.base.resize(4, 3);
    fill_rect(&mut canvas, -1, -1, 3, 3, '#', None);

    assert_eq!(raw_row_prefix(&canvas, 0, 4), "##  ");
    assert_eq!(raw_row_prefix(&canvas, 1, 4), "##  ");
    assert_eq!(raw_row_prefix(&canvas, 2, 4), "    ");
  }

  #[test]
  fn erase_rect_removes_previous_character_and_style_writes() {
    let mut canvas = CanvasService::new();
    canvas.base.resize(12, 6);
    fill_rect(
      &mut canvas,
      2,
      1,
      10,
      4,
      ' ',
      Some(TextColor::Terminal(TerminalColor::Blue)),
    );

    canvas.erase_rect(3, 2, 8, 2);

    assert!(canvas.base.is_written(2, 2));
    assert!(!canvas.base.is_written(3, 2));
    assert!(!canvas.base.is_written(10, 3));
    assert!(canvas.base.is_written(11, 3));
    assert_eq!(canvas.base.get(3, 2), Some(&CanvasCell::blank()));
  }

  #[test]
  fn signed_coordinates_are_clipped_inside_slices_and_scroll_boxes() {
    let mut layout = LayoutService::new();
    layout.resize_physical(12, 6);
    let slice = SliceId(1);
    let scroll_box = ScrollBoxId(1);
    let box_rect = Rect {
      x: 5,
      y: 0,
      width: 4,
      height: 2,
    };
    let mut canvas = CanvasService::new();
    canvas.begin_frame(&layout);
    canvas.prepare(
      1,
      vec![
        SurfaceFrame::Slice(SliceFrame {
          id: slice,
          rect: Rect {
            x: 0,
            y: 0,
            width: 4,
            height: 2,
          },
          visible: true,
          opaque: false,
          background: None,
        }),
        SurfaceFrame::ScrollBox(ScrollBoxFrame {
          id: scroll_box,
          layout: ResolvedScrollBoxLayout {
            viewport_rect: box_rect,
            content_viewport_rect: box_rect,
            occupied_rect: box_rect,
            vertical_track_rect: None,
            horizontal_track_rect: None,
            vertical_thumb_rect: None,
            horizontal_thumb_rect: None,
            max_scroll_x: 0,
            max_scroll_y: 0,
          },
          content_size: Size {
            width: 4,
            height: 2,
          },
          scroll_x: 0,
          scroll_y: 0,
          visible: true,
          opaque: false,
          scrollbar_style: ScrollbarStyle::default(),
        }),
      ],
      &layout,
    );
    let params = DrawTextParams {
      text: "abcd".to_string(),
      ..Default::default()
    };

    assert!(canvas.text_at_on(slice, -2, 0, &params));
    assert!(canvas.text_at_in_scroll_box(scroll_box, -1, 0, &params));

    assert_eq!(
      canvas.slices[&slice].buffer.row_text(0),
      "cd  ",
      "slice-local negative coordinates must clip"
    );
    assert_eq!(
      canvas.scroll_boxes[&scroll_box].buffer.row_text(0),
      "bcd ",
      "scroll-box content coordinates must clip"
    );
  }

  #[test]
  fn normal_wrap_truncates_with_marker_by_grapheme_width() {
    let mut canvas = CanvasService::new();
    canvas.text(&DrawTextParams {
      x: 0,
      y: 0,
      text: "我爱你xxxxoooo".to_string(),
      max_width: Some(10),
      overflow_marker: Some("...".to_string()),
      ..Default::default()
    });

    assert_eq!(visible_row(&canvas, 0), "我爱你x...");
  }

  #[test]
  fn none_wrap_ignores_explicit_newlines() {
    let mut canvas = CanvasService::new();
    canvas.text(&DrawTextParams {
      x: 0,
      y: 0,
      text: "ab\ncd".to_string(),
      wrap_mode: TextWrapMode::None,
      ..Default::default()
    });

    assert_eq!(visible_row(&canvas, 0), "abcd");
    assert_eq!(visible_row(&canvas, 1), "");
  }

  #[test]
  fn auto_wrap_respects_width_and_explicit_newlines() {
    let mut canvas = CanvasService::new();
    canvas.text(&DrawTextParams {
      x: 0,
      y: 0,
      text: "abcd\nefgh".to_string(),
      wrap_mode: TextWrapMode::Auto,
      max_width: Some(3),
      ..Default::default()
    });

    assert_eq!(visible_row(&canvas, 0), "abc");
    assert_eq!(visible_row(&canvas, 1), "d");
    assert_eq!(visible_row(&canvas, 2), "efg");
    assert_eq!(visible_row(&canvas, 3), "h");
  }

  #[test]
  fn auto_wrap_draws_multiline_rich_image_text() {
    let mut canvas = CanvasService::new();
    canvas.text(&DrawTextParams {
      x: 0,
      y: 0,
      text: "f%<bg:#111111><fg:#eeeeee>▅▅▅▅\n<bg:#222222><fg:#dddddd>▅▅▅▅".to_string(),
      wrap_mode: TextWrapMode::Auto,
      max_width: Some(4),
      max_height: Some(2),
      ..Default::default()
    });

    assert_eq!(visible_row(&canvas, 0), "▅▅▅▅");
    assert_eq!(visible_row(&canvas, 1), "▅▅▅▅");
  }

  #[test]
  fn max_height_marks_hidden_text() {
    let mut canvas = CanvasService::new();
    canvas.text(&DrawTextParams {
      x: 0,
      y: 0,
      text: "abcd".to_string(),
      wrap_mode: TextWrapMode::Auto,
      max_width: Some(2),
      max_height: Some(1),
      overflow_marker: Some(".".to_string()),
      ..Default::default()
    });

    assert_eq!(visible_row(&canvas, 0), "a.");
    assert_eq!(visible_row(&canvas, 1), "");
  }

  #[test]
  fn multiline_alignment_is_relative_to_first_line() {
    let mut canvas = CanvasService::new();
    canvas.text(&DrawTextParams {
      x: 0,
      y: 0,
      text: "abcd\nef".to_string(),
      line_align: TextAlign::Right,
      ..Default::default()
    });

    assert_eq!(raw_row_prefix(&canvas, 0, 4), "abcd");
    assert_eq!(raw_row_prefix(&canvas, 1, 4), "  ef");
  }

  #[test]
  fn multiline_alignment_allows_lines_wider_than_the_first_line() {
    for (align, expected_x) in [
      (TextAlign::Left, 5),
      (TextAlign::Center, 4),
      (TextAlign::Right, 2),
    ] {
      let mut canvas = CanvasService::new();
      canvas.text(&DrawTextParams {
        x: 5,
        y: 0,
        text: "Test\nabcdefg".to_string(),
        line_align: align,
        ..Default::default()
      });

      assert_eq!(canvas.base.get(expected_x, 1).unwrap().text, "a");
      assert_eq!(canvas.base.get(5, 0).unwrap().text, "T");
    }
  }

  #[test]
  fn explicit_text_modes_control_prefix_and_rich_text_parsing() {
    let mut canvas = CanvasService::new();
    let params = Some(RichTextParams::default());
    let text = "f%<fg:green>Test";
    for (y, text_mode) in [
      (0, TextMode::Plain),
      (1, TextMode::Rich),
      (2, TextMode::Auto),
    ] {
      canvas.text(&DrawTextParams {
        x: 0,
        y,
        text: text.to_string(),
        text_mode,
        params: params.clone(),
        ..Default::default()
      });
    }

    assert_eq!(raw_row_prefix(&canvas, 0, 16), "f%<fg:green>Test");
    assert_eq!(raw_row_prefix(&canvas, 1, 6), "f%Test");
    assert_eq!(raw_row_prefix(&canvas, 2, 4), "Test");
    assert_eq!(canvas.base.get(0, 0).unwrap().style.foreground, None);
    assert_eq!(
      canvas.base.get(2, 1).unwrap().style.foreground,
      Some(TextColor::Terminal(TerminalColor::Green))
    );
    assert_eq!(
      canvas.base.get(0, 2).unwrap().style.foreground,
      Some(TextColor::Terminal(TerminalColor::Green))
    );
  }

  #[test]
  fn rich_text_wrapping_preserves_segment_style() {
    let mut canvas = CanvasService::new();
    canvas.text(&DrawTextParams {
      x: 0,
      y: 0,
      text: "f%<fg:red>ab</fg><fg:blue>cd</fg>".to_string(),
      wrap_mode: TextWrapMode::Auto,
      max_width: Some(3),
      ..Default::default()
    });

    let c = canvas.base.get(2, 0).expect("c cell");
    let d = canvas.base.get(0, 1).expect("d cell");
    assert_eq!(c.text, "c");
    assert_eq!(d.text, "d");
    assert_eq!(
      c.style.foreground,
      Some(TextColor::Terminal(TerminalColor::Blue))
    );
    assert_eq!(
      d.style.foreground,
      Some(TextColor::Terminal(TerminalColor::Blue))
    );
  }

  #[test]
  fn rich_text_auto_wrap_preserves_long_segment_style() {
    let mut canvas = CanvasService::new();
    let blue_text = "[Settings -> Storage Management -> Export Data]";
    canvas.text(&DrawTextParams {
      x: 0,
      y: 0,
      text: format!(
        "f%If you need to proceed, create a backup via <fg:blue>{}</fg> first.",
        blue_text
      ),
      wrap_mode: TextWrapMode::Auto,
      max_width: Some(76),
      ..Default::default()
    });

    let mut blue = String::new();
    for y in 0..4 {
      for x in 0..canvas.base_width() {
        let Some(cell) = canvas.base.get(x, y) else {
          continue;
        };
        if cell.is_continuation() || cell.text == " " {
          continue;
        }
        if cell.style.foreground == Some(TextColor::Terminal(TerminalColor::Blue)) {
          blue.push_str(&cell.text);
        }
      }
    }

    assert!(blue.replace(' ', "").contains(&blue_text.replace(' ', "")));
  }

  #[test]
  fn draw_text_params_new_sets_required_fields() {
    let params = DrawTextParams::new(3, 4, "hello");

    assert_eq!(params.x, 3);
    assert_eq!(params.y, 4);
    assert_eq!(params.text, "hello");
    assert_eq!(params.wrap_mode, TextWrapMode::Normal);
    assert_eq!(params.line_align, TextAlign::Left);
  }

  #[test]
  fn cell_at_reads_current_text_cell() {
    let mut canvas = CanvasService::new();
    canvas.styled_text(2, 3, "a", TextStyle::default());

    let cell = canvas.cell_at(2, 3).expect("cell");
    assert_eq!(cell.text, "a");
  }

  #[test]
  fn viewport_point_ignores_physical_coordinates_before_viewport_origin() {
    let mut layout = LayoutService::new();
    layout.resize_physical(20, 10);
    layout.set_developer_viewport(Rect {
      x: 2,
      y: 2,
      width: 16,
      height: 6,
    });
    let mut canvas = CanvasService::new();
    canvas.begin_frame(&layout);
    canvas.prepare(1, Vec::new(), &layout);

    assert_eq!(
      canvas.base_size(),
      Size {
        width: 16,
        height: 6
      }
    );
    assert_eq!(canvas.base_height(), 6);
    assert_eq!(canvas.viewport_point(0, 0), None);
    assert_eq!(canvas.viewport_point(2, 2), Some((0, 0)));
  }

  #[test]
  fn styled_text_preserves_complete_graphemes() {
    let mut canvas = CanvasService::new();
    canvas.styled_text(0, 0, "e\u{301}👨‍👩", TextStyle::default());

    assert_eq!(canvas.cell_at(0, 0).unwrap().text, "e\u{301}");
    assert_eq!(canvas.cell_at(1, 0).unwrap().text, "👨‍👩");
    assert!(canvas.cell_at(2, 0).unwrap().is_continuation());
  }
}
