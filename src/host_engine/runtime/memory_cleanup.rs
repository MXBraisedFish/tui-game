//! 内存管理清理操作

use crate::host_engine::boot::environment::data_dirs;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

type CleanupResult<T> = Result<T, Box<dyn std::error::Error>>;

/// 清理缓存：删除 data/cache 和 data/log 后重建运行期必要数据文件。
pub fn clear_cache() -> CleanupResult<()> {
    let root_dir = data_dirs::root_dir();
    let data_dir = root_dir.join("data");
    remove_runtime_dir(&data_dir, &data_dir.join("cache"))?;
    remove_runtime_dir(&data_dir, &data_dir.join("log"))?;
    crate::host_engine::boot::environment::data_dirs::ensure()?;
    Ok(())
}

/// 清理全部数据：删除 data 后重建运行期必要数据文件。
pub fn clear_data() -> CleanupResult<()> {
    let root_dir = data_dirs::root_dir();
    let data_dir = root_dir.join("data");
    let language_code = read_language_preference(&data_dir);
    remove_runtime_dir(&root_dir, &data_dir)?;
    crate::host_engine::boot::environment::data_dirs::ensure()?;
    if let Some(language_code) = language_code {
        write_language_preference(&data_dir, language_code.as_str())?;
    }
    Ok(())
}

fn read_language_preference(data_dir: &Path) -> Option<String> {
    let language_path = data_dir.join("profiles").join("language.txt");
    let language_code = fs::read_to_string(language_path).ok()?;
    let language_code = language_code.trim();
    if language_code.is_empty() {
        None
    } else {
        Some(language_code.to_string())
    }
}

fn write_language_preference(data_dir: &Path, language_code: &str) -> CleanupResult<()> {
    let language_path = data_dir.join("profiles").join("language.txt");
    if let Some(parent_dir) = language_path.parent() {
        fs::create_dir_all(parent_dir)?;
    }
    fs::write(language_path, language_code)?;
    Ok(())
}

fn remove_runtime_dir(base_dir: &Path, target_dir: &Path) -> CleanupResult<()> {
    let base_dir = normalize_path(base_dir)?;
    let target_dir = normalize_path(target_dir)?;
    if !target_dir.starts_with(&base_dir) {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!(
                "refuse to clear directory outside base dir: {}",
                target_dir.display()
            ),
        )
        .into());
    }

    if target_dir.exists() {
        fs::remove_dir_all(&target_dir)?;
    }
    fs::create_dir_all(&target_dir)?;
    Ok(())
}

fn normalize_path(path: &Path) -> io::Result<PathBuf> {
    if path.exists() {
        path.canonicalize()
    } else {
        let parent = path.parent().unwrap_or_else(|| Path::new("."));
        let file_name = path.file_name().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("invalid path: {}", path.display()),
            )
        })?;
        Ok(parent.canonicalize()?.join(file_name))
    }
}
