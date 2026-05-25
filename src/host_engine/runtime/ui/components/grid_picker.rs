//! Simple grid picker component.

use crate::host_engine::runtime::ui::{Canvas, UiContext, UiEvent};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GridPicker {
    pub items: Vec<String>,
    pub selected_index: usize,
    pub columns: usize,
}

impl GridPicker {
    pub fn new(items: Vec<String>, columns: usize) -> Self {
        Self {
            items,
            selected_index: 0,
            columns: columns.max(1),
        }
    }

    pub fn handle_event(&mut self, event: &UiEvent) -> bool {
        match event {
            UiEvent::Action { name, .. } | UiEvent::Key { name, .. } => match name.as_str() {
                "prev_option" | "left_option" | "left" | "arrowleft" => {
                    self.move_left();
                    true
                }
                "next_option" | "right_option" | "right" | "arrowright" => {
                    self.move_right();
                    true
                }
                "up_option" | "up" | "arrowup" => {
                    self.move_up();
                    true
                }
                "down_option" | "down" | "arrowdown" => {
                    self.move_down();
                    true
                }
                _ => false,
            },
            _ => false,
        }
    }

    pub fn render(&self, canvas: &mut Canvas, x: u16, y: u16, cell_width: u16) {
        self.render_with_colors(
            canvas,
            x,
            y,
            cell_width,
            "black".to_string(),
            "white".to_string(),
            "cyan".to_string(),
        )
    }

    pub fn render_with_theme(
        &self,
        canvas: &mut Canvas,
        ctx: &UiContext,
        x: u16,
        y: u16,
        cell_width: u16,
    ) {
        self.render_with_colors(
            canvas,
            x,
            y,
            cell_width,
            ctx.themes.color_or("panel.background", "black"),
            ctx.themes.color_or("text.primary", "white"),
            ctx.themes.color_or("background.selected", "cyan"),
        )
    }

    fn render_with_colors(
        &self,
        canvas: &mut Canvas,
        x: u16,
        y: u16,
        cell_width: u16,
        selected_fg: String,
        normal_fg: String,
        selected_bg: String,
    ) {
        for (index, item) in self.items.iter().enumerate() {
            let column = index % self.columns;
            let row = index / self.columns;
            let item_x = x.saturating_add((column as u16).saturating_mul(cell_width));
            let item_y = y.saturating_add((row as u16).saturating_mul(2));
            let selected = index == self.selected_index;
            let text = format!("[{}]", item);
            let _ = canvas.draw_text_styled(
                item_x,
                item_y,
                text,
                Some(if selected {
                    selected_fg.clone()
                } else {
                    normal_fg.clone()
                }),
                if selected {
                    Some(selected_bg.clone())
                } else {
                    None
                },
                Vec::new(),
            );
        }
    }

    fn move_left(&mut self) {
        if self.items.is_empty() {
            return;
        }
        self.selected_index = if self.selected_index == 0 {
            self.items.len() - 1
        } else {
            self.selected_index - 1
        };
    }

    fn move_right(&mut self) {
        if !self.items.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.items.len();
        }
    }

    fn move_up(&mut self) {
        if self.items.is_empty() {
            return;
        }
        self.selected_index = if self.selected_index < self.columns {
            self.items.len().saturating_sub(1)
        } else {
            self.selected_index - self.columns
        };
    }

    fn move_down(&mut self) {
        if self.items.is_empty() {
            return;
        }
        self.selected_index = (self.selected_index + self.columns).min(self.items.len() - 1);
    }
}
