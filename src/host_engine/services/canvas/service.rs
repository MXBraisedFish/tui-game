use std::collections::HashMap;

use super::{buffer::CanvasBuffer, cell::CanvasCell};
use crate::host_engine::services::slice::resolve_rect;
use crate::host_engine::services::text_layout::{self, DrawTextParams, LayoutLine, TextAlign};
use crate::host_engine::services::unicode::graphemes;
use crate::host_engine::services::{
  LayoutService, Rect, Size, SliceId, TextColor, TextStyle, UiObjectPool,
};

/// 画布服务：管理基础层、宿主层和多切片缓冲区，协调文本绘制与区域查询。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanvasService {
  base: CanvasBuffer,
  host: CanvasBuffer,
  slices: HashMap<SliceId, PreparedSlice>,
  slice_order: Vec<SliceId>,
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

impl CanvasService {
  pub fn new() -> Self {
    let (width, height) = crossterm::terminal::size().unwrap_or((80, 24));
    Self {
      base: CanvasBuffer::new(width, height),
      host: CanvasBuffer::new(width, height),
      slices: HashMap::new(),
      slice_order: Vec::new(),
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
  }

  /// 根据 UI 对象池和布局服务预处理所有切片缓冲区。
  pub fn prepare(&mut self, pool: &UiObjectPool, layout: &LayoutService) {
    self.viewport = layout.developer_viewport_rect();
    let size = layout.developer_size();
    if self.base.width() != size.width || self.base.height() != size.height {
      self.base.resize(size.width, size.height);
    } else {
      self.base.clear();
    }
    if self.active_pool != Some(pool.id()) {
      self.slices.clear();
    }
    self.active_pool = Some(pool.id());
    self.slice_order = pool.slices.order.clone();
    self
      .slices
      .retain(|id, _| pool.slices.slices.contains_key(id));
    for (order, id) in self.slice_order.iter().copied().enumerate() {
      let state = pool.slices.slices[&id];
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
      } else {
        prepared.buffer.clear();
      }
      prepared.rect = rect;
      prepared.visible = state.visible;
      prepared.opaque = state.opaque;
      prepared.order = order;
    }
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

  /// 在宿主层上以指定样式绘制纯文本。
  pub(crate) fn host_styled_text(&mut self, x: u16, y: u16, text: &str, style: TextStyle) {
    Self::styled_text_to(&mut self.host, x, y, text, style);
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

  pub(crate) fn base_buffer(&self) -> &CanvasBuffer {
    &self.base
  }

  /// 按绘制顺序迭代所有预处理切片。
  pub(crate) fn prepared_slices(&self) -> impl Iterator<Item = (SliceId, &PreparedSlice)> {
    self
      .slice_order
      .iter()
      .filter_map(|id| self.slices.get(id).map(|slice| (*id, slice)))
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
    RichTextParams, SliceLength, SliceOptions, SliceRect, SliceService, TerminalColor, TextColor,
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
  fn styled_text_preserves_complete_graphemes() {
    let mut canvas = CanvasService::new();
    canvas.styled_text(0, 0, "e\u{301}👨‍👩", TextStyle::default());

    assert_eq!(canvas.cell_at(0, 0).unwrap().text, "e\u{301}");
    assert_eq!(canvas.cell_at(1, 0).unwrap().text, "👨‍👩");
    assert!(canvas.cell_at(2, 0).unwrap().is_continuation());
  }
}
