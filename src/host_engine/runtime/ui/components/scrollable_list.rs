//! Scrollable list component.

use crate::host_engine::runtime::ui::{Canvas, UiContext, UiEvent};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScrollableList {
    pub items: Vec<ListItem>,
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub viewport_height: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ListItem {
    pub label: String,
    pub enabled: bool,
}

impl ScrollableList {
    pub fn new(items: Vec<ListItem>, viewport_height: usize) -> Self {
        Self {
            items,
            selected_index: 0,
            scroll_offset: 0,
            viewport_height: viewport_height.max(1),
        }
    }

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
                "scroll_up" | "w" => {
                    self.scroll_up();
                    true
                }
                "scroll_down" | "s" => {
                    self.scroll_down();
                    true
                }
                _ => false,
            },
            _ => false,
        }
    }

    pub fn select_previous(&mut self) {
        if self.items.is_empty() {
            return;
        }
        self.selected_index = self.selected_index.saturating_sub(1);
        self.ensure_selected_visible();
    }

    pub fn select_next(&mut self) {
        if self.items.is_empty() {
            return;
        }
        self.selected_index = (self.selected_index + 1).min(self.items.len() - 1);
        self.ensure_selected_visible();
    }

    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    pub fn scroll_down(&mut self) {
        let max_offset = self.items.len().saturating_sub(self.viewport_height);
        self.scroll_offset = (self.scroll_offset + 1).min(max_offset);
    }

    pub fn ensure_selected_visible(&mut self) {
        if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        }
        let bottom = self.scroll_offset + self.viewport_height;
        if self.selected_index >= bottom {
            self.scroll_offset = self.selected_index + 1 - self.viewport_height;
        }
    }

    pub fn render(&self, canvas: &mut Canvas, x: u16, y: u16, width: u16) {
        self.render_with_colors(canvas, x, y, width, "dark_gray", "black", "blue");
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
            ctx.themes.color_or("text.on_selected", "black").as_str(),
            ctx.themes.color_or("background.selected", "blue").as_str(),
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
        selected_bg: &str,
    ) {
        for (row, item) in self
            .items
            .iter()
            .skip(self.scroll_offset)
            .take(self.viewport_height)
            .enumerate()
        {
            let absolute_index = self.scroll_offset + row;
            let selected = absolute_index == self.selected_index;
            let bg = if selected {
                Some(selected_bg.to_string())
            } else {
                None
            };
            let fg = if selected {
                Some(selected_fg.to_string())
            } else if item.enabled {
                None
            } else {
                Some(disabled_fg.to_string())
            };
            let text = pad_or_trim(&item.label, width as usize);
            let _ =
                canvas.draw_text_styled(x, y.saturating_add(row as u16), text, fg, bg, Vec::new());
        }
    }
}

fn pad_or_trim(text: &str, width: usize) -> String {
    let mut value = text.chars().take(width).collect::<String>();
    let len = value.chars().count();
    if len < width {
        value.push_str(&" ".repeat(width - len));
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    fn list() -> ScrollableList {
        ScrollableList::new(
            (0..10)
                .map(|index| ListItem {
                    label: format!("Item {index}"),
                    enabled: true,
                })
                .collect(),
            3,
        )
    }

    #[test]
    fn selection_scrolls_into_view() {
        let mut list = list();
        list.handle_event(&UiEvent::action("next_option"));
        list.handle_event(&UiEvent::action("next_option"));
        list.handle_event(&UiEvent::action("next_option"));
        assert_eq!(list.selected_index, 3);
        assert_eq!(list.scroll_offset, 1);
    }

    #[test]
    fn scroll_down_is_clamped() {
        let mut list = list();
        for _ in 0..20 {
            list.handle_event(&UiEvent::action("scroll_down"));
        }
        assert_eq!(list.scroll_offset, 7);
    }
}
