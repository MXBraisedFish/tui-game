//! 覆盖层脚本路径解析。

use std::io;
use std::path::{Component, Path, PathBuf};

type ScriptPathResult<T> = Result<T, Box<dyn std::error::Error>>;

pub fn resolve_script_path(script_root: &Path, logical_path: &str) -> ScriptPathResult<PathBuf> {
    let trimmed_path = logical_path.trim();
    if trimmed_path.is_empty() || Path::new(trimmed_path).is_absolute() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("invalid overlay script path: {trimmed_path}"),
        )
        .into());
    }

    let mut clean_path = PathBuf::new();
    for component in PathBuf::from(trimmed_path).components() {
        match component {
            Component::Normal(part) => clean_path.push(part),
            Component::CurDir => {}
            Component::ParentDir | Component::Prefix(_) | Component::RootDir => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("invalid overlay script path: {trimmed_path}"),
                )
                .into());
            }
        }
    }

    Ok(script_root.join(clean_path))
}
