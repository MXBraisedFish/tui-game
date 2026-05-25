//! Rust implementation of security settings page.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value};
use unicode_width::UnicodeWidthStr;

use crate::host_engine::boot::environment::data_dirs;
use crate::host_engine::boot::preload::lua_runtime::api::drawing_support::drawing_parser::STYLE_BOLD;
use crate::host_engine::boot::preload::persistent_data::security_profile::{
    SecurityProfile, load_from_default_path, persist_to_default_path,
};
use crate::host_engine::runtime::ui::pages::common::{
    MenuCommand, draw_title, key_hint, selected_menu_event, take_navigation, theme_color,
};
use crate::host_engine::runtime::ui::{Canvas, UiContext, UiEvent, UiNavigation, UiPage, UiResult};
use crate::host_engine::runtime::ui_page::page_key::UiPageKey;

const ITEM_COUNT: usize = 8;
const ACTIONS: [&str; ITEM_COUNT] = [
    "option1", "option2", "option3", "option4", "option5", "option6", "option7", "option8",
];
const FALLBACK_KEYS: [&str; ITEM_COUNT] = ["1", "2", "3", "4", "5", "6", "7", "8"];

pub struct SettingSecurityPage {
    selected_index: usize,
    reset_message: Option<String>,
    pending_navigation: Option<UiNavigation>,
}

impl SettingSecurityPage {
    pub fn new() -> Self {
        Self {
            selected_index: 0,
            reset_message: None,
            pending_navigation: None,
        }
    }
}

impl UiPage for SettingSecurityPage {
    fn page_key(&self) -> UiPageKey {
        UiPageKey::SettingSecurity
    }

    fn handle_event(&mut self, event: &UiEvent, ctx: &mut UiContext) -> UiResult<()> {
        match selected_menu_event(&mut self.selected_index, ITEM_COUNT, event) {
            Some(MenuCommand::Back) => {
                self.pending_navigation = Some(UiNavigation::Page(UiPageKey::Setting));
            }
            Some(MenuCommand::Confirm) => self.confirm_selected(ctx)?,
            _ => {}
        }
        Ok(())
    }

    fn render(&self, canvas: &mut Canvas, ctx: &UiContext) -> UiResult<()> {
        canvas.clear()?;
        draw_title(canvas, ctx, &ctx.i18n.security.title)?;
        self.render_options(canvas, ctx, &load_from_default_path())?;
        self.render_reset_message(canvas, ctx)?;
        self.render_footer(canvas, ctx)?;
        Ok(())
    }

    fn take_navigation(&mut self) -> Option<UiNavigation> {
        take_navigation(&mut self.pending_navigation)
    }
}

impl SettingSecurityPage {
    fn confirm_selected(&mut self, ctx: &UiContext) -> UiResult<()> {
        self.reset_message = None;
        let mut profile = load_from_default_path();
        match self.selected_index {
            0 => {
                if profile.default_safe_mode {
                    self.pending_navigation = Some(UiNavigation::Page(UiPageKey::WarningSecurity));
                } else {
                    profile.default_safe_mode = true;
                    persist_to_default_path(&profile)?;
                }
            }
            1 => {
                profile.default_mod_game_enabled = !profile.default_mod_game_enabled;
                persist_to_default_path(&profile)?;
            }
            2 => {
                profile.default_mod_screensaver_enabled = !profile.default_mod_screensaver_enabled;
                persist_to_default_path(&profile)?;
            }
            3 => {
                profile.default_mod_boss_enabled = !profile.default_mod_boss_enabled;
                persist_to_default_path(&profile)?;
            }
            4 => self.set_reset_message(ctx, reset_game_safe_mode()),
            5 => self.set_reset_message(ctx, reset_enabled_state("game_state.json")),
            6 => self.set_reset_message(ctx, reset_enabled_state("screensaver_state")),
            7 => self.set_reset_message(ctx, reset_enabled_state("boss_state")),
            _ => {}
        }
        Ok(())
    }

