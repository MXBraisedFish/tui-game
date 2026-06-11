use std::collections::HashMap;

use super::super::input::ActionMapEntry;

/// 富文本解析时的参数替换上下文。
///
/// 支持两种命名空间：
/// - `{value:xxx}` → 从 `values` 查找
/// - `{key:xxx}` → 从 `key_actions` 查找，自动格式化为可读按键文本
///
/// 兼容旧写法 `{xxx}`（无前缀）→ 视为 `{value:xxx}`。
#[derive(Clone, Debug, Default)]
pub struct RichTextParams {
  /// {value:xxx} 替换表
  pub values: HashMap<String, String>,
  /// {key:xxx} 替换表 —— action_name → 原始按键配置
  ///
  /// 值的格式与 `ActionMapEntry.keys` 一致：
  /// 外 Vec 是多个可选按键组合，内 Vec 是单个组合的键列表（长度 1 或 2）。
  pub key_actions: HashMap<String, Vec<Vec<String>>>,
}

impl RichTextParams {
  /// 从 action map 构建 key_actions，同时注册去掉前缀的短别名。
  ///
  /// 例如 `home.confirm` 会同时注册 `home.confirm` 和 `confirm` 两个 key，
  /// 语言文件中 `{key:confirm}` 和 `{key:home.confirm}` 都能匹配。
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
