use std::time::Duration;

use crate::host_engine::services::{
  ActionMapEntry, CanvasService, DrawTextParams, HitAreaEvent, HitAreaId, HitAreaOptions,
  HitAreaService, I18nService, KeyState, LayoutService, MouseButton, Rect, RenderService,
  RichTextParams, RuntimeObjectPool, RuntimeObjectPoolOwner, SafeModeDefault, UiEvent,
  UiObjectPool, UiObjectPoolOwner,
};

const NS: &str = "security_settings";
const MENU_LEN: usize = 7;
const ROW_LEN: usize = 7;
const DEFAULT_START: usize = 4;
const RESET_RESULT_DURATION: Duration = Duration::from_secs(3);
const LABEL_KEYS: [&str; ROW_LEN] = [
  "security_settings.security_details",
  "security_settings.mod.reset.status",
  "security_settings.mod.reset.debug",
  "security_settings.mod.reset.safe_mode",
  "security_settings.mod.default.status",
  "security_settings.mod.default.debug",
  "security_settings.mod.default.safe_mode",
];
pub struct SecuritySettingsUi {
  selected_index: usize,
  objects: UiObjectPool,
  runtime_objects: RuntimeObjectPool,
  back_area: HitAreaId,
  menu_areas: [HitAreaId; MENU_LEN],
  reset_result: Option<(bool, Duration)>,
  default_enabled: bool,
  default_debug: bool,
  default_safe_mode: SafeModeDefault,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SecuritySettingsCommand {
  Back,
  OpenDetails,
  ResetStatus,
  ResetDebug,
  ResetSafeMode,
  SetDefaultStatus(bool),
  SetDefaultDebug(bool),
  SetDefaultSafeMode(SafeModeDefault),
}

impl UiObjectPoolOwner for SecuritySettingsUi {
  fn objects(&self) -> &UiObjectPool {
    &self.objects
  }
  fn objects_mut(&mut self) -> &mut UiObjectPool {
    &mut self.objects
  }
}

impl RuntimeObjectPoolOwner for SecuritySettingsUi {
  fn runtime_objects(&self) -> &RuntimeObjectPool {
    &self.runtime_objects
  }
  fn runtime_objects_mut(&mut self) -> &mut RuntimeObjectPool {
    &mut self.runtime_objects
  }
}

impl SecuritySettingsUi {
  pub fn init(hit_area: &HitAreaService) -> Self {
    let mut objects = UiObjectPool::new();
    Self {
      selected_index: 0,
      back_area: hit_area.create(&mut objects, HitAreaOptions::default()),
      menu_areas: std::array::from_fn(|_| hit_area.create(&mut objects, HitAreaOptions::default())),
      reset_result: None,
      default_enabled: true,
      default_debug: false,
      default_safe_mode: SafeModeDefault::On,
      objects,
      runtime_objects: RuntimeObjectPool::new(),
    }
  }

  pub fn action_map() -> Vec<ActionMapEntry> {
    vec![
      action("security_settings.focus_up", "up", "Focus previous option"),
      action("security_settings.focus_down", "down", "Focus next option"),
      action(
        "security_settings.confirm",
        "enter",
        "Confirm selected option",
      ),
      action("security_settings.back", "esc", "Back to settings"),
    ]
  }

