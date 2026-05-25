//! Rust implementation of memory management page.

use unicode_width::UnicodeWidthStr;

use crate::host_engine::boot::preload::lua_runtime::api::drawing_support::drawing_parser::STYLE_BOLD;
use crate::host_engine::runtime::ui::pages::common::{
    MenuCommand, draw_title, key_hint, selected_menu_event, take_navigation, theme_color,
};
use crate::host_engine::runtime::ui::{Canvas, UiContext, UiEvent, UiNavigation, UiPage, UiResult};
use crate::host_engine::runtime::ui_page::page_key::UiPageKey;

const ACTIONS: [&str; 3] = ["option1", "option2", "option3"];
const FALLBACK_KEYS: [&str; 3] = ["1", "2", "3"];

pub struct SettingMemoryPage {
    selected_index: usize,
    pending_navigation: Option<UiNavigation>,
}

impl SettingMemoryPage {
    pub fn new() -> Self {
        Self {
            selected_index: 0,
            pending_navigation: None,
        }
    }
}

impl UiPage for SettingMemoryPage {
    fn page_key(&self) -> UiPageKey {
        UiPageKey::SettingMemory
    }

    fn handle_event(&mut self, event: &UiEvent, _ctx: &mut UiContext) -> UiResult<()> {
        match selected_menu_event(&mut self.selected_index, 3, event) {
            Some(MenuCommand::Confirm) => {
                self.pending_navigation = Some(UiNavigation::Page(match self.selected_index {
                    0 => UiPageKey::WarningClearCache,
                    1 => UiPageKey::WarningClearData,
                    _ => UiPageKey::StorageDetails,
                }));
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
        draw_title(canvas, ctx, &ctx.i18n.memory.title)?;
        self.render_options(canvas, ctx)?;
        self.render_footer(canvas, ctx)?;
        Ok(())
    }

    fn take_navigation(&mut self) -> Option<UiNavigation> {
        take_navigation(&mut self.pending_navigation)
    }
}

impl SettingMemoryPage {
    fn render_options(&self, canvas: &mut Canvas, ctx: &UiContext) -> UiResult<()> {
        let labels = [
            ctx.i18n.memory.cache.as_str(),
            ctx.i18n.memory.data.as_str(),
            ctx.i18n.memory.show.as_str(),
        ];
        let menu_width = labels
            .iter()
            .enumerate()
            .map(|(index, label)| {
                let hint = menu_key_hint(ctx, index, index == self.selected_index);
                UnicodeWidthStr::width("▶ ")
                    + UnicodeWidthStr::width(format!("[{hint}] ").as_str())
                    + UnicodeWidthStr::width(*label)
            })
            .max()
            .unwrap_or(0) as u16;
        let x = ctx.terminal_size.width.saturating_sub(menu_width) / 2;
        let y = ctx.terminal_size.height.saturating_sub(labels.len() as u16) / 2;

        for (index, label) in labels.iter().enumerate() {
            let selected = index == self.selected_index;
            let row_y = y.saturating_add(index as u16);
            let marker = if selected { "▶ " } else { "  " };
            let marker_color = if selected {
                theme_color(ctx, "accent.primary", "cyan")
            } else {
                theme_color(ctx, "text.primary", "white")
            };
            let hint = menu_key_hint(ctx, index, selected);
            let key_text = format!("[{hint}] ");
            let key_x = x.saturating_add(UnicodeWidthStr::width(marker) as u16);
            let label_x = key_x.saturating_add(UnicodeWidthStr::width(key_text.as_str()) as u16);
            canvas.draw_text_styled(
                x,
                row_y,
                marker,
                Some(marker_color),
                None,
                vec![STYLE_BOLD],
            )?;
            canvas.draw_text_styled(
                key_x,
                row_y,
                key_text,
                Some(theme_color(ctx, "text.muted", "dark_gray")),
                None,
                vec![STYLE_BOLD],
            )?;
            canvas.draw_text_styled(
                label_x,
                row_y,
                *label,
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
            ctx.i18n.key.memory_select,
            key_hint(ctx, "confirm", "Enter"),
            ctx.i18n.key.memory_confirm,
            key_hint(ctx, "back", "Esc"),
            ctx.i18n.key.memory_back,
        );
        let width = UnicodeWidthStr::width(footer.as_str()) as u16;
        let x = ctx.terminal_size.width.saturating_sub(width) / 2;
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
