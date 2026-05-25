//! Rust implementation of system keybind editor shell.

use std::collections::{BTreeMap, HashMap};
use std::fs;

use serde_json::Value;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::host_engine::boot::environment::data_dirs;
use crate::host_engine::boot::preload::lua_runtime::api::drawing_support::drawing_parser::STYLE_BOLD;
use crate::host_engine::runtime::ui::components::SplitPanel;
use crate::host_engine::runtime::ui::pages::common::{
    is_press, key_hint, take_navigation, theme_color,
};
use crate::host_engine::runtime::ui::{Canvas, UiContext, UiEvent, UiNavigation, UiPage, UiResult};
use crate::host_engine::runtime::ui_page::action_defaults;
use crate::host_engine::runtime::ui_page::page_key::UiPageKey;

const PAGE_SIZE_PADDING: u16 = 4;
const MAX_KEYS: usize = 4;

pub struct KeybindSystemPage {
    selected_page: usize,
    page_scroll: usize,
    selected_action: usize,
    action_scroll: usize,
    focus: FocusArea,
    mode: KeyMode,
    order_desc: bool,
    sort_mode: SortMode,
    jump_mode: bool,
    jump_input: String,
    listening_slot: Option<usize>,
    pending_navigation: Option<UiNavigation>,
}

impl KeybindSystemPage {
    pub fn new() -> Self {
        Self {
            selected_page: 0,
            page_scroll: 0,
            selected_action: 0,
            action_scroll: 0,
            focus: FocusArea::List,
            mode: KeyMode::AddModify,
            order_desc: false,
            sort_mode: SortMode::Name,
            jump_mode: false,
            jump_input: String::new(),
            listening_slot: None,
            pending_navigation: None,
        }
    }
}

impl UiPage for KeybindSystemPage {
    fn page_key(&self) -> UiPageKey {
        UiPageKey::KeybindSystem
    }

    fn handle_event(&mut self, event: &UiEvent, ctx: &mut UiContext) -> UiResult<()> {
        match event {
            UiEvent::Resize { .. } => {
                self.clamp_after_resize(ctx);
                return Ok(());
            }
            UiEvent::Tick { .. } => return Ok(()),
            UiEvent::Action { name, status } | UiEvent::Key { name, status } => {
                if !is_press(status) {
                    return Ok(());
                }
                if self.handle_listening_event(name) {
                    return Ok(());
                }
            }
            _ => return Ok(()),
        }
        let (UiEvent::Action { name, .. } | UiEvent::Key { name, .. }) = event else {
            return Ok(());
        };
        if self.jump_mode {
            self.handle_jump_event(name, ctx);
            return Ok(());
        }

        match self.focus {
            FocusArea::List => self.handle_list_event(name, ctx),
            FocusArea::Keys => self.handle_key_table_event(name, ctx),
        }
        Ok(())
    }

    fn render(&self, canvas: &mut Canvas, ctx: &UiContext) -> UiResult<()> {
        canvas.clear()?;
        let pages = self.sorted_pages(ctx);
        let panel = self.panel(ctx, pages.len());
        panel.render_borders_with_theme(
            canvas,
            ctx,
            "",
            &ctx.i18n.setting_keybind.system_key_title,
        )?;
        self.render_left_title(canvas, ctx, &panel, &pages)?;
        self.render_page_list(canvas, ctx, &panel)?;
        let page_key = pages.get(self.selected_page).map(|page| page.key);
        let actions = page_actions(ctx, page_key);
        self.render_action_table(canvas, ctx, &panel, &actions)?;
        self.render_footer(canvas, ctx, pages.len())?;
        Ok(())
    }

    fn take_navigation(&mut self) -> Option<UiNavigation> {
        take_navigation(&mut self.pending_navigation)
    }
}

impl KeybindSystemPage {
    fn handle_jump_event(&mut self, name: &str, ctx: &UiContext) {
        match name {
            "confirm" | "enter" => {
                let pages = self.sorted_pages(ctx);
                if let Ok(page) = self.jump_input.parse::<usize>() {
                    if !pages.is_empty() {
                        self.selected_page = page.saturating_sub(1).min(pages.len() - 1);
                        self.clamp_page_scroll(ctx, pages.len());
                    }
                }
                self.jump_mode = false;
                self.jump_input.clear();
            }
            "back" | "return" | "esc" | "q" | "jump" => {
                self.jump_mode = false;
                self.jump_input.clear();
            }
            "backspace" => {
                self.jump_input.pop();
            }
            key if key.len() == 1 && key.chars().all(|ch| ch.is_ascii_digit()) => {
                self.jump_input.push_str(key);
            }
            _ => {}
        }
    }

