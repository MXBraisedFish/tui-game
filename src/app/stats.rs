// 游戏统计数据管理模块，当前为空壳实现。定义了 GameStats 结构体，但 load_stats 返回空 HashMap，update_game_stats 不做任何操作。format_duration 已实现

use std::collections::HashMap; // 存储游戏 ID 到统计数据的映射（当前未使用）

use anyhow::Result; // 错误处理
use serde::{Deserialize, Serialize}; // 序列化/反序列化（为未来持久化准备）

// 游戏统计数据
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub struct GameStats {
    pub high_score: u32,
    pub max_duration_sec: u64,
}

// 加载所有游戏统计数据，当前返回空 HashMap
pub fn load_stats() -> HashMap<String, GameStats> {
    HashMap::new()
}

// 更新游戏统计数据，当前为空操作
pub fn update_game_stats(_game_id: &str, _score: u32, _duration_sec: u64) -> Result<()> {
    Ok(())
}

// 将秒数格式化为 HH:MM:SS 格式
pub fn format_duration(seconds: u64) -> String {
    let h = seconds / 3600;
    let m = (seconds % 3600) / 60;
    let s = seconds % 60;
    format!("{h:02}:{m:02}:{s:02}")
}
