// 集中管理应用程序运行所需的所有目录和文件路径，确保目录存在，并在首次运行时从可执行文件目录复制资源

use std::fs; // 文件系统操作：创建目录、复制文件
use std::path::{Path, PathBuf}; // 路径处理

use anyhow::Result; // 统一错误处理

// 获取当前工作目录作为项目根目录
pub fn project_root() -> Result<PathBuf> {
    Ok(std::env::current_dir()?)
}

// 获取可执行文件所在目录（发布版）或回退到项目根目录（开发时）
pub fn runtime_dir() -> Result<PathBuf> {
    if let Ok(exe) = std::env::current_exe()
        && let Some(parent) = exe.parent()
    {
        return Ok(parent.to_path_buf());
    }

    project_root()
}

// 获取应用数据目录 tui-game-data，若不存在则创建
pub fn app_data_dir() -> Result<PathBuf> {
    let dir = runtime_dir()?.join("tui-game-data");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

// 获取 assets 资源目录，优先运行目录下的，否则项目根目录下的
pub fn assets_dir() -> Result<PathBuf> {
    let runtime_assets = runtime_dir()?.join("assets");
    if runtime_assets.exists() {
        return Ok(runtime_assets);
    }
    Ok(project_root()?.join("assets"))
}

// 获取 scripts 脚本目录，优先运行目录，否则项目根目录
pub fn scripts_dir() -> Result<PathBuf> {
    let runtime_scripts = runtime_dir()?.join("scripts");
    if runtime_scripts.exists() {
        return Ok(runtime_scripts);
    }
    Ok(project_root()?.join("scripts"))
}

// 获取官方游戏目录，首次运行时从捆绑的 games/official 复制内容到数据目录
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

// 语言偏好文件路径（language.txt）
pub fn language_file() -> Result<PathBuf> {
    Ok(app_data_dir()?.join("language.txt"))
}

// 最佳成绩文件路径（best_scores.json）
pub fn best_scores_file() -> Result<PathBuf> {
    Ok(app_data_dir()?.join("best_scores.json"))
}

// 游戏存档文件路径（saves.json）
pub fn saves_file() -> Result<PathBuf> {
    Ok(app_data_dir()?.join("saves.json"))
}

// 更新缓存文件路径（updater_cache.json）
pub fn updater_cache_file() -> Result<PathBuf> {
    Ok(app_data_dir()?.join("updater_cache.json"))
}

// 日志目录路径，确保目录存在
pub fn log_dir() -> Result<PathBuf> {
    let dir = app_data_dir()?.join("log");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

// 缓存目录路径，确保目录存在
pub fn cache_dir() -> Result<PathBuf> {
    let dir = app_data_dir()?.join("cache");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

// Mod 存档目录路径，确保目录存在
pub fn mod_save_dir() -> Result<PathBuf> {
    let dir = app_data_dir()?.join("mod_save");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

// 获取主程序可执行文件路径（跨平台后缀）
pub fn main_binary_file() -> Result<PathBuf> {
    Ok(runtime_dir()?.join(binary_name("tui-game")))
}

// 确保指定路径的父目录存在
pub fn ensure_parent_dir(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

// 递归复制目录内容
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

// 根据平台返回可执行文件名，Windows 自动加 .exe
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
