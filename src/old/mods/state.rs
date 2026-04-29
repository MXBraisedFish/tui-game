// 管理 Mod 系统的全局状态，包括内存中的线程安全存储（MOD_STATE_STORE）、状态与扫描缓存的持久化读写，以及所有对外提供的状态修改 API（启用/禁用 Mod、安全模式设置、按键绑定、最佳成绩等

use std::collections::HashMap; // 存储按键绑定、游戏状态映射
use std::fs; // 读写持久化文件
use std::path::PathBuf; // 缓存文件路径
use std::sync::{LazyLock, Mutex}; // 线程安全的全局惰性初始化 + 互斥锁

use anyhow::Result; // 错误处理
use serde_json::Value as JsonValue; // 最佳成绩的 JSON 值

use crate::mods::types::*; // 所有 Mod 类型定义
use crate::utils::path_utils; // 路径工具

// LazyLock<Mutex<ModState>>：全局 Mod 状态存储。惰性初始化，首次访问时从 mod_state.json 读取，失败则使用默认值。所有状态操作通过 Mutex 保证线程安全
static MOD_STATE_STORE: LazyLock<Mutex<ModState>> = LazyLock::new(|| {
    Mutex::new(read_persisted_mod_state().unwrap_or_else(|| ModState {
        api_version: MOD_API_VERSION,
        ..Default::default()
    }))
});

// 获取或创建指定命名空间的 Mod 状态条目。新建时使用全局默认值（default_mod_enabled、default_safe_mode_enabled）
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

// 清理游戏 ID 字符串，只保留 ASCII 字母数字、_、-，替换其他字符为 _，合并连续下划线，去首尾下划线。防空则返回 "mod_save"
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

// 生成 Mod 存档文件的完整路径（mod_save/{namespace}/{sanitized_id}.json）
pub fn mod_save_path(namespace: &str, game_id: &str) -> Result<PathBuf> {
    Ok(path_utils::mod_save_dir()?.join(namespace).join(format!(
        "{}.json",
        sanitize_mod_save_file_stem(game_id)
    )))
}

// 返回 mod_state.json 的路径
fn mod_state_cache_file() -> Result<PathBuf> {
    Ok(path_utils::cache_dir()?.join("mod_state.json"))
}

// 返回 scan_cache.json 的路径
fn scan_cache_file() -> Result<PathBuf> {
    Ok(path_utils::cache_dir()?.join("scan_cache.json"))
}

// 从文件读取 ModState JSON，读取失败返回 None。读取成功后强制设置 api_version 为 MOD_API_VERSION
fn read_persisted_mod_state() -> Option<ModState> {
    let path = mod_state_cache_file().ok()?;
    let raw = fs::read_to_string(path).ok()?;
    let mut state = serde_json::from_str::<ModState>(raw.trim_start_matches('\u{feff}')).ok()?;
    state.api_version = MOD_API_VERSION;
    Some(state)
}

// 将 ModState 写入 mod_state.json
fn persist_mod_state(state: &ModState) -> Result<()> {
    let path = mod_state_cache_file()?;
    path_utils::ensure_parent_dir(&path)?;
    fs::write(path, serde_json::to_string_pretty(state)?)?;
    Ok(())
}

// 从文件读取 ModScanCache JSON
fn read_persisted_scan_cache() -> Option<ModScanCache> {
    let path = scan_cache_file().ok()?;
    let raw = fs::read_to_string(path).ok()?;
    serde_json::from_str::<ModScanCache>(raw.trim_start_matches('\u{feff}')).ok()
}

// 将 ModScanCache 写入 scan_cache.json
fn persist_scan_cache(cache: &ModScanCache) -> Result<()> {
    let path = scan_cache_file()?;
    path_utils::ensure_parent_dir(&path)?;
    fs::write(path, serde_json::to_string_pretty(cache)?)?;
    Ok(())
}

