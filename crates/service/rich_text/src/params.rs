use std::collections::HashMap;

use tg_core_input::ActionMapEntry;

/// 富文本参数：包含占位变量值和按键动作映射，供解析时替换模板标记。
#[derive(Clone, Debug, Default)]
pub struct RichTextParams {
  pub values: HashMap<String, String>,

  pub key_actions: HashMap<String, Vec<Vec<String>>>,

  pub key_default_actions: HashMap<String, Vec<Vec<String>>>,
}

impl RichTextParams {
  /// 使用可自定义动作的当前映射与默认映射创建参数。
  ///
  /// 游戏按键和宿主全局按键应走这条路径；`{key:...}` 读取当前用户映射，
  /// `{key_default:...}` 读取包或宿主提供的默认映射。
  pub fn from_key_actions(key_actions: &HashMap<String, Vec<Vec<String>>>) -> Self {
    Self::from_key_action_maps(key_actions, key_actions)
  }

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

  /// 从不可自定义的 UI 动作表创建参数。
  ///
  /// UI 页面自己的操作键没有 user/default 之分，因此两个参数读取同一份映射。
  /// 自动为每个 action 注册带前缀和不带前缀的键。
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
