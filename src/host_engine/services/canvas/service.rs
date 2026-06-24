use std::collections::HashMap;

use super::{buffer::CanvasBuffer, cell::CanvasCell};
use crate::host_engine::services::slice::resolve_rect;
use crate::host_engine::services::text_layout::{self, DrawTextParams, LayoutLine, TextAlign};
use crate::host_engine::services::unicode::graphemes;
use crate::host_engine::services::{
  LayoutService, Rect, SliceId, TextColor, TextStyle, UiObjectPool,
};

// ── 画布服务 ──

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

  pub fn width(&self) -> u16 {
    self.base.width()
  }

  pub fn height(&self) -> u16 {
    self.base.height()
  }

  pub fn size(&self) -> (u16, u16) {
    (self.width(), self.height())
  }

  pub(crate) fn physical_width(&self) -> u16 {
    self.host.width()
  }

  pub(crate) fn physical_height(&self) -> u16 {
    self.host.height()
  }

  pub fn begin_frame(&mut self, layout: &LayoutService) {
    let physical = layout.physical_size();
    if self.host.width() != physical.width || self.host.height() != physical.height {
      self.host.resize(physical.width, physical.height);
      self.force_full_redraw = true;
    } else {
      self.host.clear();
    }
  }

  pub fn prepare(&mut self, pool: &UiObjectPool, layout: &LayoutService) {
    self.viewport = layout.developer_viewport();
    let size = layout.viewport_size();
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

  // ── 文本绘制 ──

  /// 文本绘制路由入口。
  /// 检查 `f%` 前缀决定走富文本流还是普通文本流，
  /// 最终都汇聚到 `styled_text()` 写入画布单元格。
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

  /// 画布底层基元：在 (x, y) 处以指定样式绘制文本。
  /// 不做任何前缀检查，调用方给什么就画什么。
  ///
  /// 正确处理 Unicode 显示宽度：
  /// - 零宽字符（ZWJ、ZWS、组合标记）写入单元格但不推进光标
  /// - 普通字符（ASCII、拉丁）推进 1 格
  /// - 宽字符（CJK、emoji、全角）推进 2 格，并标记右侧格为 continuation
  pub fn styled_text(&mut self, x: u16, y: u16, text: &str, style: TextStyle) {
    Self::styled_text_to(&mut self.base, x, y, text, style);
  }

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
        // 零宽字符：写入当前格但不推进光标（与前一字符合并于同一 Print 输出）
        let final_style = resolve_background(style.clone(), buffer, cursor_x, y);
        buffer.set(cursor_x, y, CanvasCell::styled(&g.text, final_style));
        // cursor_x 不变
        continue;
      }

      if cursor_x as usize + g.display_width > buffer.width() as usize {
        break;
      }

      // 宽字符 ≥1：写入首格
      let final_style = resolve_background(style.clone(), buffer, cursor_x, y);
      buffer.set(cursor_x, y, CanvasCell::styled(&g.text, final_style));

      // 宽字符 ≥2：标记右侧连续格为 CONTINUATION
      for offset in 1..g.display_width {
        let cont_x = cursor_x.saturating_add(offset as u16);
        if cont_x < buffer.width() {
          buffer.set(cont_x, y, CanvasCell::continuation());
        }
      }

      cursor_x = cursor_x.saturating_add(g.display_width as u16);
    }
  }

  // ── 尺寸 ──

  /// 仅更新画布缓冲尺寸，不触发强制重绘。
  /// 需要重绘时由调用方显式调用 `request_render()`。
  pub fn resize(&mut self, width: u16, height: u16) {
    self.host.resize(width, height);
    self.force_full_redraw = true;
  }

  /// 标记下一帧为强制全屏重绘。收到 resize / focus 等系统事件时调用。
  pub fn request_render(&mut self) {
    self.force_full_redraw = true;
  }

  pub fn take_render_requested(&mut self) -> bool {
    let requested = self.force_full_redraw;
    self.force_full_redraw = false;
    requested
  }

  pub fn cell_at(&self, x: u16, y: u16) -> Option<&CanvasCell> {
    self.base.get(x, y)
  }

  pub(crate) fn host_buffer(&self) -> &CanvasBuffer {
    &self.host
  }

  pub(crate) fn base_buffer(&self) -> &CanvasBuffer {
    &self.base
  }

  pub(crate) fn prepared_slices(&self) -> impl Iterator<Item = (SliceId, &PreparedSlice)> {
    self
      .slice_order
      .iter()
      .filter_map(|id| self.slices.get(id).map(|slice| (*id, slice)))
  }

  pub(crate) fn viewport(&self) -> Rect {
    self.viewport
  }

  pub(crate) fn slice_rect(&self, id: SliceId) -> Option<Rect> {
    let slice = self.slices.get(&id)?;
    slice.visible.then_some(slice.rect)
  }

  pub(crate) fn viewport_point(&self, x: u16, y: u16) -> Option<(u16, u16)> {
    self
      .viewport
      .contains(x, y)
      .then(|| (x - self.viewport.x, y - self.viewport.y))
  }

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

/// 解析背景色：`Transparent` 从画布继承，`None` / 具体颜色保持原样。
fn resolve_background(mut style: TextStyle, buffer: &CanvasBuffer, x: u16, y: u16) -> TextStyle {
  match &style.background {
    Some(TextColor::Transparent) => {
      if buffer.is_written(x, y)
        && let Some(existing) = buffer.get(x, y)
      {
        style.background = existing.style.background.clone();
      }
    }
    _ => {} // None 或具体颜色：不继承
  }
  style
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::host_engine::services::text_layout::TextWrapMode;
  use crate::host_engine::services::{RichTextParams, TerminalColor, TextColor};
  use std::collections::HashMap;

  fn visible_row(canvas: &CanvasService, y: u16) -> String {
    (0..canvas.width())
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

  /// 模拟 home 界面的 action 提示渲染：{key:} + CJK 尾随文本。
  /// 验证富文本解析后的所有字符均被写入画布，不会被截断。
  #[test]
  fn rich_text_key_with_cjk_tail() {
    let mut canvas = CanvasService::new();

    // 构建参数：模拟 home.confirm → [Enter]
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

    // 读取第 0 行的全部非空白格，拼接为可见字符串
    // 预期：{key:home.confirm} → [Enter]，后面跟 " 确认"
    assert_eq!(
      visible_row(&canvas, 0),
      "[Enter]确认",
      "full text including CJK tail must be present"
    );
  }

  /// 验证纯 CJK 文本（不含 {key:}）也能完整写入。
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
