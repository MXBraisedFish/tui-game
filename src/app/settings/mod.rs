// app/settings 模块的入口文件，声明并重新导出所有子模块（types、common、hub、language、mods、security、keybind、memory），同时直接实现 SettingsState 的所有方法、顶层事件调度（handle_key）、Mod 热重载轮询（poll_mod_hot_reload）和页面渲染分发（render）

// 所有设置相关类型定义
pub mod types;
pub use types::*;

// 通用组件与工具函数
pub mod common;
pub use common::*;

// Hub 导航页面
pub mod hub;
pub use hub::*;

// 语言选择页面
pub mod language;
pub use language::*;

// Mod 管理页面
pub mod mods;
pub use mods::*;

// 安全设置页面
pub mod security;
pub use security::*;

// 	按键绑定页面
pub mod keybind;
pub use keybind::*;

// 内存清理页面
pub mod memory;
pub use memory::*;

use crossterm::event::{KeyEvent}; // 按键事件
use std::time::{Duration, Instant}; // 时间（Instant 在 SettingsState::new 中使用）

use crate::app::content_cache; // 内容缓存（热重载、指纹）
use crate::app::i18n; // 国际化（default_selected_index）

const MOD_HOT_RELOAD_POLL_INTERVAL: Duration = Duration::from_secs(1); // Mod 热重载的轮询间隔

impl SettingsState {
    // new()
    pub fn new() -> Self {
        let (default_safe_mode_enabled, default_mod_enabled) = crate::mods::default_mod_settings();
        let mut state = Self {
            page: SettingsPage::Hub,
            hub_selected: 0,
            lang_selected: default_selected_index(),
            mod_selected: 0,
            mod_page: 0,
            mod_detail_scroll: 0,
            mod_detail_scroll_available: false,
            mod_packages: load_mod_packages(),
            mod_safe_dialog: None,
            mod_page_jump_dialog: None,
            mod_list_view: ModListView::Detailed,
            mod_sort_mode: ModSortMode::Name,
            mod_sort_descending: false,
            mod_hot_reload_fingerprint: content_cache::current_mod_tree_fingerprint(),
            mod_hot_reload_last_checked_at: Instant::now(),
            security_selected: 0,
            default_safe_mode_enabled,
            default_mod_enabled,
            keybind_selected: 0,
            keybind_page: 0,
            keybind_page_jump_input: None,
            keybind_games: load_keybind_games(),
            keybind_focus: KeybindFocus::Games,
            keybind_action_selected: 0,
            keybind_action_scroll: 0,
            keybind_edit_mode: KeybindEditMode::Add,
            keybind_capture: None,
            keybind_sort_mode: KeybindGameSortMode::Source,
            keybind_sort_descending: false,
            memory_selected: 0,
            cleanup_dialog: None,
            default_safe_mode_disable_dialog: None,
            security_success_at: None,
        };
        state.apply_mod_sort();
        state.apply_keybind_sort();
        state
    }

    // 刷新 Mod 包列表，重新应用排序，恢复之前选中的 Mod（通过 namespace 匹配）
    pub fn refresh_mods(&mut self) {
        let previous_namespace = self
            .mod_packages
            .get(self.mod_selected)
            .map(|package| package.namespace.clone());
        self.mod_packages = load_mod_packages();
        if self.mod_packages.is_empty() {
            self.mod_selected = 0;
            self.mod_page = 0;
            self.mod_detail_scroll = 0;
            self.mod_detail_scroll_available = false;
            self.mod_hot_reload_fingerprint = content_cache::current_mod_tree_fingerprint();
            self.mod_hot_reload_last_checked_at = Instant::now();
            return;
        }
        self.apply_mod_sort();
        self.restore_selected_mod(previous_namespace.as_deref());
        self.mod_detail_scroll = 0;
        self.mod_detail_scroll_available = false;
        self.mod_hot_reload_fingerprint = content_cache::current_mod_tree_fingerprint();
        self.mod_hot_reload_last_checked_at = Instant::now();
        if let Some(dialog) = &self.mod_safe_dialog
            && !self
                .mod_packages
                .iter()
                .any(|package| package.namespace == dialog.namespace)
        {
            self.mod_safe_dialog = None;
        }
    }

    // 应用 Mod 排序（委托 common::compare_mod_packages）
    fn apply_mod_sort(&mut self) {
        let sort_mode = self.mod_sort_mode;
        let descending = self.mod_sort_descending;
        self.mod_packages.sort_by(|left, right| {
            let ordering = compare_mod_packages(left, right, sort_mode);
            if descending {
                ordering.reverse()
            } else {
                ordering
            }
        });
    }

