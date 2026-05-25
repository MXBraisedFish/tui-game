//! Rust implementation of the home page.

use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::host_engine::boot::preload::lua_runtime::api::drawing_support::drawing_parser::STYLE_BOLD;
use crate::host_engine::runtime::ui::pages::common::theme_color;
use crate::host_engine::runtime::ui::{Canvas, UiContext, UiEvent, UiNavigation, UiPage, UiResult};
use crate::host_engine::runtime::ui_page::page_key::UiPageKey;

const LOGO: [&str; 6] = [
    "████████╗██╗   ██╗██╗     ██████╗  █████╗ ███╗   ███╗███████╗",
    "╚══██╔══╝██║   ██║██║    ██╔════╝ ██╔══██╗████╗ ████║██╔════╝",
    "   ██║   ██║   ██║██║    ██║  ███╗███████║██╔████╔██║█████╗  ",
    "   ██║   ██║   ██║██║    ██║   ██║██╔══██║██║╚██╔╝██║██╔══╝  ",
    "   ██║   ╚██████╔╝██║    ╚██████╔╝██║  ██║██║ ╚═╝ ██║███████╗",
    "   ╚═╝    ╚═════╝ ╚═╝     ╚═════╝ ╚═╝  ╚═╝╚═╝     ╚═╝╚══════╝",
];

const MENU_OPTION_ACTIONS: [&str; 5] = ["option1", "option2", "option3", "option4", "option5"];
const DEFAULT_OPTION_KEYS: [&str; 5] = ["1", "2", "3", "4", "Esc"];

pub struct HomePage {
    selected_index: usize,
    pending_navigation: Option<UiNavigation>,
}

impl Default for HomePage {
    fn default() -> Self {
        Self::new()
    }
}

impl HomePage {
    pub fn new() -> Self {
        Self {
            selected_index: 0,
            pending_navigation: None,
        }
    }

    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    fn select_previous(&mut self) {
        self.selected_index = if self.selected_index == 0 {
            4
        } else {
            self.selected_index - 1
        };
    }

    fn select_next(&mut self) {
        self.selected_index = (self.selected_index + 1) % 5;
    }

    fn confirm(&mut self) {
        self.pending_navigation = match self.selected_index {
            0 => Some(UiNavigation::Page(UiPageKey::GameList)),
            // Continue is kept as a no-op until game save resume is wired into Rust UI.
            1 => None,
            2 => Some(UiNavigation::Page(UiPageKey::Setting)),
            3 => Some(UiNavigation::Page(UiPageKey::Setting)),
            4 => Some(UiNavigation::Exit),
            _ => None,
        };
    }

    fn labels(ctx: &UiContext) -> [String; 5] {
        [
            ctx.i18n.home.play.clone(),
            ctx.i18n.home.continue_game.clone(),
            ctx.i18n.home.settings.clone(),
            ctx.i18n.home.about.clone(),
            ctx.i18n.home.quit.clone(),
        ]
    }
}

