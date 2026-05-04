//! 持久化数据读取与校验

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde_json::Value;

use super::profile_data::PersistentData;

type LoaderResult<T> = Result<T, Box<dyn std::error::Error>>;
const DEFAULT_LANGUAGE_CODE: &str = "en_us";

/// 读取 data/profiles 下的持久化数据。
pub fn load_persistent_data() -> LoaderResult<PersistentData> {
    let profiles_dir = root_dir().join("data/profiles");

    Ok(PersistentData {
        saves: read_json_object(&profiles_dir.join("saves.json"))?,
        best_scores: read_json_object(&profiles_dir.join("best_scores.json"))?,
        language_code: read_language_code(&profiles_dir.join("language.txt"))?,
        keybinds: read_json_object(&profiles_dir.join("keybind.json"))?,
        mod_state: read_json_object(&profiles_dir.join("mod_state.json"))?,
    })
}

fn read_json_object(path: &Path) -> LoaderResult<Value> {
    let raw_json = fs::read_to_string(path).map_err(|error| {
        io::Error::new(
            error.kind(),
            format!("failed to read {}: {error}", path.display()),
        )
    })?;
    let value = serde_json::from_str::<Value>(raw_json.trim_start_matches('\u{feff}')).map_err(
        |error| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("failed to parse {}: {error}", path.display()),
            )
        },
    )?;

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

fn read_language_code(path: &Path) -> LoaderResult<String> {
    let raw_text = fs::read_to_string(path).map_err(|error| {
        io::Error::new(
            error.kind(),
            format!("failed to read {}: {error}", path.display()),
        )
    })?;
    let language_code = raw_text.trim().trim_start_matches('\u{feff}');
    if language_code.is_empty() {
        Ok(DEFAULT_LANGUAGE_CODE.to_string())
    } else {
        Ok(language_code.to_string())
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
