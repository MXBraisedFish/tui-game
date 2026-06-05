use crossterm::event::KeyCode;

use super::KeyboardFrameState;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyboardActionTrigger {
  Pressed,
  Released,
  Repeated,
  Held,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyboardActionBinding<Action>
where
  Action: Copy,
{
  pub key: KeyCode,
  pub trigger: KeyboardActionTrigger,
  pub action: Action,
}

impl<Action> KeyboardActionBinding<Action>
where
  Action: Copy,
{
  pub fn new(key: KeyCode, trigger: KeyboardActionTrigger, action: Action) -> Self {
    Self { key, trigger, action }
  }

  pub fn matches(&self, state: &KeyboardFrameState) -> bool {
    match self.trigger {
      KeyboardActionTrigger::Pressed => state.was_pressed(self.key),
      KeyboardActionTrigger::Released => state.was_released(self.key),
      KeyboardActionTrigger::Repeated => state.was_repeated(self.key),
      KeyboardActionTrigger::Held => state.is_held(self.key),
    }
  }
}

pub struct KeyboardActionMap<Action>
where
  Action: Copy,
{
  bindings: Vec<KeyboardActionBinding<Action>>,
}

impl<Action> KeyboardActionMap<Action>
where
  Action: Copy,
{
  pub fn new() -> Self {
    Self { bindings: Vec::new() }
  }

  pub fn add_binding(&mut self, binding: KeyboardActionBinding<Action>) {
    self.bindings.push(binding);
  }

  pub fn clear(&mut self) {
    self.bindings.clear();
  }

  pub fn bindings(&self) -> &[KeyboardActionBinding<Action>] {
    &self.bindings
  }

  pub fn resolve(&self, state: &KeyboardFrameState) -> Vec<Action> {
    let mut actions = Vec::new();

    for binding in &self.bindings {
      if binding.matches(state) {
        actions.push(binding.action);
      }
    }

    actions
  }
}
