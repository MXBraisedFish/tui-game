//! Confirm dialog component.

use crate::host_engine::runtime::ui::{Canvas, UiContext, UiEvent};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfirmDialog {
    pub title: String,
    pub message: String,
    pub confirmed: bool,
    pub cancelled: bool,
}

impl ConfirmDialog {
    pub fn new(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            confirmed: false,
            cancelled: false,
        }
    }

    pub fn handle_event(&mut self, event: &UiEvent) -> bool {
        match event {
            UiEvent::Action { name, .. } | UiEvent::Key { name, .. } => match name.as_str() {
                "confirm" | "enter" | "y" => {
                    self.confirmed = true;
                    true
                }
                "back" | "cancel" | "esc" | "q" | "n" => {
                    self.cancelled = true;
                    true
                }
                _ => false,
            },
            _ => false,
        }
    }

    pub fn render(&self, canvas: &mut Canvas, x: u16, y: u16, width: u16, ctx: &UiContext) {
        let _ = canvas.draw_text(x, y, &self.title);
        let _ = canvas.draw_text(x, y.saturating_add(2), &self.message);
        let _ = canvas.draw_text(
            x,
            y.saturating_add(4),
            format!(
                "[Enter] {}  [Esc] {}",
                ctx.i18n.key.clear_data_confirm, ctx.i18n.key.clear_data_cancel
            ),
        );
        let _ = width;
    }
}
