// mods 模块的入口文件，声明并重新导出四个子模块（types、state、scan、image），同时直接提供路径工具（mod_root_dir、mod_data_dir 等）、Mod 文本国际化解析（resolve_mod_text）、资源路径解析（resolve_asset_path）及辅助函数（mtime_secs、is_probable_lang_key）

// 所有 Mod 相关数据结构定义
pub mod types;
pub use types::*;

// Mod 状态管理、持久化读写
pub mod state;
pub use state::*;

// Mod 包扫描与验证
pub mod scan;
pub use scan::*;

// 图像处理（ASCII/光栅）
pub mod image;
pub use image::*;

use std::fs; // 读取语言 JSON 文件
use std::path::{Path, PathBuf}; // 路径处理
use std::time::{UNIX_EPOCH}; // 计算文件修改时间（mtime_secs）

use anyhow::{Result, anyhow}; // 错误处理
use serde_json::Value as JsonValue; // 解析语言包 JSON

use crate::app::i18n; // 获取当前语言代码、回退到宿主国际化
use crate::utils::path_utils; // 获取基础路径（app_data_dir、cache_dir、mod_save_dir）

const DEFAULT_PACKAGE_DESCRIPTION: &str = "No package description available."; // 当 Mod 包没有提供描述时的默认文本
const DEFAULT_GAME_DESCRIPTION: &str = "No description available."; // 当游戏清单没有描述时的默认文本
const DEFAULT_GAME_DETAIL: &str = ""; // 当游戏清单没有详情时的默认空字符串

// 获取 Mod 根目录（tui-game-data/mod）
pub fn mod_root_dir() -> Result<PathBuf> {
    Ok(path_utils::app_data_dir()?.join("mod"))
}

// 等同于 mod_root_dir()，提供语义化别名
pub fn mod_data_dir() -> Result<PathBuf> {
    mod_root_dir()
}

// 获取缓存目录
pub fn mod_cache_dir() -> Result<PathBuf> {
    path_utils::cache_dir()
}

// 获取指定命名空间的 Mod 存档子目录
pub fn mod_save_dir(namespace: &str) -> Result<PathBuf> {
    Ok(path_utils::mod_save_dir()?.join(namespace))
}

// 公开接口，解析 Mod 文本（国际化），等价于私有 resolve_mod_text
pub fn resolve_mod_text_for_display(namespace: &str, raw: &str) -> String {
    resolve_mod_text(namespace, raw)
}

// 核心解析：namespace:key 格式直接查包内语言文件；纯 i18n 键回退宿主；否则返回原文本
fn resolve_mod_text(namespace: &str, raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    if let Some((prefix, key)) = trimmed.split_once(':') {
        if prefix == namespace && !key.contains('/') && !key.contains('\\') {
            return resolve_mod_lang_key(namespace, key);
        }
    }

    if is_probable_lang_key(trimmed) {
        let resolved = resolve_mod_lang_key(namespace, trimmed);
        if !resolved.starts_with("[missing-i18n-key:") {
            return resolved;
        }
    }

    trimmed.to_string()
}

// 在 Mod 的语言文件中查找翻译，回退链：当前语言 → en_us → 缺失占位符
fn resolve_mod_lang_key(namespace: &str, key: &str) -> String {
    let current_code = i18n::current_language_code()
        .replace('-', "_")
        .to_lowercase();
    if let Some(value) = load_mod_lang_value(namespace, &current_code, key) {
        return value;
    }
    if let Some(value) = load_mod_lang_value(namespace, "en_us", key) {
        return value;
    }
    format!("[missing-i18n-key:{namespace}:{key}]")
}

// 读取指定 Mod 的语言 JSON 文件，查找单个键的值
fn load_mod_lang_value(namespace: &str, code: &str, key: &str) -> Option<String> {
    let lang_path = mod_data_dir()
        .ok()?
        .join(namespace)
        .join("assets")
        .join("lang")
        .join(format!("{code}.json"));
    let raw = fs::read_to_string(lang_path).ok()?;
    let json = serde_json::from_str::<JsonValue>(raw.trim_start_matches('\u{feff}')).ok()?;
    json.as_object()?
        .get(key)?
        .as_str()
        .map(|value| value.to_string())
}

// 安全解析 Mod 包内 assets/ 目录下的相对路径，禁止路径逃逸
fn resolve_asset_path(namespace: &str, path_str: &str) -> Result<PathBuf> {
    if path_str.starts_with('/') || path_str.starts_with('\\') {
        return Err(anyhow!("asset path must be relative"));
    }
    let asset_path = mod_data_dir()?
        .join(namespace)
        .join("assets")
        .join(path_str);
    let asset_root = mod_data_dir()?.join(namespace).join("assets");
    let normalized = asset_path.components().collect::<PathBuf>();
    if path_str
        .split(['/', '\\'])
        .any(|segment| segment == "." || segment == "..")
    {
        return Err(anyhow!("asset path cannot escape assets directory"));
    }
    if !normalized.starts_with(&asset_root) {
        return Err(anyhow!("asset path cannot escape assets directory"));
    }
    Ok(normalized)
}

// 判断字符串是否像一个 i18n 键（只包含字母数字、.、_、- 且含 .）
fn is_probable_lang_key(value: &str) -> bool {
    value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-'))
        && value.contains('.')
}

// 获取文件的修改时间戳（秒），用于缓存失效判断
fn mtime_secs(path: &Path) -> u64 {
    fs::metadata(path)
        .and_then(|meta| meta.modified())
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|value| value.as_secs())
        .unwrap_or(0)
}
