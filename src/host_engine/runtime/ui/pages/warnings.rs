//! Rust warning and confirmation pages.

use std::cell::Cell;
use std::time::{Duration, Instant};

use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::host_engine::boot::environment::data_dirs;
use crate::host_engine::boot::preload::lua_runtime::api::drawing_support::drawing_parser::STYLE_BOLD;
use crate::host_engine::runtime::memory_cleanup;
use crate::host_engine::runtime::ui::pages::common::{
    draw_title, is_press, key_hint, take_navigation, theme_color,
};
use crate::host_engine::runtime::ui::{Canvas, UiContext, UiEvent, UiNavigation, UiPage, UiResult};
use crate::host_engine::runtime::ui_page::page_key::UiPageKey;
use crate::host_engine::constant::{ROOT_UI_MIN_HEIGHT, ROOT_UI_MIN_WIDTH};
use crate::host_engine::runtime::ui_state::needed_size_state::NeededSizeMode;

const CONFIRM_DELAY: Duration = Duration::from_secs(5);
const DEFAULT_SECURITY_DELAY: Duration = Duration::from_secs(10);
const MOD_TEMPORARY_DELAY: Duration = Duration::from_secs(5);
const MOD_PERMANENT_DELAY: Duration = Duration::from_secs(10);
const WARN_WRAP_WIDTH: usize = 72;

// ---------------------------------------------------------------------------
// WarningNeededSizePage — standalone impl
// Matches official_ui/scripts/function/warning_needed_size/render.lua
// ---------------------------------------------------------------------------
// Matches official_ui/scripts/function/warning_needed_size/render.lua
// ---------------------------------------------------------------------------

pub struct WarningNeededSizePage {
    pending_navigation: Option<UiNavigation>,
}

impl WarningNeededSizePage {
    pub fn new() -> Self {
        Self {
            pending_navigation: None,
        }
    }
}

impl UiPage for WarningNeededSizePage {
    fn page_key(&self) -> UiPageKey {
        UiPageKey::WarningNeededSize
    }

    fn handle_event(&mut self, event: &UiEvent, _ctx: &mut UiContext) -> UiResult<()> {
        let (UiEvent::Action { name, status } | UiEvent::Key { name, status }) = event else {
            return Ok(());
        };
        if !is_press(status) {
            return Ok(());
        }
        if name == "return" {
            match _ctx.needed_size_mode {
                NeededSizeMode::Root => {
                    self.pending_navigation = Some(UiNavigation::Exit);
                }
                NeededSizeMode::Game => {
                    self.pending_navigation = Some(UiNavigation::Page(UiPageKey::GameList));
                }
            }
        }
        Ok(())
    }

    fn render(&self, canvas: &mut Canvas, ctx: &UiContext) -> UiResult<()> {
        canvas.clear()?;

        let needed_label = &ctx.i18n.warning.size_needed;
        let actual_label = &ctx.i18n.warning.size_actual;
        let hint = &ctx.i18n.warning.size_hint;
        let action_text = match ctx.needed_size_mode {
            NeededSizeMode::Root => &ctx.i18n.warning.size_action_exit,
            NeededSizeMode::Game => &ctx.i18n.warning.size_action_return,
        };
        let return_key = key_hint(ctx, "return", "Esc");

        // Line 1: Required terminal size: 98 x 26  (yellow, BOLD)
        let needed_line = format!("{needed_label}{ROOT_UI_MIN_WIDTH} x {ROOT_UI_MIN_HEIGHT}");
        // Line 3: Current terminal size: {w} x {h}  (label white BOLD, value cyan BOLD)
        let actual_value = format!("{} x {}", ctx.terminal_size.width, ctx.terminal_size.height);
        // Line 4: hint  (dark_gray, normal)
        // Line 6: [Esc] Exit the program  (dark_gray, normal)
        let action_line = format!("[{return_key}] {action_text}");

        let warn_color = theme_color(ctx, "text.warning", "yellow");
        let text_color = theme_color(ctx, "text.primary", "white");
        let value_color = theme_color(ctx, "accent.primary", "cyan");
        let hint_color = theme_color(ctx, "text.muted", "dark_gray");

        let content_height: u16 = 6;
        let top = ctx.terminal_size.height.saturating_sub(content_height) / 2;

        // Row 0: needed line (yellow, BOLD, centered)
        let needed_x = centered_x(ctx, &needed_line);
        canvas.draw_text_styled(
            needed_x,
            top,
            &needed_line,
            Some(warn_color),
            None,
            vec![STYLE_BOLD],
        )?;

        // Row 2: actual label (white BOLD) + actual value (cyan BOLD)
        let combined = format!("{actual_label}{actual_value}");
        let combined_x = centered_x(ctx, &combined);
        let label_width = UnicodeWidthStr::width(actual_label.as_str()) as u16;
        canvas.draw_text_styled(
            combined_x,
            top.saturating_add(2),
            actual_label,
            Some(text_color),
            None,
            vec![STYLE_BOLD],
        )?;
        canvas.draw_text_styled(
            combined_x.saturating_add(label_width),
            top.saturating_add(2),
            &actual_value,
            Some(value_color),
            None,
            vec![STYLE_BOLD],
        )?;

        // Row 3: hint (dark_gray, normal, centered)
        let hint_x = centered_x(ctx, hint);
        canvas.draw_text_styled(hint_x, top.saturating_add(3), hint, Some(hint_color.clone()), None, Vec::new())?;

        // Row 5: action line (dark_gray, normal, centered)
        let action_x = centered_x(ctx, &action_line);
        canvas.draw_text_styled(
            action_x,
            top.saturating_add(5),
            &action_line,
            Some(hint_color),
            None,
            Vec::new(),
        )?;

        Ok(())
    }

