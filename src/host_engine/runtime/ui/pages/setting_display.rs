//! Rust implementation of display settings page.

use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::PathBuf;

use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::host_engine::boot::environment::data_dirs;
use crate::host_engine::boot::preload::lua_runtime::api::drawing_support::drawing_parser::STYLE_BOLD;
use crate::host_engine::boot::preload::persistent_data::display_profile::{
    DisplayOverlayProfile, DisplayProfile, persist_display_profile,
};
use crate::host_engine::package::kind::OverlayPackage;
use crate::host_engine::package::package_id::PackageSource;
use crate::host_engine::runtime::ui::pages::common::{
    MenuCommand, is_press, key_hint, selected_menu_event, take_navigation, theme_color,
};
use crate::host_engine::runtime::ui::{Canvas, UiContext, UiEvent, UiNavigation, UiPage, UiResult};
use crate::host_engine::runtime::ui_page::page_key::UiPageKey;

const ITEM_COUNT: usize = 9;
const ACTIONS: [&str; ITEM_COUNT] = [
    "option1", "option2", "option3", "option4", "option5", "option6", "option7", "option8",
    "option9",
];
const FALLBACK_KEYS: [&str; ITEM_COUNT] = ["1", "2", "3", "4", "5", "6", "7", "8", "9"];
const IDLE_THRESHOLDS: [u64; 5] = [30, 60, 300, 600, 0];

pub struct SettingDisplayPage {
    selected_index: usize,
    panel: DisplayPanel,
    screensaver_selected: usize,
    screensaver_scroll: usize,
    boss_selected: usize,
    boss_scroll: usize,
    move_mode: bool,
    position_mode: bool,
    position_input: String,
    profile: DisplayProfile,
    pending_navigation: Option<UiNavigation>,
}

impl SettingDisplayPage {
    pub fn new() -> Self {
        Self {
            selected_index: 0,
            panel: DisplayPanel::None,
            screensaver_selected: 0,
            screensaver_scroll: 0,
            boss_selected: 0,
            boss_scroll: 0,
            move_mode: false,
            position_mode: false,
            position_input: String::new(),
            profile: load_profile(),
            pending_navigation: None,
        }
    }
}

impl UiPage for SettingDisplayPage {
    fn page_key(&self) -> UiPageKey {
        UiPageKey::SettingDisplay
    }

    fn handle_event(&mut self, event: &UiEvent, ctx: &mut UiContext) -> UiResult<()> {
        self.profile.normalize();
        if self.panel != DisplayPanel::None {
            self.handle_panel_event(event, ctx)?;
            return Ok(());
        }

        match selected_menu_event(&mut self.selected_index, ITEM_COUNT, event) {
            Some(MenuCommand::Back) => {
                self.pending_navigation = Some(UiNavigation::Page(UiPageKey::Setting));
            }
            Some(MenuCommand::Confirm) => self.confirm_setting(ctx)?,
            _ => {}
        }
        Ok(())
    }

    fn render(&self, canvas: &mut Canvas, ctx: &UiContext) -> UiResult<()> {
        canvas.clear()?;
        let left_width = if self.panel == DisplayPanel::None {
            ctx.terminal_size.width
        } else {
            split_x(ctx).saturating_sub(1)
        };
        self.render_title(canvas, ctx, left_width)?;
        self.render_settings(canvas, ctx, left_width)?;
        if self.panel != DisplayPanel::None {
            self.render_panel(canvas, ctx)?;
        }
        self.render_footer(canvas, ctx)?;
        Ok(())
    }

    fn take_navigation(&mut self) -> Option<UiNavigation> {
        take_navigation(&mut self.pending_navigation)
    }
}

