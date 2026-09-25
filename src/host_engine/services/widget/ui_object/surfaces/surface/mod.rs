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
