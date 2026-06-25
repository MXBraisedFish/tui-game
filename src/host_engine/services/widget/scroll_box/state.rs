use std::collections::{HashMap, VecDeque};

use crate::host_engine::services::MouseButton;

use super::types::{ScrollBoxEvent, ScrollBoxId, ScrollBoxOptions, ScrollbarAxis};

/// 滚动条拖动状态（内部使用）。
#[derive(Clone, Copy, Debug)]
pub(crate) struct ScrollBoxDragState {
  pub scroll_box_id: ScrollBoxId,
  pub axis: ScrollbarAxis,
  pub button: MouseButton,
  /// 拖动开始时鼠标在屏幕上的物理坐标。
  pub drag_start_mouse: u16,
  /// 拖动开始时滑块在轨道内的本地位置（非滚动值，避免往返精度损失）。
  pub drag_start_thumb_pos: u16,
  /// 滑块尺寸。
  pub thumb_size: u16,
  /// 轨道尺寸。
  pub track_size: u16,
  /// 最大滚动值。
  pub max_scroll: u16,
}

impl ScrollBoxDragState {
  /// 根据当前鼠标位置计算新的滚动值。
  /// 始终以按下时的滑块位置和鼠标位置为锚点，避免累积整数除法误差。
  pub(crate) fn scroll_from_mouse(&self, mouse_pos: u16) -> u16 {
    let travel = self.track_size.saturating_sub(self.thumb_size);
    if travel == 0 {
      return 0;
    }
    // 鼠标相对于按下位置的偏移量 + 初始滑块位置。
    let thumb_pos = (mouse_pos as i32 - self.drag_start_mouse as i32
      + self.drag_start_thumb_pos as i32)
      .max(0)
      .min(travel as i32) as u16;
    (thumb_pos as u32 * self.max_scroll as u32 / travel as u32) as u16
  }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ScrollBoxState {
  pub options: ScrollBoxOptions,
  pub scroll_x: u16,
  pub scroll_y: u16,
}

pub(crate) struct ScrollBoxObjects {
  pub next_id: u64,
  pub boxes: HashMap<ScrollBoxId, ScrollBoxState>,
  pub(crate) events: VecDeque<ScrollBoxEvent>,
  pub(crate) drag: Option<ScrollBoxDragState>,
}

impl ScrollBoxObjects {
  pub(crate) fn new() -> Self {
    Self {
      next_id: 1,
      boxes: HashMap::new(),
      events: VecDeque::new(),
      drag: None,
    }
  }
}
