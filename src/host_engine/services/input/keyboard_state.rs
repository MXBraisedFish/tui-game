use std::collections::HashSet;

use crossterm::event::KeyCode;

use super::{KeyboardInputEvent, KeyboardInputKind};

pub struct KeyboardFrameState {
  held_keys: HashSet<KeyCode>,
  pressed_keys: HashSet<KeyCode>,
  released_keys: HashSet<KeyCode>,
  repeated_keys: HashSet<KeyCode>,
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
        self.held_keys.insert(event.code);
        self.pressed_keys.insert(event.code);
      }
      KeyboardInputKind::Release => {
        self.held_keys.remove(&event.code);
        self.released_keys.insert(event.code);
      }
      KeyboardInputKind::Repeat => {
        self.held_keys.insert(event.code);
        self.repeated_keys.insert(event.code);
      }
    }
  }

  pub fn is_held(&self, code: KeyCode) -> bool {
    self.held_keys.contains(&code)
  }

  pub fn was_pressed(&self, code: KeyCode) -> bool {
    self.pressed_keys.contains(&code)
  }

  pub fn was_released(&self, code: KeyCode) -> bool {
    self.released_keys.contains(&code)
  }

  pub fn was_repeated(&self, code: KeyCode) -> bool {
    self.repeated_keys.contains(&code)
  }
}
