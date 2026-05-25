//! 持久化数据结构

use serde_json::Value;

/// data/profiles 下的持久化数据快照
#[derive(Clone, Debug)]
pub struct PersistentData {
    pub saves: Value,
    pub best_scores: Value,
    pub language_code: String,
    pub keybinds: Value,
    pub game_state: Value,
    pub screensaver_state: Value,
    pub boss_state: Value,
    pub security_state: Value,
    pub display_state: Value,
}
