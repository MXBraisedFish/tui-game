use std::collections::HashMap;

use super::{buffer::CanvasBuffer, cell::CanvasCell, top_layer::TopLayer};
use crate::host_engine::services::rich_text::RichTextSegment;
use crate::host_engine::services::text_layout::{self, DrawTextParams, LayoutLine, TextAlign};
use crate::host_engine::services::unicode::graphemes;
use crate::host_engine::services::widget::ui_object::surfaces::scroll_box::{
  ResolvedScrollBoxLayout, resolve_scroll_box_layout,
};
use crate::host_engine::services::widget::ui_object::surfaces::slice::resolve_rect;
use crate::host_engine::services::{
  LayoutService, Rect, ScrollBoxId, ScrollbarStyle, Size, SliceId, SurfaceId, TextColor, TextStyle,
  UiObjectPool,
};

/// 画布服务：管理基础层、宿主层和多切片缓冲区，协调文本绘制与区域查询。
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

/// 已预处理完成的切片：包含独立缓冲区、位置和可见性等元数据。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PreparedSlice {
  pub buffer: CanvasBuffer,
  pub rect: Rect,
  pub visible: bool,
  pub opaque: bool,
  pub order: usize,
}

/// 已预处理完成的滚动盒子：包含虚拟内容缓冲区和可视窗口元数据。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PreparedScrollBox {
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

