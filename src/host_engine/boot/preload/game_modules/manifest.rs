//! 游戏模块清单结构

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::source::GameModuleSource;

/// package.json 清单
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PackageManifest {
    pub package: String,
    pub mod_name: String,
    pub introduction: String,
    pub author: String,
    pub game_name: String,
    pub description: String,
    pub detail: String,
    pub version: String,
    pub icon: Value,
    pub banner: Value,
}

/// game.json runtime 字段
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GameRuntimeManifest {
    pub target_fps: u16,
}

/// 动作按键绑定
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GameActionBinding {
    pub key: Value,
    pub key_name: String,
}

/// game.json 清单
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GameManifest {
    pub api: Value,
    pub entry: String,
    pub save: bool,
    pub best_none: Option<String>,
    pub min_width: i64,
    pub min_height: i64,
    pub write: bool,
    pub case_sensitive: bool,
    pub actions: BTreeMap<String, GameActionBinding>,
    pub runtime: GameRuntimeManifest,
}

/// 成功读取的游戏模块
#[derive(Clone, Debug, Serialize)]
pub struct GameModule {
    pub uid: String,
    pub source: GameModuleSource,
    pub source_label: String,
    pub root_dir: PathBuf,
    pub package: PackageManifest,
    pub game: GameManifest,
}

/// 游戏模块读取错误
#[derive(Clone, Debug, Default, Serialize)]
pub struct GameModuleScanError {
    pub source: String,
    pub path: String,
    pub error: String,
}

/// 游戏模块注册表
#[derive(Clone, Debug, Default, Serialize)]
pub struct GameModuleRegistry {
    pub games: Vec<GameModule>,
    pub errors: Vec<GameModuleScanError>,
}

impl GameModuleRegistry {
    /// 合并扫描结果
    pub fn extend(&mut self, other: Self) {
        self.games.extend(other.games);
        self.errors.extend(other.errors);
    }
}
