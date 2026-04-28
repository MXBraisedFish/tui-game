// 定义 Mod 系统中所有核心数据结构，包括 Mod 图像、游戏元数据、包信息、系统持久状态、扫描缓存以及图像处理相关的辅助类型。是 mods 模块的类型基石。

use std::collections::{BTreeMap, HashMap}; // 有序映射（脚本修改时间）和哈希映射（Mod 状态、游戏状态、按键绑定）
use std::path::PathBuf; // 路径类型（用于脚本路径）

use ratatui::text::Line; // 预渲染的富文本行（ModImage 缓存）
use serde::{Deserialize, Serialize}; // 序列化/反序列化（持久化到 JSON）
use serde_json::Value as JsonValue; // 通用 JSON 值（最佳成绩字段）

pub const MOD_API_VERSION: u32 = 1; // 当前支持的 Mod API 版本号，用于兼容性检查

// 单个 Mod 包的图像数据
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ModImage {
    pub lines: Vec<String>,
    #[serde(skip, default)]
    pub rendered_lines: Vec<Line<'static>>,
}

// Mod 中单个游戏的元数据
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

// Mod 安全模式的运行状态
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModSafeModeState {
    Enabled,
    DisabledSession,
    DisabledTrusted,
}

// 一个 Mod 包的完整信息
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

// Mod 扫描结果
#[derive(Clone, Debug)]
pub struct ModScanOutput {
    pub packages: Vec<ModPackage>,
}

// 持久的 Mod 系统状态（JSON 文件）
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

// 单个 Mod 的状态条目
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

// Mod 中单个游戏的状态
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ModGameState {
    #[serde(default)]
    pub script_name: String,
    #[serde(default)]
    pub best_score: JsonValue,
    #[serde(default)]
    pub keybindings: HashMap<String, Vec<String>>,
}

// 扫描过程中产生的错误或警告
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ModScanError {
    pub namespace: String,
    pub scope: String,
    pub target: String,
    pub severity: String,
    pub message: String,
}

// 扫描缓存，加速后续扫描
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ModScanCache {
    #[serde(default)]
    pub packages: HashMap<String, CachedPackage>,
}

// 单个包的缓存元数据
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

// 图像类型
#[derive(Clone, Copy, Debug)]
pub enum ImageKind {
    Thumbnail,
    Banner,
}

// 图像色彩模式
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImageColorMode {
    Grayscale,
    Color,
}

// 从包元数据解析出的图像规格
#[derive(Clone, Debug)]
pub struct ImageSpec {
    pub namespace: String,
    pub path: String,
    pub color_mode: ImageColorMode,
}

// serde 辅助函数：当 JSON 字段缺失时，默认值为 true。用于 #[serde(default = "default_true")]
fn default_true() -> bool {
    true
}