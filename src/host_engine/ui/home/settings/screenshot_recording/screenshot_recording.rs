use std::time::Duration;

use crate::host_engine::services::{
  ActionMapEntry, CanvasService, DrawTextParams, HitAreaEvent, HitAreaId, HitAreaOptions,
  HitAreaService, I18nService, KeyState, LayoutService, MouseButton, Rect, RenderService,
  RichTextParams, RuntimeObjectPool, RuntimeObjectPoolOwner, ScrollBoxService, TextInputService,
  UiEvent, UiObjectPool, UiObjectPoolOwner,
};

use super::ScreenshotSettingsUi;

const NS: &str = "screenshot_recording";
const MENU_LEN: usize = 4;
const MENU_KEYS: [&str; MENU_LEN] = [
  "screenshot_recording.screenshot_settings",
  "screenshot_recording.recording_settings",
  "screenshot_recording.screenshot_list",
  "screenshot_recording.recording_list",
];

pub struct ScreenshotRecordingUi {
  selected_index: usize,
  objects: UiObjectPool,
  runtime_objects: RuntimeObjectPool,
  back_area: HitAreaId,
  menu_areas: [HitAreaId; MENU_LEN],
  screenshot_settings: ScreenshotSettingsUi,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScreenshotRecordingCommand {
  Back,
  OpenScreenshotSettings,
}

impl UiObjectPoolOwner for ScreenshotRecordingUi {
  fn objects(&self) -> &UiObjectPool {
    &self.objects
  }

  fn objects_mut(&mut self) -> &mut UiObjectPool {
    &mut self.objects
  }
}

impl RuntimeObjectPoolOwner for ScreenshotRecordingUi {
  fn runtime_objects(&self) -> &RuntimeObjectPool {
    &self.runtime_objects
  }

  fn runtime_objects_mut(&mut self) -> &mut RuntimeObjectPool {
    &mut self.runtime_objects
  }
}

impl ScreenshotRecordingUi {
  pub fn init(
    hit_area: &HitAreaService,
    text_input: &TextInputService,
    scroll_box: &ScrollBoxService,
  ) -> Self {
    let mut objects = UiObjectPool::new();
    Self {
      selected_index: 0,
      back_area: hit_area.create(&mut objects, HitAreaOptions::default()),
      menu_areas: std::array::from_fn(|_| hit_area.create(&mut objects, HitAreaOptions::default())),
      objects,
      runtime_objects: RuntimeObjectPool::new(),
      screenshot_settings: ScreenshotSettingsUi::init(
        hit_area,
        text_input,
        scroll_box,
        crate::host_engine::services::ScreenshotProfile::default(),
      ),
    }
  }

  pub fn action_map() -> Vec<ActionMapEntry> {
    vec![
      action(
        "screenshot_recording.focus_up",
        "up",
        "Focus previous option",
      ),
      action(
        "screenshot_recording.focus_down",
        "down",
        "Focus next option",
      ),
      action(
        "screenshot_recording.confirm",
        "enter",
        "Confirm selected option",
      ),
      action("screenshot_recording.back", "esc", "Back to settings"),
      action(
        "screenshot_recording.focus_screenshot_settings",
        "1",
        "Focus screenshot settings",
      ),
      action(
        "screenshot_recording.focus_recording_settings",
        "2",
        "Focus recording settings",
      ),
      action(
        "screenshot_recording.focus_screenshot_list",
        "3",
        "Focus screenshot list",
      ),
      action(
        "screenshot_recording.focus_recording_list",
        "4",
        "Focus recording list",
      ),
    ]
  }

  pub fn handle_event(&mut self, event: &UiEvent) -> Option<ScreenshotRecordingCommand> {
    match event {
      UiEvent::HitArea(HitAreaEvent::HoverEnter { id, .. }) => {
        self.selected_index = self.menu_areas.iter().position(|area| area == id)?;
        None
      }
      UiEvent::HitArea(HitAreaEvent::Click {
        id,
        button: MouseButton::Left,
        ..
      }) => {
        self.selected_index = self.menu_areas.iter().position(|area| area == id)?;
        self.activate_selected()
      }
      UiEvent::HitArea(HitAreaEvent::Press {
        button: MouseButton::Right,
        ..
      }) => Some(ScreenshotRecordingCommand::Back),
      UiEvent::Action(event) if event.state == KeyState::Pressed => match event.action.as_str() {
        "screenshot_recording.focus_up" => {
          self.selected_index = (self.selected_index + MENU_LEN - 1) % MENU_LEN;
          None
        }
        "screenshot_recording.focus_down" => {
          self.selected_index = (self.selected_index + 1) % MENU_LEN;
          None
        }
        "screenshot_recording.confirm" => self.activate_selected(),
        "screenshot_recording.back" => Some(ScreenshotRecordingCommand::Back),
        "screenshot_recording.focus_screenshot_settings" => self.focus(0),
        "screenshot_recording.focus_recording_settings" => self.focus(1),
        "screenshot_recording.focus_screenshot_list" => self.focus(2),
        "screenshot_recording.focus_recording_list" => self.focus(3),
        _ => None,
      },
      _ => None,
    }
  }

