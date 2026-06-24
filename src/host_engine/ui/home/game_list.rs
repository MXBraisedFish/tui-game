use crate::host_engine::services::{UiObjectPool, UiObjectPoolOwner};

pub struct GameListUi {
  objects: UiObjectPool,
}

impl GameListUi {
  pub fn init() -> Self {
    Self {
      objects: UiObjectPool::new(),
    }
  }
}

impl UiObjectPoolOwner for GameListUi {
  fn objects(&self) -> &UiObjectPool {
    &self.objects
  }

  fn objects_mut(&mut self) -> &mut UiObjectPool {
    &mut self.objects
  }
}
