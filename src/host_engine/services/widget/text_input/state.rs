use std::collections::HashMap;
use std::time::Instant;

use crate::host_engine::services::Rect;

use super::buffer::TextBuffer;
use super::types::{TextInputId, TextInputMode};

#[derive(Clone, Copy)]
pub(super) struct HitSnapshot {
  pub rect: Rect,
  pub origin: (u16, u16),
  pub surface_rank: usize,
  pub width: usize,
  pub first_line: usize,
  pub single_start: usize,
  pub order: u64,
}

pub(super) struct TextInputState {
  pub buffer: TextBuffer,
  pub mode: TextInputMode,
  pub mouse: bool,
  pub hit: Option<HitSnapshot>,
  pub pending_cursor: Option<(usize, usize)>,
  pub visual_line: Option<usize>,
}

/// UI 对象池中文本输入组件的集合。
pub(crate) struct TextInputObjects {
  pub(super) next_input_id: u64,
  pub(super) inputs: HashMap<TextInputId, TextInputState>,
}

impl TextInputObjects {
  pub(crate) fn new() -> Self {
    Self {
      next_input_id: 1,
      inputs: HashMap::new(),
    }
  }

  /// 清除所有输入组件的命中区域缓存（每帧开始时调用）。
  pub(crate) fn clear_hits(&mut self) {
    for state in self.inputs.values_mut() {
      state.hit = None;
    }
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ActiveTextInput {
  pub pool_id: u64,
  pub input_id: TextInputId,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum TextInputActive {
  #[default]
  Inactive,
  Focused(ActiveTextInput),
}

pub(super) struct DragSelection {
  pub active: ActiveTextInput,
  pub last_scroll: Instant,
}
