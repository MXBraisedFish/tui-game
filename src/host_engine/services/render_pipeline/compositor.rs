use super::{ComposedCell, ComposedFrame};
use crate::host_engine::services::canvas::buffer::CanvasBuffer;
use crate::host_engine::services::canvas::{PreparedScrollBox, PreparedSurface};
use crate::host_engine::services::unicode::graphemes;
use crate::host_engine::services::{
  CanvasCell, CanvasService, ScrollbarLayout, ScrollbarVisibility, TextColor,
};

/// 帧合成器：将基础层、切片层和宿主层按顺序叠加为一张合成帧。
pub struct FrameCompositor;

impl FrameCompositor {
  pub fn new() -> Self {
    Self
  }

  /// 执行合成：按照分层顺序（底层 → 切片 → 宿主层）合并各层像素。
  pub fn compose(&self, canvas: &CanvasService) -> ComposedFrame {
    let host = canvas.host_buffer();
    let mut frame = ComposedFrame::new(host.width(), host.height());
    for y in 0..host.height() {
      for x in 0..host.width() {
        frame.set(x, y, ComposedCell::Text(CanvasCell::blank()));
      }
    }

    let viewport = canvas.viewport();
    overlay(
      &mut frame,
      canvas.base_buffer(),
      viewport.x,
      viewport.y,
      true,
    );
    for surface in canvas.prepared_surfaces() {
      match surface {
        PreparedSurface::Slice(slice) if slice.visible => overlay(
          &mut frame,
          &slice.buffer,
          viewport.x.saturating_add(slice.rect.x),
          viewport.y.saturating_add(slice.rect.y),
          slice.opaque,
        ),
        PreparedSurface::ScrollBox(scroll_box) if scroll_box.visible => {
          overlay_scroll_box(&mut frame, scroll_box, viewport.x, viewport.y)
        }
        _ => {}
      }
    }
    overlay(&mut frame, host, 0, 0, false);

    frame
  }
}

fn overlay(frame: &mut ComposedFrame, buffer: &CanvasBuffer, ox: u16, oy: u16, opaque: bool) {
  for y in 0..buffer.height() {
    for x in 0..buffer.width() {
      if !opaque && !buffer.is_written(x, y) {
        continue;
      }
      let Some(source) = buffer.get(x, y) else {
        continue;
      };
      let px = ox.saturating_add(x);
      let py = oy.saturating_add(y);
      write_cell(frame, px, py, source);
    }
  }
}

fn overlay_scroll_box(frame: &mut ComposedFrame, scroll_box: &PreparedScrollBox, ox: u16, oy: u16) {
  let x0 = ox.saturating_add(scroll_box.rect.x);
  let y0 = oy.saturating_add(scroll_box.rect.y);
  // Inside 和 ReserveSpace 都需要缩减内容区域，Overlay 不需要。
  let needs_reduction = scroll_box.scrollbar_layout != ScrollbarLayout::Overlay;
  let visible_width = if needs_reduction && shows_vertical_scrollbar(scroll_box) {
    scroll_box.rect.width.saturating_sub(1)
  } else {
    scroll_box.rect.width
  };
  let visible_height = if needs_reduction && shows_horizontal_scrollbar(scroll_box) {
    scroll_box.rect.height.saturating_sub(1)
  } else {
    scroll_box.rect.height
  };
  for y in 0..visible_height {
    for x in 0..visible_width {
      let sx = scroll_box.scroll_x.saturating_add(x);
      let sy = scroll_box.scroll_y.saturating_add(y);
      let px = x0.saturating_add(x);
      let py = y0.saturating_add(y);
      if !scroll_box.opaque && !scroll_box.buffer.is_written(sx, sy) {
        continue;
      }
      let Some(source) = scroll_box.buffer.get(sx, sy) else {
        write_cell(frame, px, py, &CanvasCell::blank());
        continue;
      };
      if source.is_continuation() {
        if x == 0 {
          if scroll_box.opaque {
            write_cell(frame, px, py, &CanvasCell::blank());
          }
        } else {
          write_cell(frame, px, py, source);
        }
        continue;
      }
      if is_clipped_wide_cell(source, sx, scroll_box.scroll_x, visible_width) {
        if scroll_box.opaque {
          write_cell(frame, px, py, &CanvasCell::blank());
        }
        continue;
      }
      write_cell(frame, px, py, source);
    }
  }
  // 垂直滚动条最后绘制（覆盖角落）。
  draw_horizontal_scrollbar(frame, scroll_box, x0, y0);
  draw_vertical_scrollbar(frame, scroll_box, x0, y0);
}