    // 根据命名空间恢复选区，若找不到则保持位置
    fn restore_selected_mod(&mut self, namespace: Option<&str>) {
        if self.mod_packages.is_empty() {
            self.mod_selected = 0;
            self.mod_page = 0;
            return;
        }

        if let Some(namespace) = namespace
            && let Some(index) = self
                .mod_packages
                .iter()
                .position(|package| package.namespace == namespace)
        {
            self.mod_selected = index;
        } else {
            self.mod_selected = self.mod_selected.min(self.mod_packages.len().saturating_sub(1));
        }

        let page_size = current_mod_page_size(self.mod_list_view);
        self.mod_page = (self.mod_selected / page_size)
            .min(total_mod_pages(self.mod_packages.len(), page_size).saturating_sub(1));
    }

    // 切换排序模式并重新排序
    fn set_mod_sort_mode(&mut self, mode: ModSortMode) {
        let current = self
            .mod_packages
            .get(self.mod_selected)
            .map(|package| package.namespace.clone());
        self.mod_sort_mode = mode;
        self.apply_mod_sort();
        self.restore_selected_mod(current.as_deref());
    }

    // 切换升降序
    fn toggle_mod_sort_order(&mut self) {
        let current = self
            .mod_packages
            .get(self.mod_selected)
            .map(|package| package.namespace.clone());
        self.mod_sort_descending = !self.mod_sort_descending;
        self.apply_mod_sort();
        self.restore_selected_mod(current.as_deref());
    }

    // 切换列表视图（详细/简单）
    fn toggle_mod_list_view(&mut self) {
        self.mod_list_view = match self.mod_list_view {
            ModListView::Detailed => ModListView::Simple,
            ModListView::Simple => ModListView::Detailed,
        };
        self.restore_selected_mod(None);
    }

    // 刷新安全设置的默认值（从 Mod 状态读取）
    fn refresh_security_defaults(&mut self) {
        let (default_safe_mode_enabled, default_mod_enabled) = crate::mods::default_mod_settings();
        self.default_safe_mode_enabled = default_safe_mode_enabled;
        self.default_mod_enabled = default_mod_enabled;
    }

    // 刷新键位列表，尝试恢复之前选中的游戏（通过 ID 匹配）
    fn refresh_keybind_games(&mut self) {
        let previous_id = self
            .keybind_games
            .get(self.keybind_selected)
            .map(|game| game.id.clone());
        self.keybind_games = load_keybind_games();
        self.apply_keybind_sort();
        if self.keybind_games.is_empty() {
            self.keybind_selected = 0;
            self.keybind_page = 0;
            self.keybind_page_jump_input = None;
            self.keybind_focus = KeybindFocus::Games;
            self.keybind_action_selected = 0;
            self.keybind_action_scroll = 0;
            self.keybind_edit_mode = KeybindEditMode::Add;
            self.keybind_capture = None;
            return;
        }

        if let Some(previous_id) = previous_id
            && let Some(index) = self
                .keybind_games
                .iter()
                .position(|game| game.id == previous_id)
        {
            self.keybind_selected = index;
        } else {
            self.keybind_selected = self
                .keybind_selected
                .min(self.keybind_games.len().saturating_sub(1));
        }

        self.keybind_page_jump_input = None;
        self.keybind_action_selected = 0;
        self.keybind_action_scroll = 0;
        self.keybind_edit_mode = KeybindEditMode::Add;
        self.keybind_capture = None;
    }

    // 应用键位游戏排序（委托 common::compare_keybind_games）
    fn apply_keybind_sort(&mut self) {
        let mode = self.keybind_sort_mode;
        let descending = self.keybind_sort_descending;
        self.keybind_games.sort_by(|left, right| {
            let ordering = compare_keybind_games(left, right, mode);
            if descending { ordering.reverse() } else { ordering }
        });
    }

    // 切换键位排序模式
    fn set_keybind_sort_mode(&mut self, mode: KeybindGameSortMode) {
        let selected_id = self
            .keybind_games
            .get(self.keybind_selected)
            .map(|game| game.id.clone());
        self.keybind_sort_mode = mode;
        self.apply_keybind_sort();
        self.restore_keybind_selection(selected_id);
    }

