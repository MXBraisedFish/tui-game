use crate::host_engine::services::{UiObjectPool, UiObjectPoolOwner};

/// 游戏列表 UI（占位组件，尚未实现具体功能）。
pub struct GameListUi {
  objects: UiObjectPool,
}

impl GameListUi {

  /// 初始化游戏列表 UI。
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