    fn take_navigation(&mut self) -> Option<UiNavigation> {
        take_navigation(&mut self.pending_navigation)
    }
}

pub struct WarningSecurityPage {
    started_at: Cell<Option<Instant>>,
    pending_navigation: Option<UiNavigation>,
}

impl WarningSecurityPage {
    pub fn new() -> Self {
        Self {
            started_at: Cell::new(None),
            pending_navigation: None,
        }
    }
}

impl UiPage for WarningSecurityPage {
    fn page_key(&self) -> UiPageKey {
        UiPageKey::WarningSecurity
    }

    fn handle_event(&mut self, event: &UiEvent, _ctx: &mut UiContext) -> UiResult<()> {
        let (UiEvent::Action { name, status } | UiEvent::Key { name, status }) = event else {
            return Ok(());
        };
        if !is_press(status) {
            return Ok(());
        }
        match name.as_str() {
            "close_permanent" | "confirm" | "enter" => {
                if remaining_for(&self.started_at, DEFAULT_SECURITY_DELAY) == 0 {
                    let mut profile =
                        crate::host_engine::boot::preload::persistent_data::security_profile::load_from_default_path();
                    profile.default_safe_mode = false;
                    crate::host_engine::boot::preload::persistent_data::security_profile::persist_to_default_path(&profile)?;
                    self.pending_navigation = Some(UiNavigation::Page(UiPageKey::SettingSecurity));
                }
            }
            _ => {
                self.pending_navigation = Some(UiNavigation::Page(UiPageKey::SettingSecurity));
            }
        }
        Ok(())
    }

    fn render(&self, canvas: &mut Canvas, ctx: &UiContext) -> UiResult<()> {
        self.started_at.set(Some(started_at(&self.started_at)));
        let remaining = remaining_for(&self.started_at, DEFAULT_SECURITY_DELAY);
        render_security_warning(
            canvas,
            ctx,
            &ctx.i18n.default_security.title,
            &ctx.i18n.default_security.warn,
            Vec::new(),
            vec![
                WarningRow::new(
                    format!(
                        "[{}] {}",
                        key_hint(ctx, "cancel", "N"),
                        ctx.i18n.key.default_security_cancel
                    ),
                    "state.success",
                    "green",
                ),
                WarningRow::new(
                    delay_action_text(
                        key_hint(ctx, "close_permanent", "Y"),
                        &ctx.i18n.key.default_security_close_permanent,
                        &ctx.i18n.default_security.second,
                        remaining,
                    ),
                    if remaining == 0 {
                        "accent.secondary"
                    } else {
                        "text.muted"
                    },
                    if remaining == 0 { "blue" } else { "dark_gray" },
                ),
            ],
        )
    }

    fn take_navigation(&mut self) -> Option<UiNavigation> {
        let navigation = take_navigation(&mut self.pending_navigation);
        if navigation.is_some() {
            self.started_at.set(None);
        }
        navigation
    }
}

pub struct WarningModPage {
    started_at: Cell<Option<Instant>>,
    pending_navigation: Option<UiNavigation>,
}

impl WarningModPage {
    pub fn new() -> Self {
        Self {
            started_at: Cell::new(None),
            pending_navigation: None,
        }
    }
}

impl UiPage for WarningModPage {
    fn page_key(&self) -> UiPageKey {
        UiPageKey::WarningMod
    }

