use super::{KeyboardActionMap, KeyboardFrameState, ResolvedKeyboardAction};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyboardActionLayerKind {
  Root,
  Overlay,
  UiNode,
  Surface,
  ExecutionContext,
}

pub struct KeyboardActionLayer<Action>
where
  Action: Copy,
{
  kind: KeyboardActionLayerKind,
  priority: i32,
  map: KeyboardActionMap<Action>,
}

impl<Action> KeyboardActionLayer<Action>
where
  Action: Copy,
{
  pub fn new(kind: KeyboardActionLayerKind, priority: i32, map: KeyboardActionMap<Action>) -> Self {
    Self { kind, priority, map }
  }

  pub fn kind(&self) -> KeyboardActionLayerKind {
    self.kind
  }

  pub fn priority(&self) -> i32 {
    self.priority
  }

  pub fn map(&self) -> &KeyboardActionMap<Action> {
    &self.map
  }

  pub fn resolve(&self, state: &KeyboardFrameState) -> Vec<ResolvedKeyboardAction<Action>> {
    self.map.resolve_consumed(state)
  }
}
