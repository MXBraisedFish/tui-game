//! Rust implementation of the settings hub page.

use unicode_width::UnicodeWidthStr;

use crate::host_engine::boot::preload::lua_runtime::api::drawing_support::drawing_parser::STYLE_BOLD;
use crate::host_engine::runtime::ui::pages::common::theme_color;
use crate::host_engine::runtime::ui::{Canvas, UiContext, UiEvent, UiNavigation, UiPage, UiResult};
use crate::host_engine::runtime::ui_page::page_key::UiPageKey;

const MENU_ACTIONS: [&str; 6] = [
    "option1", "option2", "option3", "option4", "option5", "option6",
];
const DEFAULT_MENU_KEYS: [&str; 6] = ["1", "2", "3", "4", "5", "6"];

pub struct SettingPage {
    selected_index: usize,
    pending_navigation: Option<UiNavigation>,
}

impl Default for SettingPage {
    fn default() -> Self {
        Self::new()
    }
}

impl SettingPage {
    pub fn new() -> Self {
        Self {
            selected_index: 0,
            pending_navigation: None,
        }
    }

    fn select_previous(&mut self) {
        self.selected_index = if self.selected_index == 0 {
            5
        } else {
            self.selected_index - 1
        };
    }

    fn select_next(&mut self) {
        self.selected_index = (self.selected_index + 1) % 6;
    }

    fn confirm(&mut self) {
        self.pending_navigation = Some(UiNavigation::Page(match self.selected_index {
            0 => UiPageKey::SettingLanguage,
            1 => UiPageKey::SettingKeybind,
            2 => UiPageKey::SettingMods,
            3 => UiPageKey::SettingMemory,
            4 => UiPageKey::SettingSecurity,
            5 => UiPageKey::SettingDisplay,
            _ => UiPageKey::SettingLanguage,
        }));
    }

    fn labels(ctx: &UiContext) -> [String; 6] {
        [
            ctx.i18n.setting.language.clone(),
            ctx.i18n.setting.keybind.clone(),
            ctx.i18n.setting.mods.clone(),
            ctx.i18n.setting.memory.clone(),
            ctx.i18n.setting.security.clone(),
            ctx.i18n.setting.display.clone(),
        ]
    }
}

impl UiPage for SettingPage {
    fn page_key(&self) -> UiPageKey {
        UiPageKey::Setting
    }

