//! 统一包清单解析。

use std::fs;
use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::host_engine::boot::preload::game_modules::GameManifest;
use crate::host_engine::package::PackageError;

type ManifestResult<T> = Result<T, PackageError>;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct RawPackageManifest {
    pub api: Option<Value>,
    pub entry: Option<String>,
    pub package: Option<String>,
    pub package_name: Option<String>,
    pub introduction: Option<String>,
    pub author: Option<String>,
    pub version: Option<String>,
    pub game_name: Option<String>,
    pub screensaver_name: Option<String>,
    pub boss_name: Option<String>,
    pub description: Option<String>,
    pub detail: Option<String>,
    pub icon: Option<Value>,
    pub banner: Option<Value>,
}

pub fn parse_manifest(path: &Path) -> ManifestResult<RawPackageManifest> {
    read_json_file(path)
}

pub fn parse_game_manifest(path: &Path) -> ManifestResult<GameManifest> {
    read_json_file(path)
}

fn read_json_file<T>(path: &Path) -> ManifestResult<T>
where
    T: for<'de> Deserialize<'de>,
{
    let raw_json = fs::read_to_string(path).map_err(|error| {
        PackageError::IOError(io::Error::new(
            error.kind(),
            format!("failed to read {}: {error}", path.display()),
        ))
    })?;
    serde_json::from_str::<T>(raw_json.trim_start_matches('\u{feff}')).map_err(|error| {
        PackageError::InvalidManifest(format!("failed to parse {}: {error}", path.display()))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_manifest_accepts_partial_json() {
        let manifest =
            serde_json::from_str::<RawPackageManifest>(r#"{ "package": "demo" }"#).unwrap();
        assert_eq!(manifest.package.as_deref(), Some("demo"));
        assert!(manifest.package_name.is_none());
    }
}
