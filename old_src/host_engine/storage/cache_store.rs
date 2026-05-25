//! data/cache 可重建缓存统一读写入口。

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::host_engine::boot::environment::data_dirs;
use crate::host_engine::boot::preload::game_modules::GameModuleRegistry;

use super::types::LanguageUiText;

const CACHE_DIR: &str = "data/cache";
const IMAGE_CACHE_DIR: &str = "data/cache/images";
const GAME_SCAN_CACHE_FILE: &str = "data/cache/mod_scan_cache.json";
const SCREENSAVER_SCAN_CACHE_FILE: &str = "data/cache/screensaver_scan_cache";
const BOSS_SCAN_CACHE_FILE: &str = "data/cache/boss_scan_cache";
const LANGUAGE_UI_CACHE_FILE: &str = "data/cache/language_ui_cache.json";

type CacheStoreResult<T> = Result<T, Box<dyn std::error::Error>>;

/// 扫描缓存快照。保持现有文件格式，不改变旧缓存文件命名。
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ScanCache {
    pub games: GameModuleRegistry,
    pub screensavers: Value,
    pub bosses: Value,
}

/// 图片缓存快照。当前统一记录图片缓存目录及已存在的缓存文件。
#[derive(Clone, Debug, Default)]
pub struct ImageCache {
    pub cache_dir: PathBuf,
    pub files: Vec<PathBuf>,
}

/// data/cache 下的统一缓存快照。
#[derive(Clone, Debug, Default)]
pub struct CacheStore {
    pub scan_cache: ScanCache,
    pub image_cache: ImageCache,
    pub language_ui_cache: BTreeMap<String, LanguageUiText>,
}

impl CacheStore {
    /// 从 `data/cache/` 读取所有缓存。缓存缺失时返回空快照。
    pub fn open() -> CacheStoreResult<Self> {
        ensure_cache_dirs()?;
        Ok(Self {
            scan_cache: ScanCache {
                games: read_json_or_default(&cache_path(GAME_SCAN_CACHE_FILE)),
                screensavers: read_json_value_or_default(
                    &cache_path(SCREENSAVER_SCAN_CACHE_FILE),
                    json!({}),
                ),
                bosses: read_json_value_or_default(&cache_path(BOSS_SCAN_CACHE_FILE), json!({})),
            },
            image_cache: ImageCache {
                cache_dir: cache_path(IMAGE_CACHE_DIR),
                files: read_image_cache_files(&cache_path(IMAGE_CACHE_DIR))?,
            },
            language_ui_cache: read_json_or_default(&cache_path(LANGUAGE_UI_CACHE_FILE)),
        })
    }

    /// 仅读取扫描缓存。
    pub fn read_scan_cache(&self) -> &ScanCache {
        &self.scan_cache
    }

    /// 仅读取语言 UI 缓存。
    pub fn read_language_ui_cache(&self) -> &BTreeMap<String, LanguageUiText> {
        &self.language_ui_cache
    }

    /// 保存扫描缓存。
    pub fn save_scan_cache(&self) -> CacheStoreResult<()> {
        write_json_pretty(&cache_path(GAME_SCAN_CACHE_FILE), &self.scan_cache.games)?;
        write_json_pretty(
            &cache_path(SCREENSAVER_SCAN_CACHE_FILE),
            &self.scan_cache.screensavers,
        )?;
        write_json_pretty(&cache_path(BOSS_SCAN_CACHE_FILE), &self.scan_cache.bosses)?;
        Ok(())
    }

    /// 写入游戏扫描缓存。
    pub fn write_game_scan_cache(&self, registry: &GameModuleRegistry) -> CacheStoreResult<()> {
        write_json_pretty(&cache_path(GAME_SCAN_CACHE_FILE), registry)
    }

    /// 写入语言 UI 缓存。
    pub fn write_language_ui_cache(
        &self,
        texts: &BTreeMap<String, LanguageUiText>,
    ) -> CacheStoreResult<()> {
        write_json_pretty(&cache_path(LANGUAGE_UI_CACHE_FILE), texts)
    }

    /// 保存图像缓存索引。
    ///
    /// 实际图片 ASCII 缓存文件由 image_cache::writer 按 UID 写入；这里负责确保目录存在。
    pub fn save_image_cache(&self) -> CacheStoreResult<()> {
        fs::create_dir_all(&self.image_cache.cache_dir)?;
        Ok(())
    }

    /// 清空所有缓存，保留目录结构。
    pub fn clear_all(&self) -> CacheStoreResult<()> {
        let cache_dir = cache_path(CACHE_DIR);
        if cache_dir.exists() {
            fs::remove_dir_all(&cache_dir)?;
        }
        ensure_cache_dirs()?;
        write_json_pretty(
            &cache_path(GAME_SCAN_CACHE_FILE),
            &GameModuleRegistry::default(),
        )?;
        write_json_pretty(&cache_path(SCREENSAVER_SCAN_CACHE_FILE), &json!({}))?;
        write_json_pretty(&cache_path(BOSS_SCAN_CACHE_FILE), &json!({}))?;
        write_json_pretty(
            &cache_path(LANGUAGE_UI_CACHE_FILE),
            &BTreeMap::<String, LanguageUiText>::new(),
        )?;
        Ok(())
    }

    /// 清空扫描缓存，保留其它缓存。
    pub fn clear_scan_cache(&self) -> CacheStoreResult<()> {
        ensure_cache_dirs()?;
        write_json_pretty(
            &cache_path(GAME_SCAN_CACHE_FILE),
            &GameModuleRegistry::default(),
        )?;
        write_json_pretty(&cache_path(SCREENSAVER_SCAN_CACHE_FILE), &json!({}))?;
        write_json_pretty(&cache_path(BOSS_SCAN_CACHE_FILE), &json!({}))?;
        Ok(())
    }
}

fn cache_path(relative_path: &str) -> PathBuf {
    data_dirs::root_dir().join(relative_path)
}

fn ensure_cache_dirs() -> CacheStoreResult<()> {
    fs::create_dir_all(cache_path(CACHE_DIR))?;
    fs::create_dir_all(cache_path(IMAGE_CACHE_DIR))?;
    Ok(())
}

fn read_json_or_default<T>(path: &Path) -> T
where
    T: for<'de> Deserialize<'de> + Default + Serialize,
{
    match fs::read_to_string(path).ok().and_then(|raw_json| {
        serde_json::from_str::<T>(raw_json.trim_start_matches('\u{feff}')).ok()
    }) {
        Some(value) => value,
        None => {
            let default_value = T::default();
            let _ = write_json_pretty(path, &default_value);
            default_value
        }
    }
}

fn read_json_value_or_default(path: &Path, default_value: Value) -> Value {
    match fs::read_to_string(path).ok().and_then(|raw_json| {
        serde_json::from_str::<Value>(raw_json.trim_start_matches('\u{feff}')).ok()
    }) {
        Some(value) => value,
        None => {
            let _ = write_json_pretty(path, &default_value);
            default_value
        }
    }
}

fn write_json_pretty<T>(path: &Path, value: &T) -> CacheStoreResult<()>
where
    T: Serialize + ?Sized,
{
    if let Some(parent_dir) = path.parent() {
        fs::create_dir_all(parent_dir)?;
    }
    fs::write(path, serde_json::to_string_pretty(value)?)?;
    Ok(())
}

fn read_image_cache_files(image_cache_dir: &Path) -> CacheStoreResult<Vec<PathBuf>> {
    if !image_cache_dir.is_dir() {
        return Ok(Vec::new());
    }

    let mut files = Vec::new();
    for entry in fs::read_dir(image_cache_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}
