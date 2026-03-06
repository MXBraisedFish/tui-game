use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use mlua::{Lua, Table};

use crate::utils::path_utils;

// 游戏数据结构
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameMeta {
    pub id: String,
    pub name: String,
    pub description: String,
    pub script_path: PathBuf,
}

// 扫描脚本目录找到游戏脚本目录
// 在0.10.0更新了文件目录
// 但也向老版本兼容了
pub fn scan_scripts() -> Result<Vec<GameMeta>> {
    let scripts_dir = path_utils::scripts_dir()?;

    let mut games = Vec::new();

    let game_dir = scripts_dir.join("game");
    if game_dir.exists() {
        games.extend(scan_scripts_in(&game_dir)?);
    }

    // Backward-compatible fallback: also scan root scripts/*.lua.
    games.extend(scan_scripts_in(&scripts_dir)?);

    // Deduplicate by game id, keep first hit (scripts/game has priority).
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
        return Ok(Vec::new());
    }

    let mut entries: Vec<PathBuf> = fs::read_dir(dir)?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| {
            path.extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("lua"))
                .unwrap_or(false)
        })
        .collect();

    entries.sort();

    let mut games = Vec::new();
    for path in entries {
        let id = path
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let mut name = id.replace('_', " ");
        let mut description = "No description available.".to_string();

        if let Ok(content) = fs::read_to_string(&path) {
            let content = content.trim_start_matches('\u{feff}');
            let lua = Lua::new();
            if lua.load(content).exec().is_ok() {
                let globals = lua.globals();
                if let Ok(meta) = globals.get::<Table>("GAME_META") {
                    if let Ok(v) = meta.get::<String>("name") {
                        if !v.trim().is_empty() {
                            name = v;
                        }
                    }
                    if let Ok(v) = meta.get::<String>("description") {
                        if !v.trim().is_empty() {
                            description = v;
                        }
                    }
                }
            }
        }

        games.push(GameMeta {
            id,
            name,
            description,
            script_path: path,
        });
    }

    Ok(games)
}
