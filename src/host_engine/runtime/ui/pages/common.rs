//! Shared helpers for Rust UI pages.

use unicode_width::UnicodeWidthStr;

use crate::host_engine::boot::preload::lua_runtime::api::drawing_support::drawing_parser::STYLE_BOLD;
use crate::host_engine::runtime::ui::{Canvas, UiContext, UiEvent, UiNavigation, UiResult};
use crate::host_engine::runtime::ui_page::page_key::UiPageKey;

pub fn is_press(status: &str) -> bool {
    matches!(status, "press" | "pressed" | "down")
}

pub fn key_hint(ctx: &UiContext, action: &str, fallback: &str) -> String {
    ctx.action_hints
        .get(action)
        .cloned()
        .unwrap_or_else(|| fallback.to_string())
}

pub fn draw_title(canvas: &mut Canvas, ctx: &UiContext, title: &str) -> UiResult<()> {
    let width = UnicodeWidthStr::width(title) as u16;
    let x = ctx.terminal_size.width.saturating_sub(width) / 2;
    canvas.draw_text_styled(
        x,
        1,
        title,
        Some(theme_color(ctx, "text.primary", "white")),
        None,
        vec![STYLE_BOLD],
    )
}

pub fn draw_footer(canvas: &mut Canvas, ctx: &UiContext, text: &str) -> UiResult<()> {
    let width = UnicodeWidthStr::width(text) as u16;
    let x = ctx.terminal_size.width.saturating_sub(width) / 2;
    canvas.draw_text_styled(
        x,
        ctx.terminal_size.height.saturating_sub(2),
        text,
        Some(theme_color(ctx, "text.muted", "dark_gray")),
        None,
        vec![STYLE_BOLD],
    )
}

pub fn draw_center_menu(
    canvas: &mut Canvas,
    ctx: &UiContext,
    labels: &[String],
    selected_index: usize,
    actions: &[&str],
    fallback_keys: &[&str],
) -> UiResult<()> {
    let confirm_fallback = "Enter";
    let width = labels
        .iter()
        .enumerate()
        .map(|(index, label)| {
            let action = if index == selected_index {
                "confirm"
            } else {
                actions.get(index).copied().unwrap_or("confirm")
            };
            UnicodeWidthStr::width(label.as_str())
                + key_hint(ctx, action, confirm_fallback).len()
                + 6
        })
        .max()
        .unwrap_or(30) as u16;
    let x = ctx.terminal_size.width.saturating_sub(width) / 2;
    let y = ctx.terminal_size.height.saturating_sub(labels.len() as u16) / 2;

    for (index, label) in labels.iter().enumerate() {
        let row_y = y.saturating_add(index as u16);
        let selected = index == selected_index;
        let marker = if selected { "▶ " } else { "  " };
        let action = if selected {
            "confirm"
        } else {
            actions.get(index).copied().unwrap_or("confirm")
        };
        let fallback = if selected {
            confirm_fallback
        } else {
            fallback_keys.get(index).copied().unwrap_or("Enter")
        };
        let hint = key_hint(ctx, action, fallback);
        canvas.draw_text_styled(
            x,
            row_y,
            marker,
            Some(theme_color(ctx, "accent.primary", "cyan")),
            None,
            Vec::new(),
        )?;
        canvas.draw_text_styled(
            x.saturating_add(2),
            row_y,
            format!("[{hint}]"),
            Some(theme_color(ctx, "text.muted", "dark_gray")),
            None,
            Vec::new(),
        )?;
        canvas.draw_text_styled(
            x.saturating_add(2 + hint.len() as u16 + 3),
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

pub fn selected_menu_event(
    selected_index: &mut usize,
    item_count: usize,
    event: &UiEvent,
) -> Option<MenuCommand> {
    let (UiEvent::Action { name, status } | UiEvent::Key { name, status }) = event else {
        return None;
    };
    if !is_press(status) {
        return None;
    }
    if let Some(index) = option_index(name, item_count) {
        *selected_index = index;
        return Some(MenuCommand::Move);
    }
    match name.as_str() {
        "prev_option" | "up" | "arrowup" => {
            *selected_index = if *selected_index == 0 {
                item_count.saturating_sub(1)
            } else {
                (*selected_index).saturating_sub(1)
            };
            Some(MenuCommand::Move)
        }
        "next_option" | "down" | "arrowdown" => {
            if item_count > 0 {
                *selected_index = (*selected_index + 1) % item_count;
            }
            Some(MenuCommand::Move)
        }
        "confirm" | "enter" => Some(MenuCommand::Confirm),
        "back" | "return" | "esc" | "q" => Some(MenuCommand::Back),
        _ => None,
    }
}

fn option_index(name: &str, item_count: usize) -> Option<usize> {
    let option_number = name
        .strip_prefix("option")
        .unwrap_or(name)
        .parse::<usize>()
        .ok()?;
    let index = option_number.checked_sub(1)?;
    (index < item_count).then_some(index)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuCommand {
    Move,
    Confirm,
    Back,
}

pub fn take_navigation(pending_navigation: &mut Option<UiNavigation>) -> Option<UiNavigation> {
    pending_navigation.take()
}

pub fn back_to_setting() -> UiNavigation {
    UiNavigation::Page(UiPageKey::Setting)
}

pub fn theme_color(ctx: &UiContext, role: &str, fallback: &str) -> String {
    ctx.themes.color_or(role, fallback)
}
