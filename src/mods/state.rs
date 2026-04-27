use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{LazyLock, Mutex};

use anyhow::Result;
use serde_json::Value as JsonValue;

use crate::mods::types::*;
use crate::utils::path_utils;

/// 全局 Mod 状态存储。
static MOD_STATE_STORE: LazyLock<Mutex<ModState>> = LazyLock::new(|| {
    Mutex::new(read_persisted_mod_state().unwrap_or_else(|| ModState {
        api_version: MOD_API_VERSION,
        ..Default::default()
    }))
});

/// 获取或创建指定命名空间的 Mod 状态条目。
pub fn ensure_mod_state_entry<'a>(state: &'a mut ModState, namespace: &str) -> &'a mut ModStateEntry {
    let default_mod_enabled = state.default_mod_enabled;
    let default_safe_mode_enabled = state.default_safe_mode_enabled;
    state
        .mods
        .entry(namespace.to_string())
        .or_insert_with(|| ModStateEntry {
            enabled: default_mod_enabled,
            safe_mode_enabled: default_safe_mode_enabled,
            session_safe_mode_enabled: None,
            ..Default::default()
        })
}

/// 清理 Mod 存档文件名，仅保留 ASCII 字母数字、下划线和连字符。
pub fn sanitize_mod_save_file_stem(game_id: &str) -> String {
    let mut sanitized = String::with_capacity(game_id.len());
    for ch in game_id.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
            sanitized.push(ch);
        } else {
            sanitized.push('_');
        }
    }
    while sanitized.contains("__") {
        sanitized = sanitized.replace("__", "_");
    }
    let trimmed = sanitized.trim_matches('_');
    if trimmed.is_empty() {
        "mod_save".to_string()
    } else {
        trimmed.to_string()
    }
}

/// 返回指定命名空间和游戏 ID 的 Mod 存档路径。
pub fn mod_save_path(namespace: &str, game_id: &str) -> Result<PathBuf> {
    Ok(path_utils::mod_save_dir()?.join(namespace).join(format!(
        "{}.json",
        sanitize_mod_save_file_stem(game_id)
    )))
}

// ─── 持久化读写 ───

fn mod_state_cache_file() -> Result<PathBuf> {
    Ok(path_utils::cache_dir()?.join("mod_state.json"))
}

fn scan_cache_file() -> Result<PathBuf> {
    Ok(path_utils::cache_dir()?.join("scan_cache.json"))
}

fn read_persisted_mod_state() -> Option<ModState> {
    let path = mod_state_cache_file().ok()?;
    let raw = fs::read_to_string(path).ok()?;
    let mut state = serde_json::from_str::<ModState>(raw.trim_start_matches('\u{feff}')).ok()?;
    state.api_version = MOD_API_VERSION;
    Some(state)
}

fn persist_mod_state(state: &ModState) -> Result<()> {
    let path = mod_state_cache_file()?;
    path_utils::ensure_parent_dir(&path)?;
    fs::write(path, serde_json::to_string_pretty(state)?)?;
    Ok(())
}

fn read_persisted_scan_cache() -> Option<ModScanCache> {
    let path = scan_cache_file().ok()?;
    let raw = fs::read_to_string(path).ok()?;
    serde_json::from_str::<ModScanCache>(raw.trim_start_matches('\u{feff}')).ok()
}

fn persist_scan_cache(cache: &ModScanCache) -> Result<()> {
    let path = scan_cache_file()?;
    path_utils::ensure_parent_dir(&path)?;
    fs::write(path, serde_json::to_string_pretty(cache)?)?;
    Ok(())
}

// ─── 公开状态操作 ───

/// 从全局存储加载当前 Mod 状态。
pub fn load_mod_state() -> ModState {
    MOD_STATE_STORE
        .lock()
        .map(|state| state.clone())
        .unwrap_or_else(|_| ModState {
            api_version: MOD_API_VERSION,
            ..Default::default()
        })
}

/// 保存 Mod 状态到全局存储并持久化到磁盘。
pub fn save_mod_state(state: &ModState) -> Result<()> {
    if let Ok(mut guard) = MOD_STATE_STORE.lock() {
        *guard = state.clone();
    }
    persist_mod_state(state)?;
    Ok(())
}

/// 加载扫描缓存。
pub fn load_scan_cache() -> ModScanCache {
    read_persisted_scan_cache().unwrap_or_default()
}

/// 保存扫描缓存。
pub fn save_scan_cache(cache: &ModScanCache) -> Result<()> {
    persist_scan_cache(cache)
}

/// 设置指定 Mod 的启用状态。
pub fn set_mod_enabled(namespace: &str, enabled: bool) -> Result<()> {
    let mut state = load_mod_state();
    ensure_mod_state_entry(&mut state, namespace).enabled = enabled;
    save_mod_state(&state)
}

