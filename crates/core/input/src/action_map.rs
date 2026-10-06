//! Validation of action declarations and conversion to ordered key bindings.

use std::collections::{HashMap, HashSet};

use super::key::{KeyBinding, KeyPattern};
use super::key_token::parse_key_token;

/// An action name and description with its alternative key patterns.
///
/// # Fields
///
/// * `action` - The nonempty action identifier delivered to consumers.
/// * `description` - The visible action description.
/// * `keys` - Alternative shortcuts; each inner list contains one key or a two-key combination.
/// * `priority` - The nonnegative dispatch priority; larger values are dispatched first.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActionMapEntry {
  /// The nonempty action identifier delivered to consumers.
  pub action: String,
  /// The visible action description.
  pub description: String,
  /// Alternative shortcuts; each inner list contains one key or a two-key combination.
  pub keys: Vec<Vec<String>>,
  /// The nonnegative dispatch priority; larger values are dispatched first.
  pub priority: u64,
}

/// Failures reported by action map translate operations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ActionMapTranslateError {
  /// An action name is empty after trimming.
  EmptyAction {
    /// The slot or sequence index.
    index: usize,
  },
  /// A declared shortcut contains no keys.
  EmptyKeyPattern {
    /// The action.
    action: String,
    /// The slot or sequence index.
    index: usize,
  },
  /// A shortcut contains more keys than a supported pattern.
  TooManyKeys {
    /// The action.
    action: String,
    /// The slot or sequence index.
    index: usize,
    /// The count.
    count: usize,
  },
  /// A two-key combination repeats the same normalized key.
  RepeatedKey {
    /// The action containing the invalid combination.
    action: String,
    /// The binding index within the action.
    index: usize,
  },
  /// An action registers the same normalized binding more than once.
  DuplicateBinding {
    /// The action containing the repeated binding.
    action: String,
    /// The repeated binding index within the action.
    index: usize,
  },
  /// A shortcut contains an unrecognized portable key token.
  UnknownKeyToken {
    /// The action.
    action: String,
    /// The token.
    token: String,
  },
}

/// Validate action entries and return bindings with normalized single-key and two-key patterns.
///
/// # Errors
///
/// Return `EmptyAction`, `EmptyKeyPattern`, `TooManyKeys`, or `UnknownKeyToken` for invalid
/// declarations. Return `RepeatedKey` for a repeated combination key and `DuplicateBinding`
/// when one action registers the same normalized pattern twice.
pub fn translate_action_map(
  entries: &[ActionMapEntry],
) -> Result<Vec<KeyBinding>, ActionMapTranslateError> {
  let mut bindings = Vec::new();
  let mut registered: HashMap<&str, HashSet<KeyPattern>> = HashMap::new();

  for (entry_index, entry) in entries.iter().enumerate() {
    if entry.action.trim().is_empty() {
      return Err(ActionMapTranslateError::EmptyAction { index: entry_index });
    }

    for (pattern_index, raw_pattern) in entry.keys.iter().enumerate() {
      let pattern = translate_key_pattern(&entry.action, pattern_index, raw_pattern)?.normalized();
      if !registered.entry(&entry.action).or_default().insert(pattern) {
        return Err(ActionMapTranslateError::DuplicateBinding {
          action: entry.action.clone(),
          index: pattern_index,
        });
      }
      bindings.push(KeyBinding {
        pattern: pattern.normalized(),
        action: entry.action.clone(),
        priority: entry.priority,
      });
    }
  }

  Ok(bindings)
}

fn translate_key_pattern(
  action: &str,
  index: usize,
  raw_pattern: &[String],
) -> Result<KeyPattern, ActionMapTranslateError> {
  match raw_pattern.len() {
    0 => Err(ActionMapTranslateError::EmptyKeyPattern {
      action: action.to_string(),
      index,
    }),
    1 => {
      let key = parse_key_token(&raw_pattern[0]).ok_or_else(|| {
        ActionMapTranslateError::UnknownKeyToken {
          action: action.to_string(),
          token: raw_pattern[0].clone(),
        }
      })?;
      Ok(KeyPattern::Single(key))
    }
    2 => {
      let first = parse_key_token(&raw_pattern[0]).ok_or_else(|| {
        ActionMapTranslateError::UnknownKeyToken {
          action: action.to_string(),
          token: raw_pattern[0].clone(),
        }
      })?;
      let second = parse_key_token(&raw_pattern[1]).ok_or_else(|| {
        ActionMapTranslateError::UnknownKeyToken {
          action: action.to_string(),
          token: raw_pattern[1].clone(),
        }
      })?;
      if first == second {
        return Err(ActionMapTranslateError::RepeatedKey {
          action: action.to_string(),
          index,
        });
      }
      Ok(KeyPattern::Combo(first, second))
    }
    count => Err(ActionMapTranslateError::TooManyKeys {
      action: action.to_string(),
      index,
      count,
    }),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn shared_actions_are_valid_but_repeated_keys_and_same_action_bindings_are_rejected() {
    let entry = |action: &str, keys| ActionMapEntry {
      action: action.into(),
      description: action.into(),
      keys,
      priority: 10,
    };
    let shared = translate_action_map(&[
      entry("exit", vec![vec!["esc".into()]]),
      entry("pause", vec![vec!["esc".into()]]),
    ])
    .unwrap();
    assert_eq!(shared.len(), 2);
    assert_eq!(shared[0].priority, 10);
    assert!(matches!(
      translate_action_map(&[entry("same", vec![vec!["a".into(), "A".into()]])]),
      Err(ActionMapTranslateError::RepeatedKey { .. })
    ));
    assert!(matches!(
      translate_action_map(&[entry(
        "same",
        vec![vec!["a".into(), "b".into()], vec!["b".into(), "a".into()]]
      )]),
      Err(ActionMapTranslateError::DuplicateBinding { .. })
    ));
  }

  #[test]
  fn action_without_keys_registers_no_binding() {
    let entries = vec![ActionMapEntry {
      action: "optional".to_string(),
      description: "Optional action".to_string(),
      keys: Vec::new(),
      priority: 0,
    }];
    assert_eq!(translate_action_map(&entries), Ok(Vec::new()));
  }
}