/// 已预处理开发者 Surface 的只读引用。
pub(crate) enum PreparedSurface<'a> {
  Slice(&'a PreparedSlice),
  ScrollBox(&'a PreparedScrollBox),
}

impl CanvasService {
  pub fn new() -> Self {
    let (width, height) = crossterm::terminal::size().unwrap_or_else(|_e| {
      // TODO: log warn when terminal size query fails — fallback to (95, 24)
      (95, 24)
    });
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

  /// 开始新的一帧：调整宿主缓冲区尺寸，必要时标记全量重绘。
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

  /// 根据 UI 对象池和布局服务预处理所有切片缓冲区。
  pub fn prepare(&mut self, pool: &UiObjectPool, layout: &LayoutService) {
    self.viewport = layout.developer_viewport_rect();
    let size = layout.developer_size();
    if self.base.width() != size.width || self.base.height() != size.height {
      self.base.resize(size.width, size.height);
      self.force_full_redraw = true;
    } else {
      self.base.clear();
    }
    if self.active_pool != Some(pool.id()) {
      self.slices.clear();
      self.scroll_boxes.clear();
      self.force_full_redraw = true;
    }
    self.active_pool = Some(pool.id());
    self.surface_order = pool.surfaces.clone();
    self
      .slices
      .retain(|id, _| pool.slices.slices.contains_key(id));
    self
      .scroll_boxes
      .retain(|id, _| pool.scroll_boxes.boxes.contains_key(id));
    let order = self.surface_order.clone();
    for (order, surface) in order.into_iter().enumerate() {
      match surface {
        SurfaceId::Slice(id) => self.prepare_slice(pool, order, id, layout),
        SurfaceId::ScrollBox(id) => self.prepare_scroll_box(pool, order, id, layout),
      }
    }
  }

  fn prepare_slice(
    &mut self,
    pool: &UiObjectPool,
    order: usize,
    id: SliceId,
    layout: &LayoutService,
  ) {
    let Some(state) = pool.slices.slices.get(&id).copied() else {
      return;
    };
    let rect = resolve_rect(state.rect, layout);
    let prepared = self.slices.entry(id).or_insert_with(|| PreparedSlice {
      buffer: CanvasBuffer::new(rect.width, rect.height),
      rect,
      visible: state.visible,
      opaque: state.opaque,
      order,
    });
    if prepared.buffer.width() != rect.width || prepared.buffer.height() != rect.height {
      prepared.buffer.resize(rect.width, rect.height);
      self.force_full_redraw = true;
    } else {
      prepared.buffer.clear();
    }
    prepared.rect = rect;
    prepared.visible = state.visible;
    prepared.opaque = state.opaque;
    prepared.order = order;
  }

  fn prepare_scroll_box(
    &mut self,
    pool: &UiObjectPool,
    order: usize,
    id: ScrollBoxId,
    layout: &LayoutService,
  ) {
    let Some(state) = pool.scroll_boxes.boxes.get(&id) else {
      return;
    };
    let options = &state.options;
    let resolved_layout = resolve_scroll_box_layout(state, layout.developer_size());
    let content_size = Size {
      width: options.content_width,
      height: options.content_height,
    };
    let prepared = self
      .scroll_boxes
      .entry(id)
      .or_insert_with(|| PreparedScrollBox {
        buffer: CanvasBuffer::new(content_size.width, content_size.height),
        layout: resolved_layout,
        content_size,
        scroll_x: 0,
        scroll_y: 0,
        visible: options.visible,
        opaque: options.opaque,
        order,
        scrollbar_style: options.scrollbar_style.clone(),
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
    prepared.layout = resolved_layout;
    prepared.content_size = content_size;
    prepared.scroll_x = state.scroll_x;
    prepared.scroll_y = state.scroll_y;
    prepared.visible = options.visible;
    prepared.opaque = options.opaque;
    prepared.order = order;
    prepared.scrollbar_style = options.scrollbar_style.clone();
  }

  pub fn clear(&mut self) {
    self.base.clear();
  }

  /// 在基础层上绘制富文本（支持样式标签和排版参数）。
  pub fn text(&mut self, params: &DrawTextParams) {
    let lines = text_layout::layout_text_lines(params);
    Self::draw_layout_lines(
      &mut self.base,
      params.x,
      params.y,
      params.line_align,
      &lines,
    );
  }

  pub fn rich_text_segments(&mut self, segments: &[RichTextSegment], params: &DrawTextParams) {
    let lines = text_layout::layout_rich_text_segments(segments, params);
    Self::draw_layout_lines(
      &mut self.base,
      params.x,
      params.y,
      params.line_align,
      &lines,
    );
  }

  /// 在指定切片的缓冲区上绘制富文本，返回是否成功（切片不可见时返回 false）。
  pub fn text_on(&mut self, id: SliceId, params: &DrawTextParams) -> bool {
    let Some(slice) = self.slices.get_mut(&id).filter(|slice| slice.visible) else {
      return false;
    };
    let lines = text_layout::layout_text_lines(params);
    Self::draw_layout_lines(
      &mut slice.buffer,
      params.x,
      params.y,
      params.line_align,
      &lines,
    );
    true
  }

  pub fn rich_text_segments_on(
    &mut self,
    id: SliceId,
    segments: &[RichTextSegment],
    params: &DrawTextParams,
  ) -> bool {
    let Some(slice) = self.slices.get_mut(&id).filter(|slice| slice.visible) else {
      return false;
    };
    let lines = text_layout::layout_rich_text_segments(segments, params);
    Self::draw_layout_lines(
      &mut slice.buffer,
      params.x,
      params.y,
      params.line_align,
      &lines,
    );
    true
  }

  /// 在指定滚动盒子的虚拟内容缓冲区上绘制富文本。
  pub fn text_in_scroll_box(&mut self, id: ScrollBoxId, params: &DrawTextParams) -> bool {
    let Some(scroll_box) = self
      .scroll_boxes
      .get_mut(&id)
      .filter(|scroll_box| scroll_box.visible)
    else {
      return false;
    };
    let lines = text_layout::layout_text_lines(params);
    Self::draw_layout_lines(
      &mut scroll_box.buffer,
      params.x,
      params.y,
      params.line_align,
      &lines,
    );
    true
  }

  pub fn rich_text_segments_in_scroll_box(
    &mut self,
    id: ScrollBoxId,
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
    Self::draw_layout_lines(
      &mut scroll_box.buffer,
      params.x,
      params.y,
      params.line_align,
      &lines,
    );
    true
  }

  /// 在宿主层上绘制富文本（用于覆盖层等）。
  pub(crate) fn host_text(&mut self, params: &DrawTextParams) {
    let lines = text_layout::layout_text_lines(params);
    Self::draw_layout_lines(
      &mut self.host,
      params.x,
      params.y,
      params.line_align,
      &lines,
    );
  }

  /// 在宿主最高层上绘制富文本。
  pub(crate) fn top_text(&mut self, params: &DrawTextParams) {
    let lines = text_layout::layout_text_lines(params);
    Self::draw_layout_lines(
      &mut self.top.buffer_mut(),
      params.x,
      params.y,
      params.line_align,
      &lines,
    );
  }

  pub(crate) fn host_rich_text_segments(
    &mut self,
    segments: &[RichTextSegment],
    params: &DrawTextParams,
  ) {
    let lines = text_layout::layout_rich_text_segments(segments, params);
    Self::draw_layout_lines(
      &mut self.host,
      params.x,
      params.y,
      params.line_align,
      &lines,
    );
  }

  /// 在基础层上以指定样式绘制纯文本。
  pub fn styled_text(&mut self, x: u16, y: u16, text: &str, style: TextStyle) {
    Self::styled_text_to(&mut self.base, x, y, text, style);
  }

  /// 在指定切片的缓冲区上以指定样式绘制纯文本，返回是否成功。
  pub fn styled_text_on(
    &mut self,
    id: SliceId,
    x: u16,
    y: u16,
    text: &str,
    style: TextStyle,
  ) -> bool {
    let Some(slice) = self.slices.get_mut(&id).filter(|slice| slice.visible) else {
      return false;
    };
    Self::styled_text_to(&mut slice.buffer, x, y, text, style);
    true
  }

  /// 在指定滚动盒子的虚拟内容缓冲区上以指定样式绘制纯文本。
  pub fn styled_text_in_scroll_box(
    &mut self,
    id: ScrollBoxId,
    x: u16,
    y: u16,
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
    Self::styled_text_to(&mut scroll_box.buffer, x, y, text, style);
    true
  }

  /// 在宿主层上以指定样式绘制纯文本。
  pub(crate) fn host_styled_text(&mut self, x: u16, y: u16, text: &str, style: TextStyle) {
    Self::styled_text_to(&mut self.host, x, y, text, style);
  }

  pub(crate) fn host_cell(&mut self, x: u16, y: u16, cell: CanvasCell) {
    self.host.set(x, y, cell);
  }

  /// 在宿主最高层上以指定样式绘制纯文本。
  pub(crate) fn top_styled_text(&mut self, x: u16, y: u16, text: &str, style: TextStyle) {
    Self::styled_text_to(self.top.buffer_mut(), x, y, text, style);
  }

  fn styled_text_to(buffer: &mut CanvasBuffer, x: u16, y: u16, text: &str, style: TextStyle) {
    let gs = graphemes(text);
    let mut cursor_x = x;

    for g in &gs {
      if cursor_x >= buffer.width() || y >= buffer.height() {
        break;
      }

      if g.display_width == 0 {
        let final_style = resolve_background(style.clone(), buffer, cursor_x, y);
        buffer.set(cursor_x, y, CanvasCell::styled(&g.text, final_style));

        continue;
      }

      if cursor_x as usize + g.display_width > buffer.width() as usize {
        break;
      }
      let final_style = resolve_background(style.clone(), buffer, cursor_x, y);
      buffer.set(cursor_x, y, CanvasCell::styled(&g.text, final_style));
      for offset in 1..g.display_width {
        let cont_x = cursor_x.saturating_add(offset as u16);
        if cont_x < buffer.width() {
          buffer.set(cont_x, y, CanvasCell::continuation());
        }
      }

      cursor_x = cursor_x.saturating_add(g.display_width as u16);
    }
  }

  /// 重置画布尺寸并标记需要全量重绘。
  pub fn resize(&mut self, width: u16, height: u16) {
    self.host.resize(width, height);
    let _ = self.top.resize_or_clear(width, height);
    self.force_full_redraw = true;
  }

  /// 标记需要全量重绘（常用于样式或内容变更后）。
  pub fn request_render(&mut self) {
    self.force_full_redraw = true;
  }

  /// 取出并清除"需要全量重绘"标记，返回本次是否需要重绘。
  pub fn take_render_requested(&mut self) -> bool {
    let requested = self.force_full_redraw;
    self.force_full_redraw = false;
    requested
  }

  /// 获取基础层中指定坐标的字符单元。
  pub fn cell_at(&self, x: u16, y: u16) -> Option<&CanvasCell> {
    self.base.get(x, y)
  }

  pub(crate) fn host_buffer(&self) -> &CanvasBuffer {
    &self.host
  }

  pub(crate) fn top_buffer(&self) -> &CanvasBuffer {
    self.top.buffer()
  }

  pub(crate) fn base_buffer(&self) -> &CanvasBuffer {
    &self.base
  }

  /// 按绘制顺序迭代所有预处理切片。
  pub(crate) fn prepared_slices(&self) -> impl Iterator<Item = (SliceId, &PreparedSlice)> {
    self
      .surface_order
      .iter()
      .filter_map(|surface| match surface {
        SurfaceId::Slice(id) => self.slices.get(id).map(|slice| (*id, slice)),
        SurfaceId::ScrollBox(_) => None,
      })
  }

  /// 按共享层级顺序迭代所有开发者 Surface。
  pub(crate) fn prepared_surfaces(&self) -> impl Iterator<Item = PreparedSurface<'_>> {
    self
      .surface_order
      .iter()
      .filter_map(|surface| match surface {
        SurfaceId::Slice(id) => self.slices.get(id).map(PreparedSurface::Slice),
        SurfaceId::ScrollBox(id) => self.scroll_boxes.get(id).map(PreparedSurface::ScrollBox),
      })
  }

  pub(crate) fn viewport(&self) -> Rect {
    self.viewport
  }

  /// 获取指定切片在视口坐标系中的矩形区域（切片不可见时返回 None）。
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

  /// 查询预处理滚动盒子的内容区尺寸。
  pub fn prepared_scroll_box_content_size(&self, id: ScrollBoxId) -> Option<Size> {
    self
      .scroll_boxes
      .get(&id)
      .filter(|sb| sb.visible)
      .map(|sb| sb.content_size)
  }

  /// 查询预处理滚动盒子的 viewport 尺寸。
  pub fn prepared_scroll_box_viewport_size(&self, id: ScrollBoxId) -> Option<Size> {
    let sb = self.scroll_boxes.get(&id)?;
    sb.visible.then_some(Size {
      width: sb.layout.viewport_rect.width,
      height: sb.layout.viewport_rect.height,
    })
  }

  /// 查询预处理滚动盒子的滚动位置。
  pub fn prepared_scroll_box_scroll_position(&self, id: ScrollBoxId) -> Option<(u16, u16)> {
    let sb = self.scroll_boxes.get(&id)?;
    sb.visible.then_some((sb.scroll_x, sb.scroll_y))
  }

  /// 返回 Surface 层级顺序的只读切片。
  pub(crate) fn surface_order(&self) -> &[SurfaceId] {
    &self.surface_order
  }

  pub(crate) fn top_scroll_box_at(&self, x: u16, y: u16) -> Option<ScrollBoxId> {
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

  /// 将物理坐标转换为视口内的相对坐标。
  pub(crate) fn viewport_point(&self, x: u16, y: u16) -> Option<(u16, u16)> {
    self
      .viewport
      .contains(x, y)
      .then(|| (x - self.viewport.x, y - self.viewport.y))
  }

  /// 计算基础层上矩形区域的命中检测结果。
  pub(crate) fn base_hit_rect(&self, rect: Rect) -> Option<(Rect, (u16, u16), usize)> {
    surface_hit_rect(
      rect,
      self.viewport.x,
      self.viewport.y,
      self.base.width(),
      self.base.height(),
      0,
    )
  }

  /// 计算指定切片上矩形区域的命中检测结果。
  pub(crate) fn slice_hit_rect(
    &self,
    id: SliceId,
    rect: Rect,
  ) -> Option<(Rect, (u16, u16), usize)> {
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

  pub(crate) fn scroll_box_hit_rect(
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

  /// 计算宿主层上矩形区域的命中检测结果。
  pub(crate) fn host_hit_rect(&self, rect: Rect) -> Option<(Rect, (u16, u16), usize)> {
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
    x: u16,
    y: u16,
    align: TextAlign,
    lines: &[LayoutLine],
  ) {
    let base_width = lines.first().map(|line| line.width).unwrap_or(0);

    for (line_index, line) in lines.iter().enumerate() {
      let offset = match align {
        TextAlign::Left => 0,
        TextAlign::Center => base_width.saturating_sub(line.width) / 2,
        TextAlign::Right => base_width.saturating_sub(line.width),
      } as u16;
      let mut cursor_x = x.saturating_add(offset);
      let cursor_y = y.saturating_add(line_index as u16);
      let mut run_text = String::new();
      let mut run_style: Option<&TextStyle> = None;
      let mut run_width = 0usize;

      for item in &line.items {
        match run_style {
          Some(style) if style == &item.style => {}
          Some(style) => {
            Self::styled_text_to(buffer, cursor_x, cursor_y, &run_text, style.clone());
            cursor_x = cursor_x.saturating_add(run_width as u16);
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
}

// 计算矩形在指定表面上的裁剪命中区域。
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

// 解析背景色：当样式背景为 Transparent 时，继承已写入单元格的背景色。
fn resolve_background(mut style: TextStyle, buffer: &CanvasBuffer, x: u16, y: u16) -> TextStyle {
  match &style.background {
    Some(TextColor::Transparent) => {
      if buffer.is_written(x, y)
        && let Some(existing) = buffer.get(x, y)
      {
        style.background = existing.style.background.clone();
      }
    }
    _ => {}
  }
  style
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::host_engine::services::text_layout::TextWrapMode;
  use crate::host_engine::services::{
    Overflow, RichTextParams, ScrollBoxOptions, ScrollBoxService, ScrollbarPolicy,
    ScrollbarVisibility, SliceLength, SliceOptions, SliceRect, SliceService, TerminalColor,
    TextColor,
  };
  use std::collections::HashMap;

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
    let pool = UiObjectPool::new();
    let mut canvas = CanvasService::new();
    canvas.begin_frame(&layout);
    canvas.prepare(&pool, &layout);

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
  fn prepared_slice_queries_return_visible_prepared_size() {
    let mut layout = LayoutService::new();
    layout.resize_physical(20, 10);
    let mut pool = UiObjectPool::new();
    let slice = SliceService::new()
      .create(
        &mut pool,
        SliceOptions {
          rect: SliceRect {
            x: 1,
            y: 2,
            width: SliceLength::Fixed(5),
            height: SliceLength::Fixed(3),
          },
          ..Default::default()
        },
      )
      .unwrap();
    let mut canvas = CanvasService::new();

    canvas.begin_frame(&layout);
    canvas.prepare(&pool, &layout);

    assert_eq!(
      canvas.prepared_slice_rect(slice),
      Some(Rect {
        x: 1,
        y: 2,
        width: 5,
        height: 3
      })
    );
    assert_eq!(
      canvas.prepared_slice_size(slice),
      Some(Size {
        width: 5,
        height: 3
      })
    );
    assert_eq!(canvas.prepared_slice_width(slice), Some(5));
    assert_eq!(canvas.prepared_slice_height(slice), Some(3));
  }

  #[test]
  fn prepared_scroll_box_queries_return_visible_prepared_size() {
    let mut layout = LayoutService::new();
    layout.resize_physical(20, 10);
    let mut pool = UiObjectPool::new();
    let id = ScrollBoxService::new()
      .create(
        &mut pool,
        ScrollBoxOptions {
          rect: Rect {
            x: 18,
            y: 8,
            width: 10,
            height: 10,
          },
          content_width: 10,
          content_height: 20,
          ..Default::default()
        },
      )
      .unwrap();
    let mut canvas = CanvasService::new();

    canvas.begin_frame(&layout);
    canvas.prepare(&pool, &layout);

    assert_eq!(
      canvas.prepared_scroll_box_rect(id),
      Some(Rect {
        x: 18,
        y: 8,
        width: 2,
        height: 2
      })
    );
    assert_eq!(
      canvas.prepared_scroll_box_size(id),
      Some(Size {
        width: 2,
        height: 2
      })
    );
  }

  #[test]
  fn scroll_box_hit_rect_excludes_scrollbar_cells() {
    let mut layout = LayoutService::new();
    layout.resize_physical(4, 3);
    let service = ScrollBoxService::new();
    let mut pool = UiObjectPool::new();
    let id = service
      .create(
        &mut pool,
        ScrollBoxOptions {
          rect: Rect {
            x: 0,
            y: 0,
            width: 4,
            height: 3,
          },
          content_width: 4,
          content_height: 4,
          overflow_x: Overflow::Auto,
          scrollbar: ScrollbarPolicy {
            vertical: ScrollbarVisibility::Auto,
            horizontal: ScrollbarVisibility::Auto,
          },
          ..Default::default()
        },
      )
      .unwrap();
    let mut canvas = CanvasService::new();
    canvas.begin_frame(&layout);
    canvas.prepare(&pool, &layout);

    assert!(
      canvas
        .scroll_box_hit_rect(
          id,
          Rect {
            x: 2,
            y: 1,
            width: 1,
            height: 1
          }
        )
        .is_some()
    );
    assert_eq!(
      canvas.scroll_box_hit_rect(
        id,
        Rect {
          x: 3,
          y: 0,
          width: 1,
          height: 1
        }
      ),
      None
    );
    assert_eq!(
      canvas.scroll_box_hit_rect(
        id,
        Rect {
          x: 0,
          y: 2,
          width: 1,
          height: 1
        }
      ),
      None
    );
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
