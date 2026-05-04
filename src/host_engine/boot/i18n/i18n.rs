//! 宿主界面语言加载模块
//! 只负责读取语言文件、处理 fallback、调用分类模块注册文本

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use once_cell::sync::OnceCell;

use super::r#type::{global, home, key, loading, setting, start, warning};

const DEFAULT_LANGUAGE_CODE: &str = "en_us";
const LANGUAGE_PROFILE_PATH: &str = "data/profiles/language.txt";
const LANGUAGE_DIR: &str = "assets/lang";

static I18N_TEXT: OnceCell<I18nText> = OnceCell::new();

type I18nResult<T> = Result<T, Box<dyn std::error::Error>>;

/// 已注册的宿主语言文本集合
#[derive(Clone, Copy)]
pub struct I18nText {
    pub global: global::GlobalText,
    pub home: home::HomeText,
    pub key: key::KeyText,
    pub loading: loading::LoadingText,
    pub setting: setting::SettingText,
    pub start: start::StartText,
    pub warning: warning::WarningText,
}

/// 语言源数据，供分类注册模块按键读取
pub struct LanguageSource {
    preferred_texts: HashMap<String, String>,
    fallback_texts: HashMap<String, String>,
    is_default_language: bool,
}

/// 加载宿主语言文件并注册伪常量
pub fn load() -> I18nResult<()> {
    let language_source = load_language_source();
    let global_text = global::register(&language_source);
    let home_text = home::register(&language_source);
    let key_text = key::register(&language_source);
    let loading_text = loading::register(&language_source);
    let setting_text = setting::register(&language_source);
    let start_text = start::register(&language_source);
    let warning_text = warning::register(&language_source);

    let _ = I18N_TEXT.set(I18nText {
        global: global_text,
        home: home_text,
        key: key_text,
        loading: loading_text,
        setting: setting_text,
        start: start_text,
        warning: warning_text,
    });

    Ok(())
}

/// 获取已加载的语言文本集合
pub fn text() -> &'static I18nText {
    I18N_TEXT.get_or_init(|| {
        let language_source = load_language_source();
        I18nText {
            global: global::register(&language_source),
            home: home::register(&language_source),
            key: key::register(&language_source),
            loading: loading::register(&language_source),
            setting: setting::register(&language_source),
            start: start::register(&language_source),
            warning: warning::register(&language_source),
        }
    })
}

/// 读取指定 key，并按当前语言 -> en_us -> 修复占位的顺序回退
pub fn resolve_text(language_source: &LanguageSource, key: &str) -> String {
    language_source
        .preferred_texts
        .get(key)
        .filter(|value| !value.trim().is_empty())
        .cloned()
        .or_else(|| {
            if language_source.is_default_language {
                repair_language_files();
            }
            language_source
                .fallback_texts
                .get(key)
                .filter(|value| !value.trim().is_empty())
                .cloned()
        })
        .unwrap_or_else(|| {
            repair_language_files();
            missing_text(key, language_source)
        })
}

/// 加载语言源数据
fn load_language_source() -> LanguageSource {
    let root_dir = root_dir();
    let preferred_code =
        read_language_preference(&root_dir).unwrap_or_else(|| DEFAULT_LANGUAGE_CODE.to_string());
    let preferred_texts = read_language_file(&root_dir, &preferred_code).unwrap_or_else(|| {
        read_language_file(&root_dir, DEFAULT_LANGUAGE_CODE).unwrap_or_default()
    });
    let fallback_texts =
        read_language_file(&root_dir, DEFAULT_LANGUAGE_CODE).unwrap_or_else(|| {
            repair_language_files();
            HashMap::new()
        });

    LanguageSource {
        preferred_texts,
        fallback_texts,
        is_default_language: preferred_code == DEFAULT_LANGUAGE_CODE,
    }
}

/// 读取用户语言偏好
fn read_language_preference(root_dir: &Path) -> Option<String> {
    let raw_language = fs::read_to_string(root_dir.join(LANGUAGE_PROFILE_PATH)).ok()?;
    let language_code = raw_language.trim();
    if language_code.is_empty() {
        None
    } else {
        Some(language_code.to_string())
    }
}

/// 读取指定语言 JSON 文件
fn read_language_file(root_dir: &Path, language_code: &str) -> Option<HashMap<String, String>> {
    let language_path = root_dir
        .join(LANGUAGE_DIR)
        .join(format!("{language_code}.json"));
    let raw_json = fs::read_to_string(language_path).ok()?;
    serde_json::from_str::<HashMap<String, String>>(&raw_json).ok()
}

/// 缺失 key 的文本。优先使用已加载的缺失键模板。
fn missing_text(key: &str, language_source: &LanguageSource) -> String {
    let template = language_source
        .fallback_texts
        .get("global.error.missing_key")
        .or_else(|| {
            language_source
                .preferred_texts
                .get("global.error.missing_key")
        })
        .map(String::as_str)
        .unwrap_or("[Missing i18n key: {key}]");

    template.replace("{key}", key)
}

/// 官方语言文件修复入口，当前阶段留空
fn repair_language_files() {
    // Placeholder: official file repair will be implemented later.
}

/// 获取宿主根目录。开发环境优先使用当前目录，打包环境退回可执行文件目录。
fn root_dir() -> PathBuf {
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