    fn handle_listening_event(&mut self, name: &str) -> bool {
        if self.listening_slot.is_none() {
            return false;
        }
        // Full rebinding is intentionally deferred to the command/profile layer. Consuming
        // one key here gives the user the correct modal feedback and prevents accidental
        // navigation while the layout work remains storage-safe.
        let _captured_key = name;
        self.listening_slot = None;
        true
    }

    fn handle_list_event(&mut self, name: &str, ctx: &UiContext) {
        let pages = self.sorted_pages(ctx);
        match name {
            "prev_option" | "up" | "arrowup" => self.move_page(ctx, -1, pages.len()),
            "next_option" | "down" | "arrowdown" => self.move_page(ctx, 1, pages.len()),
            "prev_page" | "q" => self.flip_page(ctx, -1, pages.len()),
            "next_page" | "e" => self.flip_page(ctx, 1, pages.len()),
            "order" | "z" => self.order_desc = !self.order_desc,
            "sort" | "x" => self.sort_mode = self.sort_mode.next(),
            "jump" | "j" => {
                self.jump_mode = true;
                self.jump_input.clear();
            }
            "confirm" | "enter" => {
                self.focus = FocusArea::Keys;
                self.selected_action = 0;
                self.action_scroll = 0;
            }
            "back" | "return" | "esc" => {
                self.pending_navigation = Some(UiNavigation::Page(UiPageKey::SettingKeybind));
            }
            _ => {}
        }
    }

    fn handle_key_table_event(&mut self, name: &str, ctx: &UiContext) {
        let pages = self.sorted_pages(ctx);
        let actions = page_actions(ctx, pages.get(self.selected_page).map(|page| page.key));
        match name {
            "prev_option" | "up" | "arrowup" => self.move_action(ctx, &actions, -1),
            "next_option" | "down" | "arrowdown" => self.move_action(ctx, &actions, 1),
            "scroll_up" | "w" => self.scroll_action(ctx, &actions, -1),
            "scroll_down" | "s" => self.scroll_action(ctx, &actions, 1),
            "key_mode" | "a" => self.mode = self.mode.toggle(),
            "key1" | "1" | "key2" | "2" | "key3" | "3" | "key4" | "4" => {
                self.listening_slot = key_slot(name);
            }
            "reset_only" | "r" | "page_reset" | "t" => {
                // Persistence belongs to the command/profile layer. This page keeps the
                // operation visible without mutating storage during the layout pass.
                self.listening_slot = None;
            }
            "list_back" | "back" | "return" | "esc" | "q" => {
                self.focus = FocusArea::List;
            }
            _ => {}
        }
    }

    fn clamp_after_resize(&mut self, ctx: &UiContext) {
        let pages = self.sorted_pages(ctx);
        if !pages.is_empty() {
            self.selected_page = self.selected_page.min(pages.len() - 1);
            self.clamp_page_scroll(ctx, pages.len());
        }
        let actions = page_actions(ctx, pages.get(self.selected_page).map(|page| page.key));
        if !actions.is_empty() {
            self.selected_action = self.selected_action.min(actions.len() - 1);
            self.clamp_action_scroll(ctx, actions.len());
        }
    }

    fn render_left_title(
        &self,
        canvas: &mut Canvas,
        ctx: &UiContext,
        panel: &SplitPanel,
        _pages: &[SystemPage],
    ) -> UiResult<()> {
        let order = if self.order_desc {
            &ctx.i18n.setting_keybind.system_order_descending
        } else {
            &ctx.i18n.setting_keybind.system_order_ascending
        };
        let sort = match self.sort_mode {
            SortMode::Name => &ctx.i18n.setting_keybind.system_sort_name,
            SortMode::Conflict => &ctx.i18n.setting_keybind.system_sort_conflict,
        };
        let x = panel.left_x.saturating_add(2);
        let title = format!(" {} *", ctx.i18n.setting_keybind.system_list_title);
        canvas.draw_text_styled(
            x,
            panel.left_y,
            title.as_str(),
            Some(theme_color(ctx, "text.primary", "white")),
            Some(theme_color(ctx, "panel.background", "black")),
            vec![STYLE_BOLD],
        )?;
        let order_text = format!("[{order}] ");
        let order_x = x.saturating_add(width(title.as_str()));
        canvas.draw_text_styled(
            order_x,
            panel.left_y,
            &order_text,
            Some(theme_color(ctx, "state.success", "green")),
            Some(theme_color(ctx, "panel.background", "black")),
            vec![STYLE_BOLD],
        )?;
        let sort_x = order_x.saturating_add(width(order_text.as_str()));
        canvas.draw_text_styled(
            sort_x,
            panel.left_y,
            sort.as_str(),
            Some(theme_color(ctx, "state.warning", "yellow")),
            Some(theme_color(ctx, "panel.background", "black")),
            vec![STYLE_BOLD],
        )
    }

