pub(crate) mod random;
pub(crate) mod time;

use self::random::RandomGeneratorObjects;
use self::time::{
  DelayTimerEvent, DelayTimerId, DelayTimerObjects, RepeatTimerEvent, RepeatTimerId,
  RepeatTimerObjects, TimeCallbackRequest, TimerEvent, TimerObjects,
};

/// 运行时对象池，存储非 UI 组件的宿主托管对象
pub struct RuntimeObjectPool {
  pub(crate) timers: TimerObjects,
  pub(crate) delay_timers: DelayTimerObjects,
  pub(crate) repeat_timers: RepeatTimerObjects,
  pub(crate) random_generators: RandomGeneratorObjects,
  pub(crate) time_callback_requests: Vec<TimeCallbackRequest>,
}

impl RuntimeObjectPool {
  pub fn new() -> Self {
    Self {
      timers: TimerObjects::new(),
      delay_timers: DelayTimerObjects::new(),
      repeat_timers: RepeatTimerObjects::new(),
      random_generators: RandomGeneratorObjects::new(),
      time_callback_requests: Vec::new(),
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

  pub(crate) fn clear_delay_timer_events(&mut self, id: DelayTimerId) {
    self.delay_timers.events.retain(|event| event.id() != id);
    self
      .time_callback_requests
      .retain(|request| request.delay_id() != Some(id));
  }

  pub(crate) fn take_delay_timer_events(&mut self, id: DelayTimerId) -> Vec<DelayTimerEvent> {
    let mut events = Vec::new();
    self.delay_timers.events.retain(|event| {
      if event.id() == id {
        events.push(*event);
        false
      } else {
        true
      }
    });
    events
  }

  pub(crate) fn clear_repeat_timer_events(&mut self, id: RepeatTimerId) {
    self.repeat_timers.events.retain(|event| event.id() != id);
    self
      .time_callback_requests
      .retain(|request| request.repeat_id() != Some(id));
  }

  pub(crate) fn take_repeat_timer_events(&mut self, id: RepeatTimerId) -> Vec<RepeatTimerEvent> {
    let mut events = Vec::new();
    self.repeat_timers.events.retain(|event| {
      if event.id() == id {
        events.push(*event);
        false
      } else {
        true
      }
    });
    events
  }

  pub(crate) fn take_time_callback_requests(&mut self) -> Vec<TimeCallbackRequest> {
    self.time_callback_requests.drain(..).collect()
  }
}

/// 运行时对象池持有者 trait
pub trait RuntimeObjectPoolOwner {
  fn runtime_objects(&self) -> &RuntimeObjectPool;
  fn runtime_objects_mut(&mut self) -> &mut RuntimeObjectPool;
}
