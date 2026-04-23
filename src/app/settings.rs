use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use unicode_width::UnicodeWidthStr;

use crossterm::event::KeyCode;
use std::cmp::Ordering;
use std::fs;
use std::time::{Duration, Instant};

use crate::app::content_cache;
use crate::app::i18n;
use crate::app::rich_text;
use crate::mods::{self, ModPackage, ModSafeModeState};
use crate::utils::path_utils;

const MAX_COLS: usize = 12;
const H_GAP: u16 = 1;
const TRIANGLE: &str = "\u{25B6} ";
const MOD_HOT_RELOAD_POLL_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModListView {
    Detailed,
    Simple,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModSortMode {
    Name,
    Enabled,
    Author,
    SafeMode,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SettingsPage {
    Hub,
    Language,
    Mods,
    Security,
    Memory,
}

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
    pub memory_selected: usize,
    pub cleanup_dialog: Option<CleanupDialog>,
    pub default_safe_mode_disable_dialog: Option<DefaultSafeModeDisableDialog>,
    pub security_success_at: Option<Instant>,
}

#[derive(Clone, Debug)]
pub struct ModSafeDialog {
    pub namespace: String,
    pub mod_name: String,
    pub opened_at: Instant,
}

#[derive(Clone, Debug, Default)]
pub struct ModPageJumpDialog {
    pub input: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CleanupAction {
    ClearCache,
    ClearAllData,
}

#[derive(Clone, Debug)]
pub struct CleanupDialog {
    pub action: CleanupAction,
    pub opened_at: Instant,
}

#[derive(Clone, Debug)]
pub struct DefaultSafeModeDisableDialog {
    pub opened_at: Instant,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SettingsAction {
    None,
    BackToMenu,
}

#[derive(Clone, Copy, Debug)]
pub struct GridMetrics {
    pub cols: usize,
    pub inner_width: u16,
    pub outer_width: u16,
}

impl SettingsState {
    pub fn new() -> Self {
        let (default_safe_mode_enabled, default_mod_enabled) = mods::default_mod_settings();
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
            memory_selected: 0,
            cleanup_dialog: None,
            default_safe_mode_disable_dialog: None,
            security_success_at: None,
        };
        state.apply_mod_sort();
        state
    }

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

    fn set_mod_sort_mode(&mut self, mode: ModSortMode) {
        let current = self
            .mod_packages
            .get(self.mod_selected)
            .map(|package| package.namespace.clone());
        self.mod_sort_mode = mode;
        self.apply_mod_sort();
        self.restore_selected_mod(current.as_deref());
    }

    fn toggle_mod_sort_order(&mut self) {
        let current = self
            .mod_packages
            .get(self.mod_selected)
            .map(|package| package.namespace.clone());
        self.mod_sort_descending = !self.mod_sort_descending;
        self.apply_mod_sort();
        self.restore_selected_mod(current.as_deref());
    }

    fn toggle_mod_list_view(&mut self) {
        self.mod_list_view = match self.mod_list_view {
            ModListView::Detailed => ModListView::Simple,
            ModListView::Simple => ModListView::Detailed,
        };
        self.restore_selected_mod(None);
    }

    fn refresh_security_defaults(&mut self) {
        let (default_safe_mode_enabled, default_mod_enabled) = mods::default_mod_settings();
        self.default_safe_mode_enabled = default_safe_mode_enabled;
        self.default_mod_enabled = default_mod_enabled;
    }
}

pub fn default_selected_index() -> usize {
    let languages = i18n::available_languages();
    let current = i18n::current_language_code();
    languages
        .iter()
        .position(|pack| pack.code == current)
        .unwrap_or(0)
}

pub fn handle_key(state: &mut SettingsState, code: KeyCode) -> SettingsAction {
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
        SettingsPage::Memory => {
            handle_memory_key(state, code);
            SettingsAction::None
        }
    }
}

pub fn poll_mod_hot_reload(state: &mut SettingsState) -> bool {
    if state.page != SettingsPage::Mods {
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
        return true;
    }

    false
}

pub fn minimum_size(state: &SettingsState) -> (u16, u16) {
    match state.page {
        SettingsPage::Hub => minimum_size_hub(),
        SettingsPage::Language => minimum_size_language(),
        SettingsPage::Mods => minimum_size_mods(),
        SettingsPage::Security => minimum_size_security(),
        SettingsPage::Memory => minimum_size_memory(),
    }
}

pub fn render(frame: &mut ratatui::Frame<'_>, state: &mut SettingsState) {
    match state.page {
        SettingsPage::Hub => render_hub(frame, state.hub_selected),
        SettingsPage::Language => render_language_selector(frame, state.lang_selected),
        SettingsPage::Mods => render_mods(frame, state),
        SettingsPage::Security => render_security(frame, state),
        SettingsPage::Memory => render_memory(frame, state),
    }
    if let Some(dialog) = &state.cleanup_dialog {
        render_cleanup_dialog(frame, dialog);
    }
    if let Some(dialog) = &state.default_safe_mode_disable_dialog {
        render_default_safe_mode_disable_dialog(frame, dialog);
    }
}

pub fn grid_metrics(term_width: u16, languages: &[i18n::LanguagePack]) -> GridMetrics {
    if languages.is_empty() {
        return GridMetrics {
            cols: 1,
            inner_width: 6,
            outer_width: 8,
        };
    }

    let max_name_width = languages
        .iter()
        .map(|pack| UnicodeWidthStr::width(pack.name.as_str()))
        .max()
        .unwrap_or(4);

    let inner_width = (max_name_width + 2) as u16;
    let outer_width = inner_width + 2;
    let cols_by_width =
        (((term_width as usize) + H_GAP as usize) / (outer_width as usize + H_GAP as usize)).max(1);
    let cols = languages.len().min(MAX_COLS).min(cols_by_width).max(1);

    GridMetrics {
        cols,
        inner_width,
        outer_width,
    }
}

pub fn move_selection(selected: usize, key: KeyCode, metrics: GridMetrics, total: usize) -> usize {
    if total == 0 {
        return 0;
    }

    let cols = metrics.cols.max(1);
    let row = selected / cols;
    let col = selected % cols;

    match key {
        KeyCode::Left => {
            if col > 0 {
                selected - 1
            } else {
                selected
            }
        }
        KeyCode::Right => {
            if col + 1 < cols && selected + 1 < total {
                selected + 1
            } else {
                selected
            }
        }
        KeyCode::Up => {
            if row > 0 {
                selected.saturating_sub(cols)
            } else {
                selected
            }
        }
        KeyCode::Down => {
            if selected + cols < total {
                selected + cols
            } else {
                selected
            }
        }
        _ => selected,
    }
}

fn load_mod_packages() -> Vec<ModPackage> {
    content_cache::mods()
}

fn text(key: &str, fallback: &str) -> String {
    i18n::t_or(key, fallback)
}

fn compare_mod_packages(left: &ModPackage, right: &ModPackage, mode: ModSortMode) -> Ordering {
    match mode {
        ModSortMode::Name => cmp_lowercase(&left.package_name, &right.package_name)
            .then_with(|| cmp_lowercase(&left.author, &right.author))
            .then_with(|| left.namespace.cmp(&right.namespace)),
        ModSortMode::Enabled => bool_true_first(left.enabled, right.enabled)
            .then_with(|| cmp_lowercase(&left.package_name, &right.package_name))
            .then_with(|| left.namespace.cmp(&right.namespace)),
        ModSortMode::Author => cmp_lowercase(&left.author, &right.author)
            .then_with(|| cmp_lowercase(&left.package_name, &right.package_name))
            .then_with(|| left.namespace.cmp(&right.namespace)),
        ModSortMode::SafeMode => bool_true_first(left.safe_mode_enabled, right.safe_mode_enabled)
            .then_with(|| cmp_lowercase(&left.package_name, &right.package_name))
            .then_with(|| left.namespace.cmp(&right.namespace)),
    }
}

fn cmp_lowercase(left: &str, right: &str) -> Ordering {
    left.to_lowercase().cmp(&right.to_lowercase())
}

fn bool_true_first(left: bool, right: bool) -> Ordering {
    right.cmp(&left)
}

fn handle_hub_key(state: &mut SettingsState, code: KeyCode) -> SettingsAction {
    let item_count = 4;
    match code {
        KeyCode::Up | KeyCode::Char('k') => {
            state.hub_selected = state.hub_selected.saturating_sub(1);
        }
        KeyCode::Down | KeyCode::Char('j') => {
            state.hub_selected = (state.hub_selected + 1).min(item_count - 1);
        }
        KeyCode::Char('1') => state.hub_selected = 0,
        KeyCode::Char('2') => state.hub_selected = 1,
        KeyCode::Char('3') => state.hub_selected = 2,
        KeyCode::Char('4') => state.hub_selected = 3,
        KeyCode::Enter => match state.hub_selected {
            0 => {
                state.page = SettingsPage::Language;
                state.lang_selected = default_selected_index();
            }
            1 => {
                state.page = SettingsPage::Mods;
                state.refresh_mods();
            }
            2 => {
                state.page = SettingsPage::Security;
                state.refresh_security_defaults();
            }
            3 => {
                state.page = SettingsPage::Memory;
            }
            _ => {}
        },
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
            return SettingsAction::BackToMenu;
        }
        _ => {}
    }

    SettingsAction::None
}

fn handle_language_key(state: &mut SettingsState, code: KeyCode) {
    let languages = i18n::available_languages();
    if languages.is_empty() {
        if matches!(code, KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q')) {
            state.page = SettingsPage::Hub;
        }
        return;
    }

    if state.lang_selected >= languages.len() {
        state.lang_selected = languages.len() - 1;
    }

    let (term_width, _) = crossterm::terminal::size().unwrap_or((80, 24));
    let metrics = grid_metrics(term_width, &languages);

    match code {
        KeyCode::Left | KeyCode::Right | KeyCode::Up | KeyCode::Down => {
            state.lang_selected =
                move_selection(state.lang_selected, code, metrics, languages.len());
        }
        KeyCode::Enter => {
            if let Some(pack) = languages.get(state.lang_selected) {
                let _ = i18n::set_language(&pack.code);
                content_cache::reload();
                state.refresh_mods();
            }
        }
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
            state.page = SettingsPage::Hub;
        }
        _ => {}
    }
}

