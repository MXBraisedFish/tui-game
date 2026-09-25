use super::scroll_box::resolve_scroll_box_layout;
use super::slice::resolve_rect;
use crate::host_engine::services::ui::UiObjectPool;
use crate::host_engine::services::canvas::{ScrollBoxFrame, SliceFrame, SurfaceFrame};
use crate::host_engine::services::{CanvasService, LayoutService, Size};

pub use crate::host_engine::services::canvas::SurfaceId;

impl UiObjectPool {
  /// 按叠放顺序生成本帧全部绘制面的描述，交给画布预处理缓冲区。
  pub fn prepare_canvas(&self, canvas: &mut CanvasService, layout: &LayoutService) {
    let frames = self
      .surfaces
      .iter()
      .filter_map(|surface| match *surface {
        SurfaceId::Slice(id) => {
          let state = self.slices.slices.get(&id)?;
          Some(SurfaceFrame::Slice(SliceFrame {
            id,
            rect: resolve_rect(state.rect, layout),
            visible: state.visible && (!state.frame_scoped || state.drawn_this_frame),
            opaque: state.opaque,
            background: state.background.clone(),
          }))
        }
        SurfaceId::ScrollBox(id) => {
          let state = self.scroll_boxes.boxes.get(&id)?;
          let options = &state.options;
          Some(SurfaceFrame::ScrollBox(ScrollBoxFrame {
            id,
            layout: resolve_scroll_box_layout(state, layout.developer_size()),
            content_size: Size {
              width: options.content_width,
              height: options.content_height,
            },
            scroll_x: state.scroll_x,
            scroll_y: state.scroll_y,
            visible: options.visible,
            opaque: options.opaque,
            scrollbar_style: options.scrollbar_style.clone(),
          }))
        }
      })
      .collect();
    canvas.prepare(self.id(), frames, layout);
  }
}

#[cfg(test)]
mod tests {
  use crate::host_engine::services::{
    CanvasService, LayoutService, Overflow, Rect, ScrollBoxOptions, ScrollBoxService,
    ScrollbarPolicy, ScrollbarVisibility, Size, SliceLength, SliceOptions, SliceRect, SliceService,
    UiObjectPool,
  };

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
    pool.prepare_canvas(&mut canvas, &layout);

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
  fn frame_scoped_slice_is_prepared_only_after_current_frame_draw() {
    let mut layout = LayoutService::new();
    layout.resize_physical(20, 10);
    let mut pool = UiObjectPool::new();
    let service = SliceService::new();
    let slice = service
      .create(
        &mut pool,
        SliceOptions {
          rect: SliceRect {
            x: 0,
            y: 0,
            width: SliceLength::Fixed(5),
            height: SliceLength::Fixed(3),
          },
          ..Default::default()
        },
      )
      .unwrap();
    assert!(service.set_frame_scoped(&mut pool, slice, true));
    let mut canvas = CanvasService::new();

    canvas.begin_frame(&layout);
    pool.prepare_canvas(&mut canvas, &layout);
    assert_eq!(canvas.prepared_slice_rect(slice), None);

    assert!(service.draw(&mut pool, slice, 2, 1));
    pool.prepare_canvas(&mut canvas, &layout);
    assert_eq!(
      canvas.prepared_slice_rect(slice),
      Some(Rect {
        x: 2,
        y: 1,
        width: 5,
        height: 3,
      })
    );

    service.begin_frame(&mut pool);
    pool.prepare_canvas(&mut canvas, &layout);
    assert_eq!(canvas.prepared_slice_rect(slice), None);
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
    pool.prepare_canvas(&mut canvas, &layout);

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
    pool.prepare_canvas(&mut canvas, &layout);

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
}
