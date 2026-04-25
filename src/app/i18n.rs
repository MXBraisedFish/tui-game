use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use anyhow::Result;
use once_cell::sync::Lazy;
use serde_json::Value;

use crate::utils::path_utils;

const REQUIRED_KEYS: [&str; 4] = [
    "language_name",
    "language",
    "language.hint.segment.confirm",
    "language.hint.segment.back",
];

#[derive(Clone, Debug)]
/// 单个语言包的数据结构。
pub struct LanguagePack {
    pub code: String,
    pub name: String,
    pub dict: HashMap<String, String>,
}

#[derive(Clone, Debug)]
/// 国际化系统的全局状态。
struct I18nState {
    packs: Vec<LanguagePack>,
    fallback: LanguagePack,
    current_code: String,
}

static I18N: Lazy<RwLock<I18nState>> = Lazy::new(|| {
    let fallback = builtin_english_pack();
    RwLock::new(I18nState {
        packs: vec![fallback.clone()],
        fallback: fallback.clone(),
        current_code: fallback.code.clone(),
    })
});

/// 初始化国际化系统，并从 `assets/lang` 中加载全部语言包。
pub fn init(default_code: &str) -> Result<()> {
    let mut packs = load_language_packs()?;
    packs.sort_by(|a, b| b.name.cmp(&a.name));

    let fallback = packs
        .iter()
        .find(|pack| pack.code == "us-en")
        .cloned()
        .unwrap_or_else(builtin_english_pack);

    if packs.is_empty() {
        packs.push(fallback.clone());
    }

    let preferred_code = load_persisted_language_code()
        .ok()
        .flatten()
        .unwrap_or_else(|| default_code.to_string());

    let current_code = if packs.iter().any(|pack| pack.code == preferred_code) {
        preferred_code
    } else if packs.iter().any(|pack| pack.code == default_code) {
        default_code.to_string()
    } else if packs.iter().any(|pack| pack.code == "us-en") {
        "us-en".to_string()
    } else {
        fallback.code.clone()
    };

    if let Ok(mut state) = I18N.write() {
        *state = I18nState {
            packs,
            fallback,
            current_code,
        };
    }

    Ok(())
}

/// 返回所有有效语言包，并按 `language_name` 的 Unicode 值降序排列。
pub fn available_languages() -> Vec<LanguagePack> {
    if let Ok(state) = I18N.read() {
        return state.packs.clone();
    }
    vec![builtin_english_pack()]
}

/// 返回当前正在使用的语言代码。
pub fn current_language_code() -> String {
    if let Ok(state) = I18N.read() {
        return state.current_code.clone();
    }
    "us-en".to_string()
}

/// 根据语言代码切换当前语言，切换成功时返回 `true`。
pub fn set_language(code: &str) -> bool {
    if let Ok(mut state) = I18N.write() {
        if state.packs.iter().any(|pack| pack.code == code) {
            state.current_code = code.to_string();
            let _ = save_persisted_language_code(code);
            return true;
        }
    }
    false
}

/// 在当前语言中查找指定键，缺失时回退到内置英文。
pub fn t(key: &str) -> String {
    if let Ok(state) = I18N.read() {
        if let Some(current_pack) = state
            .packs
            .iter()
            .find(|pack| pack.code == state.current_code)
        {
            if let Some(value) = current_pack.dict.get(key) {
                return value.clone();
            }
        }

        if let Some(value) = state.fallback.dict.get(key) {
            return value.clone();
        }
    }

    format!("[missing-i18n-key:{}]", key)
}

/// 在指定语言中查找指定键，缺失时回退到英文。
pub fn t_for_code(code: &str, key: &str) -> String {
    if let Ok(state) = I18N.read() {
        if let Some(pack) = state.packs.iter().find(|pack| pack.code == code) {
            if let Some(value) = pack.dict.get(key) {
                return value.clone();
            }
        }

        if let Some(value) = state.fallback.dict.get(key) {
            return value.clone();
        }
    }

    format!("[missing-i18n-key:{}]", key)
}

/// 在当前语言中查找键；如果依然缺失，则回退到调用方提供的文本。
pub fn t_or(key: &str, fallback: &str) -> String {
    let value = t(key);
    if value.starts_with("[missing-i18n-key:") {
        fallback.to_string()
    } else {
        value
    }
}