  pub fn handle_event(&mut self, event: &UiEvent) -> Option<SecuritySettingsCommand> {
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
        self.confirm_selected()
      }
      UiEvent::HitArea(HitAreaEvent::Press {
        button: MouseButton::Right,
        ..
      }) => Some(SecuritySettingsCommand::Back),
      UiEvent::Action(event) if event.state == KeyState::Pressed => match event.action.as_str() {
        "security_settings.focus_up" => {
          self.selected_index = (self.selected_index + MENU_LEN - 1) % MENU_LEN;
          None
        }
        "security_settings.focus_down" => {
          self.selected_index = (self.selected_index + 1) % MENU_LEN;
          None
        }
        "security_settings.confirm" => self.confirm_selected(),
        "security_settings.back" => Some(SecuritySettingsCommand::Back),
        _ => None,
      },
      _ => None,
    }
  }

  pub fn update(&mut self, dt: Duration) {
    let Some((_, remaining)) = self.reset_result.as_mut() else {
      return;
    };
    if dt >= *remaining {
      self.reset_result = None;
    } else {
      *remaining -= dt;
    }
  }

  pub fn set_reset_result(&mut self, success: bool) {
    self.reset_result = Some((success, RESET_RESULT_DURATION));
  }

  pub fn set_defaults(&mut self, enabled: bool, debug: bool, safe_mode: SafeModeDefault) {
    self.default_enabled = enabled;
    self.default_debug = debug;
    self.default_safe_mode = safe_mode;
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
    let title = i18n.get_runtime_text(NS, "security_settings.title");
    let title_x = viewport.x.saturating_add(layout.resolve_x(
      LayoutService::ALIGN_CENTER,
      layout.get_text_width(&title, None),
      0,
    ));
    let title_y = viewport.y.saturating_add(1);
    let hint = self.hint(i18n);
    let params = RichTextParams::from_action_map(&Self::action_map(), "security_settings.");
    let hint_x = viewport.x.saturating_add(layout.resolve_x(
      LayoutService::ALIGN_CENTER,
      layout.get_text_width(&hint, Some(&params)),
      0,
    ));
    let hint_y = viewport.y.saturating_add(viewport.height.saturating_sub(1));
    let rows = self.rows(i18n, layout);
    let widths: [u16; ROW_LEN] = std::array::from_fn(|i| layout.get_text_width(&rows[i], None));
    let start_y = title_y.saturating_add(1).saturating_add(
      hint_y
        .saturating_sub(title_y)
        .saturating_sub(1)
        .saturating_sub(ROW_LEN as u16)
        / 2,
    );

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
    for (index, row) in rows.iter().enumerate() {
      let x =
        viewport
          .x
          .saturating_add(layout.resolve_x(LayoutService::ALIGN_CENTER, widths[index], 0));
      let y = start_y.saturating_add(index as u16);
      render.draw_host_text(
        canvas,
        &DrawTextParams {
          x,
          y,
          text: row.clone(),
          ..Default::default()
        },
      );
      if let Some(id) = self.menu_areas.get(index).copied() {
        hit_area.render_host(
          &mut self.objects,
          id,
          Rect {
            x,
            y,
            width: widths[index],
            height: 1,
          },
          canvas,
        );
      }
    }
    if let Some((success, _)) = self.reset_result {
      let key = if success {
        "security_settings.reset.success"
      } else {
        "security_settings.reset.fail"
      };
      let color = if success {
        "bright_green"
      } else {
        "bright_red"
      };
      let message = i18n.get_runtime_text(NS, key);
      let x = viewport.x.saturating_add(layout.resolve_x(
        LayoutService::ALIGN_CENTER,
        layout.get_text_width(&message, None),
        0,
      ));
      render.draw_host_text(
        canvas,
        &DrawTextParams {
          x,
          y: start_y.saturating_add(ROW_LEN as u16).saturating_add(1),
          text: format!("f%<fg:{color}>{message}</fg>"),
          ..Default::default()
        },
      );
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

  fn confirm_selected(&self) -> Option<SecuritySettingsCommand> {
    Some(match self.selected_index {
      0 => SecuritySettingsCommand::OpenDetails,
      1 => SecuritySettingsCommand::ResetStatus,
      2 => SecuritySettingsCommand::ResetDebug,
      3 => SecuritySettingsCommand::ResetSafeMode,
      4 => SecuritySettingsCommand::SetDefaultStatus(!self.default_enabled),
      5 => SecuritySettingsCommand::SetDefaultDebug(!self.default_debug),
      _ => SecuritySettingsCommand::SetDefaultSafeMode(match self.default_safe_mode {
        SafeModeDefault::On => SafeModeDefault::OffPermanent,
        SafeModeDefault::OffPermanent => SafeModeDefault::On,
      }),
    })
  }

  fn rows(&self, i18n: &I18nService, layout: &LayoutService) -> [String; ROW_LEN] {
    let labels: [String; ROW_LEN] =
      std::array::from_fn(|index| i18n.get_runtime_text(NS, LABEL_KEYS[index]));
    let label_width = labels
      .iter()
      .skip(DEFAULT_START)
      .map(|label| layout.get_text_width(label, None))
      .max()
      .unwrap_or_default();
    std::array::from_fn(|index| {
      let focused = index == self.selected_index;
      let label_color = if focused { "bright_cyan" } else { "white" };
      let prefix = if focused { "❯ " } else { "" };
      let suffix = if focused { " ❮" } else { "" };
      let Some(key) = self.value_key(index) else {
        return format!("f%<fg:{label_color}>{prefix}{}{suffix}</fg>", labels[index]);
      };
      let width = layout.get_text_width(&labels[index], None);
      let padding = " ".repeat(label_width.saturating_sub(width) as usize);
      let value = i18n.get_runtime_text(NS, key);
      format!(
        "f%<fg:{label_color}>{prefix}{}{padding}  </fg><fg:white>[</fg><fg:{}>{value}</fg><fg:white>]</fg><fg:{label_color}>{suffix}</fg>",
        labels[index],
        value_color(key),
      )
    })
  }

  fn value_key(&self, index: usize) -> Option<&'static str> {
    match index {
      4 if self.default_enabled => Some("security_settings.reset.status.on"),
      4 => Some("security_settings.reset.status.off"),
      5 if self.default_debug => Some("security_settings.reset.debug.on"),
      5 => Some("security_settings.reset.debug.off"),
      6 => Some(match self.default_safe_mode {
        SafeModeDefault::On => "security_settings.reset.safe_mode.on",
        SafeModeDefault::OffPermanent => "security_settings.reset.safe_mode.off",
      }),
      _ => None,
    }
  }

  fn hint(&self, i18n: &I18nService) -> String {
    format!(
      "f%<fg:rgb(85,87,83)>{}  {}  {}</fg>",
      i18n.get_runtime_text(NS, "security_settings.action.select"),
      i18n.get_runtime_text(NS, "security_settings.action.switch"),
      i18n.get_runtime_text(NS, "security_settings.action.back"),
    )
  }
}