fn handle_security_key(state: &mut SettingsState, code: KeyCode) {
    match code {
        KeyCode::Up | KeyCode::Char('k') => {
            state.security_selected = state.security_selected.saturating_sub(1);
        }
        KeyCode::Down | KeyCode::Char('j') => {
            state.security_selected = (state.security_selected + 1).min(3);
        }
        KeyCode::Char('1') => {
            state.security_selected = 0;
        }
        KeyCode::Char('2') => {
            state.security_selected = 1;
        }
        KeyCode::Char('3') => {
            state.security_selected = 2;
        }
        KeyCode::Char('4') => {
            state.security_selected = 3;
        }
        KeyCode::Enter => {
            apply_security_confirm(state);
        }
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
            state.page = SettingsPage::Hub;
        }
        _ => {}
    }
}

fn handle_memory_key(state: &mut SettingsState, code: KeyCode) {
    match code {
        KeyCode::Up | KeyCode::Char('k') => {
            state.memory_selected = state.memory_selected.saturating_sub(1);
        }
        KeyCode::Down | KeyCode::Char('j') => {
            state.memory_selected = (state.memory_selected + 1).min(1);
        }
        KeyCode::Char('1') => state.memory_selected = 0,
        KeyCode::Char('2') => state.memory_selected = 1,
        KeyCode::Enter => {
            let action = if state.memory_selected == 0 {
                CleanupAction::ClearCache
            } else {
                CleanupAction::ClearAllData
            };
            state.cleanup_dialog = Some(CleanupDialog {
                action,
                opened_at: Instant::now(),
            });
        }
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
            state.page = SettingsPage::Hub;
        }
        _ => {}
    }
}

fn handle_cleanup_dialog_key(state: &mut SettingsState, code: KeyCode) {
    let Some(dialog) = state.cleanup_dialog.as_ref() else {
        return;
    };
    let confirm_ready = dialog.opened_at.elapsed().as_secs() >= 3;
    match code {
        KeyCode::Esc | KeyCode::Char('1') | KeyCode::Char('q') | KeyCode::Char('Q') => {
            state.cleanup_dialog = None;
        }
        KeyCode::Enter | KeyCode::Char('2') if confirm_ready => {
            let action = dialog.action;
            state.cleanup_dialog = None;
            let _ = match action {
                CleanupAction::ClearCache => clear_cached_data(),
                CleanupAction::ClearAllData => clear_all_runtime_data(),
            };
            content_cache::reload();
            state.refresh_mods();
            state.refresh_security_defaults();
        }
        _ => {}
    }
}

fn apply_security_confirm(state: &mut SettingsState) {
    match state.security_selected {
        0 => {
            let next = !state.default_safe_mode_enabled;
            if next {
                let _ = mods::set_default_safe_mode_enabled(true);
                state.refresh_security_defaults();
            } else {
                state.default_safe_mode_disable_dialog = Some(DefaultSafeModeDisableDialog {
                    opened_at: Instant::now(),
                });
            }
        }
        1 => {
            let next = !state.default_mod_enabled;
            let _ = mods::set_default_mod_enabled(next);
            state.refresh_security_defaults();
        }
        2 => {
            let _ = mods::reset_all_mod_safe_modes_enabled();
            content_cache::reload();
            state.refresh_mods();
            state.refresh_security_defaults();
            state.security_success_at = Some(Instant::now());
        }
        3 => {
            let _ = mods::reset_all_mod_enabled_disabled();
            content_cache::reload();
            state.refresh_mods();
            state.refresh_security_defaults();
            state.security_success_at = Some(Instant::now());
        }
        _ => {}
    }
}

fn handle_default_safe_mode_disable_dialog_key(state: &mut SettingsState, code: KeyCode) {
    let Some(dialog) = state.default_safe_mode_disable_dialog.as_ref() else {
        return;
    };
    let confirm_ready = dialog.opened_at.elapsed().as_secs() >= 10;
    match code {
        KeyCode::Esc | KeyCode::Char('1') | KeyCode::Char('q') | KeyCode::Char('Q') => {
            state.default_safe_mode_disable_dialog = None;
        }
        KeyCode::Char('2') | KeyCode::Enter if confirm_ready => {
            state.default_safe_mode_disable_dialog = None;
            let _ = mods::set_default_safe_mode_enabled(false);
            state.refresh_security_defaults();
        }
        _ => {}
    }
}

fn handle_mods_key(state: &mut SettingsState, code: KeyCode) {
    if let Some(dialog) = &state.mod_safe_dialog {
        let countdown_done = dialog.opened_at.elapsed().as_secs() >= 5;
        match code {
            KeyCode::Esc | KeyCode::Char('1') => {
                state.mod_safe_dialog = None;
            }
            KeyCode::Char('2') if countdown_done => {
                let _ = mods::set_mod_safe_mode(&dialog.namespace, false, false);
                state.mod_safe_dialog = None;
                content_cache::reload();
                state.refresh_mods();
            }
            KeyCode::Char('3') if countdown_done => {
                let _ = mods::set_mod_safe_mode(&dialog.namespace, false, true);
                state.mod_safe_dialog = None;
                content_cache::reload();
                state.refresh_mods();
            }
            _ => {}
        }
        return;
    }

    if let Some(dialog) = state.mod_page_jump_dialog.as_mut() {
        let total_pages =
            total_mod_pages(state.mod_packages.len(), current_mod_page_size(state.mod_list_view));
        match code {
            KeyCode::Esc => state.mod_page_jump_dialog = None,
            KeyCode::Backspace => {
                dialog.input.pop();
            }
            KeyCode::Char(ch) if ch.is_ascii_digit() => {
                if dialog.input.len() < 4 {
                    dialog.input.push(ch);
                }
            }
            KeyCode::Enter => {
                if let Ok(page) = dialog.input.parse::<usize>()
                    && (1..=total_pages.max(1)).contains(&page)
                {
                    state.mod_page = page - 1;
                    let start = state.mod_page * current_mod_page_size(state.mod_list_view);
                    state.mod_selected = start.min(state.mod_packages.len().saturating_sub(1));
                    state.mod_detail_scroll = 0;
                }
                state.mod_page_jump_dialog = None;
            }
            _ => {}
        }
        return;
    }

    let page_size = current_mod_page_size(state.mod_list_view);
    match code {
        KeyCode::Up | KeyCode::Char('k') => {
            state.mod_selected = state.mod_selected.saturating_sub(1);
            state.mod_page = state.mod_selected / page_size;
            state.mod_detail_scroll = 0;
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if !state.mod_packages.is_empty() {
                state.mod_selected =
                    (state.mod_selected + 1).min(state.mod_packages.len().saturating_sub(1));
                state.mod_page = state.mod_selected / page_size;
                state.mod_detail_scroll = 0;
            }
        }
        KeyCode::Char('w') | KeyCode::Char('W') => {
            state.mod_detail_scroll = state.mod_detail_scroll.saturating_sub(1);
        }
        KeyCode::Char('s') | KeyCode::Char('S') => {
            state.mod_detail_scroll = state.mod_detail_scroll.saturating_add(1);
        }
        KeyCode::Char('q') | KeyCode::Char('Q') => {
            if state.mod_page > 0 {
                state.mod_page -= 1;
                let start = state.mod_page * page_size;
                state.mod_selected = start.min(state.mod_packages.len().saturating_sub(1));
                state.mod_detail_scroll = 0;
            }
        }
        KeyCode::Char('e') | KeyCode::Char('E') => {
            let total_pages = total_mod_pages(state.mod_packages.len(), page_size);
            if state.mod_page + 1 < total_pages {
                state.mod_page += 1;
                let start = state.mod_page * page_size;
                state.mod_selected = start.min(state.mod_packages.len().saturating_sub(1));
                state.mod_detail_scroll = 0;
            }
        }
        KeyCode::Enter | KeyCode::Char(' ') => {
            if let Some(package) = state.mod_packages.get(state.mod_selected) {
                let _ = mods::set_mod_enabled(&package.namespace, !package.enabled);
                content_cache::reload();
                state.refresh_mods();
            }
        }
        KeyCode::Char('d') | KeyCode::Char('D') => {
            if let Some(package) = state.mod_packages.get(state.mod_selected) {
                let _ = mods::set_mod_debug_enabled(&package.namespace, !package.debug_enabled);
                content_cache::reload();
                state.refresh_mods();
            }
        }
        KeyCode::Char('r') | KeyCode::Char('R') => {
            if let Some(package) = state.mod_packages.get(state.mod_selected) {
                if package.safe_mode_enabled {
                    state.mod_safe_dialog = Some(ModSafeDialog {
                        namespace: package.namespace.clone(),
                        mod_name: package.package_name.clone(),
                        opened_at: Instant::now(),
                    });
                } else {
                    let _ = mods::set_mod_safe_mode(&package.namespace, true, true);
                    content_cache::reload();
                    state.refresh_mods();
                }
            }
        }
        KeyCode::Char('h') | KeyCode::Char('H') => {
            content_cache::reload();
            state.refresh_mods();
        }
        KeyCode::Char('l') | KeyCode::Char('L') => {
            state.toggle_mod_list_view();
            state.mod_detail_scroll = 0;
        }
        KeyCode::Char('z') | KeyCode::Char('Z') => {
            let next = match state.mod_sort_mode {
                ModSortMode::Name => ModSortMode::Enabled,
                ModSortMode::Enabled => ModSortMode::Author,
                ModSortMode::Author => ModSortMode::SafeMode,
                ModSortMode::SafeMode => ModSortMode::Name,
            };
            state.set_mod_sort_mode(next);
            state.mod_detail_scroll = 0;
        }
        KeyCode::Char('x') | KeyCode::Char('X') => {
            state.toggle_mod_sort_order();
            state.mod_detail_scroll = 0;
        }
        KeyCode::Char('p') | KeyCode::Char('P') => {
            if total_mod_pages(state.mod_packages.len(), page_size) > 1 {
                state.mod_page_jump_dialog = Some(ModPageJumpDialog {
                    input: String::new(),
                });
            }
        }
        KeyCode::Esc => {
            state.page = SettingsPage::Hub;
        }
        _ => {}
    }
}

