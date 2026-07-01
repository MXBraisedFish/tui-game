use std::collections::HashMap;

use super::super::input::ActionMapEntry;

/// 富文本参数：包含占位变量值和按键动作映射，供解析时替换模板标记。
#[derive(Clone, Debug, Default)]
pub struct RichTextParams {
  pub values: HashMap<String, String>,

  pub key_actions: HashMap<String, Vec<Vec<String>>>,
}

impl RichTextParams {
  /// 从按键映射表创建参数，自动为每个 action 注册带前缀和不带前缀的键。
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
      key_actions,
    }
  }
}
