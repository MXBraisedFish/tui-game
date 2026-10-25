//! Session-owned input configuration, editing, and event conversion.

use super::*;
use tg_service_widget::{TextInputEvent, TextInputId, TextInputOptions, TextInputRenderParams};

/// Script configuration retained independently of the live native editing buffer.
#[derive(Clone)]
pub(crate) struct LuaInputBinding {
  /// The dimensions, styles, placeholder, and cursor used when drawing.
  pub params: TextInputRenderParams,
  /// The editing configuration applied to the native buffer.
  pub options: TextInputOptions,
  /// Whether Enter submits and Ctrl+Enter inserts a newline.
  pub enter_submit: bool,
  /// The original script foreground expression returned by inspection.
  pub fg: String,
  /// The original script background expression returned by inspection.
  pub bg: String,
  /// The original placeholder foreground expression returned by inspection.
  pub placeholder_fg: String,
  /// The script vertical alignment constant, including the automatic default.
  pub vertical_align: String,
  /// The script horizontal alignment constant, including the automatic default.
  pub horizontal_align: String,
  /// The script cursor grapheme, or no grapheme when hidden.
  pub cursor_shape: Option<String>,
}

impl Default for LuaInputBinding {
  fn default() -> Self {
    Self {
      params: TextInputRenderParams::default(),
      options: TextInputOptions::default(),
      enter_submit: true,
      fg: "none".into(),
      bg: "none".into(),
      placeholder_fg: "none".into(),
      vertical_align: "auto".into(),
      horizontal_align: "auto".into(),
      cursor_shape: Some("█".into()),
    }
  }
}

impl LuaObjectPool {
  /// Render a session input in its original signed base or slice coordinates.
  ///
  /// # Arguments
  ///
  /// * `id` - The pool-local input identity.
  /// * `target` - The owning base or slice.
  /// * `position` - The signed input origin before clipping.
  /// * `canvas` - The prepared destination canvas.
  pub fn draw_input(
    &mut self,
    id: u64,
    target: crate::LuaDrawTarget,
    position: (i32, i32),
    canvas: &mut tg_service_canvas::CanvasService,
  ) {
    let Some(binding) = self.inputs.get(&id) else {
      return;
    };
    let slice = match target {
      crate::LuaDrawTarget::Base => None,
      crate::LuaDrawTarget::Slice(id) => Some(id),
    };
    self.text_input.render_at(
      &mut self.ui,
      TextInputId(id),
      slice,
      position,
      &binding.params,
      canvas,
    );
  }

  /// Report whether this session currently owns a focused script input.
  pub fn input_is_focused(&self) -> bool {
    self.text_input.is_active()
  }

  /// Apply committed text to the focused script input, independently of keyboard subscriptions.
  pub fn route_input_text(&mut self, text: &str) {
    self.text_input.insert_text(&mut self.ui, text);
  }

  /// Apply a non-text terminal key to the focused input using the supplied clipboard.
  ///
  /// Ordinary characters are delivered through [`Self::route_input_text`] to avoid duplicate
  /// insertion when the terminal reports both committed text and a character key.
  pub fn route_input_key(
    &mut self,
    clipboard: &mut tg_service_clipboard::ClipboardService,
    key: tg_core_input::TerminalKeyEvent,
  ) {
    if matches!(key.code, tg_core_input::TerminalKeyCode::Char(_)) && !key.ctrl {
      return;
    }
    self
      .text_input
      .route_terminal_key(&mut self.ui, clipboard, key);
  }

  /// Clear stale pointer geometry and drag selection on focus loss or overlay takeover.
  pub fn deactivate_inputs(&mut self) {
    self.text_input.deactivate_pool(&mut self.ui);
  }

  /// Drain pending widget notifications, retaining only events owned by live script objects.
  pub fn take_widget_events(&mut self) -> Vec<LuaEventData> {
    let mut result = Vec::new();
    while let Some(event) = self.ui.pop_event() {
      if let tg_service_widget::UiEvent::TextInput(event) = event
        && let Some(event) = self.translate_input_event(event)
      {
        result.push(event);
      }
    }
    result
  }

  pub(crate) fn translate_input_event(&self, event: TextInputEvent) -> Option<LuaEventData> {
    let (id, kind, value) = match event {
      TextInputEvent::Pressed { id } => (id, "focus", None),
      TextInputEvent::PressedOutside { id } => (id, "blur", None),
      TextInputEvent::Focused { id } => (id, "focused", None),
      TextInputEvent::Blurred { id } => (id, "blurred", None),
      TextInputEvent::Changed { id, value } => (id, "change", Some(value)),
      TextInputEvent::Submit { id, value } => (id, "submit", Some(value)),
      TextInputEvent::Cancel { id, value } => (id, "cancel", Some(value)),
    };
    self
      .inputs
      .contains_key(&id.0)
      .then_some(LuaEventData::TextInput(crate::LuaTextInputEvent {
        id: id.0,
        kind,
        value,
      }))
  }

  pub(crate) fn delete_input(&mut self, id: u64) -> bool {
    if !self.inputs.contains_key(&id) {
      return false;
    }
    if self.text_input.is_focused(&self.ui, TextInputId(id)) {
      self.text_input.blur(&mut self.ui);
    }
    self.text_input.remove(&mut self.ui, TextInputId(id));
    self.inputs.remove(&id);
    true
  }
}
