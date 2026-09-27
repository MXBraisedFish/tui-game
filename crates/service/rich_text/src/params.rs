use std::collections::HashMap;

use tg_core_input::ActionMapEntry;

/// Rich text parameters: placeholder values and key action maps that replace template markers
/// while parsing.
#[derive(Clone, Debug, Default)]
pub struct RichTextParams {
  pub values: HashMap<String, String>,

  pub key_actions: HashMap<String, Vec<Vec<String>>>,

  pub key_default_actions: HashMap<String, Vec<Vec<String>>>,
}

impl RichTextParams {
  /// Creates parameters whose `{key:...}` and `{key_default:...}` placeholders both read the same
  /// key action map.
  pub fn from_key_actions(key_actions: &HashMap<String, Vec<Vec<String>>>) -> Self {
    Self::from_key_action_maps(key_actions, key_actions)
  }

  /// Creates parameters from the current and the default key action maps of customizable actions.
  ///
  /// Game keys and global host keys should take this path; `{key:...}` reads the current user
  /// map and `{key_default:...}` reads the default map provided by the package or the host.
  pub fn from_key_action_maps(
    key_actions: &HashMap<String, Vec<Vec<String>>>,
    key_default_actions: &HashMap<String, Vec<Vec<String>>>,
  ) -> Self {
    Self {
      values: HashMap::new(),
      key_actions: key_actions.clone(),
      key_default_actions: key_default_actions.clone(),
    }
  }

  /// Creates parameters from a non-customizable UI action map.
  ///
  /// A UI page's own action keys have no user/default distinction, so both maps hold the same
  /// entries. Every action is registered under its full name and, when it starts with `prefix`,
  /// also under the name without the prefix.
  pub fn from_action_map(entries: &[ActionMapEntry], prefix: &str) -> Self {
    let mut key_actions = HashMap::new();
    for entry in entries {
      key_actions.insert(entry.action.clone(), entry.keys.clone());
      if let Some(short) = entry.action.strip_prefix(prefix) {
        key_actions.insert(short.to_string(), entry.keys.clone());
      }
    }
    Self {
      values: HashMap::new(),
      key_default_actions: key_actions.clone(),
      key_actions,
    }
  }
}
