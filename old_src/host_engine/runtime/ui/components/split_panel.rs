//! Split panel layout helper.

use crate::host_engine::boot::preload::lua_runtime::api::drawing_support::border_chars::BorderChars;
use crate::host_engine::boot::preload::lua_runtime::api::drawing_support::drawing_parser::STYLE_BOLD;
use crate::host_engine::runtime::ui::{Canvas, UiContext, UiResult};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SplitPanel {
    pub left_x: u16,
    pub left_y: u16,
    pub left_width: u16,
    pub height: u16,
    pub right_x: u16,
    pub right_y: u16,
    pub right_width: u16,
}

impl SplitPanel {
    pub fn new(total_width: u16, total_height: u16, footer_height: u16) -> Self {
        let height = total_height.saturating_sub(footer_height).max(3);
        // The left panel wants at least 24 columns while the right panel keeps at least 8.
        // Runtime root UI already requires a much wider terminal, so this clamp is only a
        // defensive fallback for direct component use with undersized dimensions.
        let left_width = ((u32::from(total_width) * 33) / 100)
            .try_into()
            .unwrap_or(total_width)
            .max(24)
            .min(total_width.saturating_sub(24));
        let right_width = total_width.saturating_sub(left_width);

        Self {
            left_x: 0,
            left_y: 0,
            left_width,
            height,
            right_x: left_width,
            right_y: 0,
            right_width,
        }
    }

    pub fn render_borders(
        &self,
        canvas: &mut Canvas,
        left_title: &str,
        right_title: &str,
    ) -> UiResult<()> {
        self.render_borders_with_colors(
            canvas,
            left_title,
            right_title,
            "white".to_string(),
            "black".to_string(),
        )
    }

    pub fn render_borders_with_theme(
        &self,
        canvas: &mut Canvas,
        ctx: &UiContext,
        left_title: &str,
        right_title: &str,
    ) -> UiResult<()> {
        self.render_borders_with_colors(
            canvas,
            left_title,
            right_title,
            ctx.themes.color_or("border.primary", "white"),
            ctx.themes.color_or("panel.background", "black"),
        )
    }

    fn render_borders_with_colors(
        &self,
        canvas: &mut Canvas,
        left_title: &str,
        right_title: &str,
        border_color: String,
        title_background: String,
    ) -> UiResult<()> {
        let border = double_border();
        canvas.border_rect(
            self.left_x,
            self.left_y,
            self.left_width,
            self.height,
            border.clone(),
            Some(border_color.clone()),
            None,
        )?;
        canvas.border_rect(
            self.right_x,
            self.right_y,
            self.right_width,
            self.height,
            border,
            Some(border_color.clone()),
            None,
        )?;
        canvas.draw_text_styled(
            self.left_x.saturating_add(2),
            self.left_y,
            format!(" {left_title} "),
            Some(border_color.clone()),
            Some(title_background.clone()),
            vec![STYLE_BOLD],
        )?;
        canvas.draw_text_styled(
            self.right_x.saturating_add(2),
            self.right_y,
            format!(" {right_title} "),
            Some(border_color),
            Some(title_background),
            vec![STYLE_BOLD],
        )?;
        Ok(())
    }
}

fn double_border() -> BorderChars {
    BorderChars {
        top: Some('═'),
        top_right: Some('╗'),
        right: Some('║'),
        bottom_right: Some('╝'),
        bottom: Some('═'),
        bottom_left: Some('╚'),
        left: Some('║'),
        top_left: Some('╔'),
    }
}
