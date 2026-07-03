pub(crate) mod time;

use self::time::{TimerEvent, TimerObjects};

/// 运行时对象池，存储非 UI 组件的宿主托管对象
pub struct RuntimeObjectPool {
  pub(crate) timers: TimerObjects,
}

impl RuntimeObjectPool {
  pub fn new() -> Self {
    Self {
      timers: TimerObjects::new(),
    }
  }

  pub(crate) fn clear_timer_events(&mut self, id: time::TimerId) {
    self.timers.events.retain(|event| event.id() != id);
  }

  pub(crate) fn take_timer_events(&mut self, id: time::TimerId) -> Vec<TimerEvent> {
    let mut events = Vec::new();
    self.timers.events.retain(|event| {
      if event.id() == id {
        events.push(*event);
        false
      } else {
        true
      }
    });
    events
  }
}

/// 运行时对象池持有者 trait
pub trait RuntimeObjectPoolOwner {
  fn runtime_objects(&self) -> &RuntimeObjectPool;
  fn runtime_objects_mut(&mut self) -> &mut RuntimeObjectPool;
}
