//! Rust implementation of language selector.

use std::fs;
use std::sync::Arc;

use serde_json::Value;
use unicode_width::UnicodeWidthStr;

use crate::host_engine::boot::environment::data_dirs;
use crate::host_engine::boot::i18n::i18n;
use crate::host_engine::boot::preload::lua_runtime::api::drawing_support::border_chars::BorderChars;
use crate::host_engine::boot::preload::lua_runtime::api::drawing_support::drawing_parser::STYLE_BOLD;
use crate::host_engine::runtime::ui::pages::common::{
    draw_title, is_press, key_hint, take_navigation, theme_color,
};
use crate::host_engine::runtime::ui::{Canvas, UiContext, UiEvent, UiNavigation, UiPage, UiResult};
use crate::host_engine::runtime::ui_page::page_key::UiPageKey;

const GRID_START_Y: u16 = 3;
const CELL_HEIGHT: u16 = 3;
const MIN_CELL_WIDTH: u16 = 12;
const CELL_PADDING: u16 = 4;

#[derive(Clone, Debug)]
struct LanguageOption {
    code: String,
    name: String,
}

#[derive(Clone, Copy, Debug)]
struct LanguageLayout {
    x: u16,
    y: u16,
    columns: usize,
    rows: usize,
    cell_width: u16,
    per_page: usize,
    pages: usize,
}

pub struct SettingLanguagePage {
    languages: Vec<LanguageOption>,
    selected_index: usize,
    page: usize,
    initialized_selection: bool,
    jump_mode: bool,
    user_page: usize,
    active_language: Option<String>,
    pending_navigation: Option<UiNavigation>,
}

impl SettingLanguagePage {
    pub fn new() -> Self {
        let languages = language_options();
        let selected_index = languages
            .iter()
            .position(|language| language.code == "en_us")
            .unwrap_or(0);
        Self {
            languages,
            selected_index,
            page: 1,
            initialized_selection: false,
            jump_mode: false,
            user_page: 0,
            active_language: None,
            pending_navigation: None,
        }
    }

    fn layout(&self, ctx: &UiContext) -> LanguageLayout {
        let terminal_width = ctx.terminal_size.width.max(1);
        let terminal_height = ctx.terminal_size.height.max(1);
        let longest_name = self
            .languages
            .iter()
            .map(|language| UnicodeWidthStr::width(language.name.as_str()) as u16)
            .max()
            .unwrap_or(MIN_CELL_WIDTH);
        let cell_width = longest_name
            .max(MIN_CELL_WIDTH)
            .saturating_add(CELL_PADDING);
        let columns = (terminal_width / cell_width).max(1) as usize;
        let available_rows = terminal_height.saturating_sub(GRID_START_Y + 2);
        let rows = (available_rows / CELL_HEIGHT).max(1) as usize;
        let per_page = columns.saturating_mul(rows).max(1);
        let pages = self.languages.len().max(1).div_ceil(per_page).max(1);
        let grid_width = (columns as u16).saturating_mul(cell_width);
        let x = terminal_width.saturating_sub(grid_width) / 2;
        LanguageLayout {
            x,
            y: GRID_START_Y,
            columns,
            rows,
            cell_width,
            per_page,
            pages,
        }
    }

    fn normalize_state(&mut self, layout: LanguageLayout) {
        if self.languages.is_empty() {
            self.selected_index = 0;
            self.page = 1;
            return;
        }
        self.selected_index = self.selected_index.min(self.languages.len() - 1);
        self.page = self.page.clamp(1, layout.pages);
        let selected_page = self.selected_index / layout.per_page + 1;
        if selected_page != self.page {
            self.page = selected_page.clamp(1, layout.pages);
        }
    }

    fn current_language_code<'a>(&'a self, ctx: &'a UiContext) -> &'a str {
        self.active_language
            .as_deref()
            .unwrap_or(ctx.profiles.language.as_str())
    }

