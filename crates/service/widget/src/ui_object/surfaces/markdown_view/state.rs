//! Owned component state and per-frame hit-test caches.

use std::collections::HashMap;

use tg_service_layout::Rect;

use super::types::{MarkdownViewId, MarkdownViewOptions};

/// The retained state of markdown view.
///
/// # Fields
///
/// * `options` - The markdown view options carried by this markdown view state.
/// * `hits` - The ordered hits retained by this owner.
/// * `pressed` - The pressed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MarkdownViewState {
  /// The markdown view options carried by this markdown view state.
  pub options: MarkdownViewOptions,
  /// The ordered hits retained by this owner.
  pub hits: Vec<MarkdownLinkHit>,
  /// The pressed.
  pub pressed: Option<usize>,
}

/// The markdown link hit representation used by this module.
///
/// # Fields
///
/// * `rect` - The rectangular region in terminal cells.
/// * `order` - The order.
/// * `surface_rank` - The surface rank.
/// * `href` - The href.
/// * `text` - The text to process or display.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MarkdownLinkHit {
  /// The rectangular region in terminal cells.
  pub rect: Rect,
  /// The order.
  pub order: u64,
  /// The surface rank.
  pub surface_rank: usize,
  /// The href.
  pub href: String,
  /// The text to process or display.
  pub text: String,
}

/// The collection of owned markdown view instances and their queued events.
///
/// # Fields
///
/// * `next_id` - The identifier of the next.
/// * `views` - The views indexed by their declared keys.
pub(crate) struct MarkdownViewObjects {
  /// The identifier of the next.
  pub next_id: u64,
  /// The views indexed by their declared keys.
  pub views: HashMap<MarkdownViewId, MarkdownViewState>,
}

impl MarkdownViewObjects {
  /// Create a markdown view objects with its initial state.
  pub(crate) fn new() -> Self {
    Self {
      next_id: 1,
      views: HashMap::new(),
    }
  }

  /// Discard hit rectangles from the previous drawing pass.
  pub(crate) fn clear_hits(&mut self) {
    for state in self.views.values_mut() {
      state.hits.clear();
      state.pressed = None;
    }
  }
}
