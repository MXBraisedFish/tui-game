//! 游戏模块来源定义

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
            Self::Office => root_dir().join("scripts/game"),
            Self::Mod => root_dir().join("data/mod"),
        }
    }

    /// UID 前缀
    pub fn uid_prefix(self) -> &'static str {
        match self {
            Self::Office => "tui_game_",
            Self::Mod => "mod_game_",
        }
    }

    /// 来源标识
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Office => "office",
            Self::Mod => "mod",
        }
    }
}

/// 获取宿主根目录。开发环境优先使用当前目录，打包环境退回可执行文件目录。
fn root_dir() -> PathBuf {
    std::env::current_dir()
        .ok()
        .filter(|path| path.join("assets").exists() || path.join("Cargo.toml").exists())
        .or_else(|| {
            std::env::current_exe()
                .ok()
                .and_then(|path| path.parent().map(PathBuf::from))
        })
        .unwrap_or_else(|| PathBuf::from("."))
}
