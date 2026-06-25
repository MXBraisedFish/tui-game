use super::key_token::parse_key_token;
use super::service::{KeyBinding, KeyPattern};

/// 动作映射条目
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActionMapEntry {
  pub action: String,
  pub description: String,
  pub keys: Vec<Vec<String>>,
}

/// 动作映射翻译错误
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

/// 将动作映射条目翻译为按键绑定列表
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
