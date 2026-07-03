use crate::host_engine::services::{
  RuntimeObjectPool, RuntimeObjectPoolOwner, UiObjectPool, UiObjectPoolOwner,
};

/// 游戏列表 UI（占位组件，尚未实现具体功能）。
pub struct GameListUi {
  objects: UiObjectPool,
  runtime_objects: RuntimeObjectPool,
}

impl GameListUi {
  /// 初始化游戏列表 UI。
  pub fn init() -> Self {
    Self {
      objects: UiObjectPool::new(),
      runtime_objects: RuntimeObjectPool::new(),
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

impl RuntimeObjectPoolOwner for GameListUi {
  fn runtime_objects(&self) -> &RuntimeObjectPool {
    &self.runtime_objects
  }

  fn runtime_objects_mut(&mut self) -> &mut RuntimeObjectPool {
    &mut self.runtime_objects
  }
}
