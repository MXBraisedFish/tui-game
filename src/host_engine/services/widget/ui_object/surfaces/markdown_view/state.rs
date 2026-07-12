use std::collections::HashMap;

use crate::host_engine::services::Rect;

use super::types::{MarkdownViewId, MarkdownViewOptions};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MarkdownViewState {
  pub options: MarkdownViewOptions,
  pub hits: Vec<MarkdownLinkHit>,
  pub pressed: Option<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MarkdownLinkHit {
  pub rect: Rect,
  pub order: u64,
  pub surface_rank: usize,
  pub href: String,
  pub text: String,
}

pub(crate) struct MarkdownViewObjects {
  pub next_id: u64,
  pub views: HashMap<MarkdownViewId, MarkdownViewState>,
}

impl MarkdownViewObjects {
  pub(crate) fn new() -> Self {
    Self {
      next_id: 1,
      views: HashMap::new(),
    }
  }

  pub(crate) fn clear_hits(&mut self) {
    for state in self.views.values_mut() {
      state.hits.clear();
      state.pressed = None;
    }
  }
}