    fn ensure_profile_selection(&mut self, ctx: &UiContext) {
        if self.initialized_selection {
            return;
        }
        let active_code = self.current_language_code(ctx);
        if let Some(index) = self
            .languages
            .iter()
            .position(|language| language.code == active_code)
        {
            self.selected_index = index;
        }
        self.initialized_selection = true;
    }

    fn select_page_start(&mut self, layout: LanguageLayout) {
        if self.languages.is_empty() {
            return;
        }
        let start = (self.page - 1).saturating_mul(layout.per_page);
        self.selected_index = start.min(self.languages.len() - 1);
    }

    fn visible_range(&self, layout: LanguageLayout) -> (usize, usize) {
        visible_range_for(self.page, self.languages.len(), layout)
    }

    fn effective_selection(&self, ctx: &UiContext, layout: LanguageLayout) -> (usize, usize) {
        if self.initialized_selection {
            return (self.selected_index, self.page.clamp(1, layout.pages));
        }
        let active_code = self.current_language_code(ctx);
        let selected_index = self
            .languages
            .iter()
            .position(|language| language.code == active_code)
            .unwrap_or(self.selected_index);
        let page = selected_index / layout.per_page + 1;
        (selected_index, page.clamp(1, layout.pages))
    }

    fn visible_range_for_page(&self, page: usize, layout: LanguageLayout) -> (usize, usize) {
        visible_range_for(page, self.languages.len(), layout)
    }
}

fn visible_range_for(page: usize, language_count: usize, layout: LanguageLayout) -> (usize, usize) {
    let start = page.saturating_sub(1).saturating_mul(layout.per_page);
    let end = start.saturating_add(layout.per_page).min(language_count);
    (start, end)
}

impl SettingLanguagePage {
    fn selected_column(&self, layout: LanguageLayout) -> usize {
        let (start, _) = self.visible_range(layout);
        self.selected_index.saturating_sub(start) % layout.columns
    }

    fn move_left(&mut self, layout: LanguageLayout) {
        if self.languages.is_empty() {
            return;
        }
        let (start, end) = self.visible_range(layout);
        self.selected_index = if self.selected_index <= start {
            end.saturating_sub(1)
        } else {
            self.selected_index.saturating_sub(1)
        };
    }

    fn move_right(&mut self, layout: LanguageLayout) {
        if self.languages.is_empty() {
            return;
        }
        let (start, end) = self.visible_range(layout);
        self.selected_index = if self.selected_index + 1 >= end {
            start
        } else {
            self.selected_index + 1
        };
    }

    fn move_up(&mut self, layout: LanguageLayout) {
        if self.languages.is_empty() {
            return;
        }
        let col = self.selected_column(layout);
        let (start, end) = self.visible_range(layout);
        if self.selected_index >= start.saturating_add(layout.columns) {
            self.selected_index = self.selected_index.saturating_sub(layout.columns);
            return;
        }
        if self.page > 1 {
            self.page -= 1;
            let prev_start = (self.page - 1).saturating_mul(layout.per_page);
            let prev_end = prev_start
                .saturating_add(layout.per_page)
                .min(self.languages.len());
            let last_row_start = prev_end.saturating_sub(prev_start).saturating_sub(1)
                / layout.columns
                * layout.columns;
            self.selected_index = (prev_start + last_row_start + col).min(prev_end - 1);
        } else {
            let visible_len = end.saturating_sub(start).max(1);
            let last_row_start = (visible_len - 1) / layout.columns * layout.columns;
            self.selected_index = (start + last_row_start + col).min(end - 1);
        }
    }

    fn move_down(&mut self, layout: LanguageLayout) {
        if self.languages.is_empty() {
            return;
        }
        let col = self.selected_column(layout);
        let (start, end) = self.visible_range(layout);
        let candidate = self.selected_index.saturating_add(layout.columns);
        if candidate < end {
            self.selected_index = candidate;
            return;
        }
        if self.page < layout.pages {
            self.page += 1;
            let next_start = (self.page - 1).saturating_mul(layout.per_page);
            let next_end = next_start
                .saturating_add(layout.per_page)
                .min(self.languages.len());
            self.selected_index = (next_start + col).min(next_end - 1);
        } else {
            self.selected_index = (start + col).min(end - 1);
        }
    }

