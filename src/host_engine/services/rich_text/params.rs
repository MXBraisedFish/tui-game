use std::collections::HashMap;

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
