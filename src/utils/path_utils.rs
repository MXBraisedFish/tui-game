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

pub fn bash_lang_dir() -> Result<PathBuf> {
    Ok(assets_dir()?.join("bash_lang"))
}

pub fn bash_scripts_dir() -> Result<PathBuf> {
    Ok(scripts_dir()?.join("bash"))
}

pub fn updater_cache_file() -> Result<PathBuf> {
    Ok(app_data_dir()?.join("updater_cache.json"))
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

pub fn version_binary_file() -> Result<PathBuf> {
    Ok(runtime_dir()?.join(binary_name("version")))
}

pub fn updata_binary_file() -> Result<PathBuf> {
    Ok(runtime_dir()?.join(binary_name("updata")))
}

pub fn uninstall_script_file() -> Result<PathBuf> {
    let runtime_script = runtime_dir()?.join(root_uninstall_script_name());
    if runtime_script.exists() {
        return Ok(runtime_script);
    }
    Ok(project_root()?.join(root_uninstall_script_name()))
}

pub fn helper_script_file(name: &str) -> Result<PathBuf> {
    Ok(bash_scripts_dir()?.join(helper_script_name(name)))
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

fn helper_script_name(stem: &str) -> String {
    #[cfg(target_os = "windows")]
    {
        format!("{stem}.bat")
    }
    #[cfg(not(target_os = "windows"))]
    {
        format!("{stem}.sh")
    }
}

fn root_uninstall_script_name() -> String {
    #[cfg(target_os = "windows")]
    {
        "tg-delete.bat".to_string()
    }
    #[cfg(not(target_os = "windows"))]
    {
        "tg-delete.sh".to_string()
    }
}
