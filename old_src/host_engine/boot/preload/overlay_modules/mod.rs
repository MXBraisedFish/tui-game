//! Screensaver/老板覆盖层包预加载入口。
// TODO: 迁移至 storage::CacheStore

mod manifest;
mod scanner;
mod source;
mod uid;

pub use manifest::{OverlayPackage, OverlayPackageManifest, OverlayRegistry, OverlayScanError};
pub use source::{OverlayKind, OverlaySource};

use crate::host_engine::package::package_id::PackageId;
use crate::host_engine::package::package_id_registry::PackageIdRegistry;

type OverlayModuleResult<T> = Result<T, Box<dyn std::error::Error>>;

pub fn load() -> OverlayModuleResult<OverlayRegistry> {
    let registry = scanner::scan_all()?;
    warn_uid_conflicts(&registry);
    scan_cache::persist_scan_cache(&registry)?;
    state::sync_screensaver_state(&registry)?;
    state::sync_boss_state(&registry)?;
    Ok(registry)
}

fn warn_uid_conflicts(registry: &OverlayRegistry) {
    let mut package_id_registry = PackageIdRegistry::default();

    for screensaver in &registry.screensavers {
        let package_id = PackageId::from_legacy(
            source_legacy_text(screensaver.source),
            kind_legacy_text(screensaver.kind),
            &screensaver.uid,
        );
        if let Err(error) = package_id_registry.register(&package_id) {
            eprintln!("[warning] package uid conflict: {error}");
        }
    }

    for boss in &registry.bosses {
        let package_id = PackageId::from_legacy(
            source_legacy_text(boss.source),
            kind_legacy_text(boss.kind),
            &boss.uid,
        );
        if let Err(error) = package_id_registry.register(&package_id) {
            eprintln!("[warning] package uid conflict: {error}");
        }
    }
}

fn source_legacy_text(source: OverlaySource) -> &'static str {
    match source {
        OverlaySource::Office => "official",
        OverlaySource::ThirdParty => "mod",
    }
}

fn kind_legacy_text(kind: OverlayKind) -> &'static str {
    match kind {
        OverlayKind::Screensaver => "screensaver",
        OverlayKind::Boss => "boss",
    }
}

mod scan_cache {
    use std::fs;
    use std::path::Path;

    use serde::Serialize;

    use crate::host_engine::boot::environment::data_dirs;

    use super::{OverlayPackage, OverlayRegistry};

    #[derive(Serialize)]
    struct ScanCache<'a> {
        packages: &'a [OverlayPackage],
    }

    pub fn persist_scan_cache(
        registry: &OverlayRegistry,
    ) -> Result<(), Box<dyn std::error::Error>> {
        write_scan_cache(
            &data_dirs::root_dir().join("data/cache/screensaver_scan_cache"),
            &registry.screensavers,
        )?;
        write_scan_cache(
            &data_dirs::root_dir().join("data/cache/boss_scan_cache"),
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
}

mod state {
    use std::fs;
    use std::path::Path;

    use serde_json::{Map, Value, json};

    use crate::host_engine::boot::environment::data_dirs;
    use crate::host_engine::boot::preload::persistent_data::security_profile;

    use super::{OverlayRegistry, OverlaySource};

    pub fn sync_screensaver_state(
        registry: &OverlayRegistry,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let path = data_dirs::root_dir().join("data/profiles/screensaver_state");
        let security = security_profile::load_from_default_path();
        let mut state = read_state(&path)?;
        for package in registry
            .screensavers
            .iter()
            .filter(|package| package.source == OverlaySource::ThirdParty)
        {
            state.entry(package.uid.clone()).or_insert_with(
                || json!({ "enabled": security.default_mod_screensaver_enabled, "debug": false }),
            );
        }
        write_state(&path, &state)?;
        Ok(())
    }

    pub fn sync_boss_state(registry: &OverlayRegistry) -> Result<(), Box<dyn std::error::Error>> {
        let path = data_dirs::root_dir().join("data/profiles/boss_state");
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
}