    fn handle_event(&mut self, event: &UiEvent, _ctx: &mut UiContext) -> UiResult<()> {
        let (UiEvent::Action { name, status } | UiEvent::Key { name, status }) = event else {
            return Ok(());
        };
        if !is_press(status) {
            return Ok(());
        }
        match name.as_str() {
            "close_temporary" => {
                if remaining_for(&self.started_at, MOD_TEMPORARY_DELAY) == 0 {
                    self.pending_navigation = Some(UiNavigation::Page(UiPageKey::ModGameList));
                }
            }
            "close_permanent" => {
                if remaining_for(&self.started_at, MOD_PERMANENT_DELAY) == 0 {
                    self.pending_navigation = Some(UiNavigation::Page(UiPageKey::ModGameList));
                }
            }
            _ => {
                self.pending_navigation = Some(UiNavigation::Page(UiPageKey::ModGameList));
            }
        }
        Ok(())
    }

    fn render(&self, canvas: &mut Canvas, ctx: &UiContext) -> UiResult<()> {
        self.started_at.set(Some(started_at(&self.started_at)));
        let temporary_remaining = remaining_for(&self.started_at, MOD_TEMPORARY_DELAY);
        let permanent_remaining = remaining_for(&self.started_at, MOD_PERMANENT_DELAY);
        render_security_warning(
            canvas,
            ctx,
            &ctx.i18n.mod_security.title,
            &ctx.i18n.mod_security.warn,
            vec![if ctx.mod_warning_package_name.is_empty() {
                ctx.i18n.mod_security.mod_label.clone()
            } else {
                format!(
                    "{} {}",
                    ctx.i18n.mod_security.mod_label, ctx.mod_warning_package_name
                )
            }],
            vec![
                WarningRow::new(
                    format!(
                        "[{}] {}",
                        key_hint(ctx, "cancel", "N"),
                        ctx.i18n.key.mod_security_cancel
                    ),
                    "state.success",
                    "green",
                ),
                WarningRow::new(
                    delay_action_text(
                        key_hint(ctx, "close_temporary", "1"),
                        &ctx.i18n.key.mod_security_close_temporary,
                        &ctx.i18n.mod_security.second,
                        temporary_remaining,
                    ),
                    if temporary_remaining == 0 {
                        "state.danger"
                    } else {
                        "text.muted"
                    },
                    if temporary_remaining == 0 {
                        "red"
                    } else {
                        "dark_gray"
                    },
                ),
                WarningRow::new(
                    delay_action_text(
                        key_hint(ctx, "close_permanent", "2"),
                        &ctx.i18n.key.mod_security_close_permanent,
                        &ctx.i18n.mod_security.second,
                        permanent_remaining,
                    ),
                    if permanent_remaining == 0 {
                        "state.danger"
                    } else {
                        "text.muted"
                    },
                    if permanent_remaining == 0 {
                        "red"
                    } else {
                        "dark_gray"
                    },
                ),
            ],
        )
    }

    fn take_navigation(&mut self) -> Option<UiNavigation> {
        let navigation = take_navigation(&mut self.pending_navigation);
        if navigation.is_some() {
            self.started_at.set(None);
        }
        navigation
    }
}

pub struct WarningClearCachePage {
    started_at: Cell<Option<Instant>>,
    pending_navigation: Option<UiNavigation>,
}

impl WarningClearCachePage {
    pub fn new() -> Self {
        Self {
            started_at: Cell::new(None),
            pending_navigation: None,
        }
    }
}

impl UiPage for WarningClearCachePage {
    fn page_key(&self) -> UiPageKey {
        UiPageKey::WarningClearCache
    }

    fn handle_event(&mut self, event: &UiEvent, _ctx: &mut UiContext) -> UiResult<()> {
        handle_clear_warning_event(
            event,
            &self.started_at,
            &mut self.pending_navigation,
            memory_cleanup::clear_cache,
        )
    }

    fn render(&self, canvas: &mut Canvas, ctx: &UiContext) -> UiResult<()> {
        self.started_at.set(Some(started_at(&self.started_at)));
        let root = data_dirs::root_dir();
        let data = root.join("data");
        render_clear_warning(
            canvas,
            ctx,
            &ctx.i18n.clear_cache.title,
            &ctx.i18n.clear_cache.warn,
            vec![
                format!(
                    "{}{}",
                    ctx.i18n.clear_cache.cache_path,
                    data.join("cache").display()
                ),
                format!(
                    "{}{}",
                    ctx.i18n.clear_cache.log_path,
                    data.join("log").display()
                ),
            ],
            &ctx.i18n.key.clear_cache_cancel,
            &ctx.i18n.key.clear_cache_confirm,
            &ctx.i18n.clear_cache.second,
            remaining_seconds(&self.started_at),
        )
    }

