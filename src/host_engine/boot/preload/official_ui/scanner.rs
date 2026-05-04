//! 官方 UI 包扫描

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde_json::Value;

use super::manifest::{OfficialUiPackage, OfficialUiRegistry, OfficialUiScanError};

type ScannerResult<T> = Result<T, Box<dyn std::error::Error>>;
const DEFAULT_OFFICIAL_UI_ID: &str = "official";

/// 扫描宿主 scripts/ui 目录。
///
/// 兼容两种布局：
/// - scripts/ui/package.json 作为单个官方 UI 包
/// - scripts/ui/<package>/package.json 作为多个官方 UI 包
pub fn scan_official_ui() -> ScannerResult<OfficialUiRegistry> {
    let root_dir = root_dir().join("scripts/ui");
    let mut registry = OfficialUiRegistry::default();

    if !root_dir.is_dir() {
        return Ok(registry);
    }

    if root_dir.join("package.json").is_file() {
        match read_official_ui_package(&root_dir, DEFAULT_OFFICIAL_UI_ID.to_string()) {
            Ok(package) => registry.packages.push(package),
            Err(error) => registry.errors.push(OfficialUiScanError {
                path: root_dir.display().to_string(),
                error: error.to_string(),
            }),
        }
    }

    let mut package_dirs = fs::read_dir(&root_dir)?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.is_dir() && path.join("package.json").is_file())
        .collect::<Vec<_>>();
    package_dirs.sort();

    for package_dir in package_dirs {
        let package_id = package_dir
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(DEFAULT_OFFICIAL_UI_ID)
            .to_string();

        match read_official_ui_package(&package_dir, package_id) {
            Ok(package) => registry.packages.push(package),
            Err(error) => registry.errors.push(OfficialUiScanError {
                path: package_dir.display().to_string(),
                error: error.to_string(),
            }),
        }
    }

    Ok(registry)
}

fn read_official_ui_package(
    package_dir: &Path,
    package_id: String,
) -> ScannerResult<OfficialUiPackage> {
    let manifest = read_json_object(&package_dir.join("package.json"))?;
    Ok(OfficialUiPackage {
        id: package_id,
        root_dir: package_dir.to_path_buf(),
        manifest,
    })
}

fn read_json_object(path: &Path) -> ScannerResult<Value> {
    let raw_json = fs::read_to_string(path).map_err(|error| {
        io::Error::new(
            error.kind(),
            format!("failed to read {}: {error}", path.display()),
        )
    })?;
    let value = serde_json::from_str::<Value>(raw_json.trim_start_matches('\u{feff}'))?;
    if value.is_object() {
        Ok(value)
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("{} must be a JSON object", path.display()),
        )
        .into())
    }
}

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