    fn render_page_list(
        &self,
        canvas: &mut Canvas,
        ctx: &UiContext,
        panel: &SplitPanel,
    ) -> UiResult<()> {
        let pages = self.sorted_pages(ctx);
        let capacity = left_capacity(panel);
        let start = self.page_scroll.min(pages.len().saturating_sub(capacity));
        for (row, page) in pages.iter().skip(start).take(capacity).enumerate() {
            let index = start + row;
            let y = panel.left_y.saturating_add(1 + row as u16);
            let selected = self.focus == FocusArea::List && index == self.selected_page;
            if selected {
                canvas.fill_rect(
                    panel.left_x.saturating_add(1),
                    y,
                    panel.left_width.saturating_sub(2),
                    1,
                    ' ',
                    None,
                    Some(theme_color(ctx, "background.selected", "#78a8da")),
                )?;
            }
            canvas.draw_text_styled(
                panel.left_x.saturating_add(1),
                y,
                truncate(&page.label, panel.left_width.saturating_sub(4) as usize),
                Some(if selected {
                    theme_color(ctx, "text.on_selected", "black")
                } else {
                    theme_color(ctx, "text.primary", "white")
                }),
                if selected {
                    Some(theme_color(ctx, "background.selected", "#78a8da"))
                } else {
                    None
                },
                vec![STYLE_BOLD],
            )?;
            if page.has_empty || page.has_conflict {
                canvas.draw_text_styled(
                    panel
                        .left_x
                        .saturating_add(panel.left_width.saturating_sub(2)),
                    y,
                    " ",
                    Some(theme_color(ctx, "text.primary", "white")),
                    Some(theme_color(ctx, "state.danger", "red")),
                    Vec::new(),
                )?;
            }
        }
        self.render_page_navigation(canvas, ctx, panel, pages.len(), capacity)
    }

    fn render_page_navigation(
        &self,
        canvas: &mut Canvas,
        ctx: &UiContext,
        panel: &SplitPanel,
        total: usize,
        capacity: usize,
    ) -> UiResult<()> {
        let pages = total.div_ceil(capacity.max(1)).max(1);
        let current = self.page_scroll / capacity.max(1) + 1;
        let y = panel.left_y.saturating_add(panel.height.saturating_sub(2));
        let color = theme_color(ctx, "text.muted", "dark_gray");
        if current > 1 {
            canvas.draw_text_styled(
                panel.left_x.saturating_add(2),
                y,
                format!("◀ [{}]", key_hint(ctx, "prev_page", "Q")),
                Some(color.clone()),
                None,
                vec![STYLE_BOLD],
            )?;
        }
        let center = if self.jump_mode {
            format!(
                "{}/{}",
                if self.jump_input.is_empty() {
                    "_"
                } else {
                    &self.jump_input
                },
                pages
            )
        } else {
            format!("{current}/{pages}")
        };
        let center_x = panel
            .left_x
            .saturating_add(panel.left_width.saturating_sub(width(center.as_str())) / 2);
        canvas.draw_text_styled(
            center_x,
            y,
            center,
            Some(if self.jump_mode {
                theme_color(ctx, "text.on_warning", "black")
            } else {
                color.clone()
            }),
            if self.jump_mode {
                Some(theme_color(ctx, "state.warning", "yellow"))
            } else {
                None
            },
            vec![STYLE_BOLD],
        )?;
        if current < pages {
            let text = format!("[{}] ▶", key_hint(ctx, "next_page", "E"));
            canvas.draw_text_styled(
                panel
                    .left_x
                    .saturating_add(panel.left_width.saturating_sub(width(text.as_str()) + 2)),
                y,
                text,
                Some(color),
                None,
                vec![STYLE_BOLD],
            )?;
        }
        Ok(())
    }

