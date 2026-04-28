// 负责运行环境的准备工作，包括清理旧版本遗留数据（文件和空目录）、创建运行时所需的目录结构、初始化默认配置文件

use std::fs; // 文件系统操作：创建目录、写入文件、删除文件/目录

use anyhow::Result; // 错误处理

use crate::app::i18n; // 获取当前语言代码，写入语言偏好文件
use crate::utils::host_log; // 记录清理失败的错误日志
use crate::utils::path_utils; // 获取各类路径

// 删除旧版本的遗留文件（7 个文件）和目录（2 个目录）。这些是更早版本的持久化文件，已被新文件替代。删除失败时记录错误日志但不中断流程
pub fn cleanup_legacy_runtime_data() -> Result<()> {
    let app_data = path_utils::app_data_dir()?;
    for file_name in [
        "stats.json",
        "lua_saves.json",
        "runtime_best_scores.json",
        "latest_runtime_save.txt",
        "language_pref.txt",
        "mod_state.json",
        "scan_cache.json",
    ] {
        let path = app_data.join(file_name);
        if path.exists()
            && let Err(err) = fs::remove_file(path)
        {
            host_log::append_host_error(
                "host.error.clean_old_save_failed",
                &[("err", &err.to_string())],
            );
        }
    }
    for dir_name in ["runtime_save", "runtime-logs"] {
        let path = app_data.join(dir_name);
        if path.exists()
            && let Err(err) = fs::remove_file(path)
        {
            host_log::append_host_error(
                "host.error.clean_old_save_failed",
                &[("err", &err.to_string())],
            );
        }
    }
    Ok(())
}

// 创建 5 个子目录（mod/、official/、cache/、mod_save/、log/），若默认文件不存在则创建（language.txt、best_scores.json、saves.json、updater_cache.json），并触发官方游戏目录的首次复制
pub fn initialize_runtime_layout() -> Result<()> {
    let app_data = path_utils::app_data_dir()?;
    fs::create_dir_all(app_data.join("mod"))?;
    fs::create_dir_all(app_data.join("official"))?;
    fs::create_dir_all(app_data.join("cache"))?;
    fs::create_dir_all(app_data.join("mod_save"))?;
    fs::create_dir_all(app_data.join("log"))?;

    let language = path_utils::language_file()?;
    if !language.exists() {
        fs::write(&language, format!("{}\n", i18n::current_language_code()))?;
    }

    let best_scores = path_utils::best_scores_file()?;
    if !best_scores.exists() {
        fs::write(&best_scores, "{}\n")?;
    }

    let saves = path_utils::saves_file()?;
    if !saves.exists() {
        fs::write(&saves, "{\n  \"continue\": {},\n  \"data\": {}\n}\n")?;
    }

    let updater_cache = path_utils::updater_cache_file()?;
    if !updater_cache.exists() {
        fs::write(&updater_cache, "{}\n")?;
    }

    let _ = path_utils::official_games_dir()?;
    Ok(())
}