impl SettingDisplayPage {
    fn handle_panel_event(&mut self, event: &UiEvent, ctx: &UiContext) -> UiResult<()> {
        let (UiEvent::Action { name, status } | UiEvent::Key { name, status }) = event else {
            return Ok(());
        };
        if !is_press(status) {
            return Ok(());
        }

        if self.position_mode {
            match name.as_str() {
                "confirm" | "enter" => {
                    self.apply_position_input(ctx)?;
                    self.position_mode = false;
                    self.position_input.clear();
                }
                "back" | "return" | "esc" | "q" | "position" => {
                    self.position_mode = false;
                    self.position_input.clear();
                }
                "backspace" => {
                    self.position_input.pop();
                }
                key if key.len() == 1 && key.chars().all(|ch| ch.is_ascii_digit()) => {
                    self.position_input.push_str(key);
                }
                _ => {}
            }
            return Ok(());
        }

        match name.as_str() {
            "back" | "return" | "esc" | "q" => {
                self.panel = DisplayPanel::None;
                self.move_mode = false;
                self.position_mode = false;
                self.position_input.clear();
            }
            "scroll_up" | "w" => self.scroll_panel(ctx, -1),
            "scroll_down" | "s" => self.scroll_panel(ctx, 1),
            "prev_option" | "up" | "arrowup" => {
                if self.move_mode {
                    self.move_panel_item(ctx, -1)?;
                } else {
                    self.select_panel_item(ctx, -1);
                }
            }
            "next_option" | "down" | "arrowdown" => {
                if self.move_mode {
                    self.move_panel_item(ctx, 1)?;
                } else {
                    self.select_panel_item(ctx, 1);
                }
            }
            "confirm" | "enter" => self.toggle_panel_item(ctx)?,
            "order" | "z" => self.move_mode = !self.move_mode,
            "position" | "j" => {
                self.position_mode = !self.position_mode;
                self.position_input.clear();
                self.move_mode = false;
            }
            _ => {}
        }
        Ok(())
    }

    fn confirm_setting(&mut self, ctx: &UiContext) -> UiResult<()> {
        match self.selected_index {
            0 => self.profile.mod_badge = !self.profile.mod_badge,
            1 => self.profile.theme = "system".to_string(),
            2 => self.profile.idle_threshold = next_idle_threshold(self.profile.idle_threshold),
            3 => self.profile.idle_enter_screensaver = !self.profile.idle_enter_screensaver,
            4 => self.profile.host_status = !self.profile.host_status,
            5 => self.profile.screensaver_mode = next_mode(&self.profile.screensaver_mode),
            6 => self.profile.boss_mode = next_mode(&self.profile.boss_mode),
            7 => self.enter_panel(ctx, DisplayPanel::Screensaver)?,
            8 => self.enter_panel(ctx, DisplayPanel::Boss)?,
            _ => {}
        }
        if self.panel == DisplayPanel::None {
            self.save_profile()?;
        }
        Ok(())
    }

    fn enter_panel(&mut self, ctx: &UiContext, panel: DisplayPanel) -> UiResult<()> {
        self.panel = panel;
        self.move_mode = false;
        self.position_mode = false;
        self.position_input.clear();
        self.sync_panel_profile(ctx);
        self.clamp_panel_state(ctx);
        self.save_profile()
    }

    fn render_title(&self, canvas: &mut Canvas, ctx: &UiContext, left_width: u16) -> UiResult<()> {
        let title = ctx.i18n.display.title.as_str();
        let x = left_width.saturating_sub(width(title)) / 2;
        canvas.draw_text_styled(
            x,
            1,
            title,
            Some(theme_color(ctx, "text.primary", "white")),
            None,
            vec![STYLE_BOLD],
        )
    }

    fn render_settings(
        &self,
        canvas: &mut Canvas,
        ctx: &UiContext,
        left_width: u16,
    ) -> UiResult<()> {
        let rows = display_rows(ctx, &self.profile);
        let menu_width = rows
            .iter()
            .enumerate()
            .map(|(index, row)| setting_row_width(ctx, index, row, index == self.selected_index))
            .max()
            .unwrap_or(0) as u16;
        let x = left_width.saturating_sub(menu_width) / 2;
        let y = ctx.terminal_size.height.saturating_sub(ITEM_COUNT as u16) / 2;
        for (index, row) in rows.iter().enumerate() {
            self.render_setting_row(canvas, ctx, x, y.saturating_add(index as u16), index, row)?;
        }
        Ok(())
    }

