// 国际化多语言系统，负责加载 assets/lang/ 目录下的 JSON 语言包，提供运行时语言切换、键值查找、回退机制和编译时内置英文后备

use std::collections::HashMap; // 存储翻译键值对
use std::fs; // 读取语言文件和持久化偏好
use std::path::{Path, PathBuf}; // 路径操作
use std::sync::RwLock; // 全局语言状态的线程安全防护

use anyhow::Result; // 错误处理
use once_cell::sync::Lazy; // 惰性静态初始化
use serde_json::Value; // 解析语言 JSON

use crate::utils::path_utils; // 持久化文件路径

// 语言包必须包含的 4 个键：language_name、language、language.hint.segment.confirm、language.hint.segment.back
const REQUIRED_KEYS: [&str; 4] = [
    "language_name",
    "language",
    "language.hint.segment.confirm",
    "language.hint.segment.back",
];

// 单个语言包
#[derive(Clone, Debug)]
/// 单个语言包的数据结构。
pub struct LanguagePack {
    pub code: String,
    pub name: String,
    pub dict: HashMap<String, String>,
}

// 国际化系统全局状态（私有）
#[derive(Clone, Debug)]
struct I18nState {
    packs: Vec<LanguagePack>,
    fallback: LanguagePack,
    current_code: String,
}

// Lazy<RwLock<I18nState>>：全局国际化状态，首次访问时用内置英文包初始化
static I18N: Lazy<RwLock<I18nState>> = Lazy::new(|| {
    let fallback = builtin_english_pack();
    RwLock::new(I18nState {
        packs: vec![fallback.clone()],
        fallback: fallback.clone(),
        current_code: fallback.code.clone(),
    })
});

// 加载所有语言包 → 排序（按名称降序）→ 确定回退包（us-en）→ 确定当前语言（持久化偏好 > 默认 > us-en > 回退）→ 更新全局状态
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

// 返回所有有效语言包列表
pub fn available_languages() -> Vec<LanguagePack> {
    if let Ok(state) = I18N.read() {
        return state.packs.clone();
    }
    vec![builtin_english_pack()]
}

// 返回当前使用的语言代码
pub fn current_language_code() -> String {
    if let Ok(state) = I18N.read() {
        return state.current_code.clone();
    }
    "us-en".to_string()
}

// 切换语言：验证语言存在 → 更新当前代码 → 持久化偏好 → 返回成功/失败
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

// 当前语言查找，缺失回退英文后备，再缺失返回 [missing-i18n-key:...]
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

// 指定语言查找，缺失回退英文后备，再缺失返回占位符
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

// 当前语言查找，若缺失则返回调用方提供的回退文本
pub fn t_or(key: &str, fallback: &str) -> String {
    let value = t(key);
    if value.starts_with("[missing-i18n-key:") {
        fallback.to_string()
    } else {
        value
    }
}

// 遍历语言目录，逐个解析 JSON 文件
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

// 解析单个语言 JSON 文件：验证必须键 → 构建 HashMap → 返回 LanguagePack
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

// 从当前目录和可执行文件目录向上查找 assets/lang/
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

// 读取 language.txt 中的持久化语言偏好
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

// 将语言偏好写入 language.txt
fn save_persisted_language_code(code: &str) -> Result<()> {
    let path = path_utils::language_file()?;
    path_utils::ensure_parent_dir(&path)?;
    fs::write(path, format!("{}\n", code.trim().to_ascii_lowercase()))?;
    Ok(())
}

// 编译时内置英文语言包：通过 include_str! 嵌入 assets/lang/us-en.json，解析失败时使用最小内置包
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

// 提供最小可用的英文语言包（仅含 4 个必须键）
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
