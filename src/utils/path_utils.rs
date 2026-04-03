use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;

pub fn project_root() -> Result<PathBuf> {
    Ok(std::env::current_dir()?)
}

pub fn runtime_dir() -> Result<PathBuf> {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            return Ok(parent.to_path_buf());
        }
    }
    project_root()
}

pub fn app_data_dir() -> Result<PathBuf> {
    let dir = runtime_dir()?.join("tui-game-data");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn assets_dir() -> Result<PathBuf> {
    let runtime_assets = runtime_dir()?.join("assets");
    if runtime_assets.exists() {
        return Ok(runtime_assets);
    }
    Ok(project_root()?.join("assets"))
}

pub fn scripts_dir() -> Result<PathBuf> {
    let runtime_scripts = runtime_dir()?.join("scripts");
    if runtime_scripts.exists() {
        return Ok(runtime_scripts);
    }
    Ok(project_root()?.join("scripts"))
}

pub fn official_games_dir() -> Result<PathBuf> {
    let runtime_games = runtime_dir()?.join("games").join("official");
    if runtime_games.exists() {
        return Ok(runtime_games);
    }
    Ok(project_root()?.join("games").join("official"))
}

pub fn language_pref_file() -> Result<PathBuf> {
    Ok(app_data_dir()?.join("language_pref.txt"))
}

pub fn lua_saves_file() -> Result<PathBuf> {
    Ok(app_data_dir()?.join("lua_saves.json"))
}

pub fn stats_file() -> Result<PathBuf> {
    Ok(app_data_dir()?.join("stats.json"))
}

pub fn main_binary_file() -> Result<PathBuf> {
    Ok(runtime_dir()?.join(binary_name("tui-game")))
}

pub fn ensure_parent_dir(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

fn binary_name(stem: &str) -> String {
    #[cfg(target_os = "windows")]
    {
        format!("{stem}.exe")
    }
    #[cfg(not(target_os = "windows"))]
    {
        stem.to_string()
    }
}