    fn render_action_table(
        &self,
        canvas: &mut Canvas,
        ctx: &UiContext,
        panel: &SplitPanel,
        actions: &[ActionRow],
    ) -> UiResult<()> {
        let x = panel.right_x.saturating_add(1);
        let y = panel.right_y.saturating_add(1);
        let width = panel.right_width.saturating_sub(2).max(1);
        let action_w = width.saturating_mul(40) / 100;
        let key_w = width.saturating_sub(action_w) / 4;
        self.draw_table_header(canvas, ctx, x, y, action_w, key_w, width)?;
        let body_y = y.saturating_add(2);
        let capacity = action_capacity(panel);
        let scroll = self
            .action_scroll
            .min(actions.len().saturating_sub(capacity));
        for (row_index, row) in actions.iter().skip(scroll).take(capacity).enumerate() {
            let index = scroll + row_index;
            let row_y = body_y.saturating_add(row_index as u16);
            let selected = self.focus == FocusArea::Keys && index == self.selected_action;
            let bg = if selected {
                Some(if self.mode == KeyMode::Delete {
                    theme_color(ctx, "state.danger", "red")
                } else {
                    theme_color(ctx, "background.selected", "#78a8da")
                })
            } else {
                None
            };
            if selected {
                canvas.fill_rect(x, row_y, width, 1, ' ', None, bg.clone())?;
            }
            if row.keys.is_empty() {
                canvas.draw_text_styled(
                    x,
                    row_y,
                    " ",
                    Some(theme_color(ctx, "text.primary", "white")),
                    Some(theme_color(ctx, "state.danger", "red")),
                    Vec::new(),
                )?;
                canvas.draw_text_styled(
                    x.saturating_add(width.saturating_sub(1)),
                    row_y,
                    " ",
                    Some(theme_color(ctx, "text.primary", "white")),
                    Some(theme_color(ctx, "state.danger", "red")),
                    Vec::new(),
                )?;
            }
            let fg = if selected {
                theme_color(ctx, "text.on_selected", "black")
            } else {
                theme_color(ctx, "text.primary", "white")
            };
            canvas.draw_text_styled(
                x.saturating_add(1),
                row_y,
                truncate(&row.display_name, action_w.saturating_sub(2) as usize),
                Some(fg.clone()),
                bg.clone(),
                vec![STYLE_BOLD],
            )?;
            for slot in 0..MAX_KEYS {
                let key_x = x
                    .saturating_add(action_w)
                    .saturating_add(key_w.saturating_mul(slot as u16));
                let key = row
                    .keys
                    .get(slot)
                    .map(|key| display_key(key))
                    .unwrap_or_default();
                canvas.draw_text_styled(
                    key_x,
                    row_y,
                    truncate(
                        format!("[{}]", key).as_str(),
                        key_w.saturating_sub(1) as usize,
                    ),
                    Some(fg.clone()),
                    bg.clone(),
                    vec![STYLE_BOLD],
                )?;
            }
        }
        Ok(())
    }

    fn draw_table_header(
        &self,
        canvas: &mut Canvas,
        ctx: &UiContext,
        x: u16,
        y: u16,
        action_w: u16,
        key_w: u16,
        width: u16,
    ) -> UiResult<()> {
        let color = theme_color(ctx, "text.primary", "white");
        canvas.draw_text_styled(
            x.saturating_add(1),
            y,
            &ctx.i18n.setting_keybind.system_table_action,
            Some(color.clone()),
            None,
            vec![STYLE_BOLD],
        )?;
        let headers = [
            &ctx.i18n.setting_keybind.system_table_key1,
            &ctx.i18n.setting_keybind.system_table_key2,
            &ctx.i18n.setting_keybind.system_table_key3,
            &ctx.i18n.setting_keybind.system_table_key4,
        ];
        for (index, header) in headers.iter().enumerate() {
            let key_x = x
                .saturating_add(action_w)
                .saturating_add(key_w.saturating_mul(index as u16));
            canvas.draw_text_styled(
                key_x,
                y,
                format!("[{}]{}", index + 1, header),
                Some(color.clone()),
                None,
                vec![STYLE_BOLD],
            )?;
        }
        canvas.draw_text_styled(
            x,
            y.saturating_add(1),
            "─".repeat(width as usize),
            Some(color),
            None,
            Vec::new(),
        )
    }

    fn render_footer(
        &self,
        canvas: &mut Canvas,
        ctx: &UiContext,
        total_pages: usize,
    ) -> UiResult<()> {
        let lines = self.footer_lines(ctx, total_pages);
        let start_y = ctx
            .terminal_size
            .height
            .saturating_sub(lines.len().max(1) as u16);
        for (index, line) in lines.iter().enumerate() {
            let x = ctx.terminal_size.width.saturating_sub(width(line.as_str())) / 2;
            canvas.draw_text_styled(
                x,
                start_y.saturating_add(index as u16),
                line,
                Some(theme_color(ctx, "text.muted", "dark_gray")),
                None,
                Vec::new(),
            )?;
        }
        Ok(())
    }

