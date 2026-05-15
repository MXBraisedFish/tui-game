//! Saver/老板覆盖层包预加载入口。

mod manifest;
mod scanner;
mod source;
mod uid;

pub use manifest::{OverlayPackage, OverlayPackageManifest, OverlayRegistry, OverlayScanError};
pub use source::{OverlayKind, OverlaySource};

type OverlayModuleResult<T> = Result<T, Box<dyn std::error::Error>>;

pub fn load() -> OverlayModuleResult<OverlayRegistry> {
    let registry = scanner::scan_all()?;
    scan_cache::persist_scan_cache(&registry)?;
    state::sync_saver_state(&registry)?;
    state::sync_boss_state(&registry)?;
    Ok(registry)
}

mod scan_cache {
    use std::fs;
    use std::path::{Path, PathBuf};

    use serde::Serialize;

    use super::{OverlayPackage, OverlayRegistry};

    #[derive(Serialize)]
    struct ScanCache<'a> {
        packages: &'a [OverlayPackage],
    }

    pub fn persist_scan_cache(
        registry: &OverlayRegistry,
    ) -> Result<(), Box<dyn std::error::Error>> {
        write_scan_cache(
            &root_dir().join("data/cache/saver_scan_cache"),
            &registry.savers,
        )?;
        write_scan_cache(
            &root_dir().join("data/cache/boss_scan_cache"),
            &registry.bosses,
        )?;
        Ok(())
    }

    fn write_scan_cache(
        path: &Path,
        packages: &[OverlayPackage],
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(parent_dir) = path.parent() {
            fs::create_dir_all(parent_dir)?;
        }
        fs::write(path, serde_json::to_string_pretty(&ScanCache { packages })?)?;
        Ok(())
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
}

mod state {
    use std::fs;
    use std::path::{Path, PathBuf};

    use serde_json::{Map, Value, json};

    use crate::host_engine::boot::preload::persistent_data::security_profile;

    use super::{OverlayRegistry, OverlaySource};

    pub fn sync_saver_state(registry: &OverlayRegistry) -> Result<(), Box<dyn std::error::Error>> {
        let path = root_dir().join("data/profiles/saver_state");
        let security = security_profile::load_from_default_path();
        let mut state = read_state(&path)?;
        for package in registry
            .savers
            .iter()
            .filter(|package| package.source == OverlaySource::ThirdParty)
        {
            state.entry(package.uid.clone()).or_insert_with(
                || json!({ "enabled": security.default_mod_saver_enabled, "debug": false }),
            );
        }
        write_state(&path, &state)?;
        Ok(())
    }

    pub fn sync_boss_state(registry: &OverlayRegistry) -> Result<(), Box<dyn std::error::Error>> {
        let path = root_dir().join("data/profiles/boss_state");
        let security = security_profile::load_from_default_path();
        let mut state = read_state(&path)?;
        for package in registry
            .bosses
            .iter()
            .filter(|package| package.source == OverlaySource::ThirdParty)
        {
            state.entry(package.uid.clone()).or_insert_with(
                || json!({ "enabled": security.default_mod_boss_enabled, "debug": false }),
            );
        }
        write_state(&path, &state)?;
        Ok(())
    }

    fn read_state(path: &Path) -> Result<Map<String, Value>, Box<dyn std::error::Error>> {
        if !path.is_file() {
            return Ok(Map::new());
        }
        let raw_json = fs::read_to_string(path)?;
        let value = serde_json::from_str::<Value>(raw_json.trim_start_matches('\u{feff}'))?;
        Ok(value.as_object().cloned().unwrap_or_default())
    }

    fn write_state(
        path: &Path,
        state: &Map<String, Value>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(parent_dir) = path.parent() {
            fs::create_dir_all(parent_dir)?;
        }
        fs::write(path, serde_json::to_string_pretty(state)?)?;
        Ok(())
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
}
