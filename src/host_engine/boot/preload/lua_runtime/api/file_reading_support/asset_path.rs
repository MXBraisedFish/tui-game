//! 游戏资源路径解析

use std::path::{Component, Path, PathBuf};

/// 将相对 assets/ 的逻辑路径解析为实际文件路径。
pub fn resolve_asset_path(package_root: &Path, logical_path: &str) -> mlua::Result<PathBuf> {
    resolve_asset_file_path(package_root, logical_path, None)
}

/// 将相对 assets/ 的逻辑路径解析为指定类型的实际文件路径。
pub fn resolve_asset_file_path(
    package_root: &Path,
    logical_path: &str,
    required_extension: Option<&str>,
) -> mlua::Result<PathBuf> {
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
            Component::CurDir => {
                return Err(mlua::Error::external(
                    "asset path contains current directory",
                ));
            }
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

    if let Some(required_extension) = required_extension {
        ensure_required_extension(&mut clean_path, required_extension)?;
    }

    Ok(package_root.join("assets").join(clean_path))
}

fn ensure_required_extension(path: &mut PathBuf, required_extension: &str) -> mlua::Result<()> {
    let normalized_required_extension = required_extension
        .trim_start_matches('.')
        .to_ascii_lowercase();
    match path.extension().and_then(|extension| extension.to_str()) {
        Some(extension) if extension.eq_ignore_ascii_case(&normalized_required_extension) => Ok(()),
        Some(extension) => Err(mlua::Error::external(format!(
            "invalid file extension: expected .{normalized_required_extension}, got .{extension}"
        ))),
        None => {
            path.set_extension(normalized_required_extension);
            Ok(())
        }
    }
}
