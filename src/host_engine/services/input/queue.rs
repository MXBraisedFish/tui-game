use std::collections::VecDeque;

use super::InputEvent;

pub struct InputEventQueue {
  queue: VecDeque<InputEvent>,
}

impl InputEventQueue {
  pub fn new() -> Self {
    Self {
      queue: VecDeque::new(),
    }
  }

  pub fn push(&mut self, event: InputEvent) {
    self.queue.push_back(event);
  }

  pub fn pop(&mut self) -> Option<InputEvent> {
    self.queue.pop_front()
  }

  pub fn is_empty(&self) -> bool {
    self.queue.is_empty()
  }

  pub fn len(&self) -> usize {
    self.queue.len()
  }

  pub fn clear(&mut self) {
    self.queue.clear();
  }
}
