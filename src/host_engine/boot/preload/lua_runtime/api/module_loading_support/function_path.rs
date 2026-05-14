//! 辅助脚本路径解析

use std::path::{Component, Path, PathBuf};

/// 将相对 function/ 的路径解析为实际脚本路径。
pub fn resolve_function_path(package_root: &Path, logical_path: &str) -> mlua::Result<PathBuf> {
    let trimmed_path = logical_path.trim();
    if trimmed_path.is_empty() || Path::new(trimmed_path).is_absolute() {
        return Err(mlua::Error::external(format!(
            "invalid helper script path: {trimmed_path}"
        )));
    }

    let normalized_path = trimmed_path.trim_start_matches(['/', '\\']);
    let mut clean_path = PathBuf::new();
    for component in PathBuf::from(normalized_path).components() {
        match component {
            Component::Normal(part) => clean_path.push(part),
            Component::CurDir => {
                return Err(mlua::Error::external(
                    "helper script path contains current directory",
                ));
            }
            Component::ParentDir => {
                return Err(mlua::Error::external(
                    "helper script path contains parent directory",
                ));
            }
            Component::Prefix(_) | Component::RootDir => {
                return Err(mlua::Error::external(format!(
                    "invalid helper script path: {trimmed_path}"
                )));
            }
        }
    }

    ensure_lua_extension(&mut clean_path)?;

    Ok(package_root.join("function").join(clean_path))
}

fn ensure_lua_extension(path: &mut PathBuf) -> mlua::Result<()> {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some(extension) if extension.eq_ignore_ascii_case("lua") => Ok(()),
        Some(extension) => Err(mlua::Error::external(format!(
            "invalid helper script extension: expected .lua, got .{extension}"
        ))),
        None => {
            path.set_extension("lua");
            Ok(())
        }
    }
}
