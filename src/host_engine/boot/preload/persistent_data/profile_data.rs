//! 持久化数据结构

use serde_json::Value;

/// data/profiles 下的持久化数据快照
#[derive(Clone, Debug)]
pub struct PersistentData {
    pub saves: Value,
    pub best_scores: Value,
    pub language_code: String,
    pub keybinds: Value,
    pub mod_state: Value,
}
