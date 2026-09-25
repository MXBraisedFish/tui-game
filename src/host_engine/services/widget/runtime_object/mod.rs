use crate::host_engine::services::random::RandomGeneratorObjects;
use crate::host_engine::services::time::TimeObjects;
use crate::host_engine::services::animation::{
  AnimationPool, AnimationTarget, AnimationValuePool, CharacterEffectPool,
};

/// 运行时对象池，存储非 UI 组件的宿主托管对象
pub struct RuntimeObjectPool {
  pub(crate) time: TimeObjects,
  pub(crate) random_generators: RandomGeneratorObjects,
  pub(crate) animations: AnimationPool,
  pub(crate) animation_values: AnimationValuePool,
  pub(crate) character_effects: CharacterEffectPool,
}

impl RuntimeObjectPool {
  pub fn new() -> Self {
    Self {
      time: TimeObjects::new(),
      random_generators: RandomGeneratorObjects::new(),
      animations: AnimationPool::new(),
      animation_values: AnimationValuePool::new(),
      character_effects: CharacterEffectPool::new(),
    }
  }

  pub(crate) fn remove_animations_targeting(&mut self, target: AnimationTarget) {
    let ids = self
      .animations
      .ids()
      .into_iter()
      .filter(|id| {
        self.animations.get(*id).is_some_and(|playback| {
          playback.owner == crate::host_engine::services::animation::AnimationOwner::Object(target)
            || playback
              .bindings
              .iter()
              .any(|binding| binding.target == target)
        })
      })
      .collect::<Vec<_>>();
    for id in ids {
      self.animations.remove(id);
      self.animations.events.retain(|event| event.id != id);
      self
        .animations
        .callback_requests
        .retain(|request| request.event.id != id);
    }
  }
}

/// 运行时对象池持有者 trait
pub trait RuntimeObjectPoolOwner {
  fn runtime_objects(&self) -> &RuntimeObjectPool;
  fn runtime_objects_mut(&mut self) -> &mut RuntimeObjectPool;
}
