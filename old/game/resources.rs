// 游戏包资源访问与国际化文本解析。提供从包内读取资产文件（文本、二进制、JSON）的安全接口，并实现包级别的 i18n 文本解析（支持 namespace:key 语法）

use std::collections::HashMap; // 存储语言缓存
use std::fs; // 文件读取
use std::path::{Path, PathBuf}; // 路径操作
use std::sync::RwLock; // 线程安全的语言缓存

use anyhow::{Result, anyhow}; // 错误处理
use once_cell::sync::Lazy; // 静态缓存初始化
use serde_json::Value as JsonValue; // JSON 返回值

use crate::app::i18n; // 获取当前语言代码
use crate::game::registry::PackageDescriptor; // 包描述符

// 结构：包缓存键 -> 语言代码 -> 键值对
type PackageLangCache = HashMap<String, HashMap<String, HashMap<String, String>>>;

static PACKAGE_LANG_CACHE: Lazy<RwLock<PackageLangCache>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

// 解析可能含 namespace:key 或纯 i18n 键的字符串，返回本地化文本。若无法解析则返回原字符串
pub fn resolve_package_text(package: &PackageDescriptor, raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    if let Some((prefix, key)) = trimmed.split_once(':') {
        if prefix == package.namespace && !key.contains('/') && !key.contains('\\') {
            return resolve_package_lang_key(package, key);
        }
    }

    if is_probable_lang_key(trimmed) {
        let resolved = resolve_package_lang_key(package, trimmed);
        if !resolved.starts_with("[missing-i18n-key:") {
            return resolved;
        }
    }

    trimmed.to_string()
}

// 重建所有包的语言缓存，通常在语言切换或包变更时调用
pub fn rebuild_package_language_cache(packages: &[PackageDescriptor]) {
    let current_code = i18n::current_language_code()
        .replace('-', "_")
        .to_lowercase();
    let mut next = HashMap::new();

    for package in packages {
        let key = package_cache_key(package);
        if next.contains_key(&key) {
            continue;
        }

        let mut per_lang = HashMap::new();
        if let Some(dict) = load_package_lang_dict(package, &current_code) {
            per_lang.insert(current_code.clone(), dict);
        }
        if current_code != "en_us"
            && let Some(dict) = load_package_lang_dict(package, "en_us")
        {
            per_lang.insert("en_us".to_string(), dict);
        }
        next.insert(key, per_lang);
    }

    if let Ok(mut cache) = PACKAGE_LANG_CACHE.write() {
        *cache = next;
    }
}

// 读取文本文件（自动去除 BOM）
pub fn read_package_text(package: &PackageDescriptor, logical_path: &str) -> Result<String> {
    let path = resolve_package_asset_path(package, logical_path)?;
    let raw = fs::read_to_string(path)?;
    Ok(raw.trim_start_matches('\u{feff}').to_string())
}

// 读取二进制文件
pub fn read_package_bytes(package: &PackageDescriptor, logical_path: &str) -> Result<Vec<u8>> {
    let path = resolve_package_asset_path(package, logical_path)?;
    Ok(fs::read(path)?)
}

// 读取并解析 JSON 文件
pub fn read_package_json(package: &PackageDescriptor, logical_path: &str) -> Result<JsonValue> {
    let text = read_package_text(package, logical_path)?;
    Ok(serde_json::from_str(&text)?)
}

// 将逻辑路径解析为 assets/ 下的绝对路径，防止路径穿越
pub fn resolve_package_asset_path(
    package: &PackageDescriptor,
    logical_path: &str,
) -> Result<PathBuf> {
    resolve_relative_under(package.root_dir.join("assets"), logical_path)
}

// 解析辅助脚本路径（限定在 scripts/ 下，且必须为 .lua 文件）
pub fn resolve_package_helper_path(
    package: &PackageDescriptor,
    logical_path: &str,
) -> Result<PathBuf> {
    let helper_path = resolve_relative_under(package.root_dir.join("scripts"), logical_path)?;
    if helper_path
        .extension()
        .and_then(|value| value.to_str())
        .map(|ext| !ext.eq_ignore_ascii_case("lua"))
        .unwrap_or(true)
    {
        return Err(anyhow!("helper path must point to a .lua file"));
    }
    Ok(helper_path)
}

// 根据当前语言加载包的字典，查找键值，支持回退到 en_us 和全局 i18n
fn resolve_package_lang_key(package: &PackageDescriptor, key: &str) -> String {
    let current_code = i18n::current_language_code()
        .replace('-', "_")
        .to_lowercase();
    if let Some(value) = cached_package_lang_value(package, &current_code, key) {
        return value;
    }
    if let Some(value) = uncached_package_lang_value(package, &current_code, key) {
        return value;
    }
    if let Some(value) = cached_package_lang_value(package, "en_us", key) {
        return value;
    }
    if let Some(value) = uncached_package_lang_value(package, "en_us", key) {
        return value;
    }
    let global = i18n::t_or(key, key);
    if global != key {
        return global;
    }
    format!("[missing-i18n-key:{}:{}]", package.namespace, key)
}