    fn footer_parts(&self, ctx: &UiContext, total_pages: usize) -> Vec<String> {
        if self.listening_slot.is_some() {
            vec![
                format!(
                    "{} {}",
                    ctx.i18n.setting_keybind.system_key_any,
                    if self.mode == KeyMode::Delete {
                        &ctx.i18n.key.setting_keybind_system_delete
                    } else {
                        &ctx.i18n.key.setting_keybind_system_tip_add_modify
                    }
                ),
                format!(
                    "[{}] {}",
                    key_hint(ctx, "back", "Esc"),
                    ctx.i18n.key.setting_keybind_system_back
                ),
            ]
        } else if self.jump_mode {
            vec![
                format!("[1-9] {}", ctx.i18n.key.setting_keybind_system_select),
                format!(
                    "[{}] {}",
                    key_hint(ctx, "confirm", "Enter"),
                    ctx.i18n.key.setting_keybind_system_confirm
                ),
                format!(
                    "[{}] {}",
                    key_hint(ctx, "back", "Esc"),
                    ctx.i18n.key.setting_keybind_system_back
                ),
            ]
        } else if self.focus == FocusArea::Keys {
            vec![
                format!(
                    "[{}]/[{}] {}",
                    key_hint(ctx, "prev_option", "↑"),
                    key_hint(ctx, "next_option", "↓"),
                    ctx.i18n.key.setting_keybind_system_select
                ),
                format!(
                    "[{}/{}] {}",
                    key_hint(ctx, "scroll_up", "W"),
                    key_hint(ctx, "scroll_down", "S"),
                    ctx.i18n.key.setting_keybind_system_scroll
                ),
                format!(
                    "[1-4] {}",
                    if self.mode == KeyMode::Delete {
                        &ctx.i18n.key.setting_keybind_system_tip_delete
                    } else {
                        &ctx.i18n.key.setting_keybind_system_tip_add_modify
                    }
                ),
                format!(
                    "[{}] {}",
                    key_hint(ctx, "key_mode", "A"),
                    ctx.i18n.key.setting_keybind_system_key_mode
                ),
                format!(
                    "[{}] {}",
                    key_hint(ctx, "reset_only", "R"),
                    ctx.i18n.key.setting_keybind_system_reset_only
                ),
                format!(
                    "[{}] {}",
                    key_hint(ctx, "page_reset", "T"),
                    ctx.i18n.key.setting_keybind_system_reset_page
                ),
                format!(
                    "[{}] {}",
                    key_hint(ctx, "back", "Esc"),
                    ctx.i18n.key.setting_keybind_system_list_back
                ),
            ]
        } else {
            let mut parts = vec![
                format!(
                    "[{}]/[{}] {}",
                    key_hint(ctx, "prev_option", "↑"),
                    key_hint(ctx, "next_option", "↓"),
                    ctx.i18n.key.setting_keybind_system_select
                ),
                format!(
                    "[{}] {}",
                    key_hint(ctx, "confirm", "Enter"),
                    ctx.i18n.key.setting_keybind_system_confirm
                ),
                format!(
                    "[{}] {}",
                    key_hint(ctx, "order", "Z"),
                    ctx.i18n.key.setting_keybind_system_order
                ),
                format!(
                    "[{}] {}",
                    key_hint(ctx, "sort", "X"),
                    ctx.i18n.key.setting_keybind_system_sort
                ),
            ];
            let estimated_panel =
                SplitPanel::new(ctx.terminal_size.width, ctx.terminal_size.height, 1);
            if total_pages > left_capacity(&estimated_panel) {
                parts.push(format!(
                    "[{}] {}",
                    key_hint(ctx, "jump", "J"),
                    ctx.i18n.key.setting_keybind_system_jump
                ));
            }
            parts.push(format!(
                "[{}] {}",
                key_hint(ctx, "back", "Esc"),
                ctx.i18n.key.setting_keybind_system_back
            ));
            parts
        }
    }

    fn footer_lines(&self, ctx: &UiContext, total_pages: usize) -> Vec<String> {
        wrap_footer_parts(
            &self.footer_parts(ctx, total_pages),
            ctx.terminal_size.width.saturating_sub(2) as usize,
        )
    }

    fn footer_height(&self, ctx: &UiContext, total_pages: usize) -> u16 {
        self.footer_lines(ctx, total_pages).len().max(1) as u16
    }

    fn panel(&self, ctx: &UiContext, total_pages: usize) -> SplitPanel {
        SplitPanel::new(
            ctx.terminal_size.width,
            ctx.terminal_size.height,
            self.footer_height(ctx, total_pages),
        )
    }

    fn sorted_pages(&self, ctx: &UiContext) -> Vec<SystemPage> {
        let mut pages = system_pages(ctx);
        pages.sort_by(|left, right| match self.sort_mode {
            SortMode::Name => compare_text(&left.label, &right.label),
            SortMode::Conflict => left
                .has_conflict
                .cmp(&right.has_conflict)
                .then_with(|| left.has_empty.cmp(&right.has_empty))
                .then_with(|| compare_text(&left.label, &right.label)),
        });
        if self.order_desc {
            pages.reverse();
        }
        pages
    }

    fn move_page(&mut self, ctx: &UiContext, delta: isize, len: usize) {
        if len == 0 {
            return;
        }
        self.selected_page = wrap_index(self.selected_page, len, delta);
        self.selected_action = 0;
        self.action_scroll = 0;
        self.clamp_page_scroll(ctx, len);
    }

    fn flip_page(&mut self, ctx: &UiContext, delta: isize, len: usize) {
        if len == 0 {
            return;
        }
        let panel = self.panel(ctx, len);
        let capacity = left_capacity(&panel).max(1);
        self.selected_page = add_wrapped(self.selected_page, delta * capacity as isize, len);
        self.clamp_page_scroll(ctx, len);
    }

