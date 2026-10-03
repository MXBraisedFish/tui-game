//! Game warning overlay state, owned interactions, and clipped terminal presentation.

use crate::host_engine::services::{
  ActionMapEntry, CanvasService, DrawTextParams, I18nService, KeyState, LayoutService,
  RenderService, RichTextParams, RuntimeObjectPool, RuntimeObjectPoolOwner, TerminalColor,
  TextColor, UiEvent, UiObjectPool, UiObjectPoolOwner,
};

const NS: &str = "game_warning";

/// The state and owned widgets of the game warning view.
pub struct GameWarningUi {
  objects: UiObjectPool,
  runtime_objects: RuntimeObjectPool,
  params: RichTextParams,
}

impl GameWarningUi {
  /// Create the game warning view and allocate its owned UI objects.
  pub fn init() -> Self {
    Self {
      objects: UiObjectPool::new(),
      runtime_objects: RuntimeObjectPool::new(),
      params: RichTextParams::from_action_map(&Self::action_map(), "game_warning."),
    }
  }

  /// Return the shortcuts currently enabled by the game warning view.
  pub fn action_map() -> Vec<ActionMapEntry> {
    vec![ActionMapEntry {
      action: "game_warning.back".to_string(),
      description: "Return to game list".to_string(),
      keys: vec![vec!["esc".to_string()]],
      priority: 0,
    }]
  }

  /// Interpret a game warning UI event and return the requested application command.
  pub fn handle_event(&self, event: &UiEvent) -> Option<GameWarningCommand> {
    match event {
      UiEvent::Action(event)
        if event.state == KeyState::Pressed && event.action == "game_warning.back" =>
      {
        Some(GameWarningCommand::Back)
      }
      _ => None,
    }
  }

  /// Draw the game warning view and register interaction regions in its assigned surfaces.
  ///
  /// # Arguments
  ///
  /// * `render` - The drawing service used to render terminal cells.
  /// * `canvas` - The clipped canvas used for drawing.
  /// * `layout` - The service resolving terminal sizes and positions.
  /// * `i18n` - The service resolving localized text.
  /// * `seconds_left` - The seconds left.
  pub fn render(
    &mut self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    i18n: &I18nService,
    seconds_left: u8,
  ) {
    let size = layout.physical_size();
    let title = i18n.get_runtime_text(NS, "game_warning.title");
    let tip = i18n.get_runtime_text(NS, "game_warning.tip");
    let countdown = i18n
      .get_runtime_text(NS, "game_warning.countdown")
      .replace("{value:second}", &seconds_left.to_string());
    let back = i18n.get_runtime_text(NS, "game_warning.back");
    let red = Some(TextColor::Terminal(TerminalColor::BrightRed));

    draw_centered(render, canvas, layout, &title, 1, red.clone(), None);
    let tip_lines = tip.lines().collect::<Vec<_>>();
    let block_height = (tip_lines.len() as u16).saturating_add(4);
    let block_y = size.height.saturating_sub(block_height) / 2;
    for (offset, line) in tip_lines.iter().enumerate() {
      draw_centered(
        render,
        canvas,
        layout,
        line,
        block_y.saturating_add(offset as u16),
        None,
        None,
      );
    }
    let countdown_y = block_y
      .saturating_add(tip_lines.len() as u16)
      .saturating_add(1);
    draw_centered(
      render,
      canvas,
      layout,
      &countdown,
      countdown_y,
      red.clone(),
      None,
    );
    draw_centered(
      render,
      canvas,
      layout,
      &back,
      countdown_y.saturating_add(2),
      red,
      Some(self.params.clone()),
    );
  }
}

fn draw_centered(
  render: &mut RenderService,
  canvas: &mut CanvasService,
  layout: &LayoutService,
  text: &str,
  y: u16,
  fg: Option<TextColor>,
  params: Option<RichTextParams>,
) {
  let width = layout.get_text_width(text, params.as_ref());
  render.draw_host_text(
    canvas,
    &DrawTextParams {
      x: layout.resolve_host_x(LayoutService::ALIGN_CENTER, width, 0),
      y,
      text: text.to_string(),
      params,
      fg,
      bold: true,
      ..Default::default()
    },
  );
}

/// An application request produced by game warning interactions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameWarningCommand {
  /// A request to back.
  Back,
}

impl UiObjectPoolOwner for GameWarningUi {
  fn objects(&self) -> &UiObjectPool {
    &self.objects
  }

  fn objects_mut(&mut self) -> &mut UiObjectPool {
    &mut self.objects
  }
}

impl RuntimeObjectPoolOwner for GameWarningUi {
  fn runtime_objects(&self) -> &RuntimeObjectPool {
    &self.runtime_objects
  }

  fn runtime_objects_mut(&mut self) -> &mut RuntimeObjectPool {
    &mut self.runtime_objects
  }
}

#[cfg(test)]
mod tests {
  use super::{GameWarningCommand, GameWarningUi};
  use crate::host_engine::services::{InputActionEvent, InputEventType, KeyState, UiEvent};

  #[test]
  fn back_action_returns_to_game_list() {
    let ui = GameWarningUi::init();
    let command = ui.handle_event(&UiEvent::Action(InputActionEvent {
      event_type: InputEventType::Keyboard,
      action: "game_warning.back".to_string(),
      state: KeyState::Pressed,
    }));

    assert_eq!(command, Some(GameWarningCommand::Back));
  }
}