    fn take_navigation(&mut self) -> Option<UiNavigation> {
        let navigation = take_navigation(&mut self.pending_navigation);
        if navigation.is_some() {
            self.started_at.set(None);
        }
        navigation
    }
}

pub struct WarningClearDataPage {
    started_at: Cell<Option<Instant>>,
    pending_navigation: Option<UiNavigation>,
}

impl WarningClearDataPage {
    pub fn new() -> Self {
        Self {
            started_at: Cell::new(None),
            pending_navigation: None,
        }
    }
}

impl UiPage for WarningClearDataPage {
    fn page_key(&self) -> UiPageKey {
        UiPageKey::WarningClearData
    }

    fn handle_event(&mut self, event: &UiEvent, _ctx: &mut UiContext) -> UiResult<()> {
        handle_clear_warning_event(
            event,
            &self.started_at,
            &mut self.pending_navigation,
            memory_cleanup::clear_data,
        )
    }

    fn render(&self, canvas: &mut Canvas, ctx: &UiContext) -> UiResult<()> {
        self.started_at.set(Some(started_at(&self.started_at)));
        let data = data_dirs::root_dir().join("data");
        render_clear_warning(
            canvas,
            ctx,
            &ctx.i18n.clear_data.title,
            &ctx.i18n.clear_data.warn,
            vec![format!("{}{}", ctx.i18n.clear_data.path, data.display())],
            &ctx.i18n.key.clear_data_cancel,
            &ctx.i18n.key.clear_data_confirm,
            &ctx.i18n.clear_data.second,
            remaining_seconds(&self.started_at),
        )
    }

    fn take_navigation(&mut self) -> Option<UiNavigation> {
        let navigation = take_navigation(&mut self.pending_navigation);
        if navigation.is_some() {
            self.started_at.set(None);
        }
        navigation
    }
}

fn handle_clear_warning_event(
    event: &UiEvent,
    started_at: &Cell<Option<Instant>>,
    pending_navigation: &mut Option<UiNavigation>,
    clear_operation: impl FnOnce() -> Result<(), Box<dyn std::error::Error>>,
) -> UiResult<()> {
    let (UiEvent::Action { name, status } | UiEvent::Key { name, status }) = event else {
        return Ok(());
    };
    if !is_press(status) {
        return Ok(());
    }

    match name.as_str() {
        "confirm" | "enter" => {
            if remaining_seconds(started_at) == 0 {
                clear_operation()?;
                *pending_navigation = Some(UiNavigation::Page(UiPageKey::SettingMemory));
            }
        }
        _ => {
            *pending_navigation = Some(UiNavigation::Page(UiPageKey::SettingMemory));
        }
    }
    Ok(())
}

fn render_clear_warning(
    canvas: &mut Canvas,
    ctx: &UiContext,
    title: &str,
    warn: &str,
    path_lines: Vec<String>,
    cancel_label: &str,
    confirm_label: &str,
    second_label: &str,
    remaining_seconds: u64,
) -> UiResult<()> {
    canvas.clear()?;
    draw_title(canvas, ctx, title)?;

    let mut rows = Vec::new();
    for line in wrap_text(warn, WARN_WRAP_WIDTH) {
        rows.push(WarningRow::new(line, "text.warning", "yellow"));
    }
    rows.push(WarningRow::blank());
    for line in path_lines {
        rows.push(WarningRow::new(line, "text.primary", "white"));
    }
    rows.push(WarningRow::blank());
    rows.push(WarningRow::new(
        format!("[{}] {}", key_hint(ctx, "cancel", "Esc"), cancel_label),
        "state.success",
        "green",
    ));
    let confirm_text = if remaining_seconds == 0 {
        format!("[{}] {}", key_hint(ctx, "confirm", "Enter"), confirm_label)
    } else {
        format!(
            "[{}] {} {}{}",
            key_hint(ctx, "confirm", "Enter"),
            confirm_label,
            remaining_seconds,
            second_label
        )
    };
    rows.push(WarningRow::new(
        confirm_text,
        if remaining_seconds == 0 {
            "state.danger"
        } else {
            "text.muted"
        },
        if remaining_seconds == 0 {
            "red"
        } else {
            "dark_gray"
        },
    ));

    let content_width = rows
        .iter()
        .map(|row| UnicodeWidthStr::width(row.text.as_str()))
        .max()
        .unwrap_or(0) as u16;
    let x = ctx.terminal_size.width.saturating_sub(content_width) / 2;
    let top = ctx.terminal_size.height.saturating_sub(rows.len() as u16) / 2;
    for (index, row) in rows.iter().enumerate() {
        if row.text.is_empty() {
            continue;
        }
        canvas.draw_text_styled(
            x,
            top.saturating_add(index as u16),
            row.text.as_str(),
            Some(theme_color(ctx, row.role, row.fallback)),
            None,
            vec![STYLE_BOLD],
        )?;
    }
    Ok(())
}

