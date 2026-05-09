//! 持久化数据读取与校验

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde_json::Value;

use super::keybind_profile;
use super::profile_data::PersistentData;

type LoaderResult<T> = Result<T, Box<dyn std::error::Error>>;
const DEFAULT_LANGUAGE_CODE: &str = "en_us";
const LANGUAGE_DIR: &str = "assets/lang";

/// 读取 data/profiles 下的持久化数据。
pub fn load_persistent_data() -> LoaderResult<PersistentData> {
    let root_dir = root_dir();
    let profiles_dir = root_dir.join("data/profiles");

    Ok(PersistentData {
        saves: read_json_object(&profiles_dir.join("saves.json"))?,
        best_scores: read_json_object(&profiles_dir.join("best_scores.json"))?,
        language_code: read_language_code(&root_dir, &profiles_dir.join("language.txt"))?,
        keybinds: keybind_profile::load_keybind_profile(&profiles_dir.join("keybind.json"))?,
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

fn read_language_code(root_dir: &Path, path: &Path) -> LoaderResult<String> {
    let raw_text = fs::read_to_string(path).map_err(|error| {
        io::Error::new(
            error.kind(),
            format!("failed to read {}: {error}", path.display()),
        )
    })?;
    let language_code = raw_text.trim().trim_start_matches('\u{feff}');
    if language_code.is_empty() {
        write_default_language_code(path)?;
        Ok(DEFAULT_LANGUAGE_CODE.to_string())
    } else if !language_file_exists(root_dir, language_code) {
        write_default_language_code(path)?;
        Ok(DEFAULT_LANGUAGE_CODE.to_string())
    } else {
        Ok(language_code.to_string())
    }
}

fn language_file_exists(root_dir: &Path, language_code: &str) -> bool {
    root_dir
        .join(LANGUAGE_DIR)
        .join(format!("{language_code}.json"))
        .is_file()
}

fn write_default_language_code(path: &Path) -> LoaderResult<()> {
    if let Some(parent_dir) = path.parent() {
        fs::create_dir_all(parent_dir)?;
    }
    fs::write(path, DEFAULT_LANGUAGE_CODE)?;
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