    fn clamp_page_scroll(&mut self, ctx: &UiContext, len: usize) {
        let panel = self.panel(ctx, len);
        let capacity = left_capacity(&panel).max(1);
        if self.selected_page < self.page_scroll {
            self.page_scroll = self.selected_page;
        } else if self.selected_page >= self.page_scroll + capacity {
            self.page_scroll = self.selected_page.saturating_sub(capacity - 1);
        }
        self.page_scroll = self.page_scroll.min(len.saturating_sub(capacity));
    }

    fn move_action(&mut self, ctx: &UiContext, actions: &[ActionRow], delta: isize) {
        if actions.is_empty() {
            return;
        }
        self.selected_action = wrap_index(self.selected_action, actions.len(), delta);
        self.clamp_action_scroll(ctx, actions.len());
    }

    fn scroll_action(&mut self, ctx: &UiContext, actions: &[ActionRow], delta: isize) {
        let panel = self.panel(ctx, actions.len());
        let capacity = action_capacity(&panel).max(1);
        self.action_scroll = add_clamped(
            self.action_scroll,
            delta,
            actions.len().saturating_sub(capacity),
        );
    }

    fn clamp_action_scroll(&mut self, ctx: &UiContext, len: usize) {
        let panel = self.panel(ctx, len);
        let capacity = action_capacity(&panel).max(1);
        if self.selected_action < self.action_scroll {
            self.action_scroll = self.selected_action;
        } else if self.selected_action >= self.action_scroll + capacity {
            self.action_scroll = self.selected_action.saturating_sub(capacity - 1);
        }
        self.action_scroll = self.action_scroll.min(len.saturating_sub(capacity));
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FocusArea {
    List,
    Keys,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum KeyMode {
    AddModify,
    Delete,
}

impl KeyMode {
    fn toggle(self) -> Self {
        match self {
            Self::AddModify => Self::Delete,
            Self::Delete => Self::AddModify,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SortMode {
    Name,
    Conflict,
}

impl SortMode {
    fn next(self) -> Self {
        match self {
            Self::Name => Self::Conflict,
            Self::Conflict => Self::Name,
        }
    }
}

#[derive(Clone, Debug)]
struct SystemPage {
    label: String,
    key: &'static str,
    has_empty: bool,
    has_conflict: bool,
}

#[derive(Clone, Debug)]
struct ActionRow {
    action: String,
    display_name: String,
    keys: Vec<String>,
}

fn system_pages(ctx: &UiContext) -> Vec<SystemPage> {
    let page_defs = [
        (ctx.i18n.setting_keybind.system_page_home.clone(), "home"),
        (
            ctx.i18n.setting_keybind.system_page_setting.clone(),
            "setting",
        ),
        (
            ctx.i18n.setting_keybind.system_page_game_list.clone(),
            "game_list",
        ),
        (
            ctx.i18n.setting_keybind.system_page_storage_details.clone(),
            "storage_details",
        ),
        (
            ctx.i18n.setting_keybind.system_page_setting_keybind.clone(),
            "setting_keybind",
        ),
        (
            ctx.i18n.setting_keybind.system_page_setting_memory.clone(),
            "setting_memory",
        ),
        (
            ctx.i18n
                .setting_keybind
                .system_page_setting_language
                .clone(),
            "setting_language",
        ),
        (
            ctx.i18n.setting_keybind.system_page_setting_mods.clone(),
            "setting_mods",
        ),
        (
            ctx.i18n
                .setting_keybind
                .system_page_setting_security
                .clone(),
            "setting_security",
        ),
        (
            ctx.i18n.setting_keybind.system_page_setting_display.clone(),
            "setting_display",
        ),
        (
            ctx.i18n.setting_keybind.system_page_keybind_system.clone(),
            "keybind_system",
        ),
    ];
    let profile = ctx.keybinds.to_profile_json();
    let language_texts = language_texts();
    page_defs
        .into_iter()
        .map(|(label, key)| {
            let rows = page_actions_from_profile(&profile, key, &language_texts);
            SystemPage {
                label,
                key,
                has_empty: rows.iter().any(|row| row.keys.is_empty()),
                has_conflict: page_has_conflict(&rows),
            }
        })
        .collect()
}

fn page_actions(ctx: &UiContext, page_key: Option<&str>) -> Vec<ActionRow> {
    let Some(page_key) = page_key else {
        return Vec::new();
    };
    let profile = ctx.keybinds.to_profile_json();
    let language_texts = language_texts();
    let mut rows = page_actions_from_package_json(page_key, &language_texts);
    let profile_rows = page_actions_from_profile(&profile, page_key, &language_texts);
    for row in &mut rows {
        if let Some(profile_row) = profile_rows.iter().find(|item| item.action == row.action) {
            row.keys = profile_row.keys.clone();
        }
    }
    for profile_row in profile_rows {
        if !rows.iter().any(|row| row.action == profile_row.action) {
            rows.push(profile_row);
        }
    }
    rows.sort_by(|left, right| compare_text(&left.display_name, &right.display_name));
    rows
}

fn page_actions_from_profile(
    profile: &Value,
    page_key: &str,
    language_texts: &HashMap<String, String>,
) -> Vec<ActionRow> {
    profile
        .get("system")
        .and_then(|system| system.get(page_key))
        .and_then(Value::as_object)
        .map(|actions| {
            actions
                .iter()
                .map(|(action, value)| {
                    let keys = value
                        .get("key_user")
                        .or_else(|| value.get("key"))
                        .map(keys_from_value)
                        .unwrap_or_default();
                    let display_name = value
                        .get("key_name")
                        .and_then(Value::as_str)
                        .map(|name| localized_action_name(name, action, language_texts))
                        .unwrap_or_else(|| display_action_name(action));
                    ActionRow {
                        action: action.clone(),
                        display_name,
                        keys,
                    }
                })
                .collect()
        })
        .unwrap_or_default()
}

fn page_actions_from_package_json(
    page_key: &str,
    language_texts: &HashMap<String, String>,
) -> Vec<ActionRow> {
    action_defaults::page_actions(page_key)
        .and_then(Value::as_object)
        .map(|actions| {
            actions
                .iter()
                .map(|(action, entry)| ActionRow {
                    action: action.clone(),
                    display_name: entry
                        .get("name")
                        .and_then(Value::as_str)
                        .map(|name| localized_action_name(name, action, language_texts))
                        .unwrap_or_else(|| display_action_name(action)),
                    keys: entry.get("key").map(keys_from_value).unwrap_or_default(),
                })
                .collect()
        })
        .unwrap_or_default()
}

fn keys_from_value(value: &Value) -> Vec<String> {
    match value {
        Value::String(key) => vec![key.clone()],
        Value::Array(keys) => keys
            .iter()
            .filter_map(Value::as_str)
            .take(MAX_KEYS)
            .map(str::to_string)
            .collect(),
        _ => Vec::new(),
    }
}

fn key_slot(name: &str) -> Option<usize> {
    match name {
        "key1" | "1" => Some(1),
        "key2" | "2" => Some(2),
        "key3" | "3" => Some(3),
        "key4" | "4" => Some(4),
        _ => None,
    }
}

fn page_has_conflict(rows: &[ActionRow]) -> bool {
    let mut seen = BTreeMap::new();
    for row in rows {
        for key in &row.keys {
            if seen
                .insert(key.to_ascii_lowercase(), row.action.as_str())
                .is_some()
            {
                return true;
            }
        }
    }
    false
}

fn localized_action_name(
    pseudo_constant: &str,
    fallback_action: &str,
    language_texts: &HashMap<String, String>,
) -> String {
    pseudo_constant
        .to_language_key()
        .and_then(|key| language_texts.get(&key).cloned())
        .filter(|text| !text.trim().is_empty())
        .unwrap_or_else(|| display_action_name(fallback_action))
}

trait PseudoConstantExt {
    fn to_language_key(&self) -> Option<String>;
}

impl PseudoConstantExt for str {
    fn to_language_key(&self) -> Option<String> {
        let lower = self.trim().to_ascii_lowercase();
        let key = if let Some(rest) = lower.strip_prefix("setting_keybind_system_") {
            format!("key.setting_keybind.system.{rest}")
        } else if let Some(rest) = lower.strip_prefix("setting_keybind_list_") {
            format!("key.setting_keybind.list.{rest}")
        } else if let Some(rest) = lower.strip_prefix("game_list_") {
            format!("key.game_list.{rest}")
        } else if let Some(rest) = lower.strip_prefix("mod_list_option") {
            format!("key.mod.list.option{rest}")
        } else if let Some(rest) = lower.strip_prefix("mod_hub_") {
            format!("key.mod.list.{rest}")
        } else if let Some(rest) = lower.strip_prefix("mod_list_") {
            format!("key.mod_game_list.{rest}")
        } else if let Some(rest) = lower.strip_prefix("home_") {
            format!("key.home.{rest}")
        } else if let Some(rest) = lower.strip_prefix("setting_") {
            format!("key.setting.{rest}")
        } else if let Some(rest) = lower.strip_prefix("language_") {
            format!("key.language.{rest}")
        } else if let Some(rest) = lower.strip_prefix("memory_") {
            format!("key.memory.{rest}")
        } else if let Some(rest) = lower.strip_prefix("security_") {
            format!("key.security.{rest}")
        } else if let Some(rest) = lower.strip_prefix("display_") {
            format!("key.display.{rest}")
        } else if let Some(rest) = lower.strip_prefix("storage_details_") {
            format!("key.storage_details.{rest}")
        } else if let Some(rest) = lower.strip_prefix("clear_data_") {
            format!("key.clear_data.{rest}")
        } else if let Some(rest) = lower.strip_prefix("clear_cache_") {
            format!("key.clear_cache.{rest}")
        } else {
            return None;
        };
        Some(key)
    }
}

fn language_texts() -> HashMap<String, String> {
    let root = data_dirs::root_dir();
    let lang_code = fs::read_to_string(root.join("data/profiles/language.txt"))
        .ok()
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
        .unwrap_or_else(|| "en_us".to_string());
    let mut texts = read_language_json(&root, "en_us");
    texts.extend(read_language_json(&root, &lang_code));
    texts
}

fn read_language_json(root: &std::path::Path, lang_code: &str) -> HashMap<String, String> {
    fs::read_to_string(root.join("assets/lang").join(format!("{lang_code}.json")))
        .ok()
        .and_then(|raw| serde_json::from_str::<HashMap<String, String>>(&raw).ok())
        .unwrap_or_default()
}

fn display_action_name(value: &str) -> String {
    let mut text = value.to_ascii_lowercase();
    for prefix in [
        "setting_keybind_system_",
        "setting_keybind_list_",
        "game_list_",
        "language_",
        "security_",
        "memory_",
        "display_",
        "setting_",
        "home_",
        "mod_list_",
        "mod_",
    ] {
        if let Some(stripped) = text.strip_prefix(prefix) {
            text = stripped.to_string();
            break;
        }
    }
    text.split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn display_key(key: &str) -> String {
    match key.trim().to_ascii_lowercase().as_str() {
        "up" => "↑".to_string(),
        "down" => "↓".to_string(),
        "left" => "←".to_string(),
        "right" => "→".to_string(),
        "pageup" => "PgUp".to_string(),
        "pagedown" => "PgDn".to_string(),
        "enter" => "Enter".to_string(),
        "backspace" => "Bksp".to_string(),
        "del" => "Del".to_string(),
        "esc" => "Esc".to_string(),
        "space" => "Space".to_string(),
        "left_shift" => "LShift".to_string(),
        "right_shift" => "RShift".to_string(),
        "shift" => "Shift".to_string(),
        other if other.len() == 1 => other.to_ascii_uppercase(),
        other => other.to_string(),
    }
}

fn left_capacity(panel: &SplitPanel) -> usize {
    panel.height.saturating_sub(PAGE_SIZE_PADDING) as usize
}

fn action_capacity(panel: &SplitPanel) -> usize {
    panel.height.saturating_sub(5) as usize
}

fn width(text: &str) -> u16 {
    UnicodeWidthStr::width(text) as u16
}

fn truncate(text: &str, max_width: usize) -> String {
    if UnicodeWidthStr::width(text) <= max_width {
        return text.to_string();
    }
    if max_width <= 3 {
        return ".".repeat(max_width);
    }
    let mut output = String::new();
    let mut current = 0;
    for ch in text.chars() {
        let w = UnicodeWidthChar::width(ch).unwrap_or(0);
        if current + w > max_width - 3 {
            break;
        }
        output.push(ch);
        current += w;
    }
    output.push_str("...");
    output
}

fn wrap_footer_parts(parts: &[String], max_width: usize) -> Vec<String> {
    if parts.is_empty() {
        return Vec::new();
    }
    let max_width = max_width.max(1);
    let separator = "  ";
    let mut lines = Vec::new();
    let mut line = String::new();
    for part in parts {
        let candidate = if line.is_empty() {
            part.clone()
        } else {
            format!("{line}{separator}{part}")
        };
        if !line.is_empty() && UnicodeWidthStr::width(candidate.as_str()) > max_width {
            lines.push(line);
            line = part.clone();
        } else {
            line = candidate;
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

fn wrap_index(current: usize, len: usize, delta: isize) -> usize {
    if len == 0 {
        return 0;
    }
    if delta < 0 {
        current.checked_sub(delta.unsigned_abs()).unwrap_or(len - 1)
    } else {
        (current + delta as usize) % len
    }
}

fn add_wrapped(current: usize, delta: isize, len: usize) -> usize {
    if len == 0 {
        return 0;
    }
    let current = current as isize + delta;
    current.rem_euclid(len as isize) as usize
}

fn add_clamped(current: usize, delta: isize, max_value: usize) -> usize {
    if delta < 0 {
        current.saturating_sub(delta.unsigned_abs())
    } else {
        current.saturating_add(delta as usize).min(max_value)
    }
}

fn compare_text(left: &str, right: &str) -> std::cmp::Ordering {
    let left = left.to_lowercase();
    let right = right.to_lowercase();
    UnicodeWidthStr::width(left.as_str())
        .cmp(&UnicodeWidthStr::width(right.as_str()))
        .then_with(|| left.cmp(&right))
}