fn minimum_size_hub() -> (u16, u16) {
    let label_lang = text("settings.hub.language", "Language");
    let label_mods = text("settings.hub.mods", "Mods");
    let label_security = text("settings.hub.security", "Security");
    let label_memory = text("settings.hub.memory", "Memory Cleanup");
    let enter_key = i18n::t("menu.enter_shortcut");
    let back_hint = text("settings.hub.back_hint", "[ESC]/[Q] Return to main menu");

    let widths = [
        UnicodeWidthStr::width(format!("{}[1] {}", TRIANGLE, label_lang).as_str()),
        UnicodeWidthStr::width(format!("{}{} {}", TRIANGLE, enter_key, label_lang).as_str()),
        UnicodeWidthStr::width(format!("{}[2] {}", TRIANGLE, label_mods).as_str()),
        UnicodeWidthStr::width(format!("{}{} {}", TRIANGLE, enter_key, label_mods).as_str()),
        UnicodeWidthStr::width(format!("{}[3] {}", TRIANGLE, label_security).as_str()),
        UnicodeWidthStr::width(format!("{}{} {}", TRIANGLE, enter_key, label_security).as_str()),
        UnicodeWidthStr::width(format!("{}[4] {}", TRIANGLE, label_memory).as_str()),
        UnicodeWidthStr::width(format!("{}{} {}", TRIANGLE, enter_key, label_memory).as_str()),
        UnicodeWidthStr::width(back_hint.as_str()),
    ];

    let max_width = widths.into_iter().max().unwrap_or(30) as u16;
    (max_width + 4, 14)
}

fn minimum_size_language() -> (u16, u16) {
    let languages = i18n::available_languages();
    if languages.is_empty() {
        return (40, 8);
    }

    let max_name_width = languages
        .iter()
        .map(|pack| UnicodeWidthStr::width(pack.name.as_str()))
        .max()
        .unwrap_or(4);

    let inner_width = (max_name_width + 2) as u16;
    let outer_width = inner_width + 2;
    let cols = languages.len().min(MAX_COLS).max(1) as u16;
    let rows = ((languages.len() + cols as usize - 1) / cols as usize).max(1) as u16;

    let grid_width = cols * outer_width + cols.saturating_sub(1) * H_GAP;
    let grid_height = rows * 3;
    let hint = i18n::t("confirm_language");
    let hint_width = UnicodeWidthStr::width(hint.as_str()) as u16;

    let min_w = grid_width.max(hint_width).max(30) + 2;
    let min_h = 1 + 1 + grid_height + 1 + 2;
    (min_w, min_h.max(10))
}

fn minimum_size_mods() -> (u16, u16) {
    let min_width = 90u16;
    let hint_lines =
        wrap_mod_hint_lines(&build_mod_hint_segments(true), min_width.saturating_sub(2) as usize)
            .len()
            .max(1) as u16;
    (min_width, 25 + hint_lines)
}

fn minimum_size_security() -> (u16, u16) {
    (72, 14)
}

fn minimum_size_memory() -> (u16, u16) {
    (72, 12)
}

fn render_hub(frame: &mut ratatui::Frame<'_>, selected: usize) {
    let area = frame.area();
    let items = [
        ("[1]", text("settings.hub.language", "Language")),
        ("[2]", text("settings.hub.mods", "Mods")),
        ("[3]", text("settings.hub.security", "Security")),
        ("[4]", text("settings.hub.memory", "Memory Cleanup")),
    ];
    let enter_hint = i18n::t("menu.enter_shortcut");

    let content_width = items
        .iter()
        .map(|(shortcut, text)| {
            let normal = format!("{}{} {}", TRIANGLE, shortcut, text);
            let enter = format!("{}{} {}", TRIANGLE, enter_hint, text);
            UnicodeWidthStr::width(normal.as_str()).max(UnicodeWidthStr::width(enter.as_str()))
        })
        .max()
        .unwrap_or(1) as u16;
    let back_hint_width = UnicodeWidthStr::width(
        text("settings.hub.back_hint", "[ESC]/[Q] Return to main menu").as_str(),
    ) as u16;

    let width = area
        .width
        .saturating_sub(2)
        .max(1)
        .min(content_width.max(back_hint_width).max(1));
    let height = (items.len() + 2) as u16;
    let menu_area = Rect {
        x: area.x + area.width.saturating_sub(width) / 2,
        y: area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    };

    let left_pad = menu_area.width.saturating_sub(content_width) / 2;
    let mut lines = Vec::new();

    for (idx, (shortcut, text)) in items.iter().enumerate() {
        let is_selected = idx == selected.min(items.len().saturating_sub(1));
        let base_style = if is_selected {
            Style::default()
                .fg(Color::LightCyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        let key_style = Style::default()
            .fg(Color::DarkGray)
            .add_modifier(if is_selected {
                Modifier::BOLD
            } else {
                Modifier::empty()
            });
        let key = if is_selected {
            enter_hint.as_str()
        } else {
            shortcut
        };

        lines.push(Line::from(vec![
            Span::raw(" ".repeat(left_pad as usize)),
            Span::styled(if is_selected { TRIANGLE } else { "  " }, base_style),
            Span::styled(key.to_string(), key_style),
            Span::styled(format!(" {}", text), base_style),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        text("settings.hub.back_hint", "[ESC]/[Q] Return to main menu"),
        Style::default().fg(Color::DarkGray),
    )));

    let widget = Paragraph::new(lines).alignment(Alignment::Left);
    frame.render_widget(widget, menu_area);
}

fn render_security(frame: &mut ratatui::Frame<'_>, state: &mut SettingsState) {
    if let Some(shown_at) = state.security_success_at
        && shown_at.elapsed().as_secs() >= 1
    {
        state.security_success_at = None;
    }

    let lines = vec![
        Line::from(""),
        selection_option_with_value_line(
            0,
            state.security_selected,
            text(
                "settings.security.default_safe_mode",
                "Default mod safe mode",
            ),
            state.default_safe_mode_enabled,
        ),
        selection_option_with_value_line(
            1,
            state.security_selected,
            text(
                "settings.security.default_enabled",
                "Default mod enabled state",
            ),
            state.default_mod_enabled,
        ),
        selection_action_line(
            2,
            state.security_selected,
            text(
                "settings.security.reset_safe_mode",
                "Reset all mod safe modes to enabled",
            ),
        ),
        selection_action_line(
            3,
            state.security_selected,
            text(
                "settings.security.reset_enabled",
                "Reset all mods to disabled",
            ),
        ),
        Line::from(""),
    ];
    let rect = render_settings_box(
        frame,
        text("settings.security.title", "Security Settings"),
        56,
        lines,
    );
    if state.security_success_at.is_some() {
        render_box_success_hint(
            frame,
            rect,
            text("settings.security.reset_success", "Reset successful"),
        );
    }
    render_box_back_hint(frame, rect, text("settings.secondary.back_hint", "[ESC]/[Q] Return to main menu"));
}

fn render_memory(frame: &mut ratatui::Frame<'_>, state: &SettingsState) {
    let lines = vec![
        Line::from(""),
        selection_action_line(
            0,
            state.memory_selected,
            text(
                "settings.memory.clear_cache",
                "Clear cache",
            ),
        ),
        selection_action_line(
            1,
            state.memory_selected,
            text(
                "settings.memory.clear_all",
                "Clear all data",
            ),
        ),
        Line::from(""),
    ];
    let rect = render_settings_box(frame, text("settings.memory.title", "Memory Cleanup"), 32, lines);
    render_box_back_hint(frame, rect, text("settings.secondary.back_hint", "[ESC]/[Q] Return to main menu"));
}

fn render_language_selector(frame: &mut ratatui::Frame<'_>, selected: usize) {
    let area = frame.area();
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    let languages = i18n::available_languages();
    if languages.is_empty() {
        let empty = Paragraph::new(i18n::t("settings.no_valid_languages"))
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center);
        frame.render_widget(empty, sections[2]);
        return;
    }

    let selected_idx = selected.min(languages.len() - 1);
    let selected_pack = &languages[selected_idx];
    let title = i18n::t_for_code(&selected_pack.code, "language");
    let hint = i18n::t_for_code(&selected_pack.code, "confirm_language");

    let title_widget = Paragraph::new(title)
        .style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);
    frame.render_widget(title_widget, sections[0]);

    draw_language_grid(frame.buffer_mut(), sections[2], &languages, selected_idx);

    let hint_widget = Paragraph::new(Line::from(hint))
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Left);
    frame.render_widget(hint_widget, sections[3]);
}

fn render_settings_box(
    frame: &mut ratatui::Frame<'_>,
    title: String,
    min_content_width: u16,
    lines: Vec<Line<'static>>,
) -> Rect {
    let area = frame.area();
    let inner_width = lines
        .iter()
        .map(|line| line.width() as u16)
        .max()
        .unwrap_or(1)
        .max(min_content_width)
        .max(1);
    let width = (inner_width + 2).min(area.width.max(1));
    let height = ((lines.len() as u16) + 2).min(area.height.max(1));
    let render_area = Rect {
        x: area.x + area.width.saturating_sub(width) / 2,
        y: area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    };
    frame.render_widget(
        Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .title(Line::from(Span::styled(
                    format!("── {} ", title),
                    Style::default().fg(Color::White),
                )))
                .border_style(Style::default().fg(Color::White)),
        ),
        render_area,
    );
    render_area
}

fn render_box_success_hint(frame: &mut ratatui::Frame<'_>, rect: Rect, message: String) {
    let y = rect.y.saturating_add(rect.height);
    if y >= frame.area().y.saturating_add(frame.area().height) {
        return;
    }

    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            message,
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        )))
        .alignment(Alignment::Center),
        Rect::new(rect.x, y, rect.width, 1),
    );
}

