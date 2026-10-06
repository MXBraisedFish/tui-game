//! Owned component state and per-frame hit-test caches.

use std::collections::HashMap;
use std::time::Instant;

use tg_service_layout::Rect;

use super::buffer::TextBuffer;
use super::types::{TextInputId, TextInputMode};

/// A retained copy of hit data for later inspection or replay.
///
/// # Fields
///
/// * `rect` - The rectangular region in terminal cells.
/// * `origin` - The origin.
/// * `surface_rank` - The surface rank.
/// * `width` - The width in terminal columns.
/// * `first_line` - The first line.
/// * `single_start` - The single start.
/// * `order` - The order.
#[derive(Clone, Copy)]
pub(super) struct HitSnapshot {
  /// The rectangular region in terminal cells.
  pub rect: Rect,
  /// The origin.
  pub origin: (i32, i32),
  /// The surface rank.
  pub surface_rank: usize,
  /// The width in terminal columns.
  pub width: usize,
  /// The first line.
  pub first_line: usize,
  /// The single start.
  pub single_start: usize,
  /// The order.
  pub order: u64,
}

/// The retained state of text input.
///
/// # Fields
///
/// * `buffer` - The buffer.
/// * `mode` - The text input mode carried by this text input state.
/// * `mouse` - Whether this input accepts pointer-driven focus and selection.
/// * `hit` - The hit.
/// * `pending_cursor` - The pending cursor.
/// * `visual_line` - The visual line.
pub(super) struct TextInputState {
  /// The buffer.
  pub buffer: TextBuffer,
  /// The text input mode carried by this text input state.
  pub mode: TextInputMode,
  /// Whether this input accepts pointer-driven focus and selection.
  pub mouse: bool,
  /// The hit.
  pub hit: Option<HitSnapshot>,
  /// The pending cursor.
  pub pending_cursor: Option<(usize, usize)>,
  /// The visual line.
  pub visual_line: Option<usize>,
}

/// The collection of owned text input instances and their queued events.
///
/// # Fields
///
/// * `next_input_id` - The identifier of the next input.
/// * `inputs` - The inputs indexed by their declared keys.
pub(crate) struct TextInputObjects {
  /// The identifier of the next input.
  pub(super) next_input_id: u64,
  /// The inputs indexed by their declared keys.
  pub(super) inputs: HashMap<TextInputId, TextInputState>,
}

impl TextInputObjects {
  /// Create a text input objects with its initial state.
  pub(crate) fn new() -> Self {
    Self {
      next_input_id: 1,
      inputs: HashMap::new(),
    }
  }

  /// Discard hit rectangles from the previous drawing pass.
  pub(crate) fn clear_hits(&mut self) {
    for state in self.inputs.values_mut() {
      state.hit = None;
    }
  }
}

/// The active text input representation used by this module.
///
/// # Fields
///
/// * `pool_id` - The identity of the owning pool.
/// * `input_id` - The identifier of the input.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ActiveTextInput {
  /// The identity of the owning pool.
  pub pool_id: u64,
  /// The identifier of the input.
  pub input_id: TextInputId,
}

/// The text input active representation used by this module.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum TextInputActive {
  /// The inactive setting for text input active.
  #[default]
  Inactive,
  /// The focused setting for text input active.
  Focused(ActiveTextInput),
}

/// The drag selection representation used by this module.
///
/// # Fields
///
/// * `active` - The active.
/// * `last_scroll` - The last scroll.
pub(super) struct DragSelection {
  /// The active.
  pub active: ActiveTextInput,
  /// The last scroll.
  pub last_scroll: Instant,
}
