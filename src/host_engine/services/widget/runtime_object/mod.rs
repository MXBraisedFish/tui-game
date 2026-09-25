use crate::host_engine::services::random::RandomGeneratorObjects;
use crate::host_engine::services::time::TimeObjects;
use crate::host_engine::services::animation::AnimationObjects;

/// 运行时对象池，存储非 UI 组件的宿主托管对象
pub struct RuntimeObjectPool {
  pub(crate) time: TimeObjects,
  pub(crate) random_generators: RandomGeneratorObjects,
  pub(crate) animation: AnimationObjects,
}

impl RuntimeObjectPool {
  pub fn new() -> Self {
    Self {
      time: TimeObjects::new(),
      random_generators: RandomGeneratorObjects::new(),
      animation: AnimationObjects::new(),
    }
  }
}

/// 运行时对象池持有者 trait
pub trait RuntimeObjectPoolOwner {
  fn runtime_objects(&self) -> &RuntimeObjectPool;
  fn runtime_objects_mut(&mut self) -> &mut RuntimeObjectPool;
}