// 从全局 MOD_STATE_STORE 获取当前状态克隆。锁污染时回退默认状态
pub fn load_mod_state() -> ModState {
    MOD_STATE_STORE
        .lock()
        .map(|state| state.clone())
        .unwrap_or_else(|_| ModState {
            api_version: MOD_API_VERSION,
            ..Default::default()
        })
}

// 更新全局内存状态并持久化到磁盘
pub fn save_mod_state(state: &ModState) -> Result<()> {
    if let Ok(mut guard) = MOD_STATE_STORE.lock() {
        *guard = state.clone();
    }
    persist_mod_state(state)?;
    Ok(())
}

// 读取扫描缓存，无则返回默认值
pub fn load_scan_cache() -> ModScanCache {
    read_persisted_scan_cache().unwrap_or_default()
}

// 持久化扫描缓存
pub fn save_scan_cache(cache: &ModScanCache) -> Result<()> {
    persist_scan_cache(cache)
}

// 设置指定 Mod 的启用状态并保存
pub fn set_mod_enabled(namespace: &str, enabled: bool) -> Result<()> {
    let mut state = load_mod_state();
    ensure_mod_state_entry(&mut state, namespace).enabled = enabled;
    save_mod_state(&state)
}

// 设置指定 Mod 的调试模式并保存
pub fn set_mod_debug_enabled(namespace: &str, enabled: bool) -> Result<()> {
    let mut state = load_mod_state();
    ensure_mod_state_entry(&mut state, namespace).debug_enabled = enabled;
    save_mod_state(&state)
}

// 设置指定 Mod 的安全模式。persist=true 永久信任（持久化），persist=false 仅修改内存中的 session_safe_mode_enabled（不写入文件）
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

// 更新指定 Mod 游戏的按键绑定并保存
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

// 读取指定 Mod 游戏的按键绑定，无则返回空 HashMap
pub fn read_mod_keybindings(namespace: &str, game_id: &str) -> HashMap<String, Vec<String>> {
    load_mod_state()
        .mods
        .get(namespace)
        .and_then(|entry| entry.games.get(game_id))
        .map(|game| game.keybindings.clone())
        .unwrap_or_default()
}

// 更新指定 Mod 游戏的最佳成绩并保存
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

// 读取指定 Mod 游戏的最佳成绩，无则返回 None
pub fn read_mod_best_score(namespace: &str, game_id: &str) -> Option<JsonValue> {
    load_mod_state()
        .mods
        .get(namespace)
        .and_then(|entry| entry.games.get(game_id))
        .map(|game| game.best_score.clone())
}

// 返回全局默认设置：(default_safe_mode_enabled, default_mod_enabled)
pub fn default_mod_settings() -> (bool, bool) {
    let state = load_mod_state();
    (state.default_safe_mode_enabled, state.default_mod_enabled)
}

// 设置全局默认安全模式并保存
pub fn set_default_safe_mode_enabled(enabled: bool) -> Result<()> {
    let mut state = load_mod_state();
    state.default_safe_mode_enabled = enabled;
    save_mod_state(&state)
}

// 设置全局默认 Mod 启用状态并保存
pub fn set_default_mod_enabled(enabled: bool) -> Result<()> {
    let mut state = load_mod_state();
    state.default_mod_enabled = enabled;
    save_mod_state(&state)
}

// 批量操作：将所有 Mod 的安全模式设为启用，清除 session 级禁用，保存
pub fn reset_all_mod_safe_modes_enabled() -> Result<()> {
    let mut state = load_mod_state();
    for entry in state.mods.values_mut() {
        entry.safe_mode_enabled = true;
        entry.session_safe_mode_enabled = None;
    }
    save_mod_state(&state)
}

// 批量操作：将所有 Mod 设为禁用，保存
pub fn reset_all_mod_enabled_disabled() -> Result<()> {
    let mut state = load_mod_state();
    for entry in state.mods.values_mut() {
        entry.enabled = false;
    }
    save_mod_state(&state)
}

// Mod 日志（当前为空实现，保留了 namespace、level、message 参数供未来扩展）。仅当调试模式开启或级别为 warn/error 时写入
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