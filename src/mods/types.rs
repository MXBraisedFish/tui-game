use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;

use ratatui::text::Line;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

pub const MOD_API_VERSION: u32 = 1;

/// 单个 Mod 包的图像数据（包含 ASCII 行和预渲染的 ratatui 行）。
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ModImage {
    pub lines: Vec<String>,
    #[serde(skip, default)]
    pub rendered_lines: Vec<Line<'static>>,
}

/// Mod 中单个游戏的元数据。
#[derive(Clone, Debug)]
pub struct ModGameMeta {
    pub game_id: String,
    pub script_name: String,
    pub script_path: PathBuf,
    pub name: String,
    pub description: String,
    pub detail: String,
    pub introduction: String,
    pub best_none: Option<String>,
    pub save: bool,
    pub write: bool,
    pub min_width: Option<u16>,
    pub min_height: Option<u16>,
    pub max_width: Option<u16>,
    pub max_height: Option<u16>,
}

/// Mod 安全模式的运行状态。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModSafeModeState {
    Enabled,
    DisabledSession,
    DisabledTrusted,
}

/// 一个 Mod 包的完整信息。
#[derive(Clone, Debug)]
pub struct ModPackage {
    pub namespace: String,
    pub enabled: bool,
    pub debug_enabled: bool,
    pub safe_mode_enabled: bool,
    pub safe_mode_state: ModSafeModeState,
    pub package_name: String,
    pub package_name_allows_rich: bool,
    pub author: String,
    pub version: String,
    pub introduction: String,
    pub description: String,
    pub has_best_score_storage: bool,
    pub has_save_storage: bool,
    pub has_write_request: bool,
    pub thumbnail: ModImage,
    pub banner: ModImage,
    pub games: Vec<ModGameMeta>,
    pub errors: Vec<ModScanError>,
}

/// Mod 扫描结果。
#[derive(Clone, Debug)]
pub struct ModScanOutput {
    pub packages: Vec<ModPackage>,
}

/// 持久的 Mod 系统状态。
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ModState {
    pub api_version: u32,
    #[serde(default = "default_true")]
    pub default_mod_enabled: bool,
    #[serde(default = "default_true")]
    pub default_safe_mode_enabled: bool,
    #[serde(default)]
    pub mods: HashMap<String, ModStateEntry>,
    #[serde(default)]
    pub scan_errors: Vec<ModScanError>,
}

/// 单个 Mod 的状态条目。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModStateEntry {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub debug_enabled: bool,
    #[serde(default = "default_true")]
    pub safe_mode_enabled: bool,
    #[serde(skip)]
    pub session_safe_mode_enabled: Option<bool>,
    #[serde(default)]
    pub package_name: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub games: HashMap<String, ModGameState>,
}

impl Default for ModStateEntry {
    fn default() -> Self {
        Self {
            enabled: true,
            debug_enabled: false,
            safe_mode_enabled: true,
            session_safe_mode_enabled: None,
            package_name: String::new(),
            author: String::new(),
            version: String::new(),
            games: HashMap::new(),
        }
    }
}

/// Mod 中单个游戏的状态（成绩、按键绑定等）。
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ModGameState {
    #[serde(default)]
    pub script_name: String,
    #[serde(default)]
    pub best_score: JsonValue,
    #[serde(default)]
    pub keybindings: HashMap<String, Vec<String>>,
}

/// Mod 扫描过程中产生的错误或警告。
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ModScanError {
    pub namespace: String,
    pub scope: String,
    pub target: String,
    pub severity: String,
    pub message: String,
}

/// Mod 扫描缓存（用于加速后续扫描）。
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ModScanCache {
    #[serde(default)]
    pub packages: HashMap<String, CachedPackage>,
}

/// 单个包的缓存元数据。
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CachedPackage {
    pub meta_mtime: u64,
    #[serde(default)]
    pub script_mtimes: BTreeMap<String, u64>,
    #[serde(default)]
    pub thumbnail_cache_key: Option<String>,
    #[serde(default)]
    pub banner_cache_key: Option<String>,
    #[serde(default)]
    pub scan_ok: bool,
}

/// 图像类型（缩略图或横幅）。
#[derive(Clone, Copy, Debug)]
pub enum ImageKind {
    Thumbnail,
    Banner,
}

/// 图像色彩模式。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImageColorMode {
    Grayscale,
    Color,
}

/// 从包元数据中解析出的图像规格。
#[derive(Clone, Debug)]
pub struct ImageSpec {
    pub namespace: String,
    pub path: String,
    pub color_mode: ImageColorMode,
}

/// serde 辅助函数：默认值为 true。
fn default_true() -> bool {
    true
}