/// 设置指定 Mod 的调试模式。
pub fn set_mod_debug_enabled(namespace: &str, enabled: bool) -> Result<()> {
    let mut state = load_mod_state();
    ensure_mod_state_entry(&mut state, namespace).debug_enabled = enabled;
    save_mod_state(&state)
}

/// 设置指定 Mod 的安全模式。
/// 若 `persist` 为 true，则永久信任（写入持久状态）；
/// 否则仅本次会话生效。
pub fn set_mod_safe_mode(namespace: &str, enabled: bool, persist: bool) -> Result<()> {
    let mut state = load_mod_state();
    let entry = ensure_mod_state_entry(&mut state, namespace);
    if persist {
        entry.safe_mode_enabled = enabled;
        entry.session_safe_mode_enabled = None;
        save_mod_state(&state)
    } else {
        entry.session_safe_mode_enabled = Some(enabled);
        if let Ok(mut guard) = MOD_STATE_STORE.lock() {
            *guard = state;
        }
        Ok(())
    }
}

/// 更新 Mod 游戏的按键绑定。
pub fn update_mod_keybindings(
    namespace: &str,
    game_id: &str,
    script_name: &str,
    bindings: HashMap<String, Vec<String>>,
) -> Result<()> {
    let mut state = load_mod_state();
    let game = ensure_mod_state_entry(&mut state, namespace)
        .games
        .entry(game_id.to_string())
        .or_default();
    game.script_name = script_name.to_string();
    game.keybindings = bindings;
    save_mod_state(&state)
}

/// 读取 Mod 游戏的按键绑定。
pub fn read_mod_keybindings(namespace: &str, game_id: &str) -> HashMap<String, Vec<String>> {
    load_mod_state()
        .mods
        .get(namespace)
        .and_then(|entry| entry.games.get(game_id))
        .map(|game| game.keybindings.clone())
        .unwrap_or_default()
}

/// 更新 Mod 游戏的最佳成绩。
pub fn update_mod_best_score(
    namespace: &str,
    game_id: &str,
    script_name: &str,
    score: JsonValue,
) -> Result<()> {
    let mut state = load_mod_state();
    let game = ensure_mod_state_entry(&mut state, namespace)
        .games
        .entry(game_id.to_string())
        .or_default();
    game.script_name = script_name.to_string();
    game.best_score = score;
    save_mod_state(&state)
}

/// 读取 Mod 游戏的最佳成绩。
pub fn read_mod_best_score(namespace: &str, game_id: &str) -> Option<JsonValue> {
    load_mod_state()
        .mods
        .get(namespace)
        .and_then(|entry| entry.games.get(game_id))
        .map(|game| game.best_score.clone())
}

/// 返回默认 Mod 设置（默认安全模式、默认启用状态）。
pub fn default_mod_settings() -> (bool, bool) {
    let state = load_mod_state();
    (state.default_safe_mode_enabled, state.default_mod_enabled)
}

/// 设置默认安全模式。
pub fn set_default_safe_mode_enabled(enabled: bool) -> Result<()> {
    let mut state = load_mod_state();
    state.default_safe_mode_enabled = enabled;
    save_mod_state(&state)
}

/// 设置默认 Mod 启用状态。
pub fn set_default_mod_enabled(enabled: bool) -> Result<()> {
    let mut state = load_mod_state();
    state.default_mod_enabled = enabled;
    save_mod_state(&state)
}

/// 重置所有 Mod 的安全模式为启用。
pub fn reset_all_mod_safe_modes_enabled() -> Result<()> {
    let mut state = load_mod_state();
    for entry in state.mods.values_mut() {
        entry.safe_mode_enabled = true;
        entry.session_safe_mode_enabled = None;
    }
    save_mod_state(&state)
}

/// 重置所有 Mod 为禁用。
pub fn reset_all_mod_enabled_disabled() -> Result<()> {
    let mut state = load_mod_state();
    for entry in state.mods.values_mut() {
        entry.enabled = false;
    }
    save_mod_state(&state)
}

/// Mod 日志。
/// 仅当调试模式开启或级别为 warn/error 时才会输出。
pub fn mod_log(namespace: &str, level: &str, message: &str) -> Result<()> {
    let state = load_mod_state();
    let debug_enabled = state
        .mods
        .get(namespace)
        .map(|entry| entry.debug_enabled)
        .unwrap_or(false);

    if !debug_enabled {
        let level_lower = level.to_ascii_lowercase();
        if level_lower != "warn" && level_lower != "error" {
            return Ok(());
        }
    }

    let _ = namespace;
    let _ = level;
    let _ = message;
    Ok(())
}