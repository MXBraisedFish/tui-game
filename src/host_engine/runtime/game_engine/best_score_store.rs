//! 最佳记录持久化

use std::fs;
use std::io;
use std::path::PathBuf;

use serde_json::{Map, Value};

/// 保存游戏最佳记录字符串，并返回完整 best_scores 快照。
pub(crate) fn save_best_score(uid: &str, best_string: &str) -> io::Result<Value> {
    let path = root_dir().join("data/profiles/best_scores.json");
    let mut best_scores = read_best_scores(&path);

    if let Some(object) = best_scores.as_object_mut() {
        object.insert(uid.to_string(), Value::String(best_string.to_string()));
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(&best_scores)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    fs::write(path, content)?;

    Ok(best_scores)
}

fn read_best_scores(path: &PathBuf) -> Value {
    let Ok(raw_json) = fs::read_to_string(path) else {
        return Value::Object(Map::new());
    };
    let Ok(value) = serde_json::from_str::<Value>(raw_json.trim_start_matches('\u{feff}')) else {
        return Value::Object(Map::new());
    };
    if value.is_object() {
        value
    } else {
        Value::Object(Map::new())
    }
}

fn root_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(|parent| parent.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
}
