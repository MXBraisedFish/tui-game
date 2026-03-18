use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use mlua::{Lua, Table};

use crate::utils::path_utils;

// 游戏数据结构
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameMeta {
    pub id: String, // 游戏ID
    pub name: String, // 游戏显示名称
    pub description: String, // 游戏描述
    pub script_path: PathBuf, // 脚本文件的完整路径
}

// 扫描脚本目录找到游戏脚本目录
// 在0.10.0更新了文件目录
// 但也向老版本兼容了
pub fn scan_scripts() -> Result<Vec<GameMeta>> {
    // 获取目录
    let scripts_dir = path_utils::scripts_dir()?;

    let mut games = Vec::new();

    // 优先扫描新版本路径
    let game_dir = scripts_dir.join("game");
    if game_dir.exists() {
        games.extend(scan_scripts_in(&game_dir)?);
    }

    // 老版本兼容
    games.extend(scan_scripts_in(&scripts_dir)?);

    // ID去重,保留第一个
    let mut dedup = Vec::new();
    for g in games {
        if !dedup.iter().any(|x: &GameMeta| x.id == g.id) {
            dedup.push(g);
        }
    }

    Ok(dedup)
}

// 在目录里寻找脚本
// 也带有排序的功能
fn scan_scripts_in(dir: &Path) -> Result<Vec<GameMeta>> {
    if !dir.exists() {
        return Ok(Vec::new()); // 目标不存在,返回空列表
    }

    // 收集所有的Lua并排序
    let mut entries: Vec<PathBuf> = fs::read_dir(dir)?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| {
            path.extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("lua"))
                .unwrap_or(false)
        })
        .collect();

    entries.sort(); // 按文件名排序,保证顺序一致

    let mut games = Vec::new();
    for path in entries {
        // 从文件名获取ID
        let id = path
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "unknown".to_string());

        // 默认名字会将下划线替换为空格
        let mut name = id.replace('_', " ");
        let mut description = "No description available.".to_string();

        if let Ok(content) = fs::read_to_string(&path) {
            // 去除UTF-8BOM
            let content = content.trim_start_matches('\u{feff}');

            // 创建临时Lua环境执行脚本
            let lua = Lua::new();
            if lua.load(content).exec().is_ok() {
                let globals = lua.globals();

                // 查找GAME_META表
                if let Ok(meta) = globals.get::<Table>("GAME_META") {
                    // 读取name字段
                    if let Ok(v) = meta.get::<String>("name") {
                        if !v.trim().is_empty() {
                            name = v;
                        }
                    }
                    // 读取description字段
                    if let Ok(v) = meta.get::<String>("description") {
                        if !v.trim().is_empty() {
                            description = v;
                        }
                    }
                }
            }
        }

        // 把结果添加到列表
        games.push(GameMeta {
            id,
            name,
            description,
            script_path: path,
        });
    }

    Ok(games)
}
