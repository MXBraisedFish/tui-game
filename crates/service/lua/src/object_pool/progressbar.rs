//! Script-owned progress bar configuration and signed drawing submissions.
use super::*;
use tg_service_widget::{ProgressBarId, ProgressBarOptions, ProgressBarService};

/// A progress bar's logical width, exact script ratios and original color expressions.
#[derive(Clone)]
pub(crate) struct LuaProgressbarBinding {
  pub width: u16,
  pub completed_ratio: f64,
  pub preview_ratio: f64,
  pub options: ProgressBarOptions,
  pub completed_fg: String,
  pub preview_fg: String,
  pub remaining_fg: String,
}
impl Default for LuaProgressbarBinding {
  fn default() -> Self {
    let mut options = ProgressBarOptions::default();
    options.remaining.style.foreground = tg_core_style::parse_text_color("white");
    Self {
      width: 1,
      completed_ratio: 0.0,
      preview_ratio: 0.0,
      options,
      completed_fg: "green".into(),
      preview_fg: "bright_blue".into(),
      remaining_fg: "white".into(),
    }
  }
}
impl LuaObjectPool {
  /// Render a script progress bar in its original parent coordinates.
  ///
  /// # Arguments
  ///
  /// * `id` - The pool-local progress bar identity.
  /// * `target` - The owning base or slice surface.
  /// * `position` - The signed local column and row.
  /// * `canvas` - The prepared destination canvas.
  pub fn draw_progressbar(
    &self,
    id: u64,
    target: crate::LuaDrawTarget,
    position: (i32, i32),
    canvas: &mut tg_service_canvas::CanvasService,
  ) {
    let Some(binding) = self.progressbars.get(&id) else {
      return;
    };
    let target = match target {
      crate::LuaDrawTarget::Base => None,
      crate::LuaDrawTarget::Slice(id) => Some(id),
    };
    ProgressBarService::new().render_at(
      &self.ui,
      ProgressBarId(id),
      target,
      position,
      binding.width,
      canvas,
    );
  }
}
