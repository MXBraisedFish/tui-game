//! 临时全局按键占用表。
//!
//! 该文件只用于预设后续宿主级全局按键占用，当前不接入监听链路。

/// 全局按键占用项。
pub struct ReservedGlobalKey {
    pub key: &'static str,
    pub action: &'static str,
    pub description: &'static str,
}

/// Screensaver 键。
pub const SCREENSAVER_KEY: ReservedGlobalKey = ReservedGlobalKey {
    key: "f2",
    action: "screensaver",
    description: "Screensaver 键",
};

/// 老板键。
pub const BOSS_KEY: ReservedGlobalKey = ReservedGlobalKey {
    key: "f3",
    action: "boss_key",
    description: "老板键",
};

/// 强制终止游戏运行键。
pub const FORCE_STOP_GAME_KEY: ReservedGlobalKey = ReservedGlobalKey {
    key: "f4",
    action: "force_stop_game",
    description: "强制终止游戏运行",
};

/// 全局按键占用列表。
pub const RESERVED_GLOBAL_KEYS: [ReservedGlobalKey; 3] = [SCREENSAVER_KEY, BOSS_KEY, FORCE_STOP_GAME_KEY];
