use std::sync::atomic::{AtomicU64, Ordering};

use super::text_input::TextInputObjects;

static NEXT_POOL_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Debug)]
pub struct UiService;

impl UiService {
  pub fn new() -> Self {
    Self
  }
}

/// 页面级 UI 对象池。每个页面持有一个独立实例。
pub struct UiObjectPool {
  id: u64,
  pub(crate) text_inputs: TextInputObjects,
}

impl UiObjectPool {
  pub fn new() -> Self {
    Self {
      id: NEXT_POOL_ID.fetch_add(1, Ordering::Relaxed),
      text_inputs: TextInputObjects::new(),
    }
  }

  pub(crate) fn id(&self) -> u64 {
    self.id
  }
}

/// 所有有状态 Host UI 页面的对象池访问规范。
pub trait UiObjectPoolOwner {
  fn objects(&self) -> &UiObjectPool;
  fn objects_mut(&mut self) -> &mut UiObjectPool;
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn pools_have_distinct_internal_ids() {
    assert_ne!(UiObjectPool::new().id(), UiObjectPool::new().id());
  }
}
