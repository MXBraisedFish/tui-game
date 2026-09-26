use super::key_token::parse_key_token;
use super::key::{KeyBinding, KeyPattern};

/// Action map entry: one action, its description and the key patterns bound to it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActionMapEntry {
  pub action: String,
  pub description: String,
  pub keys: Vec<Vec<String>>,
}

/// Error raised while translating an action map into key bindings.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ActionMapTranslateError {
  EmptyAction {
    index: usize,
  },
  EmptyKeyPattern {
    action: String,
    index: usize,
  },
  TooManyKeys {
    action: String,
    index: usize,
    count: usize,
  },
  UnknownKeyToken {
    action: String,
    token: String,
  },
}

/// Translates action map entries into a list of key bindings.
///
/// Every pattern is normalized, so combo matching does not depend on key order.
///
/// # Errors
///
/// Returns [`ActionMapTranslateError::EmptyAction`] when an action name is blank,
/// [`ActionMapTranslateError::EmptyKeyPattern`] when a key pattern has no keys,
/// [`ActionMapTranslateError::TooManyKeys`] when a key pattern has more than two keys, and
/// [`ActionMapTranslateError::UnknownKeyToken`] when a key token cannot be parsed.
pub fn translate_action_map(
  entries: &[ActionMapEntry],
) -> Result<Vec<KeyBinding>, ActionMapTranslateError> {
  let mut bindings = Vec::new();

  for (entry_index, entry) in entries.iter().enumerate() {
    if entry.action.trim().is_empty() {
      return Err(ActionMapTranslateError::EmptyAction { index: entry_index });
    }

    for (pattern_index, raw_pattern) in entry.keys.iter().enumerate() {
      let pattern = translate_key_pattern(&entry.action, pattern_index, raw_pattern)?;
      bindings.push(KeyBinding {
        pattern: pattern.normalized(),
        action: entry.action.clone(),
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
  fn action_without_keys_registers_no_binding() {
    let entries = vec![ActionMapEntry {
      action: "optional".to_string(),
      description: "Optional action".to_string(),
      keys: Vec::new(),
    }];
    assert_eq!(translate_action_map(&entries), Ok(Vec::new()));
  }
}
