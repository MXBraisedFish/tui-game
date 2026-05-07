//! 图片缓存读写

use std::fs;
use std::path::{Path, PathBuf};

use super::types::{
    GameImageCacheFile, GameImageColorMode, GameImageSlot, IMAGE_CACHE_ALGORITHM_VERSION,
};

type ImageCacheResult<T> = Result<T, Box<dyn std::error::Error>>;

/// 构造单个图片缓存文件路径。
pub fn cache_file_path(image_cache_dir: &Path, uid: &str, slot: GameImageSlot) -> PathBuf {
    image_cache_dir.join(format!("{}.{}.json", uid, slot.as_str()))
}

/// 检查缓存是否匹配当前源图。
pub fn has_fresh_cache(
    path: &Path,
    source_path: &str,
    source_hash: &str,
    color_mode: GameImageColorMode,
    slot: GameImageSlot,
) -> bool {
    let Ok(raw_json) = fs::read_to_string(path) else {
        return false;
    };
    let Ok(cache_file) =
        serde_json::from_str::<GameImageCacheFile>(raw_json.trim_start_matches('\u{feff}'))
    else {
        return false;
    };
    let (columns, rows) = slot.target_size();

    cache_file.algorithm_version == IMAGE_CACHE_ALGORITHM_VERSION
        && cache_file.source_path == source_path
        && cache_file.source_hash == source_hash
        && cache_file.color_mode == color_mode
        && cache_file.columns == columns
        && cache_file.rows == rows
        && cache_file.lines.len() == rows as usize
}

/// 写入图片 ASCII 缓存。
pub fn write_cache(
    path: &Path,
    source_path: String,
    source_hash: String,
    color_mode: GameImageColorMode,
    slot: GameImageSlot,
    lines: Vec<String>,
) -> ImageCacheResult<()> {
    if let Some(parent_dir) = path.parent() {
        fs::create_dir_all(parent_dir)?;
    }
    let (columns, rows) = slot.target_size();
    let cache_file = GameImageCacheFile {
        algorithm_version: IMAGE_CACHE_ALGORITHM_VERSION,
        source_path,
        source_hash,
        color_mode,
        columns,
        rows,
        lines,
    };
    fs::write(path, serde_json::to_string_pretty(&cache_file)?)?;
    Ok(())
}
