//! Owned component state and per-frame hit-test caches.

use std::collections::HashMap;

use super::{TableId, TableOptions};

/// The retained state of table.
///
/// # Fields
///
/// * `options` - The table options carried by this table state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TableState {
  /// The table options carried by this table state.
  pub options: TableOptions,
}

/// The collection of owned table instances and their queued events.
///
/// # Fields
///
/// * `next_id` - The identifier of the next.
/// * `tables` - The tables indexed by their declared keys.
pub(crate) struct TableObjects {
  /// The identifier of the next.
  pub next_id: u64,
  /// The tables indexed by their declared keys.
  pub tables: HashMap<TableId, TableState>,
}

impl TableObjects {
  /// Create a table objects with its initial state.
  pub(crate) fn new() -> Self {
    Self {
      next_id: 1,
      tables: HashMap::new(),
    }
  }
}