fn render_security_warning(
    canvas: &mut Canvas,
    ctx: &UiContext,
    title: &str,
    warn: &str,
    info_lines: Vec<String>,
    action_rows: Vec<WarningRow>,
) -> UiResult<()> {
    canvas.clear()?;
    draw_title(canvas, ctx, title)?;

    let mut rows = Vec::new();
    for line in wrap_text(warn, WARN_WRAP_WIDTH) {
        rows.push(WarningRow::new(line, "text.warning", "yellow"));
    }
    if !info_lines.is_empty() {
        rows.push(WarningRow::blank());
        for line in info_lines {
            rows.push(WarningRow::new(line, "text.primary", "white"));
        }
    }
    rows.push(WarningRow::blank());
    rows.extend(action_rows);

    let content_width = rows
        .iter()
        .map(|row| UnicodeWidthStr::width(row.text.as_str()))
        .max()
        .unwrap_or(0) as u16;
    let x = ctx.terminal_size.width.saturating_sub(content_width) / 2;
    let top = ctx.terminal_size.height.saturating_sub(rows.len() as u16) / 2;
    for (index, row) in rows.iter().enumerate() {
        if row.text.is_empty() {
            continue;
        }
        canvas.draw_text_styled(
            x,
            top.saturating_add(index as u16),
            row.text.as_str(),
            Some(theme_color(ctx, row.role, row.fallback)),
            None,
            vec![STYLE_BOLD],
        )?;
    }
    Ok(())
}

#[derive(Clone, Debug)]
struct WarningRow {
    text: String,
    role: &'static str,
    fallback: &'static str,
}

impl WarningRow {
    fn new(text: String, role: &'static str, fallback: &'static str) -> Self {
        Self {
            text,
            role,
            fallback,
        }
    }

    fn blank() -> Self {
        Self::new(String::new(), "text.primary", "white")
    }
}

fn started_at(started_at: &Cell<Option<Instant>>) -> Instant {
    started_at.get().unwrap_or_else(Instant::now)
}

fn remaining_seconds(started_at: &Cell<Option<Instant>>) -> u64 {
    remaining_for(started_at, CONFIRM_DELAY)
}

fn remaining_for(started_at: &Cell<Option<Instant>>, delay: Duration) -> u64 {
    let elapsed = started_at
        .get()
        .map(|started_at| started_at.elapsed())
        .unwrap_or_default();
    delay.saturating_sub(elapsed).as_millis().div_ceil(1000) as u64
}

fn delay_action_text(key: String, label: &str, second_label: &str, remaining: u64) -> String {
    if remaining == 0 {
        format!("[{key}] {label}")
    } else {
        format!("[{key}] {label} {remaining}{second_label}")
    }
}

fn centered_x(ctx: &UiContext, text: &str) -> u16 {
    let width = UnicodeWidthStr::width(text) as u16;
    ctx.terminal_size.width.saturating_sub(width) / 2
}

fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    for raw_line in text.replace("\\n", "\n").lines() {
        let mut current = String::new();
        let mut current_width = 0;
        for word in raw_line.split_whitespace() {
            let word_width = UnicodeWidthStr::width(word);
            let separator_width = usize::from(!current.is_empty());
            if current_width + separator_width + word_width > max_width && !current.is_empty() {
                lines.push(current);
                current = String::new();
                current_width = 0;
            }
            if !current.is_empty() {
                current.push(' ');
                current_width += 1;
            }
            if word_width > max_width {
                for ch in word.chars() {
                    let ch_width = UnicodeWidthChar::width(ch).unwrap_or(0);
                    if current_width + ch_width > max_width && !current.is_empty() {
                        lines.push(current);
                        current = String::new();
                        current_width = 0;
                    }
                    current.push(ch);
                    current_width += ch_width;
                }
            } else {
                current.push_str(word);
                current_width += word_width;
            }
        }
        if !current.is_empty() {
            lines.push(current);
        }
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}
