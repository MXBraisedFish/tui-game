//! Rust implementation of the Mod hub page.

use crate::host_engine::runtime::ui::pages::common::{
    MenuCommand, draw_center_menu, draw_title, key_hint, selected_menu_event,
    take_navigation, theme_color,
};
use crate::host_engine::boot::preload::lua_runtime::api::drawing_support::drawing_parser::STYLE_BOLD;
use unicode_width::UnicodeWidthStr;
use crate::host_engine::runtime::ui::{Canvas, UiContext, UiEvent, UiNavigation, UiPage, UiResult};
use crate::host_engine::runtime::ui_page::page_key::UiPageKey;

const ACTIONS: [&str; 3] = ["option1", "option2", "option3"];
const FALLBACK_KEYS: [&str; 3] = ["1", "2", "3"];

pub struct ModHubPage {
    selected_index: usize,
    pending_navigation: Option<UiNavigation>,
}

impl ModHubPage {
    pub fn new() -> Self {
        Self {
            selected_index: 0,
            pending_navigation: None,
        }
    }
}

impl UiPage for ModHubPage {
    fn page_key(&self) -> UiPageKey {
        UiPageKey::SettingMods
    }

    fn handle_event(&mut self, event: &UiEvent, _ctx: &mut UiContext) -> UiResult<()> {
        match selected_menu_event(&mut self.selected_index, 3, event) {
            Some(MenuCommand::Confirm) => {
                self.pending_navigation = Some(UiNavigation::Page(match self.selected_index {
                    0 => UiPageKey::ModGameList,
                    1 => UiPageKey::ModScreensaverList,
                    2 => UiPageKey::ModBossList,
                    _ => UiPageKey::ModGameList,
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
        draw_title(canvas, ctx, &ctx.i18n.setting.mods)?;
        let labels = vec![
            ctx.i18n.mod_hub.game.clone(),
            ctx.i18n.mod_hub.screensaver.clone(),
            ctx.i18n.mod_hub.boss.clone(),
        ];
        draw_center_menu(
            canvas,
            ctx,
            &labels,
            self.selected_index,
            &ACTIONS,
            &FALLBACK_KEYS,
        )?;
        let up_key = key_hint(ctx, "prev_option", "↑");
        let down_key = key_hint(ctx, "next_option", "↓");
        let confirm_key = key_hint(ctx, "confirm", "Enter");
        let back_key = key_hint(ctx, "back", "Esc");
        let select = &ctx.i18n.key.mod_hub_select;
        let confirm = &ctx.i18n.key.mod_hub_confirm;
        let back = &ctx.i18n.key.mod_hub_back;
        let footer = format!(
            "[{up_key}]/[{down_key}] {select}   [{confirm_key}] {confirm}   [{back_key}] {back}",
        );
        let width = UnicodeWidthStr::width(footer.as_str()) as u16;
        let x = ctx.terminal_size.width.saturating_sub(width) / 2;
        canvas.draw_text_styled(
            x,
            ctx.terminal_size.height.saturating_sub(1),
            &footer,
            Some(theme_color(ctx, "text.muted", "dark_gray")),
            None,
            vec![STYLE_BOLD],
        )?;
        Ok(())
    }

    fn take_navigation(&mut self) -> Option<UiNavigation> {
        take_navigation(&mut self.pending_navigation)
    }
}
