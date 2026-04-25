use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use unicode_width::UnicodeWidthStr;

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use std::cmp::Ordering;
use std::fs;
use std::time::{Duration, Instant};

use crate::app::content_cache;
use crate::app::i18n;
use crate::app::rich_text;
use crate::core::key::{display_semantic_key, semantic_key_source};
use crate::core::save as runtime_save;
use crate::game::action::{ActionBinding, ActionKeys};
use crate::game::registry::{GameDescriptor, GameSourceKind};
use crate::game::resources;
use crate::mods::{self, ModPackage, ModSafeModeState};
use crate::utils::path_utils;

const MAX_COLS: usize = 12;
const H_GAP: u16 = 1;
const TRIANGLE: &str = "\u{25B6} ";
const MOD_HOT_RELOAD_POLL_INTERVAL: Duration = Duration::from_secs(1);
const SHIFT_BIND_HOLD: Duration = Duration::from_secs(1);
const KEYBIND_ACTION_PADDING: u16 = 2;
const KEYBIND_CAPTURE_DELAY: Duration = Duration::from_millis(120);

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
    Keybind,
    Memory,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KeybindFocus {
    Games,
    Actions,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KeybindEditMode {
    Add,
    Delete,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KeybindGameSortMode {
    Source,
    Name,
    Author,
}

#[derive(Clone, Debug)]
pub struct KeybindCaptureState {
    pub slot_index: usize,
    pub accept_after: Instant,
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

    fn apply_keybind_sort(&mut self) {
        let mode = self.keybind_sort_mode;
        let descending = self.keybind_sort_descending;
        self.keybind_games.sort_by(|left, right| {
            let ordering = compare_keybind_games(left, right, mode);
            if descending { ordering.reverse() } else { ordering }
        });
    }

    fn set_keybind_sort_mode(&mut self, mode: KeybindGameSortMode) {
        let selected_id = self
            .keybind_games
            .get(self.keybind_selected)
            .map(|game| game.id.clone());
        self.keybind_sort_mode = mode;
        self.apply_keybind_sort();
        self.restore_keybind_selection(selected_id);
    }

    fn toggle_keybind_sort_order(&mut self) {
        let selected_id = self
            .keybind_games
            .get(self.keybind_selected)
            .map(|game| game.id.clone());
        self.keybind_sort_descending = !self.keybind_sort_descending;
        self.apply_keybind_sort();
        self.restore_keybind_selection(selected_id);
    }

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

pub fn default_selected_index() -> usize {
    let languages = i18n::available_languages();
    let current = i18n::current_language_code();
    languages
        .iter()
        .position(|pack| pack.code == current)
        .unwrap_or(0)
}

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

fn compare_keybind_games(
    left: &GameDescriptor,
    right: &GameDescriptor,
    mode: KeybindGameSortMode,
) -> Ordering {
    match mode {
        KeybindGameSortMode::Source => source_rank(&left.source)
            .cmp(&source_rank(&right.source))
            .then_with(|| cmp_lowercase(&left.display_name, &right.display_name))
            .then_with(|| left.id.cmp(&right.id)),
        KeybindGameSortMode::Name => cmp_lowercase(&left.display_name, &right.display_name)
            .then_with(|| source_rank(&left.source).cmp(&source_rank(&right.source)))
            .then_with(|| left.id.cmp(&right.id)),
        KeybindGameSortMode::Author => cmp_lowercase(&left.display_author, &right.display_author)
            .then_with(|| cmp_lowercase(&left.display_name, &right.display_name))
            .then_with(|| left.id.cmp(&right.id)),
    }
}

fn source_rank(source: &GameSourceKind) -> u8 {
    match source {
        GameSourceKind::Official => 0,
        GameSourceKind::Mod => 1,
    }
}

fn cmp_lowercase(left: &str, right: &str) -> Ordering {
    left.to_lowercase().cmp(&right.to_lowercase())
}

fn bool_true_first(left: bool, right: bool) -> Ordering {
    right.cmp(&left)
}

fn handle_hub_key(state: &mut SettingsState, code: KeyCode) -> SettingsAction {
    let item_count = 5;
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
        KeyCode::Char('5') => state.hub_selected = 4,
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
                state.page = SettingsPage::Keybind;
                state.refresh_keybind_games();
            }
            4 => {
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

fn handle_keybind_key(state: &mut SettingsState, key: KeyEvent) {
    let code = key.code;
    if let Some(capture) = state.keybind_capture.clone() {
        if Instant::now() < capture.accept_after {
            return;
        }
        if let Some(bound_key) = capture_key_name(key) {
            apply_keybind_to_selected_game(state, capture.slot_index, bound_key);
            state.keybind_capture = None;
        }
        return;
    }

    if let Some(input) = state.keybind_page_jump_input.as_mut() {
        let total_pages = total_keybind_pages(state.keybind_games.len(), current_keybind_page_size());
        match code {
            KeyCode::Esc => state.keybind_page_jump_input = None,
            KeyCode::Backspace => {
                input.pop();
            }
            KeyCode::Char(ch) if ch.is_ascii_digit() => {
                if input.len() < 4 {
                    input.push(ch);
                }
            }
            KeyCode::Enter => {
                if let Ok(page) = input.parse::<usize>()
                    && (1..=total_pages.max(1)).contains(&page)
                {
                    state.keybind_page = page - 1;
                    let start = state.keybind_page * current_keybind_page_size();
                    state.keybind_selected =
                        start.min(state.keybind_games.len().saturating_sub(1));
                }
                state.keybind_page_jump_input = None;
            }
            _ => {}
        }
        return;
    }

    let page_size = current_keybind_page_size();
    match state.keybind_focus {
        KeybindFocus::Games => match code {
            KeyCode::Up | KeyCode::Char('k') => {
                state.keybind_selected = state.keybind_selected.saturating_sub(1);
                state.keybind_page = state.keybind_selected / page_size.max(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if !state.keybind_games.is_empty() {
                    state.keybind_selected = (state.keybind_selected + 1)
                        .min(state.keybind_games.len().saturating_sub(1));
                    state.keybind_page = state.keybind_selected / page_size.max(1);
                }
            }
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                if state.keybind_page > 0 {
                    state.keybind_page -= 1;
                    let start = state.keybind_page * page_size.max(1);
                    state.keybind_selected =
                        start.min(state.keybind_games.len().saturating_sub(1));
                } else if keybind_all_games_valid(state) {
                    content_cache::reload();
                    state.refresh_keybind_games();
                    state.page = SettingsPage::Hub;
                }
            }
            KeyCode::Char('e') | KeyCode::Char('E') => {
                let total_pages = total_keybind_pages(state.keybind_games.len(), page_size);
                if state.keybind_page + 1 < total_pages {
                    state.keybind_page += 1;
                    let start = state.keybind_page * page_size.max(1);
                    state.keybind_selected = start.min(state.keybind_games.len().saturating_sub(1));
                }
            }
            KeyCode::Char('p') | KeyCode::Char('P') => {
                if total_keybind_pages(state.keybind_games.len(), page_size) > 1 {
                    state.keybind_page_jump_input = Some(String::new());
                }
            }
            KeyCode::Char('z') | KeyCode::Char('Z') => {
                let next = match state.keybind_sort_mode {
                    KeybindGameSortMode::Source => KeybindGameSortMode::Name,
                    KeybindGameSortMode::Name => KeybindGameSortMode::Author,
                    KeybindGameSortMode::Author => KeybindGameSortMode::Source,
                };
                state.set_keybind_sort_mode(next);
            }
            KeyCode::Char('x') | KeyCode::Char('X') => {
                state.toggle_keybind_sort_order();
            }
            KeyCode::Enter => {
                state.keybind_focus = KeybindFocus::Actions;
                state.keybind_action_selected = 0;
                state.keybind_action_scroll = 0;
            }
            KeyCode::Esc => {
                if keybind_all_games_valid(state) {
                    content_cache::reload();
                    state.refresh_keybind_games();
                    state.page = SettingsPage::Hub;
                }
            }
            _ => {}
        },
        KeybindFocus::Actions => match code {
            KeyCode::Up | KeyCode::Char('k') => {
                state.keybind_action_selected = state.keybind_action_selected.saturating_sub(1);
                sync_keybind_action_view(state, 0);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let max_index = selected_keybind_game(state)
                    .map(keybind_action_count)
                    .unwrap_or(1)
                    .saturating_sub(1);
                state.keybind_action_selected = (state.keybind_action_selected + 1).min(max_index);
                sync_keybind_action_view(state, 0);
            }
            KeyCode::Char('w') | KeyCode::Char('W') => {
                state.keybind_action_scroll = state.keybind_action_scroll.saturating_sub(1);
            }
            KeyCode::Char('s') | KeyCode::Char('S') => {
                state.keybind_action_scroll = state.keybind_action_scroll.saturating_add(1);
                sync_keybind_action_view(state, 0);
            }
            KeyCode::Char(ch) if ('1'..='5').contains(&ch) => {
                let slot_index = (ch as u8 - b'1') as usize;
                match state.keybind_edit_mode {
                    KeybindEditMode::Add => {
                        semantic_key_source().clear_pending_keys();
                        state.keybind_capture = Some(KeybindCaptureState {
                            slot_index,
                            accept_after: Instant::now() + KEYBIND_CAPTURE_DELAY,
                        });
                    }
                    KeybindEditMode::Delete => {
                        delete_keybind_slot(state, slot_index);
                    }
                }
            }
            KeyCode::Char('z') | KeyCode::Char('Z') => {
                reset_selected_action_keybind(state);
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                reset_selected_game_keybinds(state);
            }
            KeyCode::Char('x') | KeyCode::Char('X') => {
                state.keybind_edit_mode = match state.keybind_edit_mode {
                    KeybindEditMode::Add => KeybindEditMode::Delete,
                    KeybindEditMode::Delete => KeybindEditMode::Add,
                };
            }
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
                state.keybind_focus = KeybindFocus::Games;
                state.keybind_capture = None;
            }
            _ => {}
        },
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
    let label_keybind = text("settings.hub.keybind", "Keybinding");
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
        UnicodeWidthStr::width(format!("{}[4] {}", TRIANGLE, label_keybind).as_str()),
        UnicodeWidthStr::width(format!("{}{} {}", TRIANGLE, enter_key, label_keybind).as_str()),
        UnicodeWidthStr::width(format!("{}[5] {}", TRIANGLE, label_memory).as_str()),
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
    let hint = build_language_hint_segments_for_code("");
    let hint_width = UnicodeWidthStr::width(hint.join("  ").as_str()) as u16;

    let min_w = grid_width.max(hint_width).max(30) + 2;
    let min_h = 1 + 1 + grid_height + 1 + 2;
    (min_w, min_h.max(10))
}

fn build_language_hint_segments_for_code(code: &str) -> Vec<String> {
    let translate = |key: &str, fallback: &str| {
        if code.is_empty() {
            text(key, fallback)
        } else {
            i18n::t_for_code(code, key)
        }
    };

    vec![
        translate("language.hint.segment.confirm", "[Enter] Confirm language"),
        translate("language.hint.segment.back", "[ESC]/[Q] Return to main menu"),
    ]
}

fn wrap_language_hint_lines(segments: &[String], width: usize) -> Vec<Line<'static>> {
    if width == 0 || segments.is_empty() {
        return vec![Line::from("")];
    }

    let mut lines = Vec::new();
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

fn minimum_size_keybind(state: &SettingsState) -> (u16, u16) {
    let max_name_w = state
        .keybind_games
        .iter()
        .map(|game| UnicodeWidthStr::width(game.display_name.as_str()))
        .max()
        .unwrap_or(12) as u16;
    let left_w = (max_name_w + 6).max(22);
    let right_w = 72u16;
    (left_w + right_w + 4, 16)
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
        ("[4]", text("settings.hub.keybind", "Keybinding")),
        ("[5]", text("settings.hub.memory", "Memory Cleanup")),
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
    let operation_hint_width = UnicodeWidthStr::width(
        text(
            "settings.hub.operation_hint",
            "[↑]/[↓] Select Option  [Enter] Confirm",
        )
        .as_str(),
    ) as u16;
    let back_hint_width = UnicodeWidthStr::width(
        text("settings.hub.back_hint", "[ESC]/[Q] Return to main menu").as_str(),
    ) as u16;

    let width = area
        .width
        .saturating_sub(2)
        .max(1)
        .min(content_width.max(operation_hint_width).max(back_hint_width).max(1));
    let height = (items.len() + 3) as u16;
    let menu_area = Rect {
        x: area.x + area.width.saturating_sub(width) / 2,
        y: area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    };

    let item_area = Rect {
        x: menu_area.x,
        y: menu_area.y,
        width: menu_area.width,
        height: items.len() as u16,
    };
    let hint_area = Rect {
        x: menu_area.x,
        y: menu_area.y + items.len() as u16 + 1,
        width: menu_area.width,
        height: 2,
    };

    let left_pad = item_area.width.saturating_sub(content_width) / 2;
    let mut item_lines = Vec::new();

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

        item_lines.push(Line::from(vec![
            Span::raw(" ".repeat(left_pad as usize)),
            Span::styled(if is_selected { TRIANGLE } else { "  " }, base_style),
            Span::styled(key.to_string(), key_style),
            Span::styled(format!(" {}", text), base_style),
        ]));
    }

    let item_widget = Paragraph::new(item_lines).alignment(Alignment::Left);
    frame.render_widget(item_widget, item_area);

    let hint_lines = vec![
        Line::from(Span::styled(
            text(
                "settings.hub.operation_hint",
                "[↑]/[↓] Select Option  [Enter] Confirm",
            ),
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(Span::styled(
            text("settings.hub.back_hint", "[ESC]/[Q] Return to main menu"),
            Style::default().fg(Color::DarkGray),
        )),
    ];
    let hint_widget = Paragraph::new(hint_lines).alignment(Alignment::Center);
    frame.render_widget(hint_widget, hint_area);
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
            "settings.security.enabled",
            "settings.security.disabled",
            "Enabled",
            "Disabled",
        ),
        selection_option_with_value_line(
            1,
            state.security_selected,
            text(
                "settings.security.default_enabled",
                "Default mod enabled state",
            ),
            state.default_mod_enabled,
            "settings.security.mod_enabled",
            "settings.security.mod_disabled",
            "Enabled",
            "Disabled",
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
    render_box_hint_line(
        frame,
        rect,
        2,
        text(
            "settings.security.operation_hint",
            "[↑]/[↓] Select Option  [Enter] Confirm/Toggle Option",
        ),
    );
    render_box_hint_line(
        frame,
        rect,
        3,
        text(
            "settings.secondary.back_hint",
            "[ESC]/[Q] Return to main menu",
        ),
    );
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
    render_box_hint_line(
        frame,
        rect,
        2,
        text(
            "settings.memory.operation_hint",
            "[↑]/[↓] Select Option  [Enter] Confirm",
        ),
    );
    render_box_hint_line(
        frame,
        rect,
        3,
        text(
            "settings.secondary.back_hint",
            "[ESC]/[Q] Return to main menu",
        ),
    );
}

fn render_keybind(frame: &mut ratatui::Frame<'_>, state: &mut SettingsState) {
    if state.keybind_games.is_empty() {
        state.keybind_selected = 0;
        state.keybind_page = 0;
    } else {
        let page_size = current_keybind_page_size();
        let total_pages = total_keybind_pages(state.keybind_games.len(), page_size);
        if state.keybind_page >= total_pages {
            state.keybind_page = total_pages.saturating_sub(1);
        }
        let page_start = state.keybind_page * page_size;
        let page_end = (page_start + page_size).min(state.keybind_games.len());
        if state.keybind_selected < page_start || state.keybind_selected >= page_end {
            state.keybind_selected = page_start.min(state.keybind_games.len().saturating_sub(1));
        }
    }

    let area = frame.area();
    let hint_segments = build_keybind_hint_segments(state);
    let hint_lines = wrap_keybind_hint_lines(&hint_segments, area.width.max(1) as usize);
    let hint_height = hint_lines.len().max(1) as u16;
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(hint_height)])
        .split(area);
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(root[0]);

    render_keybind_game_list(frame, columns[0], state);
    render_keybind_mapping_panel(frame, columns[1], state);
    frame.render_widget(
        Paragraph::new(hint_lines)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center),
        root[1],
    );
}

fn build_keybind_hint_segments(state: &SettingsState) -> Vec<String> {
    if state.keybind_capture.is_some() {
        return vec![
            text(
                "settings.keybind.hint.segment.capture_any",
                "[Any Key] Bind to this slot",
            ),
            text(
                "settings.keybind.hint.segment.capture_shift",
                "[Shift] Hold for 2s to bind Shift",
            ),
        ];
    }

    if state.keybind_focus == KeybindFocus::Actions {
        let mut segments = vec![
            text("settings.keybind.hint.segment.move", "[↑]/[↓] Move"),
            text("settings.keybind.hint.segment.scroll", "[W]/[S] Scroll"),
            text(
                "settings.keybind.hint.segment.reset_action",
                "[Z] Reset Action",
            ),
            text(
                "settings.keybind.hint.segment.reset_game",
                "[R] Reset Current Game",
            ),
            text(
                "settings.keybind.hint.segment.toggle_mode",
                "[X] Toggle Mode",
            ),
        ];
        segments.push(match state.keybind_edit_mode {
            KeybindEditMode::Add => text(
                "settings.keybind.hint.segment.add_key",
                "[1]/[2]/[3]/[4]/[5] Add/Rebind Key",
            ),
            KeybindEditMode::Delete => text(
                "settings.keybind.hint.segment.delete_key",
                "[1]/[2]/[3]/[4]/[5] Delete Key",
            ),
        });
        return segments;
    }

    vec![
        text("settings.keybind.hint.segment.move", "[↑]/[↓] Move"),
        text("settings.keybind.hint.segment.page", "[Q]/[E] Page"),
        text("settings.keybind.hint.segment.jump", "[P] Jump Page"),
        text("settings.keybind.hint.segment.sort_mode", "[Z] Sort Mode"),
        text("settings.keybind.hint.segment.sort_order", "[X] Sort Order"),
        text("settings.keybind.hint.segment.select", "[Enter] Select"),
        text(
            "settings.keybind.hint.segment.save_exit",
            "[Esc]/[Q] Save and Exit",
        ),
    ]
}

fn wrap_keybind_hint_lines(segments: &[String], width: usize) -> Vec<Line<'static>> {
    if width == 0 || segments.is_empty() {
        return vec![Line::from("")];
    }

    let mut lines = Vec::new();
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
    let hint_lines = wrap_language_hint_lines(
        &build_language_hint_segments_for_code(&selected_pack.code),
        sections[3].width.max(1) as usize,
    );

    let title_widget = Paragraph::new(title)
        .style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);
    frame.render_widget(title_widget, sections[0]);

    draw_language_grid(frame.buffer_mut(), sections[2], &languages, selected_idx);

    let hint_widget = Paragraph::new(hint_lines)
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

fn render_box_hint_line(frame: &mut ratatui::Frame<'_>, rect: Rect, offset: u16, message: String) {
    let y = rect.y.saturating_add(rect.height).saturating_add(offset);
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
    enabled_key: &str,
    disabled_key: &str,
    enabled_fallback: &str,
    disabled_fallback: &str,
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
        text(enabled_key, enabled_fallback)
    } else {
        text(disabled_key, disabled_fallback)
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
    clear_game_debug_logs()?;
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

fn clear_game_debug_logs() -> anyhow::Result<()> {
    let log_dir = path_utils::log_dir()?;
    if !log_dir.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(log_dir)? {
        let entry = entry?;
        let target = entry.path();
        if target.is_file()
            && target
                .file_name()
                .and_then(|name| name.to_str())
                .map(|name| name != "tui_log.txt")
                .unwrap_or(false)
        {
            fs::remove_file(target)?;
        }
    }
    Ok(())
}

fn load_keybind_games() -> Vec<GameDescriptor> {
    content_cache::games()
}

fn current_keybind_page_size() -> usize {
    let (_, height) = crossterm::terminal::size().unwrap_or((100, 24));
    height.saturating_sub(5).max(1) as usize
}

fn total_keybind_pages(total_items: usize, page_size: usize) -> usize {
    if total_items == 0 {
        1
    } else {
        ((total_items + page_size.saturating_sub(1)) / page_size).max(1)
    }
}

fn keybind_game_list_title(state: &SettingsState) -> Line<'static> {
    let order_text = if state.keybind_sort_descending {
        format!("\u{2191}{}", text("settings.mods.order.desc", "Descending"))
    } else {
        format!("\u{2193}{}", text("settings.mods.order.asc", "Ascending"))
    };

    Line::from(vec![
        Span::raw(" "),
        Span::styled(
            text("settings.keybind.games_title", "Game Selection"),
            Style::default().fg(Color::White),
        ),
        Span::styled(" *", Style::default().fg(Color::White)),
        Span::styled(
            keybind_game_sort_label(state.keybind_sort_mode),
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

fn keybind_game_sort_label(mode: KeybindGameSortMode) -> String {
    match mode {
        KeybindGameSortMode::Source => text("game_selection.sort.source", "Official & Mods"),
        KeybindGameSortMode::Name => text("game_selection.sort.name", "Name"),
        KeybindGameSortMode::Author => text("game_selection.sort.author", "Author"),
    }
}

fn keybind_game_list_line(game: &GameDescriptor, width: usize, style: Style) -> Line<'static> {
    if width == 0 {
        return Line::from("");
    }

    let name = game.display_name.clone();
    if !game.is_mod_game() {
        return Line::from(Span::styled(
            truncate_with_ellipsis_plain(&name, width),
            style,
        ));
    }

    let badge = text("mods.badge", "MOD");
    let badge_width = UnicodeWidthStr::width(badge.as_str());
    if width <= badge_width + 1 {
        return Line::from(Span::styled(
            truncate_with_ellipsis_plain(&name, width),
            style,
        ));
    }

    let left_width = width - badge_width - 1;
    let left = truncate_with_ellipsis_plain(&name, left_width);
    let pad = width.saturating_sub(UnicodeWidthStr::width(left.as_str()) + badge_width);
    let badge_fg = if matches!(style.bg, Some(Color::LightBlue)) {
        Color::Black
    } else {
        Color::Yellow
    };

    Line::from(vec![
        Span::styled(left, style),
        Span::styled(" ".repeat(pad), style),
        Span::styled(
            badge,
            Style::default()
                .fg(badge_fg)
                .bg(style.bg.unwrap_or(Color::Reset))
                .add_modifier(Modifier::BOLD),
        ),
    ])
}

fn render_keybind_game_list(frame: &mut ratatui::Frame<'_>, area: Rect, state: &SettingsState) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(keybind_game_list_title(state))
        .border_style(Style::default().fg(Color::White));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(inner);

    if state.keybind_games.is_empty() {
        frame.render_widget(
            Paragraph::new(i18n::t("game_selection.empty"))
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

    let page_size = rows[0].height.max(1) as usize;
    let total_pages = total_keybind_pages(state.keybind_games.len(), page_size);
    let page = state.keybind_page.min(total_pages.saturating_sub(1));
    let start = page * page_size;
    let page_games = state
        .keybind_games
        .iter()
        .skip(start)
        .take(page_size)
        .collect::<Vec<_>>();

    for (index, game) in page_games.iter().enumerate() {
        let y = rows[0].y + index as u16;
        if y >= rows[0].y + rows[0].height {
            break;
        }
        let selected = start + index == state.keybind_selected;
        let invalid = game_has_missing_keys(game);
        if selected {
            let buffer = frame.buffer_mut();
            fill_buffer_row(
                buffer,
                rows[0].x,
                y,
                rows[0].width,
                Style::default().bg(Color::LightBlue),
            );
        }
        let style = if selected && invalid {
            Style::default().fg(Color::Red).bg(Color::LightBlue)
        } else if selected {
            Style::default().fg(Color::Black).bg(Color::LightBlue)
        } else if invalid {
            Style::default().fg(Color::Red)
        } else {
            Style::default().fg(Color::White)
        };
        frame.render_widget(
            Paragraph::new(keybind_game_list_line(game, rows[0].width as usize, style)),
            Rect::new(rows[0].x, y, rows[0].width, 1),
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
    let pager_line = if let Some(input) = &state.keybind_page_jump_input {
        let input_text = if input.is_empty() {
            "_".to_string()
        } else {
            input.clone()
        };
        Line::from(vec![
            Span::styled(
                input_text,
                Style::default()
                    .fg(if input.is_empty() { Color::Yellow } else { Color::Black })
                    .bg(Color::Yellow),
            ),
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

fn render_keybind_mapping_panel(
    frame: &mut ratatui::Frame<'_>,
    area: Rect,
    state: &mut SettingsState,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(keybind_mapping_title(state.keybind_games.get(state.keybind_selected)))
        .border_style(Style::default().fg(Color::White));
    let inner = block.inner(area);
    let viewport_h = inner.height.saturating_sub(2) as usize;
    sync_keybind_action_view(state, viewport_h);
    let selected_game = state.keybind_games.get(state.keybind_selected);
    frame.render_widget(block, area);

    let Some(game) = selected_game else {
        frame.render_widget(
            Paragraph::new(i18n::t("game_selection.empty"))
                .style(Style::default().fg(Color::White))
                .alignment(Alignment::Center),
            inner,
        );
        return;
    };

    let action_col_width = ((inner.width as f32) * 0.20).floor() as u16;
    let key_col_width = ((inner.width as f32) * 0.16).floor() as u16;
    let header_y = inner.y;
    let data_y = inner.y + 2;
    let mut x = inner.x;
    render_cell_text(
        frame.buffer_mut(),
        x.saturating_add(KEYBIND_ACTION_PADDING),
        header_y,
        action_col_width.saturating_sub(KEYBIND_ACTION_PADDING),
        &text("settings.keybind.action", "Action"),
        Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
    );
    x += action_col_width;
    for slot in 0..5 {
        render_cell_text(
            frame.buffer_mut(),
            x,
            header_y,
            key_col_width,
            &format!(
                "[{}] {}",
                slot + 1,
                text("settings.keybind.key", "Key")
            ),
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        );
        x += key_col_width;
    }
    frame.buffer_mut().set_string(
        inner.x,
        inner.y + 1,
        "─".repeat(inner.width.max(1) as usize),
        Style::default().fg(Color::White),
    );

    let actions = keybind_action_rows(game);
    let max_scroll = actions.len().saturating_sub(viewport_h);
    let scroll = state.keybind_action_scroll.min(max_scroll);
    let selected_row = state.keybind_action_selected.min(actions.len().saturating_sub(1));

    for (visible_idx, (action_name, binding_key, binding)) in actions
        .iter()
        .skip(scroll)
        .take(viewport_h)
        .enumerate()
    {
        let y = data_y + visible_idx as u16;
        let selected = state.keybind_focus == KeybindFocus::Actions && selected_row == scroll + visible_idx;
        let slots = binding.slots();
        let missing = slots.iter().all(|slot| slot.trim().is_empty());
        if selected {
            fill_buffer_row(
                frame.buffer_mut(),
                inner.x,
                y,
                inner.width,
                if state.keybind_edit_mode == KeybindEditMode::Delete {
                    Style::default().bg(Color::LightRed)
                } else {
                    Style::default().bg(Color::LightBlue)
                },
            );
        }
        let row_style = if selected {
            let bg = if state.keybind_edit_mode == KeybindEditMode::Delete {
                Color::LightRed
            } else {
                Color::LightBlue
            };
            Style::default().fg(Color::Black).bg(bg)
        } else {
            Style::default().fg(Color::White)
        };
        let action_style = if selected {
            row_style
        } else {
            Style::default().fg(Color::White)
        };
        if missing {
            frame.buffer_mut().set_string(inner.x, y, " ", Style::default().bg(Color::Red));
            frame
                .buffer_mut()
                .set_string(inner.x + inner.width.saturating_sub(1), y, " ", Style::default().bg(Color::Red));
        }
        render_cell_text(
            frame.buffer_mut(),
            inner.x.saturating_add(KEYBIND_ACTION_PADDING),
            y,
            action_col_width.saturating_sub(KEYBIND_ACTION_PADDING),
            action_name,
            action_style,
        );
        let mut x = inner.x + action_col_width;
        for slot in 0..5 {
            let value = slots.get(slot).cloned().unwrap_or_default();
            let formatted = if value.trim().is_empty() {
                String::new()
            } else {
                display_semantic_key(&value, game.case_sensitive)
            };
            render_key_slot(
                frame.buffer_mut(),
                x,
                y,
                key_col_width,
                formatted.as_str(),
                row_style,
            );
            x += key_col_width;
        }
        let _ = binding_key;
    }
}

fn keybind_mapping_title(selected_game: Option<&GameDescriptor>) -> Line<'static> {
    let mut spans = vec![Span::styled(
        format!("── {}: ", text("settings.keybind.mapping_title", "Key Mapping")),
        Style::default().fg(Color::White),
    )];
    if let Some(game) = selected_game {
        spans.push(Span::styled(
            game.display_name.clone(),
            Style::default().fg(Color::White),
        ));
        if game.case_sensitive {
            spans.push(Span::raw(" "));
            spans.push(Span::styled(
                text(
                    "settings.keybind.case_sensitive_hint",
                    "Letter keys are case-sensitive",
                ),
                Style::default().fg(Color::Yellow),
            ));
        }
        spans.push(Span::raw(" "));
    }
    Line::from(spans)
}

fn localized_action_name(game: &GameDescriptor, binding: &crate::game::action::ActionBinding) -> String {
    if let Some(package) = game.package_info() {
        resources::resolve_package_text(package, binding.key_name())
    } else {
        binding.key_name().to_string()
    }
}

fn keybind_action_rows(game: &GameDescriptor) -> Vec<(String, String, ActionBinding)> {
    game.actions
        .iter()
        .map(|(binding_key, binding)| {
            (
                localized_action_name(game, binding),
                binding_key.clone(),
                binding.clone(),
            )
        })
        .collect()
}

fn keybind_action_count(game: &GameDescriptor) -> usize {
    game.actions.len().max(1)
}

fn selected_keybind_game(state: &SettingsState) -> Option<&GameDescriptor> {
    state.keybind_games.get(state.keybind_selected)
}

fn selected_keybind_game_mut(state: &mut SettingsState) -> Option<&mut GameDescriptor> {
    state.keybind_games.get_mut(state.keybind_selected)
}

fn game_has_missing_keys(game: &GameDescriptor) -> bool {
    game.actions.values().any(|binding| binding.keys().is_empty())
}

fn keybind_all_games_valid(state: &SettingsState) -> bool {
    state.keybind_games.iter().all(|game| !game_has_missing_keys(game))
}

fn selected_action_binding_key(state: &SettingsState) -> Option<String> {
    let selected_index = state.keybind_action_selected;
    selected_keybind_game(state).and_then(|game| {
        keybind_action_rows(game)
            .get(selected_index)
            .map(|(_, binding_key, _)| binding_key.clone())
    })
}

fn sync_keybind_action_view(state: &mut SettingsState, viewport_h: usize) {
    let action_count = selected_keybind_game(state)
        .map(keybind_action_count)
        .unwrap_or(1);
    state.keybind_action_selected = state.keybind_action_selected.min(action_count.saturating_sub(1));
    if viewport_h == 0 {
        return;
    }
    if state.keybind_action_selected < state.keybind_action_scroll {
        state.keybind_action_scroll = state.keybind_action_selected;
    } else if state.keybind_action_selected >= state.keybind_action_scroll + viewport_h {
        state.keybind_action_scroll = state
            .keybind_action_selected
            .saturating_sub(viewport_h.saturating_sub(1));
    }
}

fn render_cell_text(
    buffer: &mut Buffer,
    x: u16,
    y: u16,
    width: u16,
    text: &str,
    style: Style,
) {
    if width == 0 {
        return;
    }
    buffer.set_stringn(x, y, text, width as usize, style);
}

fn fill_buffer_row(buffer: &mut Buffer, x: u16, y: u16, width: u16, style: Style) {
    if width == 0 {
        return;
    }
    buffer.set_string(x, y, " ".repeat(width as usize), style);
}

fn render_key_slot(
    buffer: &mut Buffer,
    x: u16,
    y: u16,
    width: u16,
    value: &str,
    row_style: Style,
) {
    if width == 0 {
        return;
    }
    let bracket_style = Style::default().fg(Color::White).bg(row_style.bg.unwrap_or(Color::Reset));
    let value_style = Style::default()
        .fg(row_style.fg.unwrap_or(Color::White))
        .bg(row_style.bg.unwrap_or(Color::Reset));
    let value_width = UnicodeWidthStr::width(value);
    let slot_width = if value.is_empty() {
        2
    } else {
        value_width.saturating_add(4)
    };
    let slot_width = slot_width.min(width as usize) as u16;
    let end_x = x + slot_width.saturating_sub(1);

    buffer.set_string(x, y, "[", bracket_style);
    if !value.is_empty() {
        let value_limit = slot_width.saturating_sub(3) as usize;
        buffer.set_stringn(x + 2, y, value, value_limit, value_style);
    }
    buffer.set_string(end_x, y, "]", bracket_style);
}

fn capture_key_name(key: KeyEvent) -> Option<String> {
    if !matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat) {
        return None;
    }
    let mut names = semantic_key_source().record_crossterm_key(key);
    if names.is_empty() {
        names = semantic_key_source().drain_ready_rdev_keys(4);
    }
    names
        .into_iter()
        .find(|name| !matches!(name.as_str(), "left_shift" | "right_shift"))
}

fn normalize_bound_key_name(value: String, case_sensitive: bool) -> String {
    match value.as_str() {
        "left_shift" | "right_shift" => "shift".to_string(),
        _ if case_sensitive => value,
        _ => value.to_lowercase(),
    }
}

fn key_names_conflict(left: &str, right: &str, case_sensitive: bool) -> bool {
    if case_sensitive {
        left == right
    } else {
        left.eq_ignore_ascii_case(right)
    }
}

fn compact_key_slots(slots: Vec<String>, case_sensitive: bool) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for slot in slots.into_iter().filter(|slot| !slot.trim().is_empty()) {
        if out
            .iter()
            .any(|existing| key_names_conflict(existing, &slot, case_sensitive))
        {
            continue;
        }
        out.push(slot);
        if out.len() >= 5 {
            break;
        }
    }
    out
}

fn set_binding_slots(binding: &mut ActionBinding, slots: Vec<String>, case_sensitive: bool) {
    let slots = compact_key_slots(slots, case_sensitive);
    binding.key = match slots.len() {
        0 => ActionKeys::Multiple(Vec::new()),
        1 => ActionKeys::Single(slots[0].clone()),
        _ => ActionKeys::Multiple(slots),
    };
}

fn apply_binding_slots_to_game(
    game: &mut GameDescriptor,
    binding_key: &str,
    slots: Vec<String>,
    remove_conflicts: bool,
) {
    let normalized_slots = compact_key_slots(slots, game.case_sensitive);
    if remove_conflicts {
        let conflict_keys = normalized_slots.clone();
        for (other_key, other_binding) in &mut game.actions {
            if other_key == binding_key {
                continue;
            }
            let mut other_slots = other_binding.slots();
            let mut changed = false;
            for slot in &mut other_slots {
                if conflict_keys
                    .iter()
                    .any(|key| key_names_conflict(slot, key, game.case_sensitive))
                {
                    slot.clear();
                    changed = true;
                }
            }
            if changed {
                set_binding_slots(other_binding, other_slots, game.case_sensitive);
            }
        }
    }
    if let Some(binding) = game.actions.get_mut(binding_key) {
        set_binding_slots(binding, normalized_slots, game.case_sensitive);
    }
}

fn apply_keybind_to_selected_game(state: &mut SettingsState, slot_index: usize, key_name: String) {
    let Some(binding_key) = selected_action_binding_key(state) else {
        return;
    };
    let Some(game) = selected_keybind_game_mut(state) else {
        return;
    };

    let mut slots = game
        .actions
        .get(binding_key.as_str())
        .map(ActionBinding::slots)
        .unwrap_or_default();
    while slots.len() <= slot_index {
        slots.push(String::new());
    }
    slots[slot_index] = key_name;
    apply_binding_slots_to_game(game, binding_key.as_str(), slots, true);

    persist_selected_game_keybindings(game);
}

fn delete_keybind_slot(state: &mut SettingsState, slot_index: usize) {
    let Some(binding_key) = selected_action_binding_key(state) else {
        return;
    };
    let Some(game) = selected_keybind_game_mut(state) else {
        return;
    };
    if let Some(binding) = game.actions.get_mut(binding_key.as_str()) {
        let mut slots = binding.slots();
        if slot_index < slots.len() {
            slots[slot_index].clear();
            set_binding_slots(binding, slots, game.case_sensitive);
            persist_selected_game_keybindings(game);
        }
    }
}

fn reset_selected_action_keybind(state: &mut SettingsState) {
    let Some(binding_key) = selected_action_binding_key(state) else {
        return;
    };
    let Some(game) = selected_keybind_game_mut(state) else {
        return;
    };
    if let Some(default_binding) = game.default_actions.get(binding_key.as_str()).cloned() {
        apply_binding_slots_to_game(game, binding_key.as_str(), default_binding.slots(), true);
        persist_selected_game_keybindings(game);
    }
}

fn reset_selected_game_keybinds(state: &mut SettingsState) {
    let Some(game) = selected_keybind_game_mut(state) else {
        return;
    };
    game.actions = game.default_actions.clone();
    persist_selected_game_keybindings(game);
}

fn persist_selected_game_keybindings(game: &GameDescriptor) {
    let bindings = game
        .actions
        .iter()
        .map(|(action, binding)| (action.clone(), binding.slots()))
        .collect::<std::collections::HashMap<_, _>>();
    if let Some(package) = game.package_info()
        && game.is_mod_game()
    {
        let _ = mods::update_mod_keybindings(
            package.namespace.as_str(),
            game.id.as_str(),
            game.entry.as_str(),
            bindings,
        );
    } else {
        let _ = runtime_save::save_keybindings(game.id.as_str(), &bindings);
    }
}

fn poll_keybind_capture(state: &mut SettingsState) -> bool {
    if state.page != SettingsPage::Keybind || state.keybind_capture.is_none() {
        return false;
    }
    if state
        .keybind_capture
        .as_ref()
        .map(|capture| Instant::now() < capture.accept_after)
        .unwrap_or(false)
    {
        return false;
    }
    if semantic_key_source().is_shift_held_for(SHIFT_BIND_HOLD) {
        let slot_index = state
            .keybind_capture
            .as_ref()
            .map(|capture| capture.slot_index)
            .unwrap_or(0);
        apply_keybind_to_selected_game(state, slot_index, "shift".to_string());
        state.keybind_capture = None;
        return true;
    }
    let keys = semantic_key_source().drain_ready_rdev_keys(4);
    if let Some(key_name) = keys
        .into_iter()
        .find(|key_name| !matches!(key_name.as_str(), "left_shift" | "right_shift"))
    {
        let slot_index = state.keybind_capture.as_ref().map(|capture| capture.slot_index).unwrap_or(0);
        let case_sensitive = selected_keybind_game(state)
            .map(|game| game.case_sensitive)
            .unwrap_or(false);
        apply_keybind_to_selected_game(
            state,
            slot_index,
            normalize_bound_key_name(key_name, case_sensitive),
        );
        state.keybind_capture = None;
        return true;
    }
    false
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
    let value = if enabled {
        text("settings.mods.simple_enabled", "On")
    } else {
        text("settings.mods.simple_disabled", "Off")
    };
    let value_style = Style::default()
        .fg(if enabled { Color::Green } else { Color::Red })
        .bg(bg)
        .add_modifier(Modifier::BOLD);
    let bracket_style = Style::default().fg(Color::White).bg(bg);

    buffer.set_string(x, y, "[", bracket_style);
    buffer.set_string(x + 1, y, &value, value_style);
    let value_width = UnicodeWidthStr::width(value.as_str()) as u16;
    buffer.set_string(x + 1 + value_width, y, "]", bracket_style);
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
