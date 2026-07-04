use std::collections::VecDeque;

use super::async_runtime::EngineEvent;

pub struct EngineEventQueue {
  events: VecDeque<EngineEvent>,
}

impl EngineEventQueue {
  pub fn new() -> Self {
    Self {
      events: VecDeque::new(),
    }
  }

  pub fn push(&mut self, event: EngineEvent) {
    self.events.push_back(event);
  }

  pub fn extend(&mut self, events: impl IntoIterator<Item = EngineEvent>) {
    self.events.extend(events);
  }

  pub fn drain(&mut self) -> Vec<EngineEvent> {
    self.events.drain(..).collect()
  }

  pub fn is_empty(&self) -> bool {
    self.events.is_empty()
  }
}

impl Default for EngineEventQueue {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::host_engine::services::async_runtime::{EngineEvent, TaskId};

  #[test]
  fn engine_event_queue_drains_in_order() {
    let mut queue = EngineEventQueue::new();
    queue.push(EngineEvent::TaskFinished { id: TaskId(1) });
    queue.push(EngineEvent::TaskFinished { id: TaskId(2) });

    let events = queue.drain();

    assert!(matches!(
      events[0],
      EngineEvent::TaskFinished { id: TaskId(1) }
    ));
    assert!(matches!(
      events[1],
      EngineEvent::TaskFinished { id: TaskId(2) }
    ));
    assert!(queue.is_empty());
  }
}
