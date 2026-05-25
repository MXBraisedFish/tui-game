//! Menu component.

use crate::host_engine::runtime::ui::{Canvas, UiContext, UiEvent};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MenuComponent {
    pub items: Vec<MenuItem>,
    pub selected_index: usize,
    pub scroll_offset: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MenuItem {
    pub label: String,
    pub key_hint: Option<String>,
    pub enabled: bool,
}

impl MenuComponent {
    pub fn new(items: Vec<MenuItem>) -> Self {
        Self {
            items,
            selected_index: 0,
            scroll_offset: 0,
        }
    }

    /// Returns true when the event was consumed by the menu.
    pub fn handle_event(&mut self, event: &UiEvent) -> bool {
        match event {
            UiEvent::Action { name, .. } | UiEvent::Key { name, .. } => match name.as_str() {
                "prev_option" | "up" | "arrowup" => {
                    self.select_previous();
                    true
                }
                "next_option" | "down" | "arrowdown" => {
                    self.select_next();
                    true
                }
                "confirm" | "enter" => true,
                _ => false,
            },
            _ => false,
        }
    }

    pub fn selected_item(&self) -> Option<&MenuItem> {
        self.items.get(self.selected_index)
    }

    pub fn select_previous(&mut self) {
        if self.items.is_empty() {
            return;
        }
        for _ in 0..self.items.len() {
            self.selected_index = if self.selected_index == 0 {
                self.items.len() - 1
            } else {
                self.selected_index - 1
            };
            if self.items[self.selected_index].enabled {
                break;
            }
        }
    }

    pub fn select_next(&mut self) {
        if self.items.is_empty() {
            return;
        }
        for _ in 0..self.items.len() {
            self.selected_index = (self.selected_index + 1) % self.items.len();
            if self.items[self.selected_index].enabled {
                break;
            }
        }
    }

    pub fn render(&self, canvas: &mut Canvas, x: u16, y: u16, width: u16) {
        self.render_with_colors(canvas, x, y, width, "dark_gray", "cyan");
    }

    pub fn render_with_theme(
        &self,
        canvas: &mut Canvas,
        ctx: &UiContext,
        x: u16,
        y: u16,
        width: u16,
    ) {
        self.render_with_colors(
            canvas,
            x,
            y,
            width,
            ctx.themes.color_or("text.muted", "dark_gray").as_str(),
            ctx.themes.color_or("accent.primary", "cyan").as_str(),
        );
    }

    fn render_with_colors(
        &self,
        canvas: &mut Canvas,
        x: u16,
        y: u16,
        width: u16,
        disabled_fg: &str,
        selected_fg: &str,
    ) {
        let max_label_width = width.saturating_sub(4) as usize;
        for (row, item) in self.items.iter().skip(self.scroll_offset).enumerate() {
            let selected = self.scroll_offset + row == self.selected_index;
            let marker = if selected { "▶" } else { " " };
            let hint = item.key_hint.as_deref().unwrap_or_default();
            let label = truncate_chars(&item.label, max_label_width);
            let text = if hint.is_empty() {
                format!("{marker} {label}")
            } else {
                format!("{marker} [{hint}] {label}")
            };
            let fg = if selected {
                Some(selected_fg.to_string())
            } else if item.enabled {
                None
            } else {
                Some(disabled_fg.to_string())
            };
            let _ = canvas.draw_text_styled(
                x,
                y.saturating_add(row as u16),
                text,
                fg,
                None,
                Vec::new(),
            );
        }
    }
}

fn truncate_chars(text: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }
    text.chars().take(max_chars).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn menu() -> MenuComponent {
        MenuComponent::new(vec![
            MenuItem {
                label: "One".to_string(),
                key_hint: Some("1".to_string()),
                enabled: true,
            },
            MenuItem {
                label: "Two".to_string(),
                key_hint: Some("2".to_string()),
                enabled: true,
            },
            MenuItem {
                label: "Three".to_string(),
                key_hint: Some("3".to_string()),
                enabled: true,
            },
        ])
    }

    #[test]
    fn down_action_moves_selection() {
        let mut menu = menu();
        assert!(menu.handle_event(&UiEvent::action("next_option")));
        assert_eq!(menu.selected_index, 1);
    }

    #[test]
    fn up_action_wraps_selection() {
        let mut menu = menu();
        assert!(menu.handle_event(&UiEvent::action("prev_option")));
        assert_eq!(menu.selected_index, 2);
    }

    #[test]
    fn disabled_item_is_skipped() {
        let mut menu = menu();
        menu.items[1].enabled = false;
        menu.handle_event(&UiEvent::action("next_option"));
        assert_eq!(menu.selected_index, 2);
    }
}