    fn set_reset_message(&mut self, ctx: &UiContext, result: UiResult<()>) {
        self.reset_message = Some(if result.is_ok() {
            ctx.i18n.security.reset_success.clone()
        } else {
            ctx.i18n.security.reset_failed.clone()
        });
    }

    fn render_options(
        &self,
        canvas: &mut Canvas,
        ctx: &UiContext,
        profile: &SecurityProfile,
    ) -> UiResult<()> {
        let items = security_items(ctx, profile);
        let menu_width = items
            .iter()
            .enumerate()
            .map(|(index, item)| option_width(ctx, index, item, index == self.selected_index))
            .max()
            .unwrap_or(0) as u16;
        let x = ctx.terminal_size.width.saturating_sub(menu_width) / 2;
        let y = ctx.terminal_size.height.saturating_sub(ITEM_COUNT as u16) / 2;
        for (index, item) in items.iter().enumerate() {
            let row_y = y.saturating_add(index as u16);
            let selected = index == self.selected_index;
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
                item.label.as_str(),
                Some(if selected {
                    theme_color(ctx, "accent.primary", "cyan")
                } else {
                    theme_color(ctx, "text.primary", "white")
                }),
                None,
                vec![STYLE_BOLD],
            )?;
            if let Some(state) = item.state.as_ref() {
                let bracket_x =
                    label_x.saturating_add(UnicodeWidthStr::width(item.label.as_str()) as u16);
                canvas.draw_text_styled(
                    bracket_x,
                    row_y,
                    "[ ",
                    Some(theme_color(ctx, "text.primary", "white")),
                    None,
                    vec![STYLE_BOLD],
                )?;
                canvas.draw_text_styled(
                    bracket_x.saturating_add(2),
                    row_y,
                    state.text.as_str(),
                    Some(if state.enabled {
                        theme_color(ctx, "state.success", "green")
                    } else {
                        theme_color(ctx, "state.danger", "red")
                    }),
                    None,
                    vec![STYLE_BOLD],
                )?;
                let state_width = UnicodeWidthStr::width(state.text.as_str()) as u16;
                canvas.draw_text_styled(
                    bracket_x.saturating_add(2).saturating_add(state_width),
                    row_y,
                    " ]",
                    Some(theme_color(ctx, "text.primary", "white")),
                    None,
                    vec![STYLE_BOLD],
                )?;
            }
        }
        Ok(())
    }

    fn render_reset_message(&self, canvas: &mut Canvas, ctx: &UiContext) -> UiResult<()> {
        let Some(message) = self.reset_message.as_ref() else {
            return Ok(());
        };
        let width = UnicodeWidthStr::width(message.as_str()) as u16;
        let x = ctx.terminal_size.width.saturating_sub(width) / 2;
        canvas.draw_text_styled(
            x,
            ctx.terminal_size.height.saturating_sub(3),
            message,
            Some(theme_color(ctx, "state.success", "green")),
            None,
            vec![STYLE_BOLD],
        )
    }

    fn render_footer(&self, canvas: &mut Canvas, ctx: &UiContext) -> UiResult<()> {
        let footer = format!(
            "[{}]/[{}] {}   [{}] {}   [{}] {}",
            key_hint(ctx, "prev_option", "↑"),
            key_hint(ctx, "next_option", "↓"),
            ctx.i18n.key.security_select,
            key_hint(ctx, "confirm", "Enter"),
            ctx.i18n.key.security_toggle_confirm,
            key_hint(ctx, "back", "Esc"),
            ctx.i18n.key.security_back,
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

#[derive(Clone, Debug)]
struct SecurityItem {
    label: String,
    state: Option<SecurityItemState>,
}

#[derive(Clone, Debug)]
struct SecurityItemState {
    text: String,
    enabled: bool,
}

fn security_items(ctx: &UiContext, profile: &SecurityProfile) -> Vec<SecurityItem> {
    vec![
        SecurityItem {
            label: ctx.i18n.security.default_safe_mode.clone(),
            state: Some(SecurityItemState {
                text: if profile.default_safe_mode {
                    ctx.i18n.security.toggle_safe_mode_on.clone()
                } else {
                    ctx.i18n.security.toggle_safe_mode_off_permanent.clone()
                },
                enabled: profile.default_safe_mode,
            }),
        },
        mod_toggle_item(
            ctx.i18n.security.default_mod_game.clone(),
            profile.default_mod_game_enabled,
            ctx,
        ),
        mod_toggle_item(
            ctx.i18n.security.default_mod_screensaver.clone(),
            profile.default_mod_screensaver_enabled,
            ctx,
        ),
        mod_toggle_item(
            ctx.i18n.security.default_mod_boss.clone(),
            profile.default_mod_boss_enabled,
            ctx,
        ),
        SecurityItem {
            label: ctx.i18n.security.reset_safe_mode.clone(),
            state: None,
        },
        SecurityItem {
            label: ctx.i18n.security.reset_mod_game.clone(),
            state: None,
        },
        SecurityItem {
            label: ctx.i18n.security.reset_mod_screensaver.clone(),
            state: None,
        },
        SecurityItem {
            label: ctx.i18n.security.reset_mod_boss.clone(),
            state: None,
        },
    ]
}

fn mod_toggle_item(label: String, enabled: bool, ctx: &UiContext) -> SecurityItem {
    SecurityItem {
        label,
        state: Some(SecurityItemState {
            text: if enabled {
                ctx.i18n.security.toggle_mod_on.clone()
            } else {
                ctx.i18n.security.toggle_mod_off.clone()
            },
            enabled,
        }),
    }
}

fn option_width(ctx: &UiContext, index: usize, item: &SecurityItem, selected: bool) -> usize {
    let hint = menu_key_hint(ctx, index, selected);
    let state_width = item
        .state
        .as_ref()
        .map(|state| UnicodeWidthStr::width(format!("[ {} ]", state.text).as_str()))
        .unwrap_or(0);
    UnicodeWidthStr::width("▶ ")
        + UnicodeWidthStr::width(format!("[{hint}] ").as_str())
        + UnicodeWidthStr::width(item.label.as_str())
        + state_width
}

fn menu_key_hint(ctx: &UiContext, index: usize, selected: bool) -> String {
    if selected {
        key_hint(ctx, "confirm", "Enter")
    } else {
        key_hint(ctx, ACTIONS[index], FALLBACK_KEYS[index])
    }
}

fn reset_game_safe_mode() -> UiResult<()> {
    update_state_file("game_state.json", |state| {
        state.insert("safe_mode".to_string(), Value::Bool(true));
        state.insert("safe_mode_permanent".to_string(), Value::Bool(true));
    })
}

fn reset_enabled_state(file_name: &str) -> UiResult<()> {
    update_state_file(file_name, |state| {
        state.insert("enabled".to_string(), Value::Bool(false));
    })
}

fn update_state_file(file_name: &str, update: impl Fn(&mut Map<String, Value>)) -> UiResult<()> {
    let path = profile_path(file_name);
    let mut root = read_json_object(&path);
    for value in root.values_mut() {
        if let Value::Object(state) = value {
            update(state);
        }
    }
    write_json_object(&path, &root)?;
    Ok(())
}

fn profile_path(file_name: &str) -> PathBuf {
    data_dirs::root_dir().join("data/profiles").join(file_name)
}

fn read_json_object(path: &Path) -> Map<String, Value> {
    fs::read_to_string(path)
        .ok()
        .and_then(|raw| serde_json::from_str::<Value>(raw.trim_start_matches('\u{feff}')).ok())
        .and_then(|value| value.as_object().cloned())
        .unwrap_or_default()
}

fn write_json_object(path: &Path, value: &Map<String, Value>) -> UiResult<()> {
    if let Some(parent_dir) = path.parent() {
        fs::create_dir_all(parent_dir)?;
    }
    fs::write(
        path,
        serde_json::to_string_pretty(&Value::Object(value.clone()))?,
    )?;
    Ok(())
}