// 从缓存读取
fn cached_package_lang_value(package: &PackageDescriptor, code: &str, key: &str) -> Option<String> {
    PACKAGE_LANG_CACHE
        .read()
        .ok()?
        .get(&package_cache_key(package))?
        .get(code)?
        .get(key)
        .cloned()
}

// 实时从磁盘加载字典并读取
fn uncached_package_lang_value(package: &PackageDescriptor, code: &str, key: &str) -> Option<String> {
    load_package_lang_dict(package, code)?.get(key).cloned()
}

// 加载 assets/lang/{code}.json 文件
fn load_package_lang_dict(package: &PackageDescriptor, code: &str) -> Option<HashMap<String, String>> {
    let lang_path = package
        .root_dir
        .join("assets")
        .join("lang")
        .join(format!("{code}.json"));
    let raw = fs::read_to_string(lang_path).ok()?;
    let json = serde_json::from_str::<JsonValue>(raw.trim_start_matches('\u{feff}')).ok()?;
    let mut dict = HashMap::new();
    for (entry_key, value) in json.as_object()? {
        if let Some(text) = value.as_str() {
            dict.insert(entry_key.to_string(), text.to_string());
        }
    }
    Some(dict)
}

// 生成包缓存键（使用 root_dir 字符串）
fn package_cache_key(package: &PackageDescriptor) -> String {
    package.root_dir.to_string_lossy().to_string()
}

// 基础路径解析函数，禁止绝对路径和 .. 逃逸
fn resolve_relative_under(root: PathBuf, logical_path: &str) -> Result<PathBuf> {
    let path = Path::new(logical_path);
    if logical_path.trim().is_empty() {
        return Err(anyhow!("path cannot be blank"));
    }
    if path.is_absolute() || logical_path.starts_with('/') || logical_path.starts_with('\\') {
        return Err(anyhow!("path must be relative"));
    }
    if logical_path
        .split(['/', '\\'])
        .any(|segment| segment.is_empty() || segment == "." || segment == "..")
    {
        return Err(anyhow!("path cannot escape package root"));
    }

    let resolved = root.join(path);
    if !resolved.starts_with(&root) {
        return Err(anyhow!("path cannot escape package root"));
    }
    Ok(resolved)
}

// 启发式判断是否可能是语言键（包含点号、无斜杠、无空白）
fn is_probable_lang_key(value: &str) -> bool {
    value.contains('.')
        && !value.contains('/')
        && !value.contains('\\')
        && !value.chars().any(char::is_whitespace)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::registry::GameSourceKind;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_test_dir(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time drift")
            .as_nanos();
        std::env::temp_dir().join(format!("tui_game_resources_{name}_{unique}"))
    }

    fn test_package(root_dir: PathBuf) -> PackageDescriptor {
        PackageDescriptor {
            root_dir,
            namespace: "demo".to_string(),
            package_name: "Demo".to_string(),
            mod_name: None,
            author: "Tester".to_string(),
            version: "1.0.0".to_string(),
            debug_enabled: false,
            source: GameSourceKind::Official,
        }
    }

    #[test]
    fn package_asset_paths_reject_escape_sequences() {
        let root = temp_test_dir("asset_paths");
        fs::create_dir_all(root.join("assets")).expect("create assets dir");
        let package = test_package(root.clone());

        assert!(resolve_package_asset_path(&package, "data/file.json").is_ok());
        assert!(resolve_package_asset_path(&package, "../escape.json").is_err());
        assert!(resolve_package_asset_path(&package, "/absolute.json").is_err());
        assert!(resolve_package_asset_path(&package, "data/../escape.json").is_err());

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn package_text_prefers_current_package_language_then_en_us() {
        let root = temp_test_dir("lang_resolution");
        fs::create_dir_all(root.join("assets").join("lang")).expect("create lang dir");
        fs::write(
            root.join("assets").join("lang").join("en_us.json"),
            "{\n  \"game.demo.name\": \"Demo Name\",\n  \"game.demo.only_en\": \"English Only\"\n}\n",
        )
        .expect("write en_us");
        fs::write(
            root.join("assets").join("lang").join("zh_cn.json"),
            "{\n  \"game.demo.name\": \"演示名称\"\n}\n",
        )
        .expect("write zh_cn");

        let package = test_package(root.clone());
        let resolved = resolve_package_text(&package, "game.demo.name");
        assert!(resolved == "演示名称" || resolved == "Demo Name");

        let fallback = resolve_package_text(&package, "game.demo.only_en");
        assert_eq!(fallback, "English Only");

        let _ = fs::remove_dir_all(root);
    }
}
