use std::collections::HashMap;

use super::types::{ProgressBarId, ProgressBarOptions};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ProgressBarState {
  pub options: ProgressBarOptions,
  pub completed: f32,
  pub preview: f32,
}

pub(crate) struct ProgressBarObjects {
  pub next_id: u64,
  pub bars: HashMap<ProgressBarId, ProgressBarState>,
}

impl ProgressBarObjects {
  pub(crate) fn new() -> Self {
    Self {
      next_id: 1,
      bars: HashMap::new(),
    }
  }
}
