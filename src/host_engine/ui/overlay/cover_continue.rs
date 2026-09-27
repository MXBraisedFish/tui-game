use crate::host_engine::services::text_layout::TextWrapMode;
use crate::host_engine::services::{
  ActionMapEntry, CanvasService, DrawTextParams, HitAreaEvent, HitAreaId, HitAreaOptions,
  HitAreaService, I18nService, KeyState, LayoutService, MouseButton, Rect, RenderService,
  RichTextParams, RuntimeObjectPool, RuntimeObjectPoolOwner, TextAlign, TextMode, UiEvent,
  UiObjectPool, UiObjectPoolOwner,
};

const NS: &str = "cover_continue";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoverContinueCommand {
  Start,
  Back,
}

pub struct CoverContinueUi {
  objects: UiObjectPool,
  runtime_objects: RuntimeObjectPool,
  start_area: HitAreaId,
  back_area: HitAreaId,
  continue_game: String,
}

impl CoverContinueUi {
  pub fn init(hit_area: &HitAreaService) -> Self {
    let mut objects = UiObjectPool::new();
    Self {
      start_area: hit_area.create(&mut objects, HitAreaOptions::default()),
      back_area: hit_area.create(&mut objects, HitAreaOptions::default()),
      objects,
      runtime_objects: RuntimeObjectPool::new(),
      continue_game: String::new(),
    }
  }

  pub fn action_map() -> Vec<ActionMapEntry> {
    vec![
      ActionMapEntry {
        action: "cover_continue.start".to_string(),
        description: "Start a new game and overwrite the continue slot".to_string(),
        keys: vec![vec!["enter".to_string()]],
      },
      ActionMapEntry {
        action: "cover_continue.back".to_string(),
        description: "Return to the game list".to_string(),
        keys: vec![vec!["esc".to_string()]],
      },
    ]
  }

  pub fn start(&mut self, continue_game: String) {
    self.continue_game = continue_game;
  }

  pub fn reset(&mut self) {
    self.continue_game.clear();
  }

  pub fn handle_event(&self, event: &UiEvent) -> Option<CoverContinueCommand> {
    match event {
      UiEvent::Action(event) if event.state == KeyState::Pressed => match event.action.as_str() {
        "cover_continue.start" => Some(CoverContinueCommand::Start),
        "cover_continue.back" => Some(CoverContinueCommand::Back),
        _ => None,
      },
      UiEvent::HitArea(HitAreaEvent::Click {
        id,
        button: MouseButton::Left,
        ..
      }) if *id == self.start_area => Some(CoverContinueCommand::Start),
      UiEvent::HitArea(HitAreaEvent::Click {
        id,
        button: MouseButton::Left,
        ..
      }) if *id == self.back_area => Some(CoverContinueCommand::Back),
      _ => None,
    }
  }

  pub fn render(
    &mut self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    i18n: &I18nService,
    hit_area: &HitAreaService,
  ) {
    let size = layout.physical_size();
    let title = i18n.get_runtime_text(NS, "cover_continue.title");
    let title_width = layout.get_text_width(&title, None);
    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: layout.resolve_host_x(LayoutService::ALIGN_CENTER, title_width, 0),
        y: 1,
        text: format!("f%<fg:bright_yellow><b>{title}</b></fg>"),
        ..Default::default()
      },
    );

    let mut params = RichTextParams::from_action_map(&Self::action_map(), "cover_continue.");
    params
      .values
      .insert("continue_game".to_string(), self.continue_game.clone());
    let max_width = size.width.saturating_sub(32).max(1);
    let tip = i18n.get_runtime_text(NS, "cover_continue.tip");
    let tip_size = layout.get_draw_text_size(&DrawTextParams {
      text: tip.clone(),
      text_mode: TextMode::Rich,
      params: Some(params.clone()),
      wrap_mode: TextWrapMode::Auto,
      max_width: Some(max_width),
      ..Default::default()
    });
    let start = format!(
      "f%<fg:bright_red>{}</fg>",
      i18n.get_runtime_text(NS, "cover_continue.start")
    );
    let back = format!(
      "f%<fg:bright_green>{}</fg>",
      i18n.get_runtime_text(NS, "cover_continue.back")
    );
    let block_width = tip_size
      .width
      .max(layout.get_text_width(&start, Some(&params)))
      .max(layout.get_text_width(&back, Some(&params)))
      .min(max_width)
      .max(1);
    let block_height = tip_size.height.max(1).saturating_add(3);
    let x = size.width.saturating_sub(block_width) / 2;
    let y = size.height.saturating_sub(block_height) / 2;

    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x,
        y,
        text: tip,
        text_mode: TextMode::Rich,
        params: Some(params.clone()),
        line_align: TextAlign::Center,
        wrap_mode: TextWrapMode::Auto,
        max_width: Some(block_width),
        ..Default::default()
      },
    );
    let start_y = y.saturating_add(tip_size.height.max(1)).saturating_add(1);
    let back_y = start_y.saturating_add(1);
    let start_width = layout.get_text_width(&start, Some(&params));
    let back_width = layout.get_text_width(&back, Some(&params));
    let start_x = size.width.saturating_sub(start_width) / 2;
    let back_x = size.width.saturating_sub(back_width) / 2;
    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: start_x,
        y: start_y,
        text: start,
        params: Some(params.clone()),
        ..Default::default()
      },
    );
    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: back_x,
        y: back_y,
        text: back,
        params: Some(params),
        ..Default::default()
      },
    );

    self.register_area(
      hit_area,
      canvas,
      self.start_area,
      start_x,
      start_y,
      start_width,
    );
    self.register_area(hit_area, canvas, self.back_area, back_x, back_y, back_width);
  }

  fn register_area(
    &mut self,
    hit_area: &HitAreaService,
    canvas: &mut CanvasService,
    id: HitAreaId,
    x: u16,
    y: u16,
    width: u16,
  ) {
    hit_area.render_host(
      &mut self.objects,
      id,
      Rect {
        x,
        y,
        width: width.max(1),
        height: 1,
      },
      canvas,
    );
  }
}

impl UiObjectPoolOwner for CoverContinueUi {
  fn objects(&self) -> &UiObjectPool {
    &self.objects
  }

  fn objects_mut(&mut self) -> &mut UiObjectPool {
    &mut self.objects
  }
}

impl RuntimeObjectPoolOwner for CoverContinueUi {
  fn runtime_objects(&self) -> &RuntimeObjectPool {
    &self.runtime_objects
  }

  fn runtime_objects_mut(&mut self) -> &mut RuntimeObjectPool {
    &mut self.runtime_objects
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::host_engine::services::{InputActionEvent, InputEventType};

  fn action(name: &str) -> UiEvent {
    UiEvent::Action(InputActionEvent {
      event_type: InputEventType::Keyboard,
      action: name.to_string(),
      state: KeyState::Pressed,
    })
  }

  #[test]
  fn keyboard_actions_start_or_return() {
    let ui = CoverContinueUi::init(&HitAreaService::new());
    assert_eq!(
      ui.handle_event(&action("cover_continue.start")),
      Some(CoverContinueCommand::Start)
    );
    assert_eq!(
      ui.handle_event(&action("cover_continue.back")),
      Some(CoverContinueCommand::Back)
    );
  }
}