    fn render_setting_row(
        &self,
        canvas: &mut Canvas,
        ctx: &UiContext,
        x: u16,
        y: u16,
        index: usize,
        row: &DisplayRow,
    ) -> UiResult<()> {
        let selected = self.panel == DisplayPanel::None && index == self.selected_index;
        let marker = if selected { "▶ " } else { "  " };
        canvas.draw_text_styled(
            x,
            y,
            marker,
            Some(if selected {
                theme_color(ctx, "accent.primary", "cyan")
            } else {
                theme_color(ctx, "text.primary", "white")
            }),
            None,
            vec![STYLE_BOLD],
        )?;

        let hint = if selected {
            key_hint(ctx, "confirm", "Enter")
        } else {
            key_hint(ctx, ACTIONS[index], FALLBACK_KEYS[index])
        };
        let key_text = format!("[{hint}] ");
        let key_x = x.saturating_add(width(marker));
        canvas.draw_text_styled(
            key_x,
            y,
            key_text.as_str(),
            Some(theme_color(ctx, "text.muted", "dark_gray")),
            None,
            vec![STYLE_BOLD],
        )?;

        let label_x = key_x.saturating_add(width(key_text.as_str()));
        canvas.draw_text_styled(
            label_x,
            y,
            row.label.as_str(),
            Some(if selected {
                theme_color(ctx, "accent.primary", "cyan")
            } else {
                theme_color(ctx, "text.primary", "white")
            }),
            None,
            vec![STYLE_BOLD],
        )?;

        if let Some(value) = row.value.as_ref() {
            let bracket_x = label_x.saturating_add(width(row.label.as_str()));
            canvas.draw_text_styled(
                bracket_x,
                y,
                "[ ",
                Some(theme_color(ctx, "text.primary", "white")),
                None,
                vec![STYLE_BOLD],
            )?;
            canvas.draw_text_styled(
                bracket_x.saturating_add(2),
                y,
                value.text.as_str(),
                Some(value_color(ctx, value.color)),
                None,
                vec![STYLE_BOLD],
            )?;
            canvas.draw_text_styled(
                bracket_x
                    .saturating_add(2)
                    .saturating_add(width(value.text.as_str())),
                y,
                " ]",
                Some(theme_color(ctx, "text.primary", "white")),
                None,
                vec![STYLE_BOLD],
            )?;
        }
        Ok(())
    }