fn load_language_packs() -> Result<Vec<LanguagePack>> {
    let mut packs = Vec::new();
    for lang_dir in resolve_lang_dirs() {
        for entry in fs::read_dir(lang_dir)? {
            let path = match entry {
                Ok(e) => e.path(),
                Err(_) => continue,
            };

            if !path
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("json"))
                .unwrap_or(false)
            {
                continue;
            }

            if let Some(pack) = parse_language_pack(&path)? {
                packs.push(pack);
            }
        }
        if !packs.is_empty() {
            break;
        }
    }

    Ok(packs)
}

fn parse_language_pack(path: &Path) -> Result<Option<LanguagePack>> {
    let code = match path.file_stem().and_then(|s| s.to_str()) {
        Some(stem) => stem.to_ascii_lowercase(),
        None => return Ok(None),
    };

    let content = fs::read_to_string(path)?;
    let content = content.trim_start_matches('\u{feff}');
    let value: Value = match serde_json::from_str(content) {
        Ok(v) => v,
        Err(_) => return Ok(None),
    };

    let object = match value.as_object() {
        Some(map) => map,
        None => return Ok(None),
    };

    if !REQUIRED_KEYS
        .iter()
        .all(|key| object.get(*key).and_then(Value::as_str).is_some())
    {
        return Ok(None);
    }

    let mut dict = HashMap::new();
    for (key, value) in object {
        if let Some(text) = value.as_str() {
            dict.insert(key.to_string(), text.to_string());
        }
    }

    let name = match dict.get("language_name") {
        Some(v) => v.clone(),
        None => return Ok(None),
    };

    Ok(Some(LanguagePack { code, name, dict }))
}

fn resolve_lang_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    if let Ok(cwd) = std::env::current_dir() {
        for ancestor in cwd.ancestors() {
            let candidate = ancestor.join("assets").join("lang");
            if candidate.exists() {
                dirs.push(candidate);
            }
        }
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            for ancestor in parent.ancestors() {
                let candidate = ancestor.join("assets").join("lang");
                if candidate.exists() && !dirs.iter().any(|d| d == &candidate) {
                    dirs.push(candidate);
                }
            }
        }
    }

    dirs
}

fn load_persisted_language_code() -> Result<Option<String>> {
    let path = path_utils::language_file()?;
    if !path.exists() {
        return Ok(None);
    }

    let code = fs::read_to_string(path)?;
    let normalized = code.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        Ok(None)
    } else {
        Ok(Some(normalized))
    }
}

fn save_persisted_language_code(code: &str) -> Result<()> {
    let path = path_utils::language_file()?;
    path_utils::ensure_parent_dir(&path)?;
    fs::write(path, format!("{}\n", code.trim().to_ascii_lowercase()))?;
    Ok(())
}

fn builtin_english_pack() -> LanguagePack {
    const US_EN_JSON: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/lang/us-en.json"
    ));

    let parsed = serde_json::from_str::<Value>(US_EN_JSON)
        .ok()
        .and_then(|v| v.as_object().cloned());

    if let Some(map) = parsed {
        let mut dict = HashMap::new();
        for (key, value) in map {
            if let Some(text) = value.as_str() {
                dict.insert(key, text.to_string());
            }
        }

        if REQUIRED_KEYS.iter().all(|k| dict.contains_key(*k)) {
            let name = dict
                .get("language_name")
                .cloned()
                .unwrap_or_else(|| "English".to_string());
            return LanguagePack {
                code: "us-en".to_string(),
                name,
                dict,
            };
        }
    }

    minimal_builtin_english_pack()
}

fn minimal_builtin_english_pack() -> LanguagePack {
    let mut dict = HashMap::new();
    dict.insert("language_name".to_string(), "English".to_string());
    dict.insert("language".to_string(), "Language".to_string());
    dict.insert(
        "language.hint.segment.confirm".to_string(),
        "[Enter] Confirm language".to_string(),
    );
    dict.insert(
        "language.hint.segment.back".to_string(),
        "[ESC]/[Q] Return to main menu".to_string(),
    );

    LanguagePack {
        code: "us-en".to_string(),
        name: "English".to_string(),
        dict,
    }
}
