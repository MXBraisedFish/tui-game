//! Owned component state and per-frame hit-test caches.

use std::collections::HashMap;

use super::types::{ProgressBarId, ProgressBarOptions};

/// The retained state of progress bar.
///
/// # Fields
///
/// * `options` - The progress bar options carried by this progress bar state.
/// * `completed` - The completed.
/// * `preview` - The preview.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ProgressBarState {
  /// The progress bar options carried by this progress bar state.
  pub options: ProgressBarOptions,
  /// The completed.
  pub completed: f32,
  /// The preview.
  pub preview: f32,
}

/// The collection of owned progress bar instances and their queued events.
///
/// # Fields
///
/// * `next_id` - The identifier of the next.
/// * `bars` - The bars indexed by their declared keys.
pub(crate) struct ProgressBarObjects {
  /// The identifier of the next.
  pub next_id: u64,
  /// The bars indexed by their declared keys.
  pub bars: HashMap<ProgressBarId, ProgressBarState>,
}

impl ProgressBarObjects {
  /// Create a progress bar objects with its initial state.
  pub(crate) fn new() -> Self {
    Self {
      next_id: 1,
      bars: HashMap::new(),
    }
  }
}