impl UiPage for HomePage {
    fn page_key(&self) -> UiPageKey {
        UiPageKey::Home
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
                    "confirm" | "enter" => self.confirm(),
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
        let logo_width = LOGO
            .iter()
            .map(|line| UnicodeWidthStr::width(*line))
            .max()
            .unwrap_or(0) as u16;
        let content_height = 17u16;
        let content_top = terminal_height.saturating_sub(content_height) / 2;
        let logo_x = terminal_width.saturating_sub(logo_width) / 2;

        for (row, line) in LOGO.iter().enumerate() {
            draw_logo_line(
                canvas,
                ctx,
                logo_x,
                content_top.saturating_add(row as u16),
                line,
            )?;
        }

        let labels = Self::labels(ctx);
        let menu_width = menu_width(ctx, &labels) as u16;
        let menu_x = terminal_width.saturating_sub(menu_width) / 2;
        let menu_y = content_top.saturating_add(7);
        for (index, label) in labels.iter().enumerate() {
            let y = menu_y.saturating_add(index as u16);
            let line_x = menu_x;
            let marker = if index == self.selected_index {
                "▶ "
            } else {
                "  "
            };
            let fg = if index == 1 {
                Some(theme_color(ctx, "text.muted", "dark_gray"))
            } else if index == self.selected_index {
                Some(theme_color(ctx, "accent.primary", "cyan"))
            } else {
                Some(theme_color(ctx, "text.primary", "white"))
            };
            canvas.draw_text_styled(
                line_x,
                y,
                marker,
                Some(if index == self.selected_index {
                    theme_color(ctx, "accent.primary", "cyan")
                } else if index == 1 {
                    theme_color(ctx, "text.muted", "dark_gray")
                } else {
                    theme_color(ctx, "text.primary", "white")
                }),
                None,
                Vec::new(),
            )?;
            let hint = menu_key_hint(ctx, index, index == self.selected_index);
            let key_text = format!("[{hint}]");
            let key_x = line_x.saturating_add(UnicodeWidthStr::width(marker) as u16);
            canvas.draw_text_styled(
                key_x,
                y,
                key_text.as_str(),
                Some(theme_color(ctx, "text.muted", "dark_gray")),
                None,
                Vec::new(),
            )?;
            let label_x = key_x
                .saturating_add(UnicodeWidthStr::width(key_text.as_str()) as u16)
                .saturating_add(1);
            canvas.draw_text_styled(label_x, y, label, fg, None, vec![STYLE_BOLD])?;
        }

        let version = format!("v{}", env!("CARGO_PKG_VERSION"));
        let version_x =
            terminal_width.saturating_sub(UnicodeWidthStr::width(version.as_str()) as u16) / 2;
        canvas.draw_text_styled(
            version_x,
            menu_y.saturating_add(6),
            version.as_str(),
            Some(theme_color(ctx, "text.muted", "dark_gray")),
            None,
            Vec::new(),
        )?;

        let action_text = action_hint_text(ctx);
        let action_x =
            terminal_width.saturating_sub(UnicodeWidthStr::width(action_text.as_str()) as u16) / 2;
        canvas.draw_text_styled(
            action_x,
            menu_y.saturating_add(8),
            action_text.as_str(),
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

fn draw_logo_line(
    canvas: &mut Canvas,
    ctx: &UiContext,
    x: u16,
    y: u16,
    line: &str,
) -> UiResult<()> {
    let logo_color = theme_color(ctx, "logo.primary", "#ffa500");
    let empty_color = theme_color(ctx, "text.primary", "white");
    let mut cursor_x = x;

    for ch in line.chars() {
        let width = UnicodeWidthChar::width(ch).unwrap_or(0) as u16;
        if width > 0 {
            canvas.draw_text_styled(
                cursor_x,
                y,
                ch.to_string().as_str(),
                Some(if ch == '█' {
                    logo_color.clone()
                } else {
                    empty_color.clone()
                }),
                None,
                vec![STYLE_BOLD],
            )?;
        }
        cursor_x = cursor_x.saturating_add(width);
    }

    Ok(())
}

fn menu_width(ctx: &UiContext, labels: &[String; 5]) -> usize {
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
        .unwrap_or(30)
}

fn menu_key_hint(ctx: &UiContext, index: usize, selected: bool) -> String {
    if selected {
        key_hint(ctx, "confirm", "Enter")
    } else {
        key_hint(ctx, MENU_OPTION_ACTIONS[index], DEFAULT_OPTION_KEYS[index])
    }
}

fn action_hint_text(ctx: &UiContext) -> String {
    let prev_key = key_hint(ctx, "prev_option", "↑");
    let next_key = key_hint(ctx, "next_option", "↓");
    let confirm_key = key_hint(ctx, "confirm", "Enter");
    format!(
        "[{prev_key}/{next_key}] {}  [{confirm_key}] {}",
        ctx.i18n.key.home_select, ctx.i18n.key.home_confirm
    )
}
