use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use mlua::{Lua, Table};

use crate::mods::{self, ModGameInfo};
use crate::utils::path_utils;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameMeta {
    pub id: String,
    pub name: String,
    pub description: String,
    pub detail: String,
    pub best_none: Option<String>,
    pub save: bool,
    pub min_width: Option<u16>,
    pub min_height: Option<u16>,
    pub max_width: Option<u16>,
    pub max_height: Option<u16>,
    pub script_path: PathBuf,
    pub mod_info: Option<ModGameInfo>,
}

pub fn scan_scripts() -> Result<Vec<GameMeta>> {
    let scripts_dir = path_utils::scripts_dir()?;
    let mut games = Vec::new();

    let game_dir = scripts_dir.join("game");
    if game_dir.exists() {
        games.extend(scan_scripts_in(&game_dir)?);
    }

    games.extend(scan_scripts_in(&scripts_dir)?);

    if let Ok(mod_output) = mods::scan_mods() {
        games.extend(mod_output.games);
    }

    let mut dedup = Vec::new();
    for game in games {
        if !dedup.iter().any(|existing: &GameMeta| existing.id == game.id) {
            dedup.push(game);
        }
    }

    Ok(dedup)
}

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
        let mut detail = String::new();
        let mut save = false;
        let mut min_width = None;
        let mut min_height = None;
        let mut max_width = None;
        let mut max_height = None;

        if let Ok(content) = fs::read_to_string(&path) {
            let content = content.trim_start_matches('\u{feff}');
            let lua = Lua::new();
            if lua.load(content).exec().is_ok() {
                let globals = lua.globals();
                if let Ok(meta) = globals.get::<Table>("GAME_META") {
                    if let Ok(value) = meta.get::<String>("name") {
                        if !value.trim().is_empty() {
                            name = value;
                        }
                    }
                    if let Ok(value) = meta.get::<String>("description") {
                        if !value.trim().is_empty() {
                            description = value;
                        }
                    }
                    if let Ok(value) = meta.get::<String>("detail") {
                        if !value.trim().is_empty() {
                            detail = value;
                        }
                    }
                    if let Ok(value) = meta.get::<bool>("save") {
                        save = value;
                    }
                    if let Ok(value) = meta.get::<i64>("min_width") {
                        min_width = u16::try_from(value).ok().filter(|v| *v > 0);
                    }
                    if let Ok(value) = meta.get::<i64>("min_height") {
                        min_height = u16::try_from(value).ok().filter(|v| *v > 0);
                    }
                    if let Ok(value) = meta.get::<i64>("max_width") {
                        max_width = u16::try_from(value).ok().filter(|v| *v > 0);
                    }
                    if let Ok(value) = meta.get::<i64>("max_height") {
                        max_height = u16::try_from(value).ok().filter(|v| *v > 0);
                    }
                }
            }
        }

        games.push(GameMeta {
            id,
            name,
            description,
            detail,
            best_none: None,
            save,
            min_width,
            min_height,
            max_width,
            max_height,
            script_path: path,
            mod_info: None,
        });
    }

    Ok(games)
}
