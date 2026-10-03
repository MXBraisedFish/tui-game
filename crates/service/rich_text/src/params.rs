//! Params support for the rich text service.

use std::collections::HashMap;

use tg_core_input::ActionMapEntry;

/// Configuration values controlling rich text behavior.
///
/// # Fields
///
/// * `values` - The values indexed by their declared keys.
/// * `key_actions` - The key actions indexed by their declared keys.
/// * `key_default_actions` - The key default actions indexed by their declared keys.
#[derive(Clone, Debug, Default)]
pub struct RichTextParams {
  /// The values indexed by their declared keys.
  pub values: HashMap<String, String>,

  /// The key actions indexed by their declared keys.
  pub key_actions: HashMap<String, Vec<Vec<String>>>,

  /// The key default actions indexed by their declared keys.
  pub key_default_actions: HashMap<String, Vec<Vec<String>>>,
}

impl RichTextParams {
  /// Create text-format substitutions from named actions and their displayed shortcuts.
  pub fn from_key_actions(key_actions: &HashMap<String, Vec<Vec<String>>>) -> Self {
    Self::from_key_action_maps(key_actions, key_actions)
  }

  /// Merge action maps into the shortcut substitutions used by formatted text.
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

  /// Build shortcut substitutions from one action map.
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