    // 切换键位升降序
    fn toggle_keybind_sort_order(&mut self) {
        let selected_id = self
            .keybind_games
            .get(self.keybind_selected)
            .map(|game| game.id.clone());
        self.keybind_sort_descending = !self.keybind_sort_descending;
        self.apply_keybind_sort();
        self.restore_keybind_selection(selected_id);
    }

    // 根据游戏 ID 恢复选中项
    fn restore_keybind_selection(&mut self, selected_id: Option<String>) {
        if let Some(id) = selected_id
            && let Some(index) = self.keybind_games.iter().position(|game| game.id == id)
        {
            self.keybind_selected = index;
        }
        self.keybind_selected = self
            .keybind_selected
            .min(self.keybind_games.len().saturating_sub(1));
        self.keybind_page = self.keybind_selected / current_keybind_page_size().max(1);
    }
}

// 获取语言列表中当前语言的索引
pub fn default_selected_index() -> usize {
    let languages = i18n::available_languages();
    let current = i18n::current_language_code();
    languages
        .iter()
        .position(|pack| pack.code == current)
        .unwrap_or(0)
}

// 顶层按键分发：先检查对话框状态（安全模式/清理确认），再根据 state.page 分发到对应子页面的 handle_*_key 函数
pub fn handle_key(state: &mut SettingsState, key: KeyEvent) -> SettingsAction {
    let code = key.code;
    if state.default_safe_mode_disable_dialog.is_some() {
        handle_default_safe_mode_disable_dialog_key(state, code);
        return SettingsAction::None;
    }

    if state.cleanup_dialog.is_some() {
        handle_cleanup_dialog_key(state, code);
        return SettingsAction::None;
    }

    match state.page {
        SettingsPage::Hub => handle_hub_key(state, code),
        SettingsPage::Language => {
            handle_language_key(state, code);
            SettingsAction::None
        }
        SettingsPage::Mods => {
            handle_mods_key(state, code);
            SettingsAction::None
        }
        SettingsPage::Security => {
            handle_security_key(state, code);
            SettingsAction::None
        }
        SettingsPage::Keybind => {
            handle_keybind_key(state, key);
            SettingsAction::None
        }
        SettingsPage::Memory => {
            handle_memory_key(state, code);
            SettingsAction::None
        }
    }
}

// 轮询 Mod 热重载：检查 Mod 页或键位页是否需要刷新，检查 Mod 文件指纹变化，触发 content_cache::reload 和状态刷新
pub fn poll_mod_hot_reload(state: &mut SettingsState) -> bool {
    if state.page != SettingsPage::Mods && state.page != SettingsPage::Keybind {
        return false;
    }

    let now = Instant::now();
    if now.duration_since(state.mod_hot_reload_last_checked_at) < MOD_HOT_RELOAD_POLL_INTERVAL {
        return false;
    }
    state.mod_hot_reload_last_checked_at = now;

    let current_fingerprint = content_cache::current_mod_tree_fingerprint();
    if current_fingerprint != state.mod_hot_reload_fingerprint {
        content_cache::reload();
        state.refresh_mods();
        state.refresh_keybind_games();
        return true;
    }

    if poll_keybind_capture(state) {
        return true;
    }

    false
}

// 根据当前页面返回最小终端尺寸
pub fn minimum_size(state: &SettingsState) -> (u16, u16) {
    match state.page {
        SettingsPage::Hub => minimum_size_hub(),
        SettingsPage::Language => minimum_size_language(),
        SettingsPage::Mods => minimum_size_mods(),
        SettingsPage::Security => minimum_size_security(),
        SettingsPage::Keybind => minimum_size_keybind(state),
        SettingsPage::Memory => minimum_size_memory(),
    }
}

// 顶层渲染分发：根据 state.page 调用对应子页面的 render_* 函数，之后渲染可能存在的对话框（清理确认、安全模式确认）
pub fn render(frame: &mut ratatui::Frame<'_>, state: &mut SettingsState) {
    match state.page {
        SettingsPage::Hub => render_hub(frame, state.hub_selected),
        SettingsPage::Language => render_language_selector(frame, state.lang_selected),
        SettingsPage::Mods => render_mods(frame, state),
        SettingsPage::Security => render_security(frame, state),
        SettingsPage::Keybind => render_keybind(frame, state),
        SettingsPage::Memory => render_memory(frame, state),
    }
    if let Some(dialog) = &state.cleanup_dialog {
        render_cleanup_dialog(frame, dialog);
    }
    if let Some(dialog) = &state.default_safe_mode_disable_dialog {
        render_default_safe_mode_disable_dialog(frame, dialog);
    }
}