fn render_box_back_hint(frame: &mut ratatui::Frame<'_>, rect: Rect, message: String) {
    let y = rect.y.saturating_add(rect.height).saturating_add(2);
    if y >= frame.area().y.saturating_add(frame.area().height) {
        return;
    }

    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            message,
            Style::default().fg(Color::DarkGray),
        )))
        .alignment(Alignment::Center),
        Rect::new(rect.x, y, rect.width, 1),
    );
}

fn selection_action_line(index: usize, selected: usize, label: String) -> Line<'static> {
    let selected_row = index == selected;
    let marker_style = Style::default()
        .fg(Color::LightCyan)
        .add_modifier(Modifier::BOLD);
    let text_style = if selected_row {
        Style::default()
            .fg(Color::LightCyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    };
    let key = if selected_row {
        i18n::t("menu.enter_shortcut")
    } else {
        format!("[{}]", index + 1)
    };
    let key_style = Style::default().fg(Color::DarkGray);
    Line::from(vec![
        Span::raw(" "),
        Span::styled(if selected_row { "▶ " } else { "  " }, marker_style),
        Span::styled(key, key_style),
        Span::raw(" "),
        Span::styled(label, text_style),
    ])
}

fn selection_option_with_value_line(
    index: usize,
    selected: usize,
    label: String,
    enabled: bool,
) -> Line<'static> {
    let selected_row = index == selected;
    let marker_style = Style::default()
        .fg(Color::LightCyan)
        .add_modifier(Modifier::BOLD);
    let text_style = if selected_row {
        Style::default()
            .fg(Color::LightCyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    };
    let key = if selected_row {
        i18n::t("menu.enter_shortcut")
    } else {
        format!("[{}]", index + 1)
    };
    let key_style = Style::default().fg(Color::DarkGray);
    let status = if enabled {
        text("settings.security.enabled", "Enabled")
    } else {
        text("settings.security.disabled", "Disabled")
    };
    let status_style = Style::default()
        .fg(if enabled { Color::Green } else { Color::Red })
        .add_modifier(Modifier::BOLD);
    Line::from(vec![
        Span::raw(" "),
        Span::styled(if selected_row { "▶ " } else { "  " }, marker_style),
        Span::styled(key, key_style),
        Span::raw(" "),
        Span::styled(label, text_style),
        Span::raw(" "),
        Span::styled("[", Style::default().fg(Color::White)),
        Span::raw(" "),
        Span::styled(status, status_style),
        Span::raw(" "),
        Span::styled("]", Style::default().fg(Color::White)),
    ])
}

fn clear_cached_data() -> anyhow::Result<()> {
    clear_directory_contents(&path_utils::cache_dir()?)?;
    clear_directory_contents(&path_utils::mod_save_dir()?)?;
    fs::write(
        path_utils::saves_file()?,
        "{\n  \"continue\": {},\n  \"data\": {}\n}\n",
    )?;
    Ok(())
}

fn clear_all_runtime_data() -> anyhow::Result<()> {
    let app_data = path_utils::app_data_dir()?;
    clear_directory_contents(&app_data)?;
    fs::create_dir_all(app_data.join("official"))?;
    fs::create_dir_all(app_data.join("mod"))?;
    fs::create_dir_all(app_data.join("cache"))?;
    fs::create_dir_all(app_data.join("mod_save"))?;
    fs::create_dir_all(app_data.join("log"))?;
    fs::write(
        path_utils::language_file()?,
        format!("{}\n", i18n::current_language_code()),
    )?;
    fs::write(path_utils::best_scores_file()?, "{}\n")?;
    fs::write(
        path_utils::saves_file()?,
        "{\n  \"continue\": {},\n  \"data\": {}\n}\n",
    )?;
    fs::write(path_utils::updater_cache_file()?, "{}\n")?;
    Ok(())
}

fn clear_directory_contents(path: &std::path::Path) -> anyhow::Result<()> {
    if !path.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let target = entry.path();
        if target.is_dir() {
            fs::remove_dir_all(target)?;
        } else {
            fs::remove_file(target)?;
        }
    }
    Ok(())
}

fn render_cleanup_dialog(frame: &mut ratatui::Frame<'_>, dialog: &CleanupDialog) {
    use ratatui::widgets::Clear;

    let area = frame.area();
    frame.render_widget(Clear, area);

    let width = area.width.saturating_sub(8).clamp(40, 72);
    let remaining = 3u64.saturating_sub(dialog.opened_at.elapsed().as_secs());
    let countdown_done = remaining == 0;
    let (title, question, description) = match dialog.action {
        CleanupAction::ClearCache => (
            text("settings.memory.confirm_clear_cache_title", "Clear Cache"),
            text(
                "settings.memory.confirm_clear_cache_question",
                "Confirm clearing cache?",
            ),
            text(
                "settings.memory.confirm_clear_cache",
                "This will clear mod image cache and game save cache. This cannot be undone.",
            ),
        ),
        CleanupAction::ClearAllData => (
            text("settings.memory.confirm_clear_all_title", "Clear All Storage"),
            text(
                "settings.memory.confirm_clear_all_question",
                "Confirm clearing all storage?",
            ),
            text(
                "settings.memory.confirm_clear_all",
                "This will reset all contents inside tui-game-data. This cannot be undone.",
            ),
        ),
    };

    let content_width = width.saturating_sub(4).max(1) as usize;
    let mut lines = wrap_plain_text_lines(
        &question,
        content_width,
        Style::default().fg(Color::White),
    );
    lines.push(Line::from(""));
    lines.extend(wrap_plain_text_lines(
        &description,
        content_width,
        Style::default().fg(Color::White),
    ));
    lines.push(Line::from(""));
    let index_style = Style::default().fg(Color::White);
    lines.push(Line::from(vec![
        Span::styled("[1] ", index_style),
        Span::styled(
            text("settings.memory.confirm_cancel", "Cancel"),
            Style::default().fg(Color::LightGreen),
        ),
    ]));
    let mut confirm_spans = vec![
        Span::styled("[2] ", index_style),
        Span::styled(
            text("settings.memory.confirm_cleanup", "Confirm Cleanup"),
            Style::default().fg(if countdown_done {
                Color::Red
            } else {
                Color::DarkGray
            }),
        ),
    ];
    if !countdown_done {
        confirm_spans.push(Span::styled(
            format!(
                " {}",
                text("settings.memory.confirm_countdown", "{seconds}s")
                    .replace("{seconds}", &remaining.to_string())
            ),
            Style::default().fg(Color::DarkGray),
        ));
    }
    lines.push(Line::from(confirm_spans));

    let content_height = lines.len().max(1).min(u16::MAX as usize) as u16;
    let height = content_height
        .saturating_add(2)
        .clamp(8, area.height.saturating_sub(2).max(8));
    let rect = centered_rect(area, width, height);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(Line::from(Span::styled(
            format!(" {} ", title),
            Style::default().fg(Color::White),
        )))
        .border_style(Style::default().fg(Color::White));
    let inner = block.inner(rect);
    frame.render_widget(block, rect);
    frame.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .alignment(Alignment::Left),
        inner,
    );
    render_box_back_hint(
        frame,
        rect,
        text("settings.secondary.back_hint", "[ESC]/[Q] Return to main menu"),
    );
}

fn render_default_safe_mode_disable_dialog(
    frame: &mut ratatui::Frame<'_>,
    dialog: &DefaultSafeModeDisableDialog,
) {
    use ratatui::widgets::Clear;

    let area = frame.area();
    frame.render_widget(Clear, area);

    let width = area.width.saturating_sub(8).clamp(40, 72);
    let remaining = 10u64.saturating_sub(dialog.opened_at.elapsed().as_secs());
    let countdown_done = remaining == 0;
    let message = text(
        "settings.security.default_safe_mode_disable_dialog.message",
        "Are you sure you want to disable Safe Mode by default for all mod packages?\n\nSafe Mode is designed to protect your device. After disabling it, mod packages may perform high-risk operations such as file writes, which may cause data loss or system instability.\n\nPlease make sure you fully trust the source and authors of mod packages.",
    );

    let content_width = width.saturating_sub(4).max(1) as usize;
    let mut lines = wrap_plain_text_lines(
        &message,
        content_width,
        Style::default().fg(Color::White),
    );
    lines.push(Line::from(""));
    let index_style = Style::default().fg(Color::White);
    lines.push(Line::from(vec![
        Span::styled("[1] ", index_style),
        Span::styled(
            text(
                "settings.security.default_safe_mode_disable_dialog.cancel",
                "Cancel",
            ),
            Style::default().fg(Color::LightGreen),
        ),
    ]));
    let mut confirm_spans = vec![
        Span::styled("[2] ", index_style),
        Span::styled(
            text(
                "settings.security.default_safe_mode_disable_dialog.confirm_disable",
                "Confirm Disable",
            ),
            Style::default().fg(if countdown_done {
                Color::Red
            } else {
                Color::DarkGray
            }),
        ),
    ];
    if !countdown_done {
        confirm_spans.push(Span::styled(
            format!(
                " {}",
                text(
                    "settings.security.default_safe_mode_disable_dialog.countdown",
                    "{seconds}s",
                )
                .replace("{seconds}", &remaining.to_string())
            ),
            Style::default().fg(Color::DarkGray),
        ));
    }
    lines.push(Line::from(confirm_spans));

    let content_height = lines.len().max(1).min(u16::MAX as usize) as u16;
    let height = content_height
        .saturating_add(2)
        .clamp(8, area.height.saturating_sub(2).max(8));
    let rect = centered_rect(area, width, height);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(
            " {} ",
            text(
                "settings.security.default_safe_mode_disable_dialog.title",
                "Safe Mode",
            )
        ))
        .border_style(Style::default().fg(Color::White));
    let inner = block.inner(rect);
    frame.render_widget(block, rect);
    frame.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .alignment(Alignment::Left),
        inner,
    );
    render_box_back_hint(
        frame,
        rect,
        text("settings.secondary.back_hint", "[ESC]/[Q] Return to main menu"),
    );
}

fn centered_rect(area: Rect, width: u16, height: u16) -> Rect {
    Rect {
        x: area.x + area.width.saturating_sub(width.min(area.width)) / 2,
        y: area.y + area.height.saturating_sub(height.min(area.height)) / 2,
        width: width.min(area.width).max(1),
        height: height.min(area.height).max(1),
    }
}

