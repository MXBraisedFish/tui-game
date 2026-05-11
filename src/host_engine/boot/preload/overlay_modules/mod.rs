//! 屏保/老板覆盖层包预加载入口。

mod manifest;
mod scanner;
mod source;
mod uid;

pub use manifest::{OverlayPackage, OverlayPackageManifest, OverlayRegistry, OverlayScanError};
pub use source::{OverlayKind, OverlaySource};

type OverlayModuleResult<T> = Result<T, Box<dyn std::error::Error>>;

pub fn load() -> OverlayModuleResult<OverlayRegistry> {
    let registry = scanner::scan_all()?;
    state::sync_overlay_state(&registry)?;
    Ok(registry)
}

mod state {
    use std::fs;
    use std::path::{Path, PathBuf};

    use serde_json::{Map, Value, json};

    use super::OverlayRegistry;

    pub fn sync_overlay_state(
        registry: &OverlayRegistry,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let path = root_dir().join("data/profiles/overlay_state.json");
        let mut state = read_state(&path)?;
        for package in registry.screens.iter().chain(registry.bosses.iter()) {
            state
                .entry(package.uid.clone())
                .or_insert_with(|| json!({ "debug": false }));
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
