//! 宿主静态资源目录检查
//! 静态资源缺失或不符合要求时进入官方维修流程

use crate::host_engine::boot::environment::data_dirs;
use std::fs;
use std::io;
use std::path::Path;

use super::repair;

type EnvironmentResult<T> = Result<T, Box<dyn std::error::Error>>;

/// 检查宿主必须存在的静态资源
pub fn verify() -> EnvironmentResult<()> {
    match verify_required_files() {
        Ok(()) => Ok(()),
        Err(_) => {
            repair::repair_host_files()?;
            verify_required_files()
        }
    }
}

/// 执行实际检查。此函数不修复，只返回检查结果。
fn verify_required_files() -> EnvironmentResult<()> {
    let root_dir = data_dirs::root_dir();

    ensure_dir(&root_dir.join("assets"))?;
    ensure_dir(&root_dir.join("assets/lang"))?;
    ensure_non_empty_file(&root_dir.join("assets/lang/en_us.json"))?;

    ensure_dir(&root_dir.join("assets/bash_lang"))?;
    ensure_non_empty_file(&root_dir.join("assets/bash_lang/en_us.json"))?;

    ensure_dir(&root_dir.join("scripts"))?;
    ensure_dir(&root_dir.join("scripts/game"))?;
    ensure_creatable_dir(&root_dir.join("scripts/screensaver"))?;
    ensure_creatable_dir(&root_dir.join("scripts/boss"))?;

    Ok(())
}

/// 确保目录存在
fn ensure_dir(path: &Path) -> EnvironmentResult<()> {
    if path.is_dir() {
        return Ok(());
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        format!("required directory is missing: {}", path.display()),
    )
    .into())
}

/// 确保可为空的官方扩展目录存在。
fn ensure_creatable_dir(path: &Path) -> EnvironmentResult<()> {
    fs::create_dir_all(path)?;
    Ok(())
}

/// 确保文件存在且不为空
fn ensure_non_empty_file(path: &Path) -> EnvironmentResult<()> {
    let metadata = fs::metadata(path).map_err(|error| {
        io::Error::new(
            error.kind(),
            format!("required file is missing: {}: {error}", path.display()),
        )
    })?;

    if !metadata.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("required path is not a file: {}", path.display()),
        )
        .into());
    }

    if metadata.len() == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("required file is empty: {}", path.display()),
        )
        .into());
    }

    Ok(())
}
