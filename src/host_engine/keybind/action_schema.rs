//! 动作声明结构。

use std::collections::HashMap;

/// 单个动作默认键位。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActionDefault {
    pub default_keys: Vec<String>,
    pub display_name: String,
}

/// 包声明的动作集合。
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ActionSchema {
    pub actions: HashMap<String, ActionDefault>,
}