fn render_mods(frame: &mut ratatui::Frame<'_>, state: &mut SettingsState) {
    if state.mod_packages.is_empty() {
        state.mod_selected = 0;
        state.mod_page = 0;
        state.mod_detail_scroll = 0;
        state.mod_detail_scroll_available = false;
    } else {
        let page_size = current_mod_page_size(state.mod_list_view);
        let total_pages = total_mod_pages(state.mod_packages.len(), page_size);
        if state.mod_page >= total_pages {
            state.mod_page = total_pages.saturating_sub(1);
        }
        let page_start = state.mod_page * page_size;
        let page_end = (page_start + page_size).min(state.mod_packages.len());
        if state.mod_selected < page_start || state.mod_selected >= page_end {
            state.mod_selected = page_start.min(state.mod_packages.len().saturating_sub(1));
            state.mod_detail_scroll = 0;
        }
    }

    let area = frame.area();
    let root_preview = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(area);
    let columns_preview = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
        .split(root_preview[0]);
    state.mod_detail_scroll_available = compute_mod_detail_scroll_available(columns_preview[1], state);

    let hint_lines = wrap_mod_hint_lines(
        &build_mod_hint_segments(state.mod_detail_scroll_available),
        area.width.max(1) as usize,
    );
    let hint_height = hint_lines
        .len()
        .max(1)
        .min(u16::MAX as usize) as u16;
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(hint_height)])
        .split(area);
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
        .split(root[0]);

    render_mod_list(frame, columns[0], state);
    render_mod_detail(frame, columns[1], state);

    frame.render_widget(
        Paragraph::new(hint_lines)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center),
        root[1],
    );
    if let Some(dialog) = &state.mod_safe_dialog {
        render_mod_safe_dialog(frame, dialog);
    }
}

fn compute_mod_detail_scroll_available(area: Rect, state: &SettingsState) -> bool {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ", text("settings.mods.detail", "Mod Details")))
        .border_style(Style::default().fg(Color::White));
    let inner = block.inner(area);

    let Some(package) = state.mod_packages.get(state.mod_selected) else {
        return false;
    };

    let build_detail_lines = |content_width: usize| {
        let mut lines = rich_lines_from_image(
            &package.banner,
            content_width,
            Style::default().fg(Color::White),
        );
        lines.push(Line::from(""));
        lines.push(section_title_line(text(
            "settings.mods.section.basic_info",
            "Basic Info",
        )));
        lines.extend(label_value_lines(
            text("settings.mods.package_label", "Mod Package:"),
            package.package_name.clone(),
            package.package_name_allows_rich,
            content_width,
            Style::default().fg(Color::White),
        ));
        lines.extend(label_value_lines(
            text("settings.mods.author", "Author:"),
            package.author.clone(),
            true,
            content_width,
            Style::default().fg(Color::White),
        ));
        lines.extend(label_value_lines(
            text("settings.mods.version", "Version:"),
            package.version.clone(),
            true,
            content_width,
            Style::default().fg(Color::White),
        ));

        lines.push(Line::from(""));
        lines.push(section_title_line(text(
            "settings.mods.section.storage",
            "Data Storage",
        )));
        lines.push(label_value_line(
            text("settings.mods.best_score", "Best Score:"),
            if package.has_best_score_storage {
                text("settings.mods.storage_has", "Available")
            } else {
                text("settings.mods.storage_none", "None")
            },
            if package.has_best_score_storage {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::DarkGray)
            },
        ));
        lines.push(label_value_line(
            text("settings.mods.save_data", "Game Save:"),
            if package.has_save_storage {
                text("settings.mods.storage_has", "Available")
            } else {
                text("settings.mods.storage_none", "None")
            },
            if package.has_save_storage {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::DarkGray)
            },
        ));

        lines.push(Line::from(""));
        lines.push(section_title_line(text(
            "settings.mods.section.security",
            "Security",
        )));
        lines.push(label_value_line(
            text("settings.mods.write_request", "Direct Write Request:"),
            if package.has_write_request {
                text("settings.mods.storage_has", "Available")
            } else {
                text("settings.mods.storage_none", "None")
            },
            if package.has_write_request {
                Style::default().fg(Color::Red)
            } else {
                Style::default().fg(Color::DarkGray)
            },
        ));
        let safe_mode_text = match package.safe_mode_state {
            ModSafeModeState::Enabled => text("settings.mods.safe_mode_on", "On"),
            ModSafeModeState::DisabledSession => text(
                "settings.mods.safe_mode_session_off",
                "Disabled (This Session)",
            ),
            ModSafeModeState::DisabledTrusted => text(
                "settings.mods.safe_mode_trusted_off",
                "Disabled (Permanently Trusted)",
            ),
        };
        lines.push(label_value_line(
            text("settings.mods.safe_mode", "Safe Mode:"),
            safe_mode_text,
            if package.safe_mode_enabled {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Red)
            },
        ));

        lines.push(Line::from(""));
        lines.push(section_title_line(text(
            "settings.mods.section.introduction",
            "Mod Introduction",
        )));
        lines.extend(rich_text::parse_rich_text_wrapped(
            &package.introduction,
            content_width,
            Style::default().fg(Color::White),
        ));
        lines
    };

    let viewport_h = inner.height as usize;
    let full_width = inner.width.max(1) as usize;
    let wide_lines = build_detail_lines(full_width);
    let needs_scroll = wide_lines.len() > viewport_h;
    let lines = if needs_scroll && inner.width > 2 {
        build_detail_lines(inner.width.saturating_sub(2).max(1) as usize)
    } else {
        wide_lines
    };
    lines.len().saturating_sub(viewport_h) > 0
}

fn render_mod_list(frame: &mut ratatui::Frame<'_>, area: Rect, state: &SettingsState) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(mod_list_title(state))
        .border_style(Style::default().fg(Color::White));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(inner);

    if state.mod_packages.is_empty() {
        frame.render_widget(
            Paragraph::new(text("settings.mods.empty", "No mods found."))
                .style(Style::default().fg(Color::White))
                .alignment(Alignment::Center),
            rows[0],
        );
        frame.render_widget(
            Paragraph::new("1/1")
                .style(Style::default().fg(Color::White))
                .alignment(Alignment::Center),
            rows[1],
        );
        return;
    }

    let item_height = mod_item_height(state.mod_list_view);
    let page_size = ((rows[0].height / item_height).max(1)) as usize;
    let total_pages = total_mod_pages(state.mod_packages.len(), page_size);
    let page = state.mod_page.min(total_pages.saturating_sub(1));
    let start = page * page_size;

    for (index, package) in state
        .mod_packages
        .iter()
        .enumerate()
        .skip(start)
        .take(page_size)
    {
        let local = (index - start) as u16;
        let item_area = Rect::new(
            rows[0].x,
            rows[0].y + local * item_height,
            rows[0].width,
            item_height.min(rows[0].height.saturating_sub(local * item_height)),
        );
        render_mod_list_item(
            frame.buffer_mut(),
            item_area,
            package,
            index == state.mod_selected,
            state.mod_list_view,
        );
    }

    let left = if page > 0 {
        i18n::t("game_selection.pager.prev")
    } else {
        String::new()
    };
    let right = if page + 1 < total_pages {
        i18n::t("game_selection.pager.next")
    } else {
        String::new()
    };
    let pager_line = if let Some(dialog) = &state.mod_page_jump_dialog {
        let input_text = if dialog.input.is_empty() {
            "_".to_string()
        } else {
            dialog.input.clone()
        };
        let input_style = Style::default()
            .fg(if dialog.input.is_empty() {
                Color::Yellow
            } else {
                Color::Black
            })
            .bg(Color::Yellow);
        Line::from(vec![
            Span::styled(input_text, input_style),
            Span::styled(
                format!("/{}", total_pages.max(1)),
                Style::default().fg(Color::White),
            ),
        ])
    } else {
        Line::from(Span::styled(
            format!("{}/{}", page + 1, total_pages.max(1)),
            Style::default().fg(Color::White),
        ))
    };
    frame.render_widget(
        Paragraph::new(left)
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Left),
        rows[1],
    );
    frame.render_widget(
        Paragraph::new(pager_line).alignment(Alignment::Center),
        rows[1],
    );
    frame.render_widget(
        Paragraph::new(right)
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Right),
        rows[1],
    );
}

