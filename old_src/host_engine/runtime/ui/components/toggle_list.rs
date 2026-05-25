//! Toggle list component.

use crate::host_engine::runtime::ui::{Canvas, UiContext, UiEvent};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToggleList {
    pub items: Vec<ToggleItem>,
    pub selected_index: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToggleItem {
    pub label: String,
    pub enabled: bool,
}

impl ToggleList {
    pub fn new(items: Vec<ToggleItem>) -> Self {
        Self {
            items,
            selected_index: 0,
        }
    }

    pub fn handle_event(&mut self, event: &UiEvent) -> bool {
        match event {
            UiEvent::Action { name, .. } | UiEvent::Key { name, .. } => match name.as_str() {
                "prev_option" | "up" | "arrowup" => {
                    self.selected_index = self.selected_index.saturating_sub(1);
                    true
                }
                "next_option" | "down" | "arrowdown" => {
                    if !self.items.is_empty() {
                        self.selected_index = (self.selected_index + 1).min(self.items.len() - 1);
                    }
                    true
                }
                "confirm" | "toggle" | "enter" => {
                    if let Some(item) = self.items.get_mut(self.selected_index) {
                        item.enabled = !item.enabled;
                    }
                    true
                }
                _ => false,
            },
            _ => false,
        }
    }

    pub fn render(&self, canvas: &mut Canvas, x: u16, y: u16, width: u16, ctx: &UiContext) {
        for (row, item) in self.items.iter().enumerate() {
            let marker = if row == self.selected_index {
                "▶"
            } else {
                " "
            };
            let status = if item.enabled {
                &ctx.i18n.display.option_list_on
            } else {
                &ctx.i18n.display.option_list_off
            };
            let text = format!("{marker} {} [ {status} ]", item.label);
            let _ = canvas.draw_text(x, y.saturating_add(row as u16), text);
        }
        let _ = width;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn confirm_toggles_selected_item() {
        let mut list = ToggleList::new(vec![ToggleItem {
            label: "Feature".to_string(),
            enabled: false,
        }]);

        assert!(list.handle_event(&UiEvent::action("confirm")));

        assert!(list.items[0].enabled);
    }
}
