use anyhow::Result;
use std::fs;

use crate::utils::path_utils;

/// 新 runtime 的统一存档目录。
pub fn runtime_save_dir() -> Result<std::path::PathBuf> {
    let dir = path_utils::app_data_dir()?.join("runtime_save");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// 记录“最近一次由新 runtime 保存的游戏”。
fn latest_runtime_save_marker_path() -> Result<std::path::PathBuf> {
    Ok(path_utils::app_data_dir()?.join("latest_runtime_save.txt"))
}

/// 将游戏 ID 转成适合文件系统使用的文件名主干。
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

/// 新 runtime 单游戏存档文件路径。
pub fn runtime_game_save_path(game_id: &str) -> Result<std::path::PathBuf> {
    Ok(runtime_save_dir()?.join(format!("{}.json", sanitize_runtime_save_stem(game_id))))
}

pub fn set_latest_runtime_save_game(game_id: &str) -> Result<()> {
    let path = latest_runtime_save_marker_path()?;
    path_utils::ensure_parent_dir(&path)?;
    fs::write(path, game_id.trim())?;
    Ok(())
}

pub fn latest_runtime_save_game_id() -> Option<String> {
    let path = latest_runtime_save_marker_path().ok()?;
    let raw = fs::read_to_string(path).ok()?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

pub fn clear_latest_runtime_save_game() -> Result<()> {
    let path = latest_runtime_save_marker_path()?;
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

/// 宿主统一解析当前可继续的最近存档目标。
pub fn latest_saved_game_id() -> Option<String> {
    let runtime_latest = latest_runtime_save_game_id();
    let runtime_pair = runtime_latest.and_then(|game_id| {
        runtime_game_save_path(&game_id)
            .ok()
            .map(|path| (game_id, path))
    });

    [runtime_pair]
        .into_iter()
        .flatten()
        .filter_map(|(game_id, path)| {
            let modified = fs::metadata(path).ok()?.modified().ok()?;
            Some((game_id, modified))
        })
        .max_by_key(|(_, modified)| *modified)
        .map(|(game_id, _)| game_id)
}

/// 统一清理“当前活动存档”。
pub fn clear_active_game_save() -> Result<()> {
    if let Some(game_id) = latest_runtime_save_game_id() {
        if let Ok(path) = runtime_game_save_path(&game_id) {
            let _ = fs::remove_file(path);
        }
        let _ = clear_latest_runtime_save_game();
    }

    Ok(())
}
