//! 宿主动态数据目录检查
//! 动态文件只保证存在，不校验内容

use std::fs;
use std::path::{Path, PathBuf};

type EnvironmentResult<T> = Result<T, Box<dyn std::error::Error>>;

const EMPTY_JSON_OBJECT: &str = "{}";
const DEFAULT_LANGUAGE_CODE: &str = "en_us";

/// 确保宿主运行期动态目录与文件存在
pub fn ensure() -> EnvironmentResult<()> {
    let root_dir = root_dir();

    ensure_dir(&root_dir.join("data"))?;

    ensure_dir(&root_dir.join("data/cache"))?;
    ensure_dir(&root_dir.join("data/cache/images"))?;
    ensure_file(
        &root_dir.join("data/cache/mod_scan_cache.json"),
        EMPTY_JSON_OBJECT,
    )?;
    ensure_file(
        &root_dir.join("data/cache/screensaver_scan_cache"),
        EMPTY_JSON_OBJECT,
    )?;
    ensure_file(
        &root_dir.join("data/cache/boss_scan_cache"),
        EMPTY_JSON_OBJECT,
    )?;
    ensure_file(
        &root_dir.join("data/cache/language_ui_cache.json"),
        EMPTY_JSON_OBJECT,
    )?;

    ensure_dir(&root_dir.join("data/profiles"))?;
    ensure_file(
        &root_dir.join("data/profiles/saves.json"),
        EMPTY_JSON_OBJECT,
    )?;
    ensure_file(
        &root_dir.join("data/profiles/best_scores.json"),
        EMPTY_JSON_OBJECT,
    )?;
    ensure_file(
        &root_dir.join("data/profiles/language.txt"),
        DEFAULT_LANGUAGE_CODE,
    )?;
    ensure_file(
        &root_dir.join("data/profiles/keybind.json"),
        EMPTY_JSON_OBJECT,
    )?;
    ensure_file(
        &root_dir.join("data/profiles/game_state.json"),
        EMPTY_JSON_OBJECT,
    )?;
    ensure_file(
        &root_dir.join("data/profiles/screensaver_state"),
        EMPTY_JSON_OBJECT,
    )?;
    ensure_file(
        &root_dir.join("data/profiles/boss_state"),
        EMPTY_JSON_OBJECT,
    )?;
    ensure_file(
        &root_dir.join("data/profiles/display_state.json"),
        r#"{
  "mod_badge": true,
  "theme": "system",
  "idle_threshold": 60,
  "idle_enter_screensaver": false,
  "host_status": false,
  "screensaver_mode": "ordered",
  "boss_mode": "ordered",
  "screensaver_list": { "order": [], "enabled": {}, "cursor": 0 },
  "boss_list": { "order": [], "enabled": {}, "cursor": 0 }
}"#,
    )?;
    ensure_file(
        &root_dir.join("data/profiles/security_state.json"),
        r#"{
  "default_safe_mode": true,
  "default_mod_game_enabled": true,
  "default_mod_screensaver_enabled": true,
  "default_mod_boss_enabled": true
}"#,
    )?;

    ensure_dir(&root_dir.join("data/log"))?;
    ensure_file(&root_dir.join("data/log/tui_log.txt"), "")?;

    ensure_dir(&root_dir.join("data/mod"))?;
    ensure_dir(&root_dir.join("data/mod/game"))?;
    ensure_dir(&root_dir.join("data/mod/screensaver"))?;
    ensure_dir(&root_dir.join("data/mod/boss"))?;

    Ok(())
}

/// 确保目录存在，不存在则创建
fn ensure_dir(path: &Path) -> EnvironmentResult<()> {
    fs::create_dir_all(path)?;
    Ok(())
}

/// 确保文件存在，不存在则写入默认内容
fn ensure_file(path: &Path, default_content: &str) -> EnvironmentResult<()> {
    if path.exists() {
        return Ok(());
    }

    if let Some(parent_dir) = path.parent() {
        fs::create_dir_all(parent_dir)?;
    }
    fs::write(path, default_content)?;

    Ok(())
}

/// 获取宿主根目录。开发环境优先使用当前目录，打包环境退回可执行文件目录。
pub fn root_dir() -> PathBuf {
    std::env::current_dir()
        .ok()
        .filter(|path| path.join("assets").exists() || path.join("Cargo.toml").exists())
        .or_else(|| {
            std::env::current_exe()
                .ok()
                .and_then(|path| path.parent().map(Path::to_path_buf))
        })
        .unwrap_or_else(|| PathBuf::from("."))
}
