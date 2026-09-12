use std::cell::RefCell;
use std::rc::{Rc, Weak};

use crate::host_engine::services::{
  RuntimeObjectPool, RuntimeObjectPoolOwner, SliceService, UiObjectPool, UiObjectPoolOwner,
};

pub(crate) type SharedLuaObjectPool = Rc<RefCell<Option<LuaObjectPool>>>;
pub(crate) type WeakLuaObjectPool = Weak<RefCell<Option<LuaObjectPool>>>;

pub(crate) fn shared_lua_object_pool() -> SharedLuaObjectPool {
  Rc::new(RefCell::new(Some(LuaObjectPool::new())))
}

/// 单个 Lua Session 拥有的全部宿主管理对象。
///
/// UI 对象与非 UI 运行时对象使用现有宿主对象池，因而继续沿用各服务的
/// 显式 ID API。Game 与 Screensaver Session 各自持有一个实例，彼此隔离。
pub struct LuaObjectPool {
  ui: UiObjectPool,
  runtime: RuntimeObjectPool,
}

impl LuaObjectPool {
  pub fn new() -> Self {
    Self {
      ui: UiObjectPool::new(),
      runtime: RuntimeObjectPool::new(),
    }
  }

  pub fn ui(&self) -> &UiObjectPool {
    &self.ui
  }

  pub fn ui_mut(&mut self) -> &mut UiObjectPool {
    &mut self.ui
  }

  pub fn runtime(&self) -> &RuntimeObjectPool {
    &self.runtime
  }

  pub fn runtime_mut(&mut self) -> &mut RuntimeObjectPool {
    &mut self.runtime
  }

  /// 开始新的宿主帧，清除仅对上一帧有效的 Lua UI 提交状态。
  pub(crate) fn begin_frame(&mut self) {
    SliceService::new().begin_frame(&mut self.ui);
  }
}

impl Default for LuaObjectPool {
  fn default() -> Self {
    Self::new()
  }
}

impl UiObjectPoolOwner for LuaObjectPool {
  fn objects(&self) -> &UiObjectPool {
    self.ui()
  }

  fn objects_mut(&mut self) -> &mut UiObjectPool {
    self.ui_mut()
  }
}

impl RuntimeObjectPoolOwner for LuaObjectPool {
  fn runtime_objects(&self) -> &RuntimeObjectPool {
    self.runtime()
  }

  fn runtime_objects_mut(&mut self) -> &mut RuntimeObjectPool {
    self.runtime_mut()
  }
}