    fn render_panel(&self, canvas: &mut Canvas, ctx: &UiContext) -> UiResult<()> {
        let divider_x = split_x(ctx).saturating_sub(1);
        for y in 0..ctx.terminal_size.height.saturating_sub(1) {
            canvas.draw_text_styled(
                divider_x,
                y,
                "║",
                Some(theme_color(ctx, "text.primary", "white")),
                None,
                vec![STYLE_BOLD],
            )?;
        }

        let items = panel_items(ctx, &self.profile, self.panel);
        let panel_x = divider_x.saturating_add(2);
        let panel_width = ctx.terminal_size.width.saturating_sub(panel_x).max(1);
        let panel_height = ctx.terminal_size.height.saturating_sub(2) as usize;
        let selected = self.panel_selected().min(items.len().saturating_sub(1));
        let scroll = self.panel_scroll().min(selected);
        let visible_count = panel_height.min(items.len().saturating_sub(scroll));
        for (row, item) in items.iter().skip(scroll).take(visible_count).enumerate() {
            self.render_panel_item(
                canvas,
                ctx,
                panel_x,
                row as u16,
                panel_width,
                scroll + row,
                item,
                scroll + row == selected,
            )?;
        }
        self.render_panel_scroll_hints(
            canvas,
            ctx,
            panel_x,
            panel_width,
            &items,
            scroll,
            visible_count,
        )?;
        self.render_panel_mode(canvas, ctx, panel_x, panel_width)?;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn render_panel_item(
        &self,
        canvas: &mut Canvas,
        ctx: &UiContext,
        x: u16,
        y: u16,
        panel_width: u16,
        index: usize,
        item: &PanelItem,
        selected: bool,
    ) -> UiResult<()> {
        let bg = selected.then(|| theme_color(ctx, "background.selected", "#78a8da"));
        let fg = if selected {
            theme_color(ctx, "text.on_selected", "black")
        } else {
            theme_color(ctx, "text.primary", "white")
        };
        canvas.draw_text_styled(
            x,
            y,
            " ".repeat(panel_width as usize),
            Some(fg.clone()),
            bg.clone(),
            Vec::new(),
        )?;

        let number = format!("{:>4} ", index + 1);
        canvas.draw_text_styled(
            x,
            y,
            number.as_str(),
            Some(theme_color(ctx, "text.muted", "dark_gray")),
            bg.clone(),
            vec![STYLE_BOLD],
        )?;

        let indicator_width = 1;
        let mod_badge_width = if self.profile.mod_badge && item.is_mod {
            4
        } else {
            0
        };
        let name_x = x.saturating_add(width(number.as_str()));
        let used_right = indicator_width + mod_badge_width + 1;
        let name_width = panel_width
            .saturating_sub(width(number.as_str()))
            .saturating_sub(used_right);
        canvas.draw_text_styled(
            name_x,
            y,
            truncate_with_ellipsis(item.name.as_str(), name_width as usize),
            Some(fg),
            bg.clone(),
            vec![STYLE_BOLD],
        )?;
        if self.profile.mod_badge && item.is_mod && panel_width > 6 {
            let badge_x = x.saturating_add(panel_width.saturating_sub(indicator_width + 4));
            canvas.draw_text_styled(
                badge_x,
                y,
                "MOD",
                Some(theme_color(ctx, "state.warning", "yellow")),
                bg.clone(),
                vec![STYLE_BOLD],
            )?;
        }
        let indicator_x = x.saturating_add(panel_width.saturating_sub(1));
        canvas.draw_text_styled(
            indicator_x,
            y,
            " ",
            Some(theme_color(ctx, "text.primary", "white")),
            Some(if item.display_enabled {
                theme_color(ctx, "state.success", "green")
            } else {
                theme_color(ctx, "state.danger", "red")
            }),
            Vec::new(),
        )
    }

    fn render_panel_scroll_hints(
        &self,
        canvas: &mut Canvas,
        ctx: &UiContext,
        panel_x: u16,
        panel_width: u16,
        items: &[PanelItem],
        scroll: usize,
        visible_count: usize,
    ) -> UiResult<()> {
        if items.len() <= visible_count {
            return Ok(());
        }
        let x = panel_x.saturating_add(panel_width.saturating_sub(2));
        let hint_color = theme_color(ctx, "text.muted", "dark_gray");
        if scroll > 0 {
            canvas.draw_text_styled(x, 0, "↑", Some(hint_color.clone()), None, vec![STYLE_BOLD])?;
            canvas.draw_text_styled(
                x,
                1,
                key_hint(ctx, "scroll_up", "W"),
                Some(hint_color.clone()),
                None,
                vec![STYLE_BOLD],
            )?;
        }
        if scroll + visible_count < items.len() {
            let y = ctx.terminal_size.height.saturating_sub(3);
            canvas.draw_text_styled(
                x,
                y,
                key_hint(ctx, "scroll_down", "S"),
                Some(hint_color.clone()),
                None,
                vec![STYLE_BOLD],
            )?;
            canvas.draw_text_styled(
                x,
                y.saturating_add(1),
                "↓",
                Some(hint_color),
                None,
                vec![STYLE_BOLD],
            )?;
        }
        Ok(())
    }

    fn render_panel_mode(
        &self,
        canvas: &mut Canvas,
        ctx: &UiContext,
        panel_x: u16,
        panel_width: u16,
    ) -> UiResult<()> {
        let text = if self.move_mode {
            Some(format!(
                "[{}] {}",
                key_hint(ctx, "order", "Z"),
                ctx.i18n.key.display_order
            ))
        } else if self.position_mode {
            let value = if self.position_input.is_empty() {
                "_"
            } else {
                self.position_input.as_str()
            };
            Some(format!("[{}] {}", key_hint(ctx, "position", "J"), value))
        } else {
            None
        };
        let Some(text) = text else {
            return Ok(());
        };
        let x = panel_x.saturating_add(panel_width.saturating_sub(width(text.as_str())) / 2);
        canvas.draw_text_styled(
            x,
            ctx.terminal_size.height.saturating_sub(2),
            text,
            Some(theme_color(ctx, "state.warning", "yellow")),
            None,
            vec![STYLE_BOLD],
        )
    }

    fn render_footer(&self, canvas: &mut Canvas, ctx: &UiContext) -> UiResult<()> {
        let footer = if self.panel == DisplayPanel::None {
            format!(
                "[{}]/[{}] {}   [{}] {}   [{}] {}",
                key_hint(ctx, "prev_option", "↑"),
                key_hint(ctx, "next_option", "↓"),
                ctx.i18n.key.display_select,
                key_hint(ctx, "confirm", "Enter"),
                ctx.i18n.key.display_toggle_confirm,
                key_hint(ctx, "back", "Esc"),
                ctx.i18n.key.display_back,
            )
        } else {
            format!(
                "[{}]/[{}] {}   [{}] {}   [{}/{}] {}   [{}] {}   [{}] {}   [{}] {}",
                key_hint(ctx, "prev_option", "↑"),
                key_hint(ctx, "next_option", "↓"),
                ctx.i18n.key.display_select,
                key_hint(ctx, "confirm", "Enter"),
                ctx.i18n.key.display_toggle,
                key_hint(ctx, "scroll_up", "W"),
                key_hint(ctx, "scroll_down", "S"),
                ctx.i18n.key.display_scroll,
                key_hint(ctx, "order", "Z"),
                ctx.i18n.key.display_order,
                key_hint(ctx, "position", "J"),
                ctx.i18n.key.display_position,
                key_hint(ctx, "back", "Esc"),
                ctx.i18n.key.display_back,
            )
        };
        let x = ctx
            .terminal_size
            .width
            .saturating_sub(width(footer.as_str()))
            / 2;
        canvas.draw_text_styled(
            x,
            ctx.terminal_size.height.saturating_sub(1),
            footer,
            Some(theme_color(ctx, "text.muted", "dark_gray")),
            None,
            Vec::new(),
        )
    }

    fn select_panel_item(&mut self, ctx: &UiContext, delta: isize) {
        let len = panel_items(ctx, &self.profile, self.panel).len();
        if len == 0 {
            return;
        }
        let selected = self.panel_selected_mut();
        *selected = wrap_index(*selected, len, delta);
        self.clamp_panel_state(ctx);
    }

    fn scroll_panel(&mut self, ctx: &UiContext, delta: isize) {
        let len = panel_items(ctx, &self.profile, self.panel).len();
        let visible = self.panel_visible_count(ctx).max(1);
        let max_scroll = len.saturating_sub(visible);
        let scroll = self.panel_scroll_mut();
        *scroll = add_clamped(*scroll, delta, max_scroll);
    }

    fn move_panel_item(&mut self, ctx: &UiContext, delta: isize) -> UiResult<()> {
        let items = panel_items(ctx, &self.profile, self.panel);
        if items.len() < 2 {
            return Ok(());
        }
        let selected = self.panel_selected().min(items.len() - 1);
        let target = add_clamped(selected, delta, items.len() - 1);
        if target == selected {
            return Ok(());
        }
        let uid = items[selected].uid.clone();
        self.move_uid_to(ctx, uid.as_str(), target)?;
        *self.panel_selected_mut() = target;
        self.clamp_panel_state(ctx);
        Ok(())
    }

    fn apply_position_input(&mut self, ctx: &UiContext) -> UiResult<()> {
        let items = panel_items(ctx, &self.profile, self.panel);
        if items.is_empty() {
            return Ok(());
        }
        let Ok(position) = self.position_input.parse::<usize>() else {
            return Ok(());
        };
        let target = position.saturating_sub(1).min(items.len() - 1);
        let selected = self.panel_selected().min(items.len() - 1);
        let uid = items[selected].uid.clone();
        self.move_uid_to(ctx, uid.as_str(), target)?;
        *self.panel_selected_mut() = target;
        self.clamp_panel_state(ctx);
        Ok(())
    }

    fn toggle_panel_item(&mut self, ctx: &UiContext) -> UiResult<()> {
        let items = panel_items(ctx, &self.profile, self.panel);
        if let Some(item) = items.get(self.panel_selected()) {
            let enabled = match self.panel {
                DisplayPanel::Screensaver => &mut self.profile.screensaver_list.enabled,
                DisplayPanel::Boss => &mut self.profile.boss_list.enabled,
                DisplayPanel::None => return Ok(()),
            };
            let current = enabled.get(&item.uid).copied().unwrap_or(true);
            enabled.insert(item.uid.clone(), !current);
            self.save_profile()?;
        }
        Ok(())
    }

    fn move_uid_to(&mut self, ctx: &UiContext, uid: &str, target: usize) -> UiResult<()> {
        self.sync_panel_profile(ctx);
        let list = match self.panel {
            DisplayPanel::Screensaver => &mut self.profile.screensaver_list.order,
            DisplayPanel::Boss => &mut self.profile.boss_list.order,
            DisplayPanel::None => return Ok(()),
        };
        if let Some(current) = list.iter().position(|item| item == uid) {
            let item = list.remove(current);
            let target = target.min(list.len());
            list.insert(target, item);
            self.save_profile()?;
        }
        Ok(())
    }

    fn sync_panel_profile(&mut self, ctx: &UiContext) {
        sync_overlay_profile(
            &mut self.profile.screensaver_list,
            overlay_packages(ctx, DisplayPanel::Screensaver),
        );
        sync_overlay_profile(
            &mut self.profile.boss_list,
            overlay_packages(ctx, DisplayPanel::Boss),
        );
    }

    fn clamp_panel_state(&mut self, ctx: &UiContext) {
        let len = panel_items(ctx, &self.profile, self.panel).len();
        let visible = self.panel_visible_count(ctx).max(1);
        let selected = self.panel_selected_mut();
        if len == 0 {
            *selected = 0;
        } else if *selected >= len {
            *selected = len - 1;
        }
        let selected_value = *selected;
        let scroll = self.panel_scroll_mut();
        if selected_value < *scroll {
            *scroll = selected_value;
        } else if selected_value >= *scroll + visible {
            *scroll = selected_value.saturating_sub(visible - 1);
        }
        *scroll = (*scroll).min(len.saturating_sub(visible));
    }

    fn panel_selected(&self) -> usize {
        match self.panel {
            DisplayPanel::Screensaver => self.screensaver_selected,
            DisplayPanel::Boss => self.boss_selected,
            DisplayPanel::None => 0,
        }
    }

    fn panel_selected_mut(&mut self) -> &mut usize {
        match self.panel {
            DisplayPanel::Screensaver => &mut self.screensaver_selected,
            DisplayPanel::Boss => &mut self.boss_selected,
            DisplayPanel::None => &mut self.screensaver_selected,
        }
    }

    fn panel_scroll(&self) -> usize {
        match self.panel {
            DisplayPanel::Screensaver => self.screensaver_scroll,
            DisplayPanel::Boss => self.boss_scroll,
            DisplayPanel::None => 0,
        }
    }

    fn panel_scroll_mut(&mut self) -> &mut usize {
        match self.panel {
            DisplayPanel::Screensaver => &mut self.screensaver_scroll,
            DisplayPanel::Boss => &mut self.boss_scroll,
            DisplayPanel::None => &mut self.screensaver_scroll,
        }
    }

    fn panel_visible_count(&self, ctx: &UiContext) -> usize {
        ctx.terminal_size.height.saturating_sub(2) as usize
    }

    fn save_profile(&mut self) -> UiResult<()> {
        self.profile.normalize();
        persist_display_profile(&display_state_path(), &self.profile)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DisplayPanel {
    None,
    Screensaver,
    Boss,
}

#[derive(Clone, Debug)]
struct DisplayRow {
    label: String,
    value: Option<DisplayValue>,
}

#[derive(Clone, Debug)]
struct DisplayValue {
    text: String,
    color: ValueColor,
}

#[derive(Clone, Copy, Debug)]
enum ValueColor {
    On,
    Off,
    Neutral,
}

#[derive(Clone, Debug)]
struct PanelItem {
    uid: String,
    name: String,
    is_mod: bool,
    display_enabled: bool,
}

fn display_rows(ctx: &UiContext, profile: &DisplayProfile) -> Vec<DisplayRow> {
    let display = &ctx.i18n.display;
    vec![
        DisplayRow {
            label: display.option_mod.clone(),
            value: Some(DisplayValue {
                text: if profile.mod_badge {
                    display.toggle_mod_on.clone()
                } else {
                    display.toggle_mod_off.clone()
                },
                color: if profile.mod_badge {
                    ValueColor::On
                } else {
                    ValueColor::Off
                },
            }),
        },
        DisplayRow {
            label: display.option_theme.clone(),
            value: Some(DisplayValue {
                text: display.toggle_theme_system.clone(),
                color: ValueColor::Neutral,
            }),
        },
        DisplayRow {
            label: display.option_afk_time.clone(),
            value: Some(DisplayValue {
                text: idle_threshold_text(ctx, profile.idle_threshold),
                color: if profile.idle_threshold == 0 {
                    ValueColor::Off
                } else {
                    ValueColor::On
                },
            }),
        },
        DisplayRow {
            label: display.option_afk_screensaver.clone(),
            value: Some(DisplayValue {
                text: if profile.idle_enter_screensaver {
                    display.toggle_afk_screensaver_on.clone()
                } else {
                    display.toggle_afk_screensaver_off.clone()
                },
                color: if profile.idle_enter_screensaver {
                    ValueColor::On
                } else {
                    ValueColor::Off
                },
            }),
        },
        DisplayRow {
            label: display.option_info.clone(),
            value: Some(DisplayValue {
                text: if profile.host_status {
                    display.option_info_on.clone()
                } else {
                    display.option_info_off.clone()
                },
                color: if profile.host_status {
                    ValueColor::On
                } else {
                    ValueColor::Off
                },
            }),
        },
        DisplayRow {
            label: display.option_screensaver_sort.clone(),
            value: Some(DisplayValue {
                text: mode_text(ctx, &profile.screensaver_mode),
                color: mode_color(&profile.screensaver_mode),
            }),
        },
        DisplayRow {
            label: display.option_boss_sort.clone(),
            value: Some(DisplayValue {
                text: mode_text(ctx, &profile.boss_mode),
                color: mode_color(&profile.boss_mode),
            }),
        },
        DisplayRow {
            label: display.option_screensaver_list.clone(),
            value: None,
        },
        DisplayRow {
            label: display.option_boss_list.clone(),
            value: None,
        },
    ]
}

fn setting_row_width(ctx: &UiContext, index: usize, row: &DisplayRow, selected: bool) -> usize {
    let hint = if selected {
        key_hint(ctx, "confirm", "Enter")
    } else {
        key_hint(ctx, ACTIONS[index], FALLBACK_KEYS[index])
    };
    let value_width = row
        .value
        .as_ref()
        .map(|value| width(format!("[ {} ]", value.text).as_str()) as usize)
        .unwrap_or(0);
    width("▶ ") as usize
        + width(format!("[{hint}] ").as_str()) as usize
        + width(row.label.as_str()) as usize
        + value_width
}

fn panel_items(ctx: &UiContext, profile: &DisplayProfile, panel: DisplayPanel) -> Vec<PanelItem> {
    let packages = overlay_packages(ctx, panel);
    let overlay_profile = match panel {
        DisplayPanel::Screensaver => &profile.screensaver_list,
        DisplayPanel::Boss => &profile.boss_list,
        DisplayPanel::None => return Vec::new(),
    };
    let package_by_uid = packages
        .iter()
        .map(|package| (package.uid.as_str(), *package))
        .collect::<BTreeMap<_, _>>();
    let mut result = Vec::new();
    let mut seen = HashSet::new();
    for uid in &overlay_profile.order {
        if let Some(package) = package_by_uid.get(uid.as_str()) {
            result.push(panel_item(package, overlay_profile));
            seen.insert(uid.clone());
        }
    }
    let mut remaining = packages
        .into_iter()
        .filter(|package| !seen.contains(&package.uid))
        .collect::<Vec<_>>();
    remaining.sort_by(|left, right| {
        left.display_name
            .len()
            .cmp(&right.display_name.len())
            .then_with(|| left.display_name.cmp(&right.display_name))
    });
    for package in remaining {
        result.push(panel_item(package, overlay_profile));
    }
    result
}

fn overlay_packages<'a>(ctx: &'a UiContext, panel: DisplayPanel) -> Vec<&'a OverlayPackage> {
    match panel {
        DisplayPanel::Screensaver => ctx
            .packages
            .screensavers()
            .iter()
            .filter(|package| ctx.packages.screensavers.is_enabled(&package.uid))
            .collect(),
        DisplayPanel::Boss => ctx
            .packages
            .bosses()
            .iter()
            .filter(|package| ctx.packages.bosses.is_enabled(&package.uid))
            .collect(),
        DisplayPanel::None => Vec::new(),
    }
}

fn panel_item(package: &OverlayPackage, profile: &DisplayOverlayProfile) -> PanelItem {
    PanelItem {
        uid: package.uid.clone(),
        name: package.display_name.clone(),
        is_mod: package.id.source == PackageSource::ThirdParty,
        display_enabled: profile.enabled.get(&package.uid).copied().unwrap_or(true),
    }
}

fn sync_overlay_profile(profile: &mut DisplayOverlayProfile, packages: Vec<&OverlayPackage>) {
    let valid_uids = packages
        .iter()
        .map(|package| package.uid.clone())
        .collect::<HashSet<_>>();
    profile.order.retain(|uid| valid_uids.contains(uid));
    profile.enabled.retain(|uid, _| valid_uids.contains(uid));
    for package in packages {
        if !profile.order.contains(&package.uid) {
            profile.order.push(package.uid.clone());
        }
        profile.enabled.entry(package.uid.clone()).or_insert(true);
    }
    if profile.cursor >= profile.order.len() {
        profile.cursor = 0;
    }
}

fn load_profile() -> DisplayProfile {
    fs::read_to_string(display_state_path())
        .ok()
        .and_then(|raw| {
            serde_json::from_str::<serde_json::Value>(raw.trim_start_matches('\u{feff}')).ok()
        })
        .map(|value| {
            let mut profile = DisplayProfile::from_value(&value);
            profile.normalize();
            profile
        })
        .unwrap_or_else(DisplayProfile::default)
}

fn display_state_path() -> PathBuf {
    data_dirs::root_dir().join("data/profiles/display_state.json")
}

fn next_idle_threshold(current: u64) -> u64 {
    let index = IDLE_THRESHOLDS
        .iter()
        .position(|value| *value == current)
        .unwrap_or(1);
    IDLE_THRESHOLDS[(index + 1) % IDLE_THRESHOLDS.len()]
}

fn next_mode(current: &str) -> String {
    match current {
        "ordered" => "random".to_string(),
        "random" => "off".to_string(),
        _ => "ordered".to_string(),
    }
}

fn idle_threshold_text(ctx: &UiContext, value: u64) -> String {
    if value == 0 {
        ctx.i18n.display.toggle_afk_time_never.clone()
    } else if value >= 60 && value.is_multiple_of(60) {
        format!("{}{}", value / 60, ctx.i18n.display.toggle_afk_time_minute)
    } else {
        format!("{}{}", value, ctx.i18n.display.toggle_afk_time_second)
    }
}

fn mode_text(ctx: &UiContext, mode: &str) -> String {
    match mode {
        "random" => ctx.i18n.display.toggle_sort_random.clone(),
        "off" => ctx.i18n.display.toggle_sort_off.clone(),
        _ => ctx.i18n.display.toggle_sort_order.clone(),
    }
}

fn mode_color(mode: &str) -> ValueColor {
    if mode == "off" {
        ValueColor::Off
    } else {
        ValueColor::On
    }
}

fn value_color(ctx: &UiContext, color: ValueColor) -> String {
    match color {
        ValueColor::On => theme_color(ctx, "state.success", "green"),
        ValueColor::Off => theme_color(ctx, "state.danger", "red"),
        ValueColor::Neutral => theme_color(ctx, "accent.primary", "cyan"),
    }
}

fn width(text: &str) -> u16 {
    UnicodeWidthStr::width(text) as u16
}

fn split_x(ctx: &UiContext) -> u16 {
    ((ctx.terminal_size.width as f32) * 0.7).round().max(30.0) as u16
}

fn truncate_with_ellipsis(text: &str, max_width: usize) -> String {
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
    let mut current_width = 0;
    for ch in text.chars() {
        let ch_width = UnicodeWidthChar::width(ch).unwrap_or(0);
        if current_width + ch_width > max_width - 3 {
            break;
        }
        result.push(ch);
        current_width += ch_width;
    }
    result.push_str("...");
    result
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

fn add_clamped(current: usize, delta: isize, max_value: usize) -> usize {
    if delta < 0 {
        current.saturating_sub(delta.unsigned_abs())
    } else {
        current.saturating_add(delta as usize).min(max_value)
    }
}
