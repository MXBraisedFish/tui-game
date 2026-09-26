use super::{ComposedCell, ComposedFrame};
use tg_service_canvas::buffer::CanvasBuffer;
use tg_service_canvas::{PreparedScrollBox, PreparedSurface};
use tg_core_unicode::graphemes;
use tg_core_style::{CanvasCell, TextColor};
use tg_service_canvas::CanvasService;

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
        PreparedSurface::Slice(slice) if slice.visible => overlay_slice(
          &mut frame,
          slice,
          viewport.x.saturating_add(slice.rect.x),
          viewport.y.saturating_add(slice.rect.y),
        ),
        PreparedSurface::ScrollBox(scroll_box) if scroll_box.visible => {
          overlay_scroll_box(&mut frame, scroll_box, viewport.x, viewport.y)
        }
        _ => {}
      }
    }
    overlay(&mut frame, host, 0, 0, false);
    overlay(&mut frame, canvas.top_buffer(), 0, 0, false);

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

fn overlay_slice(
  frame: &mut ComposedFrame,
  slice: &tg_service_canvas::PreparedSlice,
  ox: u16,
  oy: u16,
) {
  for y in 0..slice.buffer.height() {
    for x in 0..slice.buffer.width() {
      if !slice.opaque && !slice.buffer.is_written(x, y) {
        continue;
      }
      let Some(source) = slice.buffer.get(x, y) else {
        continue;
      };
      let mut source = source.clone();
      if source.style.background.is_none() {
        source.style.background = slice.background.clone();
      }
      write_cell(frame, ox.saturating_add(x), oy.saturating_add(y), &source);
    }
  }
}

fn overlay_scroll_box(frame: &mut ComposedFrame, scroll_box: &PreparedScrollBox, ox: u16, oy: u16) {
  let content = scroll_box.layout.content_viewport_rect;
  let x0 = ox.saturating_add(content.x);
  let y0 = oy.saturating_add(content.y);
  let visible_width = content.width;
  let visible_height = content.height;
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
  draw_horizontal_scrollbar(frame, scroll_box, ox, oy);
  draw_vertical_scrollbar(frame, scroll_box, ox, oy);
}

fn draw_vertical_scrollbar(
  frame: &mut ComposedFrame,
  scroll_box: &PreparedScrollBox,
  ox: u16,
  oy: u16,
) {
  let Some(track) = scroll_box.layout.vertical_track_rect else {
    return;
  };
  let thumb = scroll_box.layout.vertical_thumb_rect;
  for y in 0..track.height {
    let cell_y = track.y.saturating_add(y);
    let is_thumb =
      thumb.is_some_and(|thumb| cell_y >= thumb.y && cell_y < thumb.y.saturating_add(thumb.height));
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
      ox.saturating_add(track.x),
      oy.saturating_add(cell_y),
      &CanvasCell::styled(ch.to_string(), style),
    );
  }
}

fn draw_horizontal_scrollbar(
  frame: &mut ComposedFrame,
  scroll_box: &PreparedScrollBox,
  ox: u16,
  oy: u16,
) {
  let Some(track) = scroll_box.layout.horizontal_track_rect else {
    return;
  };
  let thumb = scroll_box.layout.horizontal_thumb_rect;
  for x in 0..track.width {
    let cell_x = track.x.saturating_add(x);
    let is_thumb =
      thumb.is_some_and(|thumb| cell_x >= thumb.x && cell_x < thumb.x.saturating_add(thumb.width));
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
      ox.saturating_add(cell_x),
      oy.saturating_add(track.y),
      &CanvasCell::styled(ch.to_string(), style),
    );
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
  use tg_core_style::{TerminalColor, TextColor, TextStyle};

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

}
