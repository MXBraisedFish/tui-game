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
  pub priority: i32,
}

impl<Action> KeyboardActionBinding<Action>
where
  Action: Copy,
{
  pub fn new(key: KeyCode, trigger: KeyboardActionTrigger, action: Action) -> Self {
    Self::with_priority(key, trigger, action, 0)
  }

  pub fn with_priority(key: KeyCode, trigger: KeyboardActionTrigger, action: Action, priority: i32) -> Self {
    Self {
      key,
      trigger,
      action,
      priority,
    }
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
    let mut matched = Vec::new();

    for (index, binding) in self.bindings.iter().enumerate() {
      if binding.matches(state) {
        matched.push((binding.priority, index, binding.action));
      }
    }

    matched.sort_by(|left, right| right.0.cmp(&left.0).then_with(|| left.1.cmp(&right.1)));

    matched.into_iter().map(|(_, _, action)| action).collect()
  }
}
