//! 游戏包图片缓存类型
// TODO: 迁移至 storage::CacheStore

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// 图片缓存算法版本。
pub const IMAGE_CACHE_ALGORITHM_VERSION: u32 = 2;

/// ASCII 亮度映射表，左侧最亮，右侧最暗。
pub const ASCII_GRADIENT: &str = r#"M@N%W$E#RK&FXYI*l]}1/+i>"!~';,`:."#;

/// 游戏包图片槽位。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GameImageSlot {
    Icon,
    Banner,
}

impl GameImageSlot {
    /// 缓存文件名后缀。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Icon => "icon",
            Self::Banner => "banner",
        }
    }

    /// 目标终端字符尺寸，返回值为 `(columns, rows)`。
    pub fn target_size(self) -> (u32, u32) {
        match self {
            Self::Icon => (8, 4),
            Self::Banner => (86, 13),
        }
    }
}

/// 图片色彩输出模式。
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum GameImageColorMode {
    Grayscale,
    Color,
}

/// 已解析的 image: 规格。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameImageSpec {
    pub relative_path: String,
    pub absolute_path: PathBuf,
    pub color_mode: GameImageColorMode,
}

/// 图片 ASCII 缓存文件。
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GameImageCacheFile {
    pub algorithm_version: u32,
    pub source_path: String,
    pub source_hash: String,
    pub color_mode: GameImageColorMode,
    pub columns: u32,
    pub rows: u32,
    pub lines: Vec<String>,
}