fn draw_vertical_scrollbar(
  frame: &mut ComposedFrame,
  scroll_box: &PreparedScrollBox,
  x0: u16,
  y0: u16,
) {
  let height = scroll_box.rect.height;
  if height == 0 || scroll_box.rect.width == 0 || !shows_vertical_scrollbar(scroll_box) {
    return;
  }
  let x = match scroll_box.scrollbar_layout {
    ScrollbarLayout::Overlay | ScrollbarLayout::Inside => {
      x0.saturating_add(scroll_box.rect.width - 1)
    }
    ScrollbarLayout::ReserveSpace => x0.saturating_add(scroll_box.rect.width),
  };
  let max_scroll = scroll_box.content_size.height.saturating_sub(height);
  let thumb_height = if max_scroll == 0 {
    height
  } else {
    ((height as u32 * height as u32) / scroll_box.content_size.height.max(1) as u32)
      .max(scroll_box.scrollbar_style.minimum_thumb_height as u32)
      .min(height as u32) as u16
  };
  let travel = height.saturating_sub(thumb_height);
  let thumb_y = if max_scroll == 0 {
    0
  } else {
    (scroll_box.scroll_y as u32 * travel as u32 / max_scroll as u32) as u16
  };
  for y in 0..height {
    let is_thumb = y >= thumb_y && y < thumb_y.saturating_add(thumb_height);
    let (ch, style) = if is_thumb {
      (
        scroll_box.scrollbar_style.thumb_char,
        scroll_box.scrollbar_style.thumb_style.clone(),
      )
    } else {
      (
        scroll_box.scrollbar_style.track_char,
        scroll_box.scrollbar_style.track_style.clone(),
      )
    };
    write_cell(
      frame,
      x,
      y0.saturating_add(y),
      &CanvasCell::styled(ch.to_string(), style),
    );
  }
}

fn draw_horizontal_scrollbar(
  frame: &mut ComposedFrame,
  scroll_box: &PreparedScrollBox,
  x0: u16,
  y0: u16,
) {
  let width = scroll_box.rect.width;
  if width == 0 || scroll_box.rect.height == 0 || !shows_horizontal_scrollbar(scroll_box) {
    return;
  }
  let y = match scroll_box.scrollbar_layout {
    ScrollbarLayout::Overlay | ScrollbarLayout::Inside => {
      y0.saturating_add(scroll_box.rect.height - 1)
    }
    ScrollbarLayout::ReserveSpace => y0.saturating_add(scroll_box.rect.height),
  };
  let max_scroll = scroll_box.content_size.width.saturating_sub(width);
  let thumb_width = if max_scroll == 0 {
    width
  } else {
    ((width as u32 * width as u32) / scroll_box.content_size.width.max(1) as u32)
      .max(scroll_box.scrollbar_style.minimum_thumb_height as u32)
      .min(width as u32) as u16
  };
  let travel = width.saturating_sub(thumb_width);
  let thumb_x = if max_scroll == 0 {
    0
  } else {
    (scroll_box.scroll_x as u32 * travel as u32 / max_scroll as u32) as u16
  };
  for x in 0..width {
    let is_thumb = x >= thumb_x && x < thumb_x.saturating_add(thumb_width);
    let (ch, style) = if is_thumb {
      (
        scroll_box.scrollbar_style.h_thumb_char,
        scroll_box.scrollbar_style.h_thumb_style.clone(),
      )
    } else {
      (
        scroll_box.scrollbar_style.h_track_char,
        scroll_box.scrollbar_style.h_track_style.clone(),
      )
    };
    write_cell(
      frame,
      x0.saturating_add(x),
      y,
      &CanvasCell::styled(ch.to_string(), style),
    );
  }
}

