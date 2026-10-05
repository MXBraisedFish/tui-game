//! Session-owned keyboard subscriptions and balanced delivery tracking.

use crate::{LuaActionState, LuaEventData};

#[derive(Debug)]
pub(crate) struct LuaInputState {
  pub actions: bool,
  pub keys: bool,
  pub focus_release: bool,
  action_generation: u64,
  key_generation: u64,
  active_actions: Vec<String>,
  active_keys: Vec<String>,
  pending_releases: Vec<LuaEventData>,
}

impl Default for LuaInputState {
  fn default() -> Self {
    Self {
      actions: true,
      keys: false,
      focus_release: true,
      action_generation: 0,
      key_generation: 0,
      active_actions: Vec::new(),
      active_keys: Vec::new(),
      pending_releases: Vec::new(),
    }
  }
}

impl LuaInputState {
  pub fn generation(&self, data: &LuaEventData) -> u64 {
    match data {
      LuaEventData::Key { .. } => self.key_generation,
      _ => self.action_generation,
    }
  }

  pub fn close(&mut self, actions: bool, keys: bool) {
    self.close_with_releases(actions, keys, true);
  }

  /// Forget game input on terminal focus loss or overlay takeover under its release policy.
  pub fn focus_lost(&mut self) {
    self.close_with_releases(true, true, self.focus_release);
  }

  fn close_with_releases(&mut self, actions: bool, keys: bool, emit_releases: bool) {
    if keys {
      self.key_generation = self.key_generation.wrapping_add(1);
      for key in std::mem::take(&mut self.active_keys) {
        if emit_releases {
          self.pending_releases.push(LuaEventData::Key {
            key,
            state: LuaActionState::Released,
          });
        }
      }
    }
    if actions {
      self.action_generation = self.action_generation.wrapping_add(1);
      for action in std::mem::take(&mut self.active_actions) {
        if emit_releases {
          self.pending_releases.push(LuaEventData::Action {
            action,
            state: LuaActionState::Released,
          });
        }
      }
    }
  }

  pub fn take_releases(&mut self) -> Vec<LuaEventData> {
    std::mem::take(&mut self.pending_releases)
  }

  pub fn accept(&mut self, data: &LuaEventData) -> bool {
    let (enabled, active, name, state) = match data {
      LuaEventData::Action { action, state } => {
        (self.actions, &mut self.active_actions, action, state)
      }
      LuaEventData::Key { key, state } => (self.keys, &mut self.active_keys, key, state),
      _ => return true,
    };
    if !enabled {
      return false;
    }
    match state {
      LuaActionState::Pressed => {
        if active.contains(name) {
          false
        } else {
          active.push(name.clone());
          true
        }
      }
      LuaActionState::Held => active.contains(name),
      LuaActionState::Released => {
        let found = active.contains(name);
        active.retain(|active| active != name);
        found
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn silent_focus_cleanup_preserves_pending_releases_from_rejection() {
    let mut input = LuaInputState::default();
    input.keys = true;
    input.focus_release = false;
    let action = LuaEventData::Action {
      action: "pause".into(),
      state: LuaActionState::Pressed,
    };
    let key = LuaEventData::Key {
      key: "esc".into(),
      state: LuaActionState::Pressed,
    };
    assert!(input.accept(&action));
    assert!(input.accept(&key));
    input.close(true, false);
    input.focus_lost();
    assert_eq!(
      input.take_releases(),
      [LuaEventData::Action {
        action: "pause".into(),
        state: LuaActionState::Released,
      }]
    );
    input.focus_release = true;
    input.focus_lost();
    assert!(input.take_releases().is_empty());
    assert!(!input.accept(&LuaEventData::Key {
      key: "esc".into(),
      state: LuaActionState::Held,
    }));
    assert!(input.accept(&key));
    input.focus_lost();
    assert_eq!(
      input.take_releases(),
      [LuaEventData::Key {
        key: "esc".into(),
        state: LuaActionState::Released,
      }]
    );
  }
}