    fn handle_event(&mut self, event: &UiEvent, _ctx: &mut UiContext) -> UiResult<()> {
        match event {
            UiEvent::Action { name, status } | UiEvent::Key { name, status } => {
                if !is_press(status) {
                    return Ok(());
                }
                match name.as_str() {
                    "prev_option" | "up" | "arrowup" => self.select_previous(),
                    "next_option" | "down" | "arrowdown" => self.select_next(),
                    "option1" | "1" => self.selected_index = 0,
                    "option2" | "2" => self.selected_index = 1,
                    "option3" | "3" => self.selected_index = 2,
                    "option4" | "4" => self.selected_index = 3,
                    "option5" | "5" => self.selected_index = 4,
                    "option6" | "6" => self.selected_index = 5,
                    "confirm" | "enter" => self.confirm(),
                    "back" | "return" | "esc" | "q" => {
                        self.pending_navigation = Some(UiNavigation::Page(UiPageKey::Home));
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn render(&self, canvas: &mut Canvas, ctx: &UiContext) -> UiResult<()> {
        canvas.clear()?;

        let terminal_width = ctx.terminal_size.width;
        let terminal_height = ctx.terminal_size.height;
        let title = ctx.i18n.setting.title.as_str();
        let title_width = UnicodeWidthStr::width(title) as u16;
        let title_x = terminal_width.saturating_sub(title_width) / 2;
        canvas.draw_text_styled(
            title_x,
            1,
            title,
            Some(theme_color(ctx, "text.primary", "white")),
            None,
            vec![STYLE_BOLD],
        )?;

        let labels = Self::labels(ctx);
        let menu_width = menu_width(ctx, &labels) as u16;
        let menu_x = terminal_width.saturating_sub(menu_width) / 2;
        let menu_y = terminal_height.saturating_sub(6) / 2;

        for (index, label) in labels.iter().enumerate() {
            let y = menu_y.saturating_add(index as u16);
            let marker = if index == self.selected_index {
                "▶ "
            } else {
                "  "
            };
            canvas.draw_text_styled(
                menu_x,
                y,
                marker,
                Some(if index == self.selected_index {
                    theme_color(ctx, "accent.primary", "cyan")
                } else {
                    theme_color(ctx, "text.primary", "white")
                }),
                None,
                vec![STYLE_BOLD],
            )?;
            let hint = menu_key_hint(ctx, index, index == self.selected_index);
            let key_text = format!("[{hint}]");
            let key_x = menu_x.saturating_add(UnicodeWidthStr::width(marker) as u16);
            canvas.draw_text_styled(
                key_x,
                y,
                key_text.as_str(),
                Some(theme_color(ctx, "text.muted", "dark_gray")),
                None,
                vec![STYLE_BOLD],
            )?;
            let label_x = key_x
                .saturating_add(UnicodeWidthStr::width(key_text.as_str()) as u16)
                .saturating_add(1);
            canvas.draw_text_styled(
                label_x,
                y,
                label,
                if index == self.selected_index {
                    Some(theme_color(ctx, "accent.primary", "cyan"))
                } else {
                    Some(theme_color(ctx, "text.primary", "white"))
                },
                None,
                vec![STYLE_BOLD],
            )?;
        }

        let footer = footer_hint_text(ctx);
        let footer_x =
            terminal_width.saturating_sub(UnicodeWidthStr::width(footer.as_str()) as u16) / 2;
        let footer_y = terminal_height.saturating_sub(1);
        canvas.draw_text_styled(
            footer_x,
            footer_y,
            footer.as_str(),
            Some(theme_color(ctx, "text.muted", "dark_gray")),
            None,
            Vec::new(),
        )?;

        Ok(())
    }

    fn take_navigation(&mut self) -> Option<UiNavigation> {
        self.pending_navigation.take()
    }
}

fn is_press(status: &str) -> bool {
    matches!(status, "press" | "pressed" | "down")
}

fn key_hint(ctx: &UiContext, action: &str, fallback: &str) -> String {
    ctx.action_hints
        .get(action)
        .cloned()
        .unwrap_or_else(|| fallback.to_string())
}

fn menu_width(ctx: &UiContext, labels: &[String; 6]) -> usize {
    labels
        .iter()
        .enumerate()
        .map(|(index, label)| {
            UnicodeWidthStr::width("▶ ")
                + UnicodeWidthStr::width(format!("[{}]", menu_key_hint(ctx, index, true)).as_str())
                + 1
                + UnicodeWidthStr::width(label.as_str())
        })
        .max()
        .unwrap_or(34)
}

fn menu_key_hint(ctx: &UiContext, index: usize, selected: bool) -> String {
    if selected {
        key_hint(ctx, "confirm", "Enter")
    } else {
        key_hint(ctx, MENU_ACTIONS[index], DEFAULT_MENU_KEYS[index])
    }
}

fn footer_hint_text(ctx: &UiContext) -> String {
    let prev_key = key_hint(ctx, "prev_option", "↑");
    let next_key = key_hint(ctx, "next_option", "↓");
    let confirm_key = key_hint(ctx, "confirm", "Enter");
    let back_key = key_hint(ctx, "back", "Esc");
    format!(
        "[{prev_key}/{next_key}] {}  [{confirm_key}] {}  [{back_key}] {}",
        ctx.i18n.key.setting_select, ctx.i18n.key.setting_confirm, ctx.i18n.key.setting_back
    )
}
