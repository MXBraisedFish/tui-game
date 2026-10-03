//! Owned application event buffering between service completion and host dispatch.

use std::collections::VecDeque;

use super::async_runtime::EngineEvent;

/// An ordered queue of owned service completion events awaiting application routing.
pub struct EngineEventQueue {
  events: VecDeque<EngineEvent>,
}

impl EngineEventQueue {
  /// Create an engine event queue with its initial state.
  pub fn new() -> Self {
    Self {
      events: VecDeque::new(),
    }
  }

  /// Append an owned completion event to the delivery queue.
  pub fn push(&mut self, event: EngineEvent) {
    self.events.push_back(event);
  }

  /// Append completion events in the order supplied by the iterator.
  pub fn extend(&mut self, events: impl IntoIterator<Item = EngineEvent>) {
    self.events.extend(events);
  }

  /// Return the current drain.
  pub fn drain(&mut self) -> Vec<EngineEvent> {
    self.events.drain(..).collect()
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
  #[test]
  fn engine_event_queue_drains_in_order() {
    let mut queue = EngineEventQueue::new();
    queue.push(EngineEvent::TaskFinished);
    queue.push(EngineEvent::TaskFinished);

    let events = queue.drain();

    assert!(matches!(events[0], EngineEvent::TaskFinished));
    assert!(matches!(events[1], EngineEvent::TaskFinished));
    assert!(queue.drain().is_empty());
  }
}
