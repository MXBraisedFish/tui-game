use std::collections::HashSet;

use super::{KeyboardFrameState, PhysicalKey};

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
  pub key: PhysicalKey,
  pub trigger: KeyboardActionTrigger,
  pub action: Action,
  pub priority: i32,
  pub consume: bool,
}

impl<Action> KeyboardActionBinding<Action>
where
  Action: Copy,
{
  pub fn new(key: PhysicalKey, trigger: KeyboardActionTrigger, action: Action) -> Self {
    Self::with_priority(key, trigger, action, 0)
  }

  pub fn with_priority(key: PhysicalKey, trigger: KeyboardActionTrigger, action: Action, priority: i32) -> Self {
    Self::with_options(key, trigger, action, priority, true)
  }

  pub fn with_options(key: PhysicalKey, trigger: KeyboardActionTrigger, action: Action, priority: i32, consume: bool) -> Self {
    Self {
      key,
      trigger,
      action,
      priority,
      consume,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResolvedKeyboardAction<Action>
where
  Action: Copy,
{
  pub action: Action,
  pub key: PhysicalKey,
  pub priority: i32,
  pub consume: bool,
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
    self
      .resolve_consumed(state)
      .into_iter()
      .map(|resolved| resolved.action)
      .collect()
  }

  pub fn resolve_consumed(&self, state: &KeyboardFrameState) -> Vec<ResolvedKeyboardAction<Action>> {
    let mut consumed_keys = HashSet::new();
    let mut actions = Vec::new();

    for resolved in self.resolve_detailed(state) {
      if consumed_keys.contains(&resolved.key) {
        continue;
      }

      if resolved.consume {
        consumed_keys.insert(resolved.key);
      }

      actions.push(resolved);
    }

    actions
  }

  pub fn resolve_detailed(&self, state: &KeyboardFrameState) -> Vec<ResolvedKeyboardAction<Action>> {
    let mut matched = Vec::new();

    for (index, binding) in self.bindings.iter().enumerate() {
      if binding.matches(state) {
        matched.push((
          binding.priority,
          index,
          ResolvedKeyboardAction {
            action: binding.action,
            key: binding.key,
            priority: binding.priority,
            consume: binding.consume,
          },
        ));
      }
    }

    matched.sort_by(|left, right| right.0.cmp(&left.0).then_with(|| left.1.cmp(&right.1)));

    matched.into_iter().map(|(_, _, action)| action).collect()
  }
}