fn render_mod_list_item(
    buffer: &mut Buffer,
    area: Rect,
    package: &ModPackage,
    selected: bool,
    list_view: ModListView,
) {
    if area.height == 0 || area.width == 0 {
        return;
    }

    let base_style = if selected {
        Style::default().bg(Color::DarkGray)
    } else {
        Style::default()
    };
    let meta_style = if selected {
        Style::default().fg(Color::White).bg(Color::DarkGray)
    } else {
        Style::default().fg(Color::Gray)
    };

    for dy in 0..area.height {
        buffer.set_string(
            area.x,
            area.y + dy,
            " ".repeat(area.width as usize),
            Style::default(),
        );
    }
    let highlight_rows = match list_view {
        ModListView::Detailed if selected => area.height.min(4),
        ModListView::Simple if selected => area.height.min(1),
        _ => 0,
    };
    for dy in 0..highlight_rows {
        buffer.set_string(
            area.x,
            area.y + dy,
            " ".repeat(area.width as usize),
            base_style,
        );
    }

    match list_view {
        ModListView::Detailed => {
            let thumb_width = 8u16;
            let text_x = area.x + thumb_width + 2;
            let content_height = area.height.min(4);
            let safe_marker_width = if package.safe_mode_enabled { 0 } else { 1 };
            let text_width = area.width.saturating_sub(thumb_width + 2 + safe_marker_width) as usize;
            if text_width == 0 {
                return;
            }

            for (idx, line) in package
                .thumbnail
                .rendered_lines
                .iter()
                .take(content_height as usize)
                .enumerate()
            {
                render_compiled_line_to_buffer(
                    buffer,
                    area.x,
                    area.y + idx as u16,
                    thumb_width as usize,
                    line,
                );
            }

            render_mod_debug_prefix(buffer, text_x, area.y, package.debug_enabled, selected);
            let name_x = text_x + if package.debug_enabled { 3 } else { 0 };
            let name_width = text_width.saturating_sub(if package.debug_enabled { 3 } else { 0 });
            render_manifest_text_to_buffer(
                buffer,
                name_x,
                area.y,
                name_width,
                &package.package_name,
                package.package_name_allows_rich,
                base_style.add_modifier(Modifier::BOLD),
            );

            if content_height > 1 {
                render_label_manifest_value_to_buffer(
                    buffer,
                    text_x,
                    area.y + 1,
                    text_width,
                    &text("settings.mods.author", "Author:"),
                    &package.author,
                    true,
                    meta_style,
                );
            }
            if content_height > 2 {
                render_label_manifest_value_to_buffer(
                    buffer,
                    text_x,
                    area.y + 2,
                    text_width,
                    &text("settings.mods.version", "Version:"),
                    &package.version,
                    true,
                    meta_style,
                );
            }
            if content_height > 3 {
                render_mod_status_line(buffer, text_x, area.y + 3, text_width, package, selected);
            }

            if !package.safe_mode_enabled {
                render_safe_mode_marker_column(buffer, area, content_height);
            }
        }
        ModListView::Simple => {
            let safe_marker_width = if package.safe_mode_enabled { 0 } else { 1 };
            let status_width = 6usize;
            let text_width = area.width.saturating_sub(safe_marker_width) as usize;
            let name_width = text_width.saturating_sub(status_width + 1);
            let text_x = area.x;

            render_mod_debug_prefix(buffer, text_x, area.y, package.debug_enabled, selected);
            let name_x = text_x + if package.debug_enabled { 3 } else { 0 };
            let actual_name_width = name_width.saturating_sub(if package.debug_enabled { 3 } else { 0 });
            render_manifest_text_to_buffer(
                buffer,
                name_x,
                area.y,
                actual_name_width,
                &package.package_name,
                package.package_name_allows_rich,
                base_style,
            );

            let status_x = area.x + name_width as u16 + 1;
            render_enabled_tag(buffer, status_x, area.y, package.enabled, selected);

            if !package.safe_mode_enabled {
                let marker_x = area.x + area.width - 1;
                buffer.set_string(
                    marker_x,
                    area.y,
                    " ",
                    Style::default().bg(Color::Red),
                );
            }
        }
    }
}


fn mod_list_title(state: &SettingsState) -> Line<'static> {
    let order_text = if state.mod_sort_descending {
        format!("\u{2191}{}", text("settings.mods.order.desc", "Descending"))
    } else {
        format!("\u{2193}{}", text("settings.mods.order.asc", "Ascending"))
    };
    Line::from(vec![
        Span::raw(" "),
        Span::styled(
            text("settings.mods.title", "Mods"),
            Style::default().fg(Color::White),
        ),
        Span::styled(" *", Style::default().fg(Color::White)),
        Span::styled(
            mod_sort_label(state.mod_sort_mode),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" ", Style::default().fg(Color::White)),
        Span::styled("[", Style::default().fg(Color::White)),
        Span::styled(order_text, Style::default().fg(Color::DarkGray)),
        Span::styled("]", Style::default().fg(Color::White)),
        Span::raw(" "),
    ])
}

fn mod_sort_label(mode: ModSortMode) -> String {
    match mode {
        ModSortMode::Name => text("settings.mods.sort.name", "Name"),
        ModSortMode::Enabled => text("settings.mods.sort.enabled", "Enabled"),
        ModSortMode::Author => text("settings.mods.sort.author", "Author"),
        ModSortMode::SafeMode => text("settings.mods.sort.safe_mode", "Safe Mode"),
    }
}

fn mod_item_height(list_view: ModListView) -> u16 {
    match list_view {
        ModListView::Detailed => 5,
        ModListView::Simple => 1,
    }
}

fn render_mod_debug_prefix(
    buffer: &mut Buffer,
    x: u16,
    y: u16,
    enabled: bool,
    selected: bool,
) {
    if !enabled {
        return;
    }

    let bg = if selected { Color::DarkGray } else { Color::Reset };
    buffer.set_string(x, y, "[", Style::default().fg(Color::White).bg(bg));
    buffer.set_string(x + 1, y, "D", Style::default().fg(Color::LightBlue).bg(bg));
    buffer.set_string(x + 2, y, "]", Style::default().fg(Color::White).bg(bg));
}

fn render_enabled_tag(buffer: &mut Buffer, x: u16, y: u16, enabled: bool, selected: bool) {
    let bg = if selected { Color::DarkGray } else { Color::Reset };
    let value = if enabled { "ON" } else { "OFF" };
    let value_style = Style::default()
        .fg(if enabled { Color::Green } else { Color::Red })
        .bg(bg)
        .add_modifier(Modifier::BOLD);
    let bracket_style = Style::default().fg(Color::White).bg(bg);

    buffer.set_string(x, y, "[", bracket_style);
    buffer.set_string(x + 1, y, value, value_style);
    buffer.set_string(x + 1 + value.len() as u16, y, "]", bracket_style);
}

fn render_mod_status_line(
    buffer: &mut Buffer,
    x: u16,
    y: u16,
    width: usize,
    package: &ModPackage,
    selected: bool,
) {
    let bg = if selected { Color::DarkGray } else { Color::Reset };
    let label = format!("{} ", text("settings.mods.state", "State:"));
    let label_style = Style::default().fg(Color::Gray).bg(bg);
    let value_style = Style::default()
        .fg(if package.enabled { Color::Green } else { Color::Red })
        .bg(bg)
        .add_modifier(Modifier::BOLD);

    buffer.set_stringn(x, y, &label, width, label_style);
    let value_x = x + UnicodeWidthStr::width(label.as_str()) as u16;
    buffer.set_stringn(
        value_x,
        y,
        if package.enabled {
            text("settings.mods.enabled", "Enabled")
        } else {
            text("settings.mods.disabled", "Disabled")
        },
        width.saturating_sub(UnicodeWidthStr::width(label.as_str())),
        value_style,
    );
}

fn render_safe_mode_marker_column(
    buffer: &mut Buffer,
    area: Rect,
    content_height: u16,
) {
    let marker_x = area.x + area.width - 1;
    let style = Style::default().bg(Color::Red);
    for dy in 0..content_height {
        buffer.set_string(marker_x, area.y + dy, " ", style);
    }
}

fn section_title_line(title: String) -> Line<'static> {
    Line::from(Span::styled(
        title,
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    ))
}

fn label_value_line(label: String, value: String, value_style: Style) -> Line<'static> {
    Line::from(vec![
        Span::styled(label, Style::default().fg(Color::White)),
        Span::raw(" "),
        Span::styled(value, value_style.add_modifier(Modifier::BOLD)),
    ])
}

fn label_value_lines(
    label: String,
    value: String,
    allow_rich: bool,
    width: usize,
    value_style: Style,
) -> Vec<Line<'static>> {
    if !allow_rich || !value.starts_with("f%") {
        return vec![label_value_line(label, value, value_style)];
    }

    let mut parsed = rich_text::parse_rich_text_wrapped(&value, usize::MAX / 8, value_style);
    if parsed.is_empty() {
        return vec![label_value_line(label, String::new(), value_style)];
    }

    let mut first_spans = vec![
        Span::styled(label.clone(), Style::default().fg(Color::White)),
        Span::raw(" "),
    ];
    first_spans.extend(parsed.remove(0).spans);

    let mut lines = vec![Line::from(first_spans)];
    let indent = " ".repeat(UnicodeWidthStr::width(label.as_str()) + 1);
    let continuation_width = width.saturating_sub(indent.len()).max(1);
    for line in parsed {
        let mut spans = vec![Span::styled(indent.clone(), Style::default().fg(Color::White))];
        let wrapped = crop_line_center_to_width(&line, continuation_width);
        spans.extend(wrapped.spans);
        lines.push(Line::from(spans));
    }
    lines
}

fn build_mod_hint_segments(include_scroll: bool) -> Vec<String> {
    let mut segments = vec![
        text("settings.mods.hint.toggle", "[Enter] Toggle"),
        text("settings.mods.hint.debug", "[D] Debug"),
        text("settings.mods.hint.safe_mode", "[R] Safe Mode"),
        text("settings.mods.hint.hot_reload", "[H] Hot Reload"),
        text("settings.mods.hint.view", "[L] View"),
        text("settings.mods.hint.jump", "[P] Jump"),
        text("settings.mods.hint.sort_mode", "[Z] Sort"),
        text("settings.mods.hint.sort_order", "[X] Order"),
        text("settings.mods.hint.move", "[\u{2191}]/[\u{2193}] Move"),
        text("settings.mods.hint.page", "[Q]/[E] Page"),
        text("settings.hub.back_hint", "[ESC] Return to main menu"),
    ];
    if include_scroll {
        segments.push(text("settings.mods.hint.scroll", "[W]/[S] Scroll Details"));
    }
    segments
}

fn wrap_mod_hint_lines(segments: &[String], width: usize) -> Vec<Line<'static>> {
    if width == 0 || segments.is_empty() {
        return vec![Line::from("")];
    }

    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut current_segments: Vec<Span<'static>> = Vec::new();
    let mut current_width = 0usize;

    for segment in segments {
        let segment_width = UnicodeWidthStr::width(segment.as_str());
        let separator_width = if current_segments.is_empty() { 0 } else { 2 };

        if !current_segments.is_empty() && current_width + separator_width + segment_width > width {
            lines.push(Line::from(std::mem::take(&mut current_segments)));
            current_width = 0;
        }

        if !current_segments.is_empty() {
            current_segments.push(Span::raw("  "));
            current_width += 2;
        }
        current_segments.push(Span::raw(segment.clone()));
        current_width += segment_width;
    }

    if !current_segments.is_empty() {
        lines.push(Line::from(current_segments));
    }

    if lines.is_empty() {
        lines.push(Line::from(""));
    }

    lines
}