fn value_color(key: &str) -> &'static str {
  match key {
    "security_settings.reset.status.off" => "bright_red",
    "security_settings.reset.status.on" | "security_settings.reset.safe_mode.on" => "bright_green",
    "security_settings.reset.debug.on" => "bright_magenta",
    "security_settings.reset.debug.off" => "rgb(85,87,83)",
    "security_settings.reset.safe_mode.off" => "bright_red",
    _ => "white",
  }
}

fn action(name: &str, key: &str, description: &str) -> ActionMapEntry {
  ActionMapEntry {
    action: name.to_string(),
    description: description.to_string(),
    keys: vec![vec![key.to_string()]],
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn reset_result_disappears_after_three_seconds() {
    let mut ui = SecuritySettingsUi::init(&HitAreaService::new());
    ui.set_reset_result(true);
    ui.update(Duration::from_millis(2999));
    assert!(ui.reset_result.is_some());
    ui.update(Duration::from_millis(1));
    assert!(ui.reset_result.is_none());
  }

  #[test]
  fn default_options_are_focusable_and_emit_switch_commands() {
    let mut ui = SecuritySettingsUi::init(&HitAreaService::new());
    ui.selected_index = 4;
    assert_eq!(
      ui.confirm_selected(),
      Some(SecuritySettingsCommand::SetDefaultStatus(false))
    );
    ui.selected_index = 5;
    assert_eq!(
      ui.confirm_selected(),
      Some(SecuritySettingsCommand::SetDefaultDebug(true))
    );
    ui.selected_index = 6;
    assert_eq!(
      ui.confirm_selected(),
      Some(SecuritySettingsCommand::SetDefaultSafeMode(
        SafeModeDefault::OffPermanent
      ))
    );
  }
}
