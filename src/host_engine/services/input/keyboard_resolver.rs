use std::collections::HashSet;

use super::{KeyboardActionLayer, KeyboardFrameState, PhysicalKey, ResolvedKeyboardAction};

pub struct KeyboardActionResolver<Action>
where
  Action: Copy,
{
  layers: Vec<KeyboardActionLayer<Action>>,
}

impl<Action> KeyboardActionResolver<Action>
where
  Action: Copy,
{
  pub fn new() -> Self {
    Self { layers: Vec::new() }
  }

  pub fn add_layer(&mut self, layer: KeyboardActionLayer<Action>) {
    self.layers.push(layer);
  }

  pub fn clear(&mut self) {
    self.layers.clear();
  }

  pub fn layers(&self) -> &[KeyboardActionLayer<Action>] {
    &self.layers
  }

  pub fn resolve(&self, state: &KeyboardFrameState) -> Vec<Action> {
    self
      .resolve_detailed(state)
      .into_iter()
      .map(|resolved| resolved.action)
      .collect()
  }

  pub fn resolve_detailed(&self, state: &KeyboardFrameState) -> Vec<ResolvedKeyboardAction<Action>> {
    let mut matched = Vec::new();

    for (layer_index, layer) in self.layers.iter().enumerate() {
      for action in layer.resolve(state) {
        matched.push((layer.priority(), layer_index, action.priority, action));
      }
    }

    matched.sort_by(|left, right| {
      right
        .0
        .cmp(&left.0)
        .then_with(|| left.1.cmp(&right.1))
        .then_with(|| right.2.cmp(&left.2))
    });

    let mut consumed_keys = HashSet::<PhysicalKey>::new();
    let mut resolved_actions = Vec::new();

    for (_, _, _, action) in matched {
      if consumed_keys.contains(&action.key) {
        continue;
      }

      if action.consume {
        consumed_keys.insert(action.key);
      }

      resolved_actions.push(action);
    }

    resolved_actions
  }
}
