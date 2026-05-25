//! Rust implementation of keybind settings hub.

use unicode_width::UnicodeWidthStr;

use crate::host_engine::boot::preload::lua_runtime::api::drawing_support::drawing_parser::STYLE_BOLD;
use crate::host_engine::runtime::ui::pages::common::{
    MenuCommand, draw_title, key_hint, selected_menu_event, take_navigation, theme_color,
};
use crate::host_engine::runtime::ui::{Canvas, UiContext, UiEvent, UiNavigation, UiPage, UiResult};
use crate::host_engine::runtime::ui_page::page_key::UiPageKey;

const ITEM_COUNT: usize = 3;
const ACTIONS: [&str; ITEM_COUNT] = ["option1", "option2", "option3"];
const FALLBACK_KEYS: [&str; ITEM_COUNT] = ["1", "2", "3"];

pub struct SettingKeybindPage {
    selected_index: usize,
    pending_navigation: Option<UiNavigation>,
}

impl SettingKeybindPage {
    pub fn new() -> Self {
        Self {
            selected_index: 0,
            pending_navigation: None,
        }
    }
}

impl UiPage for SettingKeybindPage {
    fn page_key(&self) -> UiPageKey {
        UiPageKey::SettingKeybind
    }

    fn handle_event(&mut self, event: &UiEvent, _ctx: &mut UiContext) -> UiResult<()> {
        match selected_menu_event(&mut self.selected_index, ITEM_COUNT, event) {
            Some(MenuCommand::Confirm) => {
                self.pending_navigation = match self.selected_index {
                    0 => Some(UiNavigation::Page(UiPageKey::KeybindSystem)),
                    1 => Some(UiNavigation::Page(UiPageKey::KeybindSystem)),
                    2 => Some(UiNavigation::Page(UiPageKey::KeybindSystem)),
                    _ => None,
                };
            }
            Some(MenuCommand::Back) => {
                self.pending_navigation = Some(UiNavigation::Page(UiPageKey::Setting));
            }
            _ => {}
        }
        Ok(())
    }

    fn render(&self, canvas: &mut Canvas, ctx: &UiContext) -> UiResult<()> {
        canvas.clear()?;
        draw_title(canvas, ctx, &ctx.i18n.setting_keybind.system_list_title)?;
        let labels = [
            ctx.i18n.setting_keybind.list_global.clone(),
            ctx.i18n.setting_keybind.list_system.clone(),
            ctx.i18n.setting_keybind.list_game.clone(),
        ];
        self.render_menu(canvas, ctx, &labels)?;
        self.render_footer(canvas, ctx)?;
        Ok(())
    }

    fn take_navigation(&mut self) -> Option<UiNavigation> {
        take_navigation(&mut self.pending_navigation)
    }
}

impl SettingKeybindPage {
    fn render_menu(
        &self,
        canvas: &mut Canvas,
        ctx: &UiContext,
        labels: &[String; 3],
    ) -> UiResult<()> {
        let menu_width = labels
            .iter()
            .enumerate()
            .map(|(index, label)| {
                UnicodeWidthStr::width("▶ ")
                    + UnicodeWidthStr::width(
                        format!("[{}] ", menu_key_hint(ctx, index, true)).as_str(),
                    )
                    + UnicodeWidthStr::width(label.as_str())
            })
            .max()
            .unwrap_or(0) as u16;
        let x = ctx.terminal_size.width.saturating_sub(menu_width) / 2;
        let y = ctx.terminal_size.height.saturating_sub(ITEM_COUNT as u16) / 2;
        for (index, label) in labels.iter().enumerate() {
            let selected = index == self.selected_index;
            let row_y = y.saturating_add(index as u16);
            let marker = if selected { "▶ " } else { "  " };
            canvas.draw_text_styled(
                x,
                row_y,
                marker,
                Some(if selected {
                    theme_color(ctx, "accent.primary", "cyan")
                } else {
                    theme_color(ctx, "text.primary", "white")
                }),
                None,
                vec![STYLE_BOLD],
            )?;
            let hint = menu_key_hint(ctx, index, selected);
            let key_text = format!("[{hint}] ");
            let key_x = x.saturating_add(UnicodeWidthStr::width(marker) as u16);
            canvas.draw_text_styled(
                key_x,
                row_y,
                key_text.as_str(),
                Some(theme_color(ctx, "text.muted", "dark_gray")),
                None,
                vec![STYLE_BOLD],
            )?;
            let label_x = key_x.saturating_add(UnicodeWidthStr::width(key_text.as_str()) as u16);
            canvas.draw_text_styled(
                label_x,
                row_y,
                label,
                Some(if selected {
                    theme_color(ctx, "accent.primary", "cyan")
                } else {
                    theme_color(ctx, "text.primary", "white")
                }),
                None,
                vec![STYLE_BOLD],
            )?;
        }
        Ok(())
    }

    fn render_footer(&self, canvas: &mut Canvas, ctx: &UiContext) -> UiResult<()> {
        let footer = format!(
            "[{}]/[{}] {}   [{}] {}   [{}] {}",
            key_hint(ctx, "prev_option", "↑"),
            key_hint(ctx, "next_option", "↓"),
            ctx.i18n.key.setting_keybind_list_select,
            key_hint(ctx, "confirm", "Enter"),
            ctx.i18n.key.setting_keybind_list_confirm,
            key_hint(ctx, "back", "Esc"),
            ctx.i18n.key.setting_keybind_list_back,
        );
        let x = ctx
            .terminal_size
            .width
            .saturating_sub(UnicodeWidthStr::width(footer.as_str()) as u16)
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
}

fn menu_key_hint(ctx: &UiContext, index: usize, selected: bool) -> String {
    if selected {
        key_hint(ctx, "confirm", "Enter")
    } else {
        key_hint(ctx, ACTIONS[index], FALLBACK_KEYS[index])
    }
}
