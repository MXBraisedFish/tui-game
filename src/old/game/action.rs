// 定义游戏动作的按键绑定数据结构，用于将语义键名（如 "space"、"enter"）映射到游戏逻辑中的动作（如 "jump"、"confirm"）。属于 game 模块的配置数据层，供 manifest.rs 和 registry.rs 使用

use serde::{Deserialize, Serialize}; // 支持 JSON 序列化/反序列化，用于从游戏清单文件（game.json）中读取绑定配置

// 表示一个游戏动作的完整绑定信息
#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct ActionBinding {
    pub key: ActionKeys, // 绑定的键名（单个或多个）
    pub key_name: String, // 用于 UI 显示的友好名称（如 "跳跃"、"确认"）
}

// untagged 表示在 JSON 中直接以字符串或数组形式出现，无需额外标签
#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(untagged)]
pub enum ActionKeys {
    Single(String), // 单个键，如 "space"
    Multiple(Vec<String>), // 多个备选键，如 ["enter", "space"]
}

impl ActionBinding {
    // 返回有效的键名列表（过滤空字符串）。单键若为空则返回空向量，多键过滤空字符串
    pub fn keys(&self) -> Vec<String> {
        match &self.key {
            ActionKeys::Single(key) => {
                if key.trim().is_empty() {
                    Vec::new()
                } else {
                    vec![key.clone()]
                }
            }
            ActionKeys::Multiple(keys) => keys
                .iter()
                .filter(|key| !key.trim().is_empty())
                .cloned()
                .collect(),
        }
    }

    // 获取 key_name 字段的引用
    pub fn key_name(&self) -> &str {
        &self.key_name
    }

    // 返回原始键列表（不去空，用于存档/比较）。单键返回单元素向量，多键克隆整个向量
    pub fn slots(&self) -> Vec<String> {
        match &self.key {
            ActionKeys::Single(key) => vec![key.clone()],
            ActionKeys::Multiple(keys) => keys.clone(),
        }
    }
}
