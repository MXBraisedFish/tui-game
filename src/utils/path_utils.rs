use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;

// 项目根目录
pub fn project_root() -> Result<PathBuf> {
    Ok(std::env::current_dir()?)
}

// 运行目录
pub fn runtime_dir() -> Result<PathBuf> {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            return Ok(parent.to_path_buf());
        }
    }
    project_root()
}

// 程序可执行文件附近的程序数据目录
pub fn app_data_dir() -> Result<PathBuf> {
    let dir = runtime_dir()?.join("tui-game-data");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

// 脚本目录
pub fn scripts_dir() -> Result<PathBuf> {
    let runtime_scripts = runtime_dir()?.join("scripts");
    if runtime_scripts.exists() {
        return Ok(runtime_scripts);
    }
    Ok(project_root()?.join("scripts"))
}

// 程序数据中的更新缓存目录
pub fn updater_cache_file() -> Result<PathBuf> {
    Ok(app_data_dir()?.join("updater_cache.json"))
}

// 程序数据中的语言文件目录
pub fn language_pref_file() -> Result<PathBuf> {
    Ok(app_data_dir()?.join("language_pref.txt"))
}

// 程序数据中的Lua脚本保存目录
pub fn lua_saves_file() -> Result<PathBuf> {
    Ok(app_data_dir()?.join("lua_saves.json"))
}

// 程序数据中的游戏数据统计目录
pub fn stats_file() -> Result<PathBuf> {
    Ok(app_data_dir()?.join("stats.json"))
}

// 执行文件附近的外部更新脚本文件路径
pub fn version_script_file() -> Result<PathBuf> {
    // 依旧条件编译
    #[cfg(target_os = "windows")]
    {
        return Ok(runtime_dir()?.join("version.bat"));
    }
    #[cfg(not(target_os = "windows"))]
    {
        Ok(runtime_dir()?.join("version.sh"))
    }
}

// 确保文件路径父目录的存在
pub fn ensure_parent_dir(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}
