//! 游戏模块来源定义

use crate::host_engine::boot::environment::data_dirs;
use std::path::PathBuf;

/// 游戏模块来源
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum GameModuleSource {
    Office,
    Mod,
}

impl GameModuleSource {
    /// 来源扫描目录
    pub fn root_dir(self) -> PathBuf {
        match self {
            Self::Office => data_dirs::root_dir().join("scripts/game"),
            Self::Mod => data_dirs::root_dir().join("data/mod/game"),
        }
    }

    /// UID 前缀
    pub fn uid_prefix(self) -> &'static str {
        match self {
            Self::Office => "game_",
            Self::Mod => "mod_game_",
        }
    }

    /// 来源标识
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Office => "game",
            Self::Mod => "mod_game",
        }
    }
}

