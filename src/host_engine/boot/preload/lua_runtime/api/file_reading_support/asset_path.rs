//! 游戏资源路径解析

use std::path::{Component, Path, PathBuf};

/// 将相对 assets/ 的逻辑路径解析为实际文件路径。
pub fn resolve_asset_path(package_root: &Path, logical_path: &str) -> mlua::Result<PathBuf> {
    let trimmed_path = logical_path.trim();
    if trimmed_path.is_empty() || Path::new(trimmed_path).is_absolute() {
        return Err(mlua::Error::external(format!(
            "invalid asset path: {trimmed_path}"
        )));
    }

    let normalized_path = trimmed_path.trim_start_matches(['/', '\\']);
    let mut clean_path = PathBuf::new();
    for component in PathBuf::from(normalized_path).components() {
        match component {
            Component::Normal(part) => clean_path.push(part),
            Component::CurDir => {}
            Component::ParentDir => {
                return Err(mlua::Error::external(
                    "asset path contains parent directory",
                ));
            }
            Component::Prefix(_) | Component::RootDir => {
                return Err(mlua::Error::external(format!(
                    "invalid asset path: {trimmed_path}"
                )));
            }
        }
    }

    Ok(package_root.join("assets").join(clean_path))
}