fn shows_vertical_scrollbar(scroll_box: &PreparedScrollBox) -> bool {
  match scroll_box.scrollbar.vertical {
    ScrollbarVisibility::Always => scroll_box.rect.height > 0,
    ScrollbarVisibility::Auto => scroll_box.content_size.height > scroll_box.rect.height,
    ScrollbarVisibility::Never => false,
  }
}

fn shows_horizontal_scrollbar(scroll_box: &PreparedScrollBox) -> bool {
  match scroll_box.scrollbar.horizontal {
    ScrollbarVisibility::Always => scroll_box.rect.width > 0,
    ScrollbarVisibility::Auto => scroll_box.content_size.width > scroll_box.rect.width,
    ScrollbarVisibility::Never => false,
  }
}

fn is_clipped_wide_cell(cell: &CanvasCell, sx: u16, scroll_x: u16, visible_width: u16) -> bool {
  let width = graphemes(&cell.text)
    .first()
    .map(|grapheme| grapheme.display_width)
    .unwrap_or(1);
  width > 1 && sx as usize + width > scroll_x as usize + visible_width as usize
}

fn write_cell(frame: &mut ComposedFrame, x: u16, y: u16, source: &CanvasCell) {
  let Some(lower) = frame.get(x, y) else {
    return;
  };
  let mut cell = source.clone();
  if cell.style.background == Some(TextColor::Transparent) {
    cell.style.background = match lower {
      ComposedCell::Text(lower) => lower.style.background.clone(),
      ComposedCell::Empty => None,
    };
  }
  frame.set(x, y, ComposedCell::Text(cell));
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::host_engine::services::{
    LayoutService, ScrollBoxOptions, ScrollBoxService, SliceLength, SliceOptions, SliceRect,
    SliceService, SurfaceId, TerminalColor, TextColor, TextStyle, UiObjectPool,
  };

  #[test]
  fn compose_copies_text() {
    let mut canvas = CanvasService::new();
    canvas.styled_text(1, 2, "a", TextStyle::default());

    let frame = FrameCompositor::new().compose(&canvas);

    assert!(matches!(
      frame.get(1, 2),
      Some(ComposedCell::Text(cell)) if cell.text == "a"
    ));
  }

  fn text(frame: &ComposedFrame, x: u16, y: u16) -> &str {
    match frame.get(x, y).unwrap() {
      ComposedCell::Text(cell) => &cell.text,
      ComposedCell::Empty => "",
    }
  }

  #[test]
  fn viewport_slices_and_host_compose_in_order() {
    let mut layout = LayoutService::new();
    layout.resize_physical(12, 6);
    layout.set_developer_viewport(crate::host_engine::services::Rect {
      x: 2,
      y: 1,
      width: 8,
      height: 4,
    });
    let slices = SliceService::new();
    let mut pool = UiObjectPool::new();
    let opaque = slices
      .create(
        &mut pool,
        SliceOptions {
          rect: SliceRect {
            x: 2,
            y: 1,
            width: SliceLength::Fixed(3),
            height: SliceLength::Fixed(2),
          },
          ..Default::default()
        },
      )
      .unwrap();
    let transparent = slices
      .create(
        &mut pool,
        SliceOptions {
          rect: SliceRect {
            x: 2,
            y: 1,
            width: SliceLength::Fixed(3),
            height: SliceLength::Fixed(2),
          },
          opaque: false,
          ..Default::default()
        },
      )
      .unwrap();
    let mut canvas = CanvasService::new();
    canvas.begin_frame(&layout);
    canvas.prepare(&pool, &layout);
    canvas.styled_text(2, 1, "B", TextStyle::default());
    canvas.styled_text_on(transparent, 1, 0, "T", TextStyle::default());
    canvas.host_styled_text(5, 2, "H", TextStyle::default());

    let frame = FrameCompositor::new().compose(&canvas);
    assert_eq!(text(&frame, 4, 2), " ");
    assert_eq!(text(&frame, 5, 2), "H");
    assert_eq!(text(&frame, 6, 2), " ");
    assert!(canvas.prepared_slice_rect(opaque).is_some());
  }

  #[test]
  fn later_slice_wins_and_wide_grapheme_is_not_split() {
    let mut layout = LayoutService::new();
    layout.resize_physical(5, 2);
    let service = SliceService::new();
    let mut pool = UiObjectPool::new();
    let options = SliceOptions {
      rect: SliceRect {
        x: 0,
        y: 0,
        width: SliceLength::Fixed(1),
        height: SliceLength::Fixed(1),
      },
      opaque: false,
      ..Default::default()
    };
    let a = service.create(&mut pool, options).unwrap();
    let b = service.create(&mut pool, options).unwrap();
    let mut canvas = CanvasService::new();
    canvas.begin_frame(&layout);
    canvas.prepare(&pool, &layout);
    canvas.styled_text_on(a, 0, 0, "A", TextStyle::default());
    canvas.styled_text_on(b, 0, 0, "B", TextStyle::default());
    canvas.styled_text_on(b, 0, 0, "我", TextStyle::default());
    let frame = FrameCompositor::new().compose(&canvas);
    assert_eq!(text(&frame, 0, 0), "B");
  }

  #[test]
  fn transparent_slice_explicit_space_inherits_lower_background() {
    let mut layout = LayoutService::new();
    layout.resize_physical(3, 1);
    let slices = SliceService::new();
    let mut pool = UiObjectPool::new();
    let slice = slices
      .create(
        &mut pool,
        SliceOptions {
          opaque: false,
          ..Default::default()
        },
      )
      .unwrap();
    let mut canvas = CanvasService::new();
    canvas.begin_frame(&layout);
    canvas.prepare(&pool, &layout);
    let background = TextColor::Terminal(TerminalColor::Blue);
    canvas.styled_text(
      0,
      0,
      "B",
      TextStyle {
        background: Some(background.clone()),
        ..Default::default()
      },
    );
    canvas.styled_text_on(
      slice,
      0,
      0,
      " ",
      TextStyle {
        background: Some(TextColor::Transparent),
        ..Default::default()
      },
    );

    let frame = FrameCompositor::new().compose(&canvas);
    let ComposedCell::Text(cell) = frame.get(0, 0).unwrap() else {
      panic!("expected text cell")
    };
    assert_eq!(cell.text, " ");
    assert_eq!(cell.style.background, Some(background));
  }

  #[test]
  fn slice_state_changes_apply_on_next_prepare() {
    let mut layout = LayoutService::new();
    layout.resize_physical(3, 1);
    let slices = SliceService::new();
    let mut pool = UiObjectPool::new();
    let slice = slices.create(&mut pool, SliceOptions::default()).unwrap();
    let mut canvas = CanvasService::new();
    canvas.begin_frame(&layout);
    canvas.prepare(&pool, &layout);
    slices.set_visible(&mut pool, slice, false);
    assert!(canvas.styled_text_on(slice, 0, 0, "A", TextStyle::default()));
    assert_eq!(text(&FrameCompositor::new().compose(&canvas), 0, 0), "A");

    canvas.begin_frame(&layout);
    canvas.prepare(&pool, &layout);
    assert!(!canvas.styled_text_on(slice, 0, 0, "B", TextStyle::default()));
    assert_eq!(text(&FrameCompositor::new().compose(&canvas), 0, 0), " ");
  }

  #[test]
  fn scroll_box_clips_content_by_scroll_y_and_draws_scrollbar() {
    let mut layout = LayoutService::new();
    layout.resize_physical(8, 4);
    let service = ScrollBoxService::new();
    let mut pool = UiObjectPool::new();
    let id = service
      .create(
        &mut pool,
        ScrollBoxOptions {
          rect: crate::host_engine::services::Rect {
            x: 0,
            y: 0,
            width: 6,
            height: 2,
          },
          content_width: 6,
          content_height: 4,
          ..Default::default()
        },
      )
      .unwrap();
    service.scroll_to(&mut pool, id, 0, 1, &layout);
    let mut canvas = CanvasService::new();
    canvas.begin_frame(&layout);
    canvas.prepare(&pool, &layout);
    canvas.styled_text_in_scroll_box(id, 0, 0, "row0", TextStyle::default());
    canvas.styled_text_in_scroll_box(id, 0, 1, "row1", TextStyle::default());
    canvas.styled_text_in_scroll_box(id, 0, 2, "row2", TextStyle::default());

    let frame = FrameCompositor::new().compose(&canvas);

    assert_eq!(text(&frame, 0, 0), "r");
    assert_eq!(text(&frame, 3, 0), "1");
    assert_eq!(text(&frame, 3, 1), "2");
    assert_eq!(text(&frame, 5, 0), "█");
  }

  #[test]
  fn scroll_box_preserves_wide_character_continuations() {
    let mut layout = LayoutService::new();
    layout.resize_physical(4, 1);
    let service = ScrollBoxService::new();
    let mut pool = UiObjectPool::new();
    let id = service
      .create(
        &mut pool,
        ScrollBoxOptions {
          rect: crate::host_engine::services::Rect {
            x: 0,
            y: 0,
            width: 4,
            height: 1,
          },
          content_width: 4,
          content_height: 1,
          ..Default::default()
        },
      )
      .unwrap();
    let mut canvas = CanvasService::new();
    canvas.begin_frame(&layout);
    canvas.prepare(&pool, &layout);
    canvas.styled_text_in_scroll_box(id, 0, 0, "中文", TextStyle::default());

    let frame = FrameCompositor::new().compose(&canvas);
    assert_eq!(text(&frame, 0, 0), "中");
    assert!(matches!(
      frame.get(1, 0),
      Some(ComposedCell::Text(cell)) if cell.is_continuation()
    ));
    assert_eq!(text(&frame, 2, 0), "文");
    assert!(matches!(
      frame.get(3, 0),
      Some(ComposedCell::Text(cell)) if cell.is_continuation()
    ));
  }

  #[test]
  fn scroll_box_and_slice_share_surface_order() {
    let mut layout = LayoutService::new();
    layout.resize_physical(4, 2);
    let slices = SliceService::new();
    let scroll = ScrollBoxService::new();
    let mut pool = UiObjectPool::new();
    let slice = slices.create(&mut pool, SliceOptions::default()).unwrap();
    let box_id = scroll
      .create(
        &mut pool,
        ScrollBoxOptions {
          rect: crate::host_engine::services::Rect {
            x: 0,
            y: 0,
            width: 1,
            height: 1,
          },
          content_width: 1,
          content_height: 1,
          ..Default::default()
        },
      )
      .unwrap();
    scroll.move_below(&mut pool, box_id, SurfaceId::Slice(slice));
    let mut canvas = CanvasService::new();
    canvas.begin_frame(&layout);
    canvas.prepare(&pool, &layout);
    canvas.styled_text_on(slice, 0, 0, "S", TextStyle::default());
    canvas.styled_text_in_scroll_box(box_id, 0, 0, "B", TextStyle::default());

    assert_eq!(text(&FrameCompositor::new().compose(&canvas), 0, 0), "S");

    scroll.move_above(&mut pool, box_id, SurfaceId::Slice(slice));
    canvas.begin_frame(&layout);
    canvas.prepare(&pool, &layout);
    canvas.styled_text_on(slice, 0, 0, "S", TextStyle::default());
    canvas.styled_text_in_scroll_box(box_id, 0, 0, "B", TextStyle::default());

    assert_eq!(text(&FrameCompositor::new().compose(&canvas), 0, 0), "B");
  }
}
