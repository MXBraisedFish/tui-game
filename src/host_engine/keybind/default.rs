//! 宿主全局默认按键。

use std::collections::HashMap;

use super::binding::Key;

/// 系统默认全局绑定。
pub fn system_defaults() -> HashMap<String, Vec<Key>> {
    HashMap::from([
        ("screensaver".to_string(), vec![Key::F(2)]),
        ("boss_key".to_string(), vec![Key::F(3)]),
        ("force_stop_game".to_string(), vec![Key::F(4)]),
    ])
}
