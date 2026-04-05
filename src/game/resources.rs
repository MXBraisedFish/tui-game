use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};
use serde_json::Value as JsonValue;

use crate::app::i18n;
use crate::game::registry::PackageDescriptor;

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

pub fn read_package_text(package: &PackageDescriptor, logical_path: &str) -> Result<String> {
    let path = resolve_package_asset_path(package, logical_path)?;
    let raw = fs::read_to_string(path)?;
    Ok(raw.trim_start_matches('\u{feff}').to_string())
}

pub fn read_package_bytes(package: &PackageDescriptor, logical_path: &str) -> Result<Vec<u8>> {
    let path = resolve_package_asset_path(package, logical_path)?;
    Ok(fs::read(path)?)
}

pub fn read_package_json(package: &PackageDescriptor, logical_path: &str) -> Result<JsonValue> {
    let text = read_package_text(package, logical_path)?;
    Ok(serde_json::from_str(&text)?)
}

pub fn resolve_package_asset_path(
    package: &PackageDescriptor,
    logical_path: &str,
) -> Result<PathBuf> {
    resolve_relative_under(package.root_dir.join("assets"), logical_path)
}

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

fn resolve_package_lang_key(package: &PackageDescriptor, key: &str) -> String {
    let current_code = i18n::current_language_code()
        .replace('-', "_")
        .to_lowercase();
    if let Some(value) = load_package_lang_value(package, &current_code, key) {
        return value;
    }
    if let Some(value) = load_package_lang_value(package, "en_us", key) {
        return value;
    }
    let global = i18n::t_or(key, key);
    if global != key {
        return global;
    }
    format!("[missing-i18n-key:{}:{}]", package.namespace, key)
}

fn load_package_lang_value(package: &PackageDescriptor, code: &str, key: &str) -> Option<String> {
    let lang_path = package
        .root_dir
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
            author: "Tester".to_string(),
            version: "1.0.0".to_string(),
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