fn wrap_plain_text_lines(text: &str, width: usize, style: Style) -> Vec<Line<'static>> {
    let width = width.max(1);
    let mut out = Vec::new();

    for raw_line in text.lines() {
        let mut remaining = raw_line.trim_end().to_string();
        if remaining.is_empty() {
            out.push(Line::from(""));
            continue;
        }

        while !remaining.is_empty() {
            if UnicodeWidthStr::width(remaining.as_str()) <= width {
                out.push(Line::from(Span::styled(remaining.clone(), style)));
                break;
            }

            let mut cur = String::new();
            let mut cur_width = 0usize;
            let mut last_space_byte = None;

            for (idx, ch) in remaining.char_indices() {
                let ch_width = UnicodeWidthStr::width(ch.encode_utf8(&mut [0; 4]));
                if cur_width + ch_width > width {
                    break;
                }
                cur.push(ch);
                cur_width += ch_width;
                if ch.is_whitespace() {
                    last_space_byte = Some(idx);
                }
            }

            if cur.is_empty() {
                if let Some(ch) = remaining.chars().next() {
                    out.push(Line::from(Span::styled(ch.to_string(), style)));
                    remaining = remaining[ch.len_utf8()..].trim_start().to_string();
                }
                continue;
            }

            if let Some(space_idx) = last_space_byte {
                let head = remaining[..space_idx].trim_end().to_string();
                if !head.is_empty() {
                    out.push(Line::from(Span::styled(head, style)));
                }
                remaining = remaining[space_idx + 1..].trim_start().to_string();
            } else {
                out.push(Line::from(Span::styled(cur.clone(), style)));
                remaining = remaining[cur.len()..].trim_start().to_string();
            }
        }
    }

    if out.is_empty() {
        out.push(Line::from(""));
    }
    out
}

fn render_mod_detail(frame: &mut ratatui::Frame<'_>, area: Rect, state: &mut SettingsState) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ", text("settings.mods.detail", "Mod Details")))
        .border_style(Style::default().fg(Color::White));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(package) = state.mod_packages.get(state.mod_selected) else {
        frame.render_widget(
            Paragraph::new(text("settings.mods.empty", "No mods found."))
                .style(Style::default().fg(Color::White))
                .alignment(Alignment::Center),
            inner,
        );
        return;
    };

    let build_detail_lines = |content_width: usize| {
        let mut lines = rich_lines_from_image(
            &package.banner,
            content_width,
            Style::default().fg(Color::White),
        );
        lines.push(Line::from(""));
        lines.push(section_title_line(text(
            "settings.mods.section.basic_info",
            "Basic Info",
        )));
        lines.extend(label_value_lines(
            text("settings.mods.package_label", "Mod Package:"),
            package.package_name.clone(),
            package.package_name_allows_rich,
            content_width,
            Style::default().fg(Color::White),
        ));
        lines.extend(label_value_lines(
            text("settings.mods.author", "Author:"),
            package.author.clone(),
            true,
            content_width,
            Style::default().fg(Color::White),
        ));
        lines.extend(label_value_lines(
            text("settings.mods.version", "Version:"),
            package.version.clone(),
            true,
            content_width,
            Style::default().fg(Color::White),
        ));

        lines.push(Line::from(""));
        lines.push(section_title_line(text(
            "settings.mods.section.storage",
            "Data Storage",
        )));
        lines.push(label_value_line(
            text("settings.mods.best_score", "Best Score:"),
            if package.has_best_score_storage {
                text("settings.mods.storage_has", "Available")
            } else {
                text("settings.mods.storage_none", "None")
            },
            if package.has_best_score_storage {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::DarkGray)
            },
        ));
        lines.push(label_value_line(
            text("settings.mods.save_data", "Game Save:"),
            if package.has_save_storage {
                text("settings.mods.storage_has", "Available")
            } else {
                text("settings.mods.storage_none", "None")
            },
            if package.has_save_storage {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::DarkGray)
            },
        ));

        lines.push(Line::from(""));
        lines.push(section_title_line(text(
            "settings.mods.section.security",
            "Security",
        )));
        lines.push(label_value_line(
            text("settings.mods.write_request", "Direct Write Request:"),
            if package.has_write_request {
                text("settings.mods.storage_has", "Available")
            } else {
                text("settings.mods.storage_none", "None")
            },
            if package.has_write_request {
                Style::default().fg(Color::Red)
            } else {
                Style::default().fg(Color::DarkGray)
            },
        ));
        let safe_mode_text = match package.safe_mode_state {
            ModSafeModeState::Enabled => text("settings.mods.safe_mode_on", "On"),
            ModSafeModeState::DisabledSession => text(
                "settings.mods.safe_mode_session_off",
                "Disabled (This Session)",
            ),
            ModSafeModeState::DisabledTrusted => text(
                "settings.mods.safe_mode_trusted_off",
                "Disabled (Permanently Trusted)",
            ),
        };
        lines.push(label_value_line(
            text("settings.mods.safe_mode", "Safe Mode:"),
            safe_mode_text,
            if package.safe_mode_enabled {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Red)
            },
        ));

        lines.push(Line::from(""));
        lines.push(section_title_line(text(
            "settings.mods.section.introduction",
            "Mod Introduction",
        )));
        lines.extend(rich_text::parse_rich_text_wrapped(
            &package.introduction,
            content_width,
            Style::default().fg(Color::White),
        ));
        lines
    };

    let viewport_h = inner.height as usize;
    let full_width = inner.width.max(1) as usize;
    let wide_lines = build_detail_lines(full_width);
    let needs_scroll = wide_lines.len() > viewport_h;

    let (lines, text_area) = if needs_scroll && inner.width > 2 {
        (
            build_detail_lines(inner.width.saturating_sub(2).max(1) as usize),
            Rect::new(inner.x, inner.y, inner.width - 2, inner.height),
        )
    } else {
        (wide_lines, inner)
    };

    let max_scroll = lines.len().saturating_sub(viewport_h);
    if state.mod_detail_scroll > max_scroll {
        state.mod_detail_scroll = max_scroll;
    }
    state.mod_detail_scroll_available = max_scroll > 0;

    frame.render_widget(
        Paragraph::new(lines)
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: false })
            .scroll((state.mod_detail_scroll as u16, 0)),
        text_area,
    );

    if state.mod_detail_scroll_available && inner.width > 2 {
        let scroll_x = inner.x + inner.width - 1;
        let can_up = state.mod_detail_scroll > 0;
        let can_down = state.mod_detail_scroll < max_scroll;

        frame.render_widget(
            Paragraph::new(if can_up { "↑" } else { " " }).style(Style::default().fg(Color::White)),
            Rect::new(scroll_x, inner.y, 1, 1),
        );
        frame.render_widget(
            Paragraph::new(if can_up { "W" } else { " " }).style(Style::default().fg(Color::White)),
            Rect::new(scroll_x, inner.y.saturating_add(1), 1, 1),
        );

        if inner.height > 4 {
            let track_start = inner.y.saturating_add(2);
            let track_len = inner.height.saturating_sub(4);
            let pos = if max_scroll == 0 {
                0
            } else {
                ((state.mod_detail_scroll * (track_len as usize - 1)) / max_scroll) as u16
            };
            frame.render_widget(
                Paragraph::new("█").style(Style::default().fg(Color::White)),
                Rect::new(scroll_x, track_start.saturating_add(pos), 1, 1),
            );
        }

        let d_y = inner.y + inner.height.saturating_sub(2);
        frame.render_widget(
            Paragraph::new(if can_down { "S" } else { " " })
                .style(Style::default().fg(Color::White)),
            Rect::new(scroll_x, d_y, 1, 1),
        );
        frame.render_widget(
            Paragraph::new(if can_down { "↓" } else { " " })
                .style(Style::default().fg(Color::White)),
            Rect::new(scroll_x, d_y.saturating_add(1), 1, 1),
        );
    }
}

fn rich_lines_from_image(image: &mods::ModImage, width: usize, base: Style) -> Vec<Line<'static>> {
    image
        .rendered_lines
        .iter()
        .take(13)
        .map(|line| center_or_crop_line_to_width(line, width, base))
        .collect()
}

