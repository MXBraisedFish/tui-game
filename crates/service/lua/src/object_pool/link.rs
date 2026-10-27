//! Session-owned link configuration and native click conversion.

use super::*;
use tg_service_widget::{HyperlinkEvent, HyperlinkId, HyperlinkOptions, HyperlinkService};

/// Script options retained with a native link object.
#[derive(Clone)]
pub(crate) struct LuaLinkBinding {
  /// The native text, target, and normal/hover styles.
  pub options: HyperlinkOptions,
  /// The original foreground expression.
  pub fg: String,
  /// The original background expression.
  pub bg: String,
  /// The original hovered foreground expression.
  pub hover_fg: String,
  /// The original hovered background expression.
  pub hover_bg: String,
  /// Whether activation copies the resolved destination.
  pub copy: bool,
}
impl LuaLinkBinding {
  pub(crate) fn new(text: String, link: String) -> Self {
    Self {
      options: HyperlinkOptions::new(link, text),
      fg: "bright_blue".into(),
      bg: "none".into(),
      hover_fg: "bright_blue".into(),
      hover_bg: "none".into(),
      copy: false,
    }
  }
}
impl LuaObjectPool {
  /// Render a link in its owning base or slice, preserving signed clipping coordinates.
  ///
  /// # Arguments
  ///
  /// * `id` - The pool-local native link identity.
  /// * `target` - The parent base or slice surface.
  /// * `position` - The original signed column and row.
  /// * `canvas` - The prepared destination canvas.
  pub fn draw_link(
    &mut self,
    id: u64,
    target: crate::LuaDrawTarget,
    position: (i32, i32),
    canvas: &mut tg_service_canvas::CanvasService,
  ) {
    if !self.links.contains_key(&id) {
      return;
    }
    let slice = match target {
      crate::LuaDrawTarget::Base => None,
      crate::LuaDrawTarget::Slice(id) => Some(id),
    };
    HyperlinkService::new().render_at(&mut self.ui, HyperlinkId(id), slice, position, canvas);
  }
  pub(crate) fn translate_link_event(&self, event: HyperlinkEvent) -> Option<LuaEventData> {
    let HyperlinkEvent::Clicked { id, link } = event;
    self
      .links
      .contains_key(&id.0)
      .then_some(LuaEventData::Hyperlink(crate::LuaHyperlinkEvent {
        id: id.0,
        link,
      }))
  }
}
