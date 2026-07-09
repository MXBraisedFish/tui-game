use std::collections::HashMap;

use super::{TableId, TableOptions};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TableState {
  pub options: TableOptions,
}

pub(crate) struct TableObjects {
  pub next_id: u64,
  pub tables: HashMap<TableId, TableState>,
}

impl TableObjects {
  pub(crate) fn new() -> Self {
    Self {
      next_id: 1,
      tables: HashMap::new(),
    }
  }
}
