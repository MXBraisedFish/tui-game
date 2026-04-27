/// 统一管理应用程序所有文件与目录路径
/// 业务逻辑：
/// 运行时目录定位
/// 应用数据目录
/// 资源与脚本路径
/// 官方游戏目录
/// 持久化文件路径
/// 工具函数
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;

pub fn project_root() -> Result<PathBuf> {
    Ok(std::env::current_dir()?)
}

pub fn runtime_dir() -> Result<PathBuf> {
    if let Ok(exe) = std::env::current_exe()
        && let Some(parent) = exe.parent()
    {
        return Ok(parent.to_path_buf());
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
    let dir = app_data_dir()?.join("official");
    fs::create_dir_all(&dir)?;

    let bundled = runtime_dir()?.join("games").join("official");
    if bundled.exists() {
        copy_dir_contents(&bundled, &dir)?;
    } else {
        let project = project_root()?.join("games").join("official");
        if project.exists() {
            copy_dir_contents(&project, &dir)?;
        }
    }

    Ok(dir)
}

pub fn language_file() -> Result<PathBuf> {
    Ok(app_data_dir()?.join("language.txt"))
}

pub fn best_scores_file() -> Result<PathBuf> {
    Ok(app_data_dir()?.join("best_scores.json"))
}

pub fn saves_file() -> Result<PathBuf> {
    Ok(app_data_dir()?.join("saves.json"))
}

pub fn updater_cache_file() -> Result<PathBuf> {
    Ok(app_data_dir()?.join("updater_cache.json"))
}

pub fn log_dir() -> Result<PathBuf> {
    let dir = app_data_dir()?.join("log");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn cache_dir() -> Result<PathBuf> {
    let dir = app_data_dir()?.join("cache");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn mod_save_dir() -> Result<PathBuf> {
    let dir = app_data_dir()?.join("mod_save");
    fs::create_dir_all(&dir)?;
    Ok(dir)
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

fn copy_dir_contents(from: &Path, to: &Path) -> Result<()> {
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let source = entry.path();
        let target = to.join(entry.file_name());
        if source.is_dir() {
            fs::create_dir_all(&target)?;
            copy_dir_contents(&source, &target)?;
        } else {
            ensure_parent_dir(&target)?;
            fs::copy(&source, &target)?;
        }
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