    fn prev_page(&mut self, layout: LanguageLayout) {
        if self.page > 1 {
            self.page -= 1;
            self.select_page_start(layout);
        }
    }

    fn next_page(&mut self, layout: LanguageLayout) {
        if self.page < layout.pages {
            self.page += 1;
            self.select_page_start(layout);
        }
    }

    fn confirm_jump(&mut self, layout: LanguageLayout) {
        if self.user_page > 0 {
            self.page = self.user_page.clamp(1, layout.pages);
            self.select_page_start(layout);
        }
        self.jump_mode = false;
        self.user_page = 0;
    }

    fn handle_normal_action(&mut self, name: &str, ctx: &mut UiContext) -> UiResult<()> {
        self.ensure_profile_selection(ctx);
        let layout = self.layout(ctx);
        self.normalize_state(layout);
        match name {
            "left_option" | "left" | "arrowleft" => self.move_left(layout),
            "right_option" | "right" | "arrowright" => self.move_right(layout),
            "up_option" | "up" | "arrowup" => self.move_up(layout),
            "down_option" | "down" | "arrowdown" => self.move_down(layout),
            "prev_page" => self.prev_page(layout),
            "next_page" => self.next_page(layout),
            "jump" | "j" if layout.pages > 1 => {
                self.jump_mode = true;
                self.user_page = 0;
            }
            "confirm" | "enter" => {
                if let Some(language) = self.languages.get(self.selected_index) {
                    ctx.profiles.save_language(&language.code)?;
                    i18n::reload(&language.code)?;
                    ctx.i18n = Arc::new(i18n::text());
                    self.active_language = Some(language.code.clone());
                }
            }
            "back" | "return" | "esc" | "q" => {
                self.pending_navigation = Some(UiNavigation::Page(UiPageKey::Setting));
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_jump_action(&mut self, name: &str, ctx: &UiContext) {
        let layout = self.layout(ctx);
        match name {
            "confirm" | "enter" => self.confirm_jump(layout),
            "back" | "return" | "esc" | "q" => {
                self.jump_mode = false;
                self.user_page = 0;
            }
            _ => {
                if let Some(digit) = digit_from_action(name) {
                    self.user_page = self
                        .user_page
                        .saturating_mul(10)
                        .saturating_add(digit as usize)
                        .min(9999);
                }
            }
        }
    }
}

impl UiPage for SettingLanguagePage {
    fn page_key(&self) -> UiPageKey {
        UiPageKey::SettingLanguage
    }

    fn handle_event(&mut self, event: &UiEvent, ctx: &mut UiContext) -> UiResult<()> {
        if let UiEvent::Resize { .. } = event {
            let layout = self.layout(ctx);
            self.normalize_state(layout);
            return Ok(());
        }
        let (UiEvent::Action { name, status } | UiEvent::Key { name, status }) = event else {
            return Ok(());
        };
        if !is_press(status) {
            return Ok(());
        }
        if self.jump_mode {
            self.handle_jump_action(name, ctx);
        } else {
            self.handle_normal_action(name, ctx)?;
        }
        Ok(())
    }

    fn render(&self, canvas: &mut Canvas, ctx: &UiContext) -> UiResult<()> {
        canvas.clear()?;
        draw_title(canvas, ctx, &ctx.i18n.language.title)?;
        let layout = self.layout(ctx);
        let (selected_index, page) = self.effective_selection(ctx, layout);
        self.render_grid(canvas, ctx, layout, selected_index, page)?;
        self.render_page_line(canvas, ctx, layout, page)?;
        self.render_footer(canvas, ctx, layout)?;
        Ok(())
    }

    fn take_navigation(&mut self) -> Option<UiNavigation> {
        take_navigation(&mut self.pending_navigation)
    }
}

impl SettingLanguagePage {
    fn render_grid(
        &self,
        canvas: &mut Canvas,
        ctx: &UiContext,
        layout: LanguageLayout,
        selected_index: usize,
        page: usize,
    ) -> UiResult<()> {
        let active_code = self.current_language_code(ctx);
        let (start, end) = self.visible_range_for_page(page, layout);
        for (visible_index, language) in self.languages[start..end].iter().enumerate() {
            let column = visible_index % layout.columns;
            let row = visible_index / layout.columns;
            if row >= layout.rows {
                break;
            }
            let cell_x = layout
                .x
                .saturating_add((column as u16).saturating_mul(layout.cell_width));
            let cell_y = layout
                .y
                .saturating_add((row as u16).saturating_mul(CELL_HEIGHT));
            let selected = start + visible_index == selected_index;
            if selected {
                canvas.border_rect(
                    cell_x,
                    cell_y,
                    layout.cell_width,
                    CELL_HEIGHT,
                    double_border(),
                    Some(theme_color(ctx, "accent.primary", "cyan")),
                    None,
                )?;
            }
            let name_width = UnicodeWidthStr::width(language.name.as_str()) as u16;
            let text_x = cell_x.saturating_add(layout.cell_width.saturating_sub(name_width) / 2);
            let fg = if language.code == active_code {
                theme_color(ctx, "state.success", "green")
            } else {
                theme_color(ctx, "text.primary", "white")
            };
            canvas.draw_text_styled(
                text_x,
                cell_y.saturating_add(1),
                &language.name,
                Some(fg),
                None,
                vec![STYLE_BOLD],
            )?;
        }
        Ok(())
    }

    fn render_page_line(
        &self,
        canvas: &mut Canvas,
        ctx: &UiContext,
        layout: LanguageLayout,
        page: usize,
    ) -> UiResult<()> {
        let y = ctx.terminal_size.height.saturating_sub(2);
        if self.jump_mode {
            let input = if self.user_page == 0 {
                format!("_/{}", layout.pages)
            } else {
                format!("{}/{}", self.user_page, layout.pages)
            };
            let input_width = UnicodeWidthStr::width(input.as_str()) as u16;
            let x = ctx.terminal_size.width.saturating_sub(input_width) / 2;
            canvas.draw_text_styled(
                x,
                y,
                input,
                Some("black".to_string()),
                Some("yellow".to_string()),
                vec![STYLE_BOLD],
            )?;
            return Ok(());
        }

        let page_text = format!("{}/{}", page.min(layout.pages), layout.pages);
        let page_width = UnicodeWidthStr::width(page_text.as_str()) as u16;
        let center_x = ctx.terminal_size.width.saturating_sub(page_width) / 2;
        canvas.draw_text_styled(
            center_x,
            y,
            page_text,
            Some(theme_color(ctx, "text.muted", "dark_gray")),
            None,
            vec![STYLE_BOLD],
        )?;

        if page > 1 {
            canvas.draw_text_styled(
                0,
                y,
                format!("◀ [{}]", key_hint(ctx, "prev_page", "Q")),
                Some(theme_color(ctx, "text.muted", "dark_gray")),
                None,
                vec![STYLE_BOLD],
            )?;
        }
        if page < layout.pages {
            let text = format!("[{}] ▶", key_hint(ctx, "next_page", "E"));
            let text_width = UnicodeWidthStr::width(text.as_str()) as u16;
            let x = ctx.terminal_size.width.saturating_sub(text_width);
            canvas.draw_text_styled(
                x,
                y,
                text,
                Some(theme_color(ctx, "text.muted", "dark_gray")),
                None,
                vec![STYLE_BOLD],
            )?;
        }
        Ok(())
    }

    fn render_footer(
        &self,
        canvas: &mut Canvas,
        ctx: &UiContext,
        layout: LanguageLayout,
    ) -> UiResult<()> {
        let text = if self.jump_mode {
            format!(
                "[1]-[9] {}   [{}] {}   [{}] {}",
                ctx.i18n.key.language_page,
                key_hint(ctx, "confirm", "Enter"),
                ctx.i18n.key.language_confirm,
                key_hint(ctx, "back", "Esc"),
                ctx.i18n.key.language_cancel,
            )
        } else {
            let select_keys = format!(
                "[{}]/[{}]/[{}]/[{}]",
                key_hint(ctx, "up_option", "↑"),
                key_hint(ctx, "down_option", "↓"),
                key_hint(ctx, "left_option", "←"),
                key_hint(ctx, "right_option", "→")
            );
            let mut parts = vec![
                format!("{select_keys} {}", ctx.i18n.key.language_select),
                format!(
                    "[{}] {}",
                    key_hint(ctx, "confirm", "Enter"),
                    ctx.i18n.key.language_confirm
                ),
            ];
            if layout.pages > 1 {
                parts.push(format!(
                    "[{}] {}",
                    key_hint(ctx, "jump", "J"),
                    ctx.i18n.key.language_jump
                ));
                parts.push(format!(
                    "[{}]/[{}] {}",
                    key_hint(ctx, "prev_page", "Q"),
                    key_hint(ctx, "next_page", "E"),
                    ctx.i18n.key.language_flip
                ));
            }
            parts.push(format!(
                "[{}] {}",
                key_hint(ctx, "back", "Esc"),
                ctx.i18n.key.language_back
            ));
            parts.join("   ")
        };
        let text_width = UnicodeWidthStr::width(text.as_str()) as u16;
        let x = ctx.terminal_size.width.saturating_sub(text_width) / 2;
        let y = ctx.terminal_size.height.saturating_sub(1);
        canvas.draw_text_styled(
            x,
            y,
            text,
            Some(theme_color(ctx, "text.muted", "dark_gray")),
            None,
            Vec::new(),
        )
    }
}

fn language_options() -> Vec<LanguageOption> {
    let lang_dir = data_dirs::root_dir().join("assets/lang");
    let mut languages: Vec<_> = fs::read_dir(lang_dir)
        .ok()
        .into_iter()
        .flat_map(|entries| entries.filter_map(Result::ok))
        .filter_map(|entry| {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                return None;
            }
            let code = path.file_stem()?.to_str()?.to_string();
            let name = language_display_name(&path).unwrap_or_else(|| code.clone());
            Some(LanguageOption { code, name })
        })
        .collect();
    languages.sort_by(|left, right| left.code.cmp(&right.code));
    if languages.is_empty() {
        languages.push(LanguageOption {
            code: "en_us".to_string(),
            name: "English".to_string(),
        });
    }
    languages
}

fn language_display_name(path: &std::path::Path) -> Option<String> {
    let raw_json = fs::read_to_string(path).ok()?;
    let value: Value = serde_json::from_str(&raw_json).ok()?;
    value
        .get("language.name")
        .and_then(Value::as_str)
        .map(str::to_string)
}

fn digit_from_action(name: &str) -> Option<u8> {
    let raw = name.strip_prefix("option").unwrap_or(name);
    match raw {
        "1" => Some(1),
        "2" => Some(2),
        "3" => Some(3),
        "4" => Some(4),
        "5" => Some(5),
        "6" => Some(6),
        "7" => Some(7),
        "8" => Some(8),
        "9" => Some(9),
        _ => None,
    }
}

fn double_border() -> BorderChars {
    BorderChars {
        top: Some('═'),
        top_right: Some('╗'),
        right: Some('║'),
        bottom_right: Some('╝'),
        bottom: Some('═'),
        bottom_left: Some('╚'),
        left: Some('║'),
        top_left: Some('╔'),
    }
}
