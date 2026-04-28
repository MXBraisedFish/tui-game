// 定义 app/settings 模块的所有核心数据类型，包括设置页面结构体、导航和排序枚举、对话框数据、按键捕获状态和网格布局参数

use std::time::Instant; // 时间戳（对话框打开时间、按键捕获延时）

use crate::game::registry::GameDescriptor; // 游戏描述符（SettingsState 中使用）
use crate::mods::{ModPackage}; // Mod 类型（SettingsState 中使用）

// Mod 列表的视图模式
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModListView {
    Detailed,
    Simple,
}

// Mod 包的排序方式
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModSortMode {
    Name,
    Enabled,
    Author,
    SafeMode,
}

// 设置子页面标识
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SettingsPage {
    Hub,
    Language,
    Mods,
    Security,
    Keybind,
    Memory,
}

// 按键绑定页面的焦点区域
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KeybindFocus {
    Games,
    Actions,
}

// 按键绑定的编辑模式
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KeybindEditMode {
    Add,
    Delete,
}

// 按键绑定页面的游戏排序
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KeybindGameSortMode {
    Source,
    Name,
    Author,
}

// 按键捕获的等待状态
#[derive(Clone, Debug)]
pub struct KeybindCaptureState {
    pub slot_index: usize,
    pub accept_after: Instant,
}

// 整个设置页面的状态
#[derive(Clone, Debug)]
pub struct SettingsState {
    pub page: SettingsPage,
    pub hub_selected: usize,
    pub lang_selected: usize,
    pub mod_selected: usize,
    pub mod_page: usize,
    pub mod_detail_scroll: usize,
    pub mod_detail_scroll_available: bool,
    pub mod_packages: Vec<ModPackage>,
    pub mod_safe_dialog: Option<ModSafeDialog>,
    pub mod_page_jump_dialog: Option<ModPageJumpDialog>,
    pub mod_list_view: ModListView,
    pub mod_sort_mode: ModSortMode,
    pub mod_sort_descending: bool,
    pub mod_hot_reload_fingerprint: Option<u64>,
    pub mod_hot_reload_last_checked_at: Instant,
    pub security_selected: usize,
    pub default_safe_mode_enabled: bool,
    pub default_mod_enabled: bool,
    pub keybind_selected: usize,
    pub keybind_page: usize,
    pub keybind_page_jump_input: Option<String>,
    pub keybind_games: Vec<GameDescriptor>,
    pub keybind_focus: KeybindFocus,
    pub keybind_action_selected: usize,
    pub keybind_action_scroll: usize,
    pub keybind_edit_mode: KeybindEditMode,
    pub keybind_capture: Option<KeybindCaptureState>,
    pub keybind_sort_mode: KeybindGameSortMode,
    pub keybind_sort_descending: bool,
    pub memory_selected: usize,
    pub cleanup_dialog: Option<CleanupDialog>,
    pub default_safe_mode_disable_dialog: Option<DefaultSafeModeDisableDialog>,
    pub security_success_at: Option<Instant>,
}

// Mod 安全模式禁用确认对话框
#[derive(Clone, Debug)]
pub struct ModSafeDialog {
    pub namespace: String,
    pub mod_name: String,
    pub opened_at: Instant,
}

// Mod 页码跳转输入
#[derive(Clone, Debug, Default)]
pub struct ModPageJumpDialog {
    pub input: String,
}

// 清理操作类型枚举
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CleanupAction {
    ClearCache,
    ClearAllData,
}

// 清理确认对话框
#[derive(Clone, Debug)]
pub struct CleanupDialog {
    pub action: CleanupAction,
    pub opened_at: Instant,
}

// 关闭默认安全模式确认对话框
#[derive(Clone, Debug)]
pub struct DefaultSafeModeDisableDialog {
    pub opened_at: Instant,
}

// 设置页面的动作结果枚举
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SettingsAction {
    None,
    BackToMenu,
}

// 	网格布局的参数
#[derive(Clone, Copy, Debug)]
pub struct GridMetrics {
    pub cols: usize,
    pub inner_width: u16,
    pub outer_width: u16,
}