use std::collections::HashSet;

use super::{KeyboardInputEvent, KeyboardInputKind, PhysicalKey};

pub struct KeyboardFrameState {
  held_keys: HashSet<PhysicalKey>,
  pressed_keys: HashSet<PhysicalKey>,
  released_keys: HashSet<PhysicalKey>,
  repeated_keys: HashSet<PhysicalKey>,
}

impl KeyboardFrameState {
  pub fn new() -> Self {
    Self {
      held_keys: HashSet::new(),
      pressed_keys: HashSet::new(),
      released_keys: HashSet::new(),
      repeated_keys: HashSet::new(),
    }
  }

  pub fn clear(&mut self) {
    self.held_keys.clear();
    self.pressed_keys.clear();
    self.released_keys.clear();
    self.repeated_keys.clear();
  }

  pub fn begin_frame(&mut self) {
    self.pressed_keys.clear();
    self.released_keys.clear();
    self.repeated_keys.clear();
  }

  pub fn apply_event(&mut self, event: KeyboardInputEvent) {
    match event.kind {
      KeyboardInputKind::Press => {
        self.held_keys.insert(event.key);
        self.pressed_keys.insert(event.key);
      }
      KeyboardInputKind::Release => {
        self.held_keys.remove(&event.key);
        self.released_keys.insert(event.key);
      }
      KeyboardInputKind::Repeat => {
        self.held_keys.insert(event.key);
        self.repeated_keys.insert(event.key);
      }
    }
  }

  pub fn is_held(&self, key: PhysicalKey) -> bool {
    self.held_keys.contains(&key)
  }

  pub fn was_pressed(&self, key: PhysicalKey) -> bool {
    self.pressed_keys.contains(&key)
  }

  pub fn was_released(&self, key: PhysicalKey) -> bool {
    self.released_keys.contains(&key)
  }

  pub fn was_repeated(&self, key: PhysicalKey) -> bool {
    self.repeated_keys.contains(&key)
  }
}
