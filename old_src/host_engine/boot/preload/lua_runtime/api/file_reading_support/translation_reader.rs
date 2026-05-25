//! 包内语言读取

use std::path::Path;

use serde_json::Value as JsonValue;

/// 翻译结果。
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TranslationResult {
    Found(String),
    MissingInCurrentLanguage,
    MissingInFallback,
}

/// 从包内语言文件读取指定语言键。
pub fn read_translation(
    package_root: &Path,
    language_code: &str,
    key: &str,
) -> mlua::Result<TranslationResult> {
    if key.trim().is_empty() {
        return Err(mlua::Error::external("translation key is empty"));
    }

    if let Some(value) = read_language_value(package_root, language_code, key)? {
        return Ok(TranslationResult::Found(value));
    }

    if language_code != "en_us" {
        if let Some(value) = read_language_value(package_root, "en_us", key)? {
            return Ok(TranslationResult::Found(value));
        }
        return Ok(TranslationResult::MissingInFallback);
    }

    Ok(TranslationResult::MissingInCurrentLanguage)
}

fn read_language_value(
    package_root: &Path,
    language_code: &str,
    key: &str,
) -> mlua::Result<Option<String>> {
    let language_path = package_root
        .join("assets")
        .join("lang")
        .join(format!("{language_code}.json"));
    if !language_path.is_file() {
        return Ok(None);
    }

    let raw_text = std::fs::read_to_string(language_path)
        .map_err(|error| mlua::Error::external(format!("failed to read language file: {error}")))?;
    let language_data = serde_json::from_str::<JsonValue>(raw_text.trim_start_matches('\u{feff}'))
        .map_err(|error| mlua::Error::external(format!("invalid language file: {error}")))?;

    Ok(language_data
        .as_object()
        .and_then(|object| object.get(key))
        .and_then(JsonValue::as_str)
        .map(ToString::to_string))
}
