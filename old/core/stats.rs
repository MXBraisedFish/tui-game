// 游戏最佳成绩存储管理，基于 best_scores.json 文件持久化各游戏的最高分或最优数据。支持合并更新、按游戏 ID 清理无效条目

use std::collections::{HashMap, HashSet}; // 存储所有成绩和有效游戏 ID 集合
use std::fs; // 文件读写

use anyhow::Result; // 错误处理
use serde_json::{Map, Value as JsonValue}; // JSON 操作

use crate::utils::path_utils; // 获取成绩文件路径（best_scores_file()）

// 深度合并两个 JSON 对象：递归合并，若字段均为对象则合并，否则覆盖
fn merge_objects(base: &mut Map<String, JsonValue>, overlay: &Map<String, JsonValue>) {
    for (key, value) in overlay {
        match (base.get_mut(key), value) {
            (Some(JsonValue::Object(existing)), JsonValue::Object(incoming)) => {
                merge_objects(existing, incoming);
            }
            _ => {
                base.insert(key.clone(), value.clone());
            }
        }
    }
}

// 读取 best_scores.json，返回 Map<String, JsonValue>，键为游戏 ID，值为任意 JSON
fn read_store() -> Result<Map<String, JsonValue>> {
    let path = path_utils::best_scores_file()?;
    if !path.exists() {
        return Ok(Map::new());
    }
    let raw = fs::read_to_string(path)?;
    Ok(
        serde_json::from_str::<Map<String, JsonValue>>(raw.trim_start_matches('\u{feff}'))
            .unwrap_or_default(),
    )
}

// 将存储写回文件
fn write_store(store: &Map<String, JsonValue>) -> Result<()> {
    let path = path_utils::best_scores_file()?;
    path_utils::ensure_parent_dir(&path)?;
    fs::write(path, serde_json::to_string_pretty(store)?)?;
    Ok(())
}

// 读取该游戏的最高成绩数据
pub fn read_runtime_best_score(game_id: &str) -> Option<JsonValue> {
    read_store().ok()?.get(game_id).cloned()
}

// 写入/更新成绩。若原值存在且为对象，则与 value 进行深度合并（保留旧字段），否则直接替换
pub fn write_runtime_best_score(game_id: &str, value: &JsonValue) -> Result<()> {
    let mut store = read_store()?;
    let merged = match (store.remove(game_id), value) {
        (Some(JsonValue::Object(mut existing)), JsonValue::Object(incoming)) => {
            merge_objects(&mut existing, incoming);
            JsonValue::Object(existing)
        }
        _ => value.clone(),
    };
    store.insert(game_id.to_string(), merged);
    write_store(&store)
}

// 删除 best_scores.json 中不在 valid_game_ids 里的条目（用于卸载 Mod 后清理）
pub fn prune_runtime_scores(valid_game_ids: impl IntoIterator<Item = String>) -> Result<()> {
    let valid: HashSet<String> = valid_game_ids.into_iter().collect();
    let mut store = read_store()?;
    store.retain(|key, _| valid.contains(key));
    write_store(&store)
}

// 加载所有游戏的成绩到 HashMap，失败返回空
pub fn load_all() -> HashMap<String, JsonValue> {
    read_store()
        .map(|store| store.into_iter().collect())
        .unwrap_or_default()
}
