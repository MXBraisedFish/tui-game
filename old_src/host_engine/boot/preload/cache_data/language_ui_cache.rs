//! 语言选择 UI 文本缓存
// TODO: 迁移至 storage::CacheStore

use crate::host_engine::boot::environment::data_dirs;
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::Path;

use super::cache_snapshot::LanguageUiText;
use crate::host_engine::storage::cache_store::CacheStore;

type LanguageUiCacheResult<T> = Result<T, Box<dyn std::error::Error>>;

const LANGUAGE_DIR: &str = "assets/lang";
const DEFAULT_LANGUAGE_CODE: &str = "en_us";

/// 扫描 assets/lang，并同步语言选择 UI 需要的文本缓存。
pub fn sync_language_ui_cache(
    cache_store: &CacheStore,
) -> LanguageUiCacheResult<BTreeMap<String, LanguageUiText>> {
    let root_dir = data_dirs::root_dir();
    let language_dir = root_dir.join(LANGUAGE_DIR);
    let fallback_texts =
        read_language_json(&language_dir.join(format!("{DEFAULT_LANGUAGE_CODE}.json")))
            .unwrap_or_default();
    let mut language_ui_texts = BTreeMap::new();

    if language_dir.is_dir() {
        for entry in fs::read_dir(&language_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
                continue;
            }
            let Some(language_code) = path.file_stem().and_then(|file_stem| file_stem.to_str())
            else {
                continue;
            };
            let current_texts = read_language_json(&path).unwrap_or_default();
            language_ui_texts.insert(
                language_code.to_string(),
                build_language_ui_text(&current_texts, &fallback_texts),
            );
        }
    }

    cache_store.write_language_ui_cache(&language_ui_texts)?;
    Ok(language_ui_texts)
}

fn read_language_json(path: &Path) -> Option<HashMap<String, String>> {
    let raw_json = fs::read_to_string(path).ok()?;
    serde_json::from_str::<HashMap<String, String>>(raw_json.trim_start_matches('\u{feff}')).ok()
}

fn build_language_ui_text(
    current_texts: &HashMap<String, String>,
    fallback_texts: &HashMap<String, String>,
) -> LanguageUiText {
    LanguageUiText {
        key_language_up_option: text(current_texts, fallback_texts, "key.language.up_option"),
        key_language_down_option: text(current_texts, fallback_texts, "key.language.down_option"),
        key_language_left_option: text(current_texts, fallback_texts, "key.language.left_option"),
        key_language_right_option: text(current_texts, fallback_texts, "key.language.right_option"),
        key_language_select: text(current_texts, fallback_texts, "key.language.select"),
        key_language_confirm: text(current_texts, fallback_texts, "key.language.confirm"),
        key_language_jump: text(current_texts, fallback_texts, "key.language.jump"),
        key_language_prev_page: text(current_texts, fallback_texts, "key.language.prev_page"),
        key_language_next_page: text(current_texts, fallback_texts, "key.language.next_page"),
        key_language_back_cancel: text(current_texts, fallback_texts, "key.language.back_cancel"),
        key_language_back: text(current_texts, fallback_texts, "key.language.back"),
        key_language_cancel: text(current_texts, fallback_texts, "key.language.cancel"),
        key_language_page: text(current_texts, fallback_texts, "key.language.page"),
        key_language_flip: text(current_texts, fallback_texts, "key.language.flip"),
        language_title: text(current_texts, fallback_texts, "language.title"),
        language_name: text(current_texts, fallback_texts, "language.name"),
    }
}

fn text(
    current_texts: &HashMap<String, String>,
    fallback_texts: &HashMap<String, String>,
    key: &str,
) -> String {
    current_texts
        .get(key)
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            fallback_texts
                .get(key)
                .filter(|value| !value.trim().is_empty())
        })
        .cloned()
        .unwrap_or_else(|| format!("[Missing i18n key: {key}]"))
}