fn center_or_crop_line_to_width(line: &Line<'static>, width: usize, base: Style) -> Line<'static> {
    let line_width = line_width(line);
    if width == 0 {
        return Line::from("");
    }
    if line_width > width {
        return crop_line_center_to_width(line, width);
    }
    if line_width == width {
        return line.clone();
    }

    let pad = width.saturating_sub(line_width);
    let left = pad / 2;
    let right = pad.saturating_sub(left);
    let mut spans = Vec::new();
    if left > 0 {
        spans.push(Span::styled(" ".repeat(left), base));
    }
    spans.extend(line.spans.iter().cloned());
    if right > 0 {
        spans.push(Span::styled(" ".repeat(right), base));
    }
    Line::from(spans)
}

fn line_width(line: &Line<'static>) -> usize {
    line.spans
        .iter()
        .map(|span| UnicodeWidthStr::width(span.content.as_ref()))
        .sum()
}

fn crop_line_center_to_width(line: &Line<'static>, width: usize) -> Line<'static> {
    if width == 0 {
        return Line::from("");
    }

    let mut cells = Vec::<(char, Style, usize)>::new();
    for span in &line.spans {
        for ch in span.content.chars() {
            let ch_width = UnicodeWidthStr::width(ch.encode_utf8(&mut [0; 4]));
            if ch_width == 0 {
                continue;
            }
            cells.push((ch, span.style, ch_width));
        }
    }

    let mut total_width: usize = cells.iter().map(|(_, _, w)| *w).sum();
    if total_width <= width {
        return line.clone();
    }

    let mut trim_left = true;
    while total_width > width && !cells.is_empty() {
        if trim_left {
            if let Some((_, _, w)) = cells.first().copied() {
                total_width = total_width.saturating_sub(w);
            }
            cells.remove(0);
        } else if let Some((_, _, w)) = cells.pop() {
            total_width = total_width.saturating_sub(w);
        }
        trim_left = !trim_left;
    }

    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut current_style: Option<Style> = None;
    let mut current_text = String::new();
    for (ch, style, _) in cells {
        match current_style {
            Some(existing) if existing == style => current_text.push(ch),
            Some(existing) => {
                spans.push(Span::styled(current_text.clone(), existing));
                current_text.clear();
                current_text.push(ch);
                current_style = Some(style);
            }
            None => {
                current_text.push(ch);
                current_style = Some(style);
            }
        }
    }
    if let Some(style) = current_style {
        spans.push(Span::styled(current_text, style));
    }
    Line::from(spans)
}

fn render_rich_line_to_buffer(
    buffer: &mut Buffer,
    x: u16,
    y: u16,
    width: usize,
    text: &str,
    base: Style,
) {
    let lines = rich_text::parse_rich_text_wrapped(text, width.max(1), base);
    let Some(first_line) = lines.first() else {
        return;
    };
    let line = crop_line_center_to_width(first_line, width.max(1));

    let mut cursor_x = x;
    for span in &line.spans {
        let content = span.content.as_ref();
        if content.is_empty() {
            continue;
        }
        let remaining = width.saturating_sub(cursor_x.saturating_sub(x) as usize);
        if remaining == 0 {
            break;
        }
        buffer.set_stringn(cursor_x, y, content, remaining, span.style);
        cursor_x = cursor_x.saturating_add(UnicodeWidthStr::width(content) as u16);
        if cursor_x >= x.saturating_add(width as u16) {
            break;
        }
    }
}

fn render_compiled_line_to_buffer(
    buffer: &mut Buffer,
    x: u16,
    y: u16,
    width: usize,
    line: &Line<'static>,
) {
    let line = crop_line_center_to_width(line, width.max(1));

    let mut cursor_x = x;
    for span in &line.spans {
        let content = span.content.as_ref();
        if content.is_empty() {
            continue;
        }
        let remaining = width.saturating_sub(cursor_x.saturating_sub(x) as usize);
        if remaining == 0 {
            break;
        }
        buffer.set_stringn(cursor_x, y, content, remaining, span.style);
        cursor_x = cursor_x.saturating_add(UnicodeWidthStr::width(content) as u16);
        if cursor_x >= x.saturating_add(width as u16) {
            break;
        }
    }
}

fn render_manifest_text_to_buffer(
    buffer: &mut Buffer,
    x: u16,
    y: u16,
    width: usize,
    text: &str,
    allow_rich: bool,
    base: Style,
) {
    if allow_rich && text.starts_with("f%") {
        render_rich_line_to_buffer(buffer, x, y, width, text, base);
        return;
    }
    let line = truncate_with_ellipsis_plain(text, width);
    buffer.set_stringn(x, y, line, width, base);
}

fn render_label_manifest_value_to_buffer(
    buffer: &mut Buffer,
    x: u16,
    y: u16,
    width: usize,
    label: &str,
    value: &str,
    allow_rich: bool,
    base: Style,
) {
    let prefix = format!("{label} ");
    let prefix_width = UnicodeWidthStr::width(prefix.as_str());
    buffer.set_stringn(x, y, &prefix, width, base);
    let value_x = x.saturating_add(prefix_width as u16);
    let value_width = width.saturating_sub(prefix_width);
    render_manifest_text_to_buffer(buffer, value_x, y, value_width, value, allow_rich, base);
}

fn truncate_with_ellipsis_plain(text: &str, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }
    if UnicodeWidthStr::width(text) <= max_width {
        return text.to_string();
    }
    if max_width <= 3 {
        return ".".repeat(max_width);
    }

    let mut result = String::new();
    for ch in text.chars() {
        let next = format!("{result}{ch}");
        if UnicodeWidthStr::width(next.as_str()) + 3 > max_width {
            break;
        }
        result.push(ch);
    }
    result.push_str("...");
    result
}

fn current_mod_page_size(list_view: ModListView) -> usize {
    let (_, term_height) = crossterm::terminal::size().unwrap_or((90, 26));
    let root_height = term_height.saturating_sub(1);
    let inner_height = root_height.saturating_sub(2);
    let content_height = inner_height.saturating_sub(1);
    (content_height / mod_item_height(list_view)).max(1) as usize
}

fn total_mod_pages(total_items: usize, page_size: usize) -> usize {
    if total_items == 0 {
        1
    } else {
        ((total_items + page_size.saturating_sub(1)) / page_size).max(1)
    }
}

fn render_mod_safe_dialog(frame: &mut ratatui::Frame<'_>, dialog: &ModSafeDialog) {
    use ratatui::widgets::Clear;

    let area = frame.area();
    frame.render_widget(Clear, area);

    let width = area.width.saturating_sub(8).clamp(40, 72);
    let remaining = 5u64.saturating_sub(dialog.opened_at.elapsed().as_secs());
    let countdown_done = remaining == 0;
    let message = text(
        "settings.mods.safe_mode_dialog.message",
        "Are you sure you want to disable Safe Mode for mod \"{mod_name}\"?\n\nSafe Mode is designed to protect your device. After disabling it, this mod may perform high-risk operations such as file writes or system calls, which may cause data loss or system instability.\nPlease make sure you fully trust the source and author of this mod.",
    )
    .replace("{mod_name}", &dialog.mod_name);

    let content_width = width.saturating_sub(4).max(1) as usize;
    let mut lines = wrap_plain_text_lines(
        &message,
        content_width,
        Style::default().fg(Color::White),
    );
    lines.push(Line::from(""));
    let index_style = Style::default().fg(Color::White);
    lines.push(Line::from(vec![
        Span::styled("[1] ", index_style),
        Span::styled(
            text("settings.mods.safe_mode_dialog.cancel", "Cancel"),
            Style::default().fg(Color::LightGreen),
        ),
    ]));
    let gated_style = Style::default().fg(if countdown_done {
        Color::Red
    } else {
        Color::DarkGray
    });
    lines.push(Line::from(vec![
        Span::styled("[2] ", index_style),
        Span::styled(
            format!(
                "{} {}{}",
                text(
                    "settings.mods.safe_mode_dialog.disable_once",
                    "Confirm Disable"
                ),
                text(
                    "settings.mods.safe_mode_dialog.disable_once_sub",
                    "(Only This Time)"
                ),
                if countdown_done {
                    String::new()
                } else {
                    format!(
                        " {}",
                        text("settings.mods.safe_mode_dialog.countdown", "{seconds}s")
                            .replace("{seconds}", &remaining.to_string())
                    )
                }
            ),
            gated_style,
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled("[3] ", index_style),
        Span::styled(
            format!(
                "{} {}{}",
                text(
                    "settings.mods.safe_mode_dialog.disable_forever",
                    "Confirm Disable"
                ),
                text(
                    "settings.mods.safe_mode_dialog.disable_forever_sub",
                    "(Permanently Trust This Mod)"
                ),
                if countdown_done {
                    String::new()
                } else {
                    format!(
                        " {}",
                        text("settings.mods.safe_mode_dialog.countdown", "{seconds}s")
                            .replace("{seconds}", &remaining.to_string())
                    )
                }
            ),
            gated_style,
        ),
    ]));

    let content_height = lines.len().max(1).min(u16::MAX as usize) as u16;
    let height = content_height
        .saturating_add(2)
        .clamp(8, area.height.saturating_sub(2).max(8));
    let rect = centered_rect(area, width, height);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(
            " {} ",
            text("settings.mods.safe_mode_dialog.title", "Safe Mode")
        ))
        .border_style(Style::default().fg(Color::White));
    let inner = block.inner(rect);
    frame.render_widget(block, rect);

    frame.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .alignment(Alignment::Left),
        inner,
    );
    render_box_back_hint(
        frame,
        rect,
        text("settings.secondary.back_hint", "[ESC]/[Q] Return to main menu"),
    );
}

fn draw_language_grid(
    buffer: &mut Buffer,
    area: Rect,
    languages: &[i18n::LanguagePack],
    selected: usize,
) {
    let metrics = grid_metrics(area.width, languages);
    let cols = metrics.cols;
    let rows = ((languages.len() + cols - 1) / cols).max(1);

    let grid_width = cols as u16 * metrics.outer_width + (cols.saturating_sub(1) as u16) * H_GAP;
    let grid_height = rows as u16 * 3;

    let start_x = area.x + area.width.saturating_sub(grid_width) / 2;
    let start_y = area.y + area.height.saturating_sub(grid_height) / 2;

    let current_code = i18n::current_language_code();

    for (idx, pack) in languages.iter().enumerate() {
        let row = idx / cols;
        let col = idx % cols;
        let x = start_x + col as u16 * (metrics.outer_width + H_GAP);
        let y = start_y + row as u16 * 3;

        let is_selected = idx == selected;
        let is_current = pack.code == current_code;

        let text_style = if is_current {
            Style::default()
                .fg(Color::Rgb(173, 255, 47))
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        let border_style = Style::default().fg(Color::White);
        let label = center_text(pack.name.as_str(), metrics.inner_width as usize);

        if is_selected {
            let top = format!(
                "\u{2554}{}\u{2557}",
                "\u{2550}".repeat(metrics.inner_width as usize)
            );
            let bottom = format!(
                "\u{255A}{}\u{255D}",
                "\u{2550}".repeat(metrics.inner_width as usize)
            );
            buffer.set_string(x, y, top, border_style);
            buffer.set_string(x, y + 1, "\u{2551}", border_style);
            buffer.set_string(x + 1, y + 1, label.clone(), text_style);
            buffer.set_string(x + 1 + metrics.inner_width, y + 1, "\u{2551}", border_style);
            buffer.set_string(x, y + 2, bottom, border_style);
        } else {
            let blank = " ".repeat(metrics.outer_width as usize);
            let mid = format!(" {} ", label);
            buffer.set_string(x, y, blank.clone(), Style::default());
            buffer.set_string(x, y + 1, mid, text_style);
            buffer.set_string(x, y + 2, blank, Style::default());
        }
    }
}

fn center_text(text: &str, width: usize) -> String {
    let current = UnicodeWidthStr::width(text);
    if current >= width {
        return text.to_string();
    }

    let remaining = width - current;
    let left = remaining / 2;
    let right = remaining - left;
    format!("{}{}{}", " ".repeat(left), text, " ".repeat(right))
}