  pub fn update(&mut self, dt: Duration) {
    let _ = dt;
  }

  pub fn screenshot_settings_mut(&mut self) -> &mut ScreenshotSettingsUi {
    &mut self.screenshot_settings
  }

  pub fn render(
    &mut self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    i18n: &I18nService,
    hit_area: &HitAreaService,
  ) {
    let viewport = layout.developer_viewport_rect();
    let title = i18n.get_runtime_text(NS, "screenshot_recording.title");
    let title_x = viewport.x.saturating_add(layout.resolve_x(
      LayoutService::ALIGN_CENTER,
      layout.get_text_width(&title, None),
      0,
    ));
    let title_y = viewport.y.saturating_add(1);
    let hint = self.hint(i18n);
    let params = RichTextParams::from_action_map(&Self::action_map(), "screenshot_recording.");
    let hint_x = viewport.x.saturating_add(layout.resolve_x(
      LayoutService::ALIGN_CENTER,
      layout.get_text_width(&hint, Some(&params)),
      0,
    ));
    let hint_y = viewport.y.saturating_add(viewport.height.saturating_sub(1));
    let available = hint_y.saturating_sub(title_y).saturating_sub(1);
    let menu_y = title_y
      .saturating_add(1)
      .saturating_add(available.saturating_sub(MENU_LEN as u16) / 2);

    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: title_x,
        y: title_y,
        text: format!("f%<fg:bright_magenta>{title}</fg>"),
        bold: true,
        ..Default::default()
      },
    );

    hit_area.render_host(&mut self.objects, self.back_area, viewport, canvas);
    for index in 0..MENU_LEN {
      let text = self.menu_item(i18n, index);
      let width = layout.get_text_width(&text, None);
      let rect = Rect {
        x: viewport
          .x
          .saturating_add(layout.resolve_x(LayoutService::ALIGN_CENTER, width, 0)),
        y: menu_y.saturating_add(index as u16),
        width,
        height: 1,
      };
      render.draw_host_text(
        canvas,
        &DrawTextParams {
          x: rect.x,
          y: rect.y,
          text,
          ..Default::default()
        },
      );
      hit_area.render_host(&mut self.objects, self.menu_areas[index], rect, canvas);
    }

    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: hint_x,
        y: hint_y,
        text: hint,
        params: Some(params),
        ..Default::default()
      },
    );
  }

  fn focus(&mut self, index: usize) -> Option<ScreenshotRecordingCommand> {
    self.selected_index = index;
    None
  }

  fn activate_selected(&self) -> Option<ScreenshotRecordingCommand> {
    (self.selected_index == 0).then_some(ScreenshotRecordingCommand::OpenScreenshotSettings)
  }

  fn menu_item(&self, i18n: &I18nService, index: usize) -> String {
    let label = i18n.get_runtime_text(NS, MENU_KEYS[index]);
    if index == self.selected_index {
      format!("f%<fg:bright_cyan>❯ {label} ❮</fg>")
    } else {
      label
    }
  }

  fn hint(&self, i18n: &I18nService) -> String {
    format!(
      "f%<fg:rgb(85,87,83)>{}  {}  {}  {}</fg>",
      i18n.get_runtime_text(NS, "screenshot_recording.action.focus"),
      i18n.get_runtime_text(NS, "screenshot_recording.action.select"),
      i18n.get_runtime_text(NS, "screenshot_recording.action.confirm"),
      i18n.get_runtime_text(NS, "screenshot_recording.action.back"),
    )
  }
}

fn action(name: &str, key: &str, description: &str) -> ActionMapEntry {
  ActionMapEntry {
    action: name.to_string(),
    description: description.to_string(),
    keys: vec![vec![key.to_string()]],
  }
}
