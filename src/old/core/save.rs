// 游戏存档管理，基于 saves.json 文件进行持久化。支持多游戏、多数据槽、继续游戏存档和全局键位绑定保存

use anyhow::Result; // 错误处理
use serde_json::{Map, Value as JsonValue}; // JSON 数据结构操作
use std::fs; // 文件读写
use std::collections::HashMap; // 键位绑定的存储格式

use crate::utils::path_utils; // 获取存档文件路径（saves_file()）

// 创建空的存档存储结构：{"continue": {}, "data": {}}
fn empty_store() -> Map<String, JsonValue> {
    let mut store = Map::new();
    store.insert("continue".to_string(), JsonValue::Object(Map::new()));
    store.insert("data".to_string(), JsonValue::Object(Map::new()));
    store
}

// 读取 saves.json 并解析为 Map<String, JsonValue>，处理 BOM 和缺失字段
fn read_store() -> Result<Map<String, JsonValue>> {
    let path = path_utils::saves_file()?;
    if !path.exists() {
        return Ok(empty_store());
    }
    let raw = fs::read_to_string(path)?;
    let mut store =
        serde_json::from_str::<Map<String, JsonValue>>(raw.trim_start_matches('\u{feff}'))
            .unwrap_or_else(|_| empty_store());
    if !matches!(store.get("continue"), Some(JsonValue::Object(_))) {
        store.insert("continue".to_string(), JsonValue::Object(Map::new()));
    }
    if !matches!(store.get("data"), Some(JsonValue::Object(_))) {
        store.insert("data".to_string(), JsonValue::Object(Map::new()));
    }
    Ok(store)
}

// 将存储结构写回文件
fn write_store(store: &Map<String, JsonValue>) -> Result<()> {
    let path = path_utils::saves_file()?;
    path_utils::ensure_parent_dir(&path)?;
    fs::write(path, serde_json::to_string_pretty(store)?)?;
    Ok(())
}

//保留的数据槽名称，用于存储键位配置
const KEYBINDINGS_SLOT: &str = "__keybindings";

// 清理存档名：保留字母数字、下划线、连字符，其余转为下划线，去除首尾下划线，空则返回 "runtime_save"
pub fn sanitize_runtime_save_stem(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-') {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    let trimmed = out.trim_matches('_');
    if trimmed.is_empty() {
        "runtime_save".to_string()
    } else {
        trimmed.to_string()
    }
}

// 在 data.<game_id>.<slot> 路径保存任意数据
pub fn save_data_slot(game_id: &str, slot: &str, value: &JsonValue) -> Result<()> {
    let mut store = read_store()?;
    let data = store
        .get_mut("data")
        .and_then(JsonValue::as_object_mut)
        .expect("data object");
    let game_slots = data
        .entry(game_id.to_string())
        .or_insert_with(|| JsonValue::Object(Map::new()))
        .as_object_mut()
        .expect("game data object");
    game_slots.insert(slot.to_string(), value.clone());
    write_store(&store)
}

// 加载指定数据槽的值
pub fn load_data_slot(game_id: &str, slot: &str) -> Result<Option<JsonValue>> {
    let store = read_store()?;
    Ok(store
        .get("data")
        .and_then(JsonValue::as_object)
        .and_then(|data| data.get(game_id))
        .and_then(JsonValue::as_object)
        .and_then(|slots| slots.get(slot))
        .cloned())
}

// 保存为"继续游戏"存档（覆盖同游戏之前的 continue）
pub fn save_continue(game_id: &str, value: &JsonValue) -> Result<()> {
    let mut store = read_store()?;
    let continue_map = store
        .get_mut("continue")
        .and_then(JsonValue::as_object_mut)
        .expect("continue object");
    continue_map.clear();
    continue_map.insert(game_id.to_string(), value.clone());
    write_store(&store)
}

// 加载继续存档
pub fn load_continue(game_id: &str) -> Result<Option<JsonValue>> {
    let store = read_store()?;
    Ok(store
        .get("continue")
        .and_then(JsonValue::as_object)
        .and_then(|continue_map| continue_map.get(game_id))
        .cloned())
}

// 获取最近有 continue 存档的游戏 ID
pub fn latest_saved_game_id() -> Option<String> {
    let store = read_store().ok()?;
    store
        .get("continue")
        .and_then(JsonValue::as_object)
        .and_then(|continue_map| continue_map.keys().next().cloned())
}

// 清空所有 continue 存档（清除当前游戏的保存进度）
pub fn clear_active_game_save() -> Result<()> {
    let mut store = read_store()?;
    if let Some(continue_map) = store.get_mut("continue").and_then(JsonValue::as_object_mut) {
        continue_map.clear();
    }
    write_store(&store)
}

// 检查是否存在 continue 存档
pub fn game_has_continue_save(game_id: &str) -> bool {
    load_continue(game_id).ok().flatten().is_some()
}

// 保存键位绑定到 __keybindings 槽
pub fn save_keybindings(game_id: &str, bindings: &HashMap<String, Vec<String>>) -> Result<()> {
    let value = serde_json::to_value(bindings)?;
    save_data_slot(game_id, KEYBINDINGS_SLOT, &value)
}

// 加载键位绑定，若不存在返回空 HashMap
pub fn load_keybindings(game_id: &str) -> Result<HashMap<String, Vec<String>>> {
    let Some(value) = load_data_slot(game_id, KEYBINDINGS_SLOT)? else {
        return Ok(HashMap::new());
    };
    Ok(serde_json::from_value(value).unwrap_or_default())
}
