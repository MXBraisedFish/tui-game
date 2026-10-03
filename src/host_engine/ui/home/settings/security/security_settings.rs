//! Security settings page state, user commands, and terminal-cell presentation.

use std::time::Duration;

use crate::host_engine::services::{
  ActionMapEntry, CanvasService, DrawTextParams, HitAreaEvent, HitAreaId, HitAreaOptions,
  HitAreaService, I18nService, KeyState, LayoutService, MouseButton, Rect, RenderService,
  RichTextParams, RuntimeObjectPool, RuntimeObjectPoolOwner, UiEvent, UiObjectPool,
  UiObjectPoolOwner,
};

const NS: &str = "security_settings";
const MENU_LEN: usize = 5;
const ROW_LEN: usize = 5;
const DEFAULT_START: usize = 3;
const LABEL_KEYS: [&str; ROW_LEN] = [
  "security_settings.reset_terminal",
  "security_settings.mod.reset.status",
  "security_settings.mod.reset.debug",
  "security_settings.mod.default.status",
  "security_settings.mod.default.debug",
];
/// The state and owned widgets of the security settings view.
pub struct SecuritySettingsUi {
  selected_index: usize,
  objects: UiObjectPool,
  runtime_objects: RuntimeObjectPool,
  back_area: HitAreaId,
  menu_areas: [HitAreaId; MENU_LEN],
  default_enabled: bool,
  default_debug: bool,
}

/// An application request produced by security settings interactions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SecuritySettingsCommand {
  /// A request to back.
  Back,
  /// The reset terminal setting for security settings command.
  ResetTerminal,
  /// The reset status setting for security settings command.
  ResetStatus,
  /// The reset debug setting for security settings command.
  ResetDebug,
  /// A request to set default status.
  SetDefaultStatus(bool),
  /// A request to set default debug.
  SetDefaultDebug(bool),
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
  /// Create the security settings view and allocate its owned UI objects.
  pub fn init(hit_area: &HitAreaService) -> Self {
    let mut objects = UiObjectPool::new();
    Self {
      selected_index: 0,
      back_area: hit_area.create(&mut objects, HitAreaOptions::default()),
      menu_areas: std::array::from_fn(|_| hit_area.create(&mut objects, HitAreaOptions::default())),
      default_enabled: true,
      default_debug: false,
      objects,
      runtime_objects: RuntimeObjectPool::new(),
    }
  }

  /// Return the shortcuts currently enabled by the security settings view.
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
      action(
        "security_settings.focus_reset_terminal",
        "1",
        "Focus terminal capability reset",
      ),
      action(
        "security_settings.focus_reset_status",
        "2",
        "Focus package status reset",
      ),
      action(
        "security_settings.focus_reset_debug",
        "3",
        "Focus package debug reset",
      ),
      action(
        "security_settings.focus_default_status",
        "4",
        "Focus default package status",
      ),
      action(
        "security_settings.focus_default_debug",
        "5",
        "Focus default package debug",
      ),
    ]
  }

  /// Interpret a security settings UI event and return the requested application command.
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
        "security_settings.focus_reset_terminal" => self.focus(0),
        "security_settings.focus_reset_status" => self.focus(1),
        "security_settings.focus_reset_debug" => self.focus(2),
        "security_settings.focus_default_status" => self.focus(3),
        "security_settings.focus_default_debug" => self.focus(4),
        _ => None,
      },
      _ => None,
    }
  }

  /// Advance the security settings view's transient state for this host frame.
  pub fn update(&mut self, _dt: Duration) {}

  fn focus(&mut self, index: usize) -> Option<SecuritySettingsCommand> {
    self.selected_index = index.min(MENU_LEN.saturating_sub(1));
    None
  }

  /// Update the defaults used by this security settings ui.
  pub fn set_defaults(&mut self, enabled: bool, debug: bool) {
    self.default_enabled = enabled;
    self.default_debug = debug;
  }

  /// Draw the security settings view and register interaction regions in its assigned surfaces.
  ///
  /// # Arguments
  ///
  /// * `render` - The drawing service used to render terminal cells.
  /// * `canvas` - The clipped canvas used for drawing.
  /// * `layout` - The service resolving terminal sizes and positions.
  /// * `i18n` - The service resolving localized text.
  /// * `hit_area` - The hit area.
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
    let default_row_width = self.default_row_width(i18n, layout);
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
      let width = if index >= DEFAULT_START {
        default_row_width
      } else {
        widths[index]
      };
      let x = viewport
        .x
        .saturating_add(layout.resolve_x(LayoutService::ALIGN_CENTER, width, 0));
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
            width,
            height: 1,
          },
          canvas,
        );
      }
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
      0 => SecuritySettingsCommand::ResetTerminal,
      1 => SecuritySettingsCommand::ResetStatus,
      2 => SecuritySettingsCommand::ResetDebug,
      3 => SecuritySettingsCommand::SetDefaultStatus(!self.default_enabled),
      _ => SecuritySettingsCommand::SetDefaultDebug(!self.default_debug),
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
    let bracket_width = (DEFAULT_START..ROW_LEN)
      .filter_map(|index| self.value_key(index))
      .map(|key| layout.get_text_width(&i18n.get_runtime_text(NS, key), None))
      .max()
      .unwrap_or_default()
      .saturating_add(2);
    std::array::from_fn(|index| {
      let focused = index == self.selected_index;
      let label_color = if focused { "bright_cyan" } else { "white" };
      let prefix = if focused { "❯ " } else { "" };
      let suffix = if focused { " ❮" } else { "" };
      let Some(key) = self.value_key(index) else {
        return format!("f%<fg:{label_color}>{prefix}{}{suffix}</fg>", labels[index]);
      };
      let prefix = if focused { "❯ " } else { "  " };
      let suffix = if focused { " ❮" } else { "  " };
      let width = layout.get_text_width(&labels[index], None);
      let value = i18n.get_runtime_text(NS, key);
      let current_bracket_width = layout.get_text_width(&value, None).saturating_add(2);
      let padding = " ".repeat(
        label_width
          .saturating_sub(width)
          .saturating_add(bracket_width.saturating_sub(current_bracket_width)) as usize,
      );
      format!(
        "f%<fg:{label_color}>{prefix}{}{padding}  </fg><fg:white>[</fg><fg:{}>{value}</fg><fg:white>]</fg><fg:{label_color}>{suffix}</fg>",
        labels[index],
        value_color(key),
      )
    })
  }

  fn default_row_width(&self, i18n: &I18nService, layout: &LayoutService) -> u16 {
    let label_width = LABEL_KEYS
      .iter()
      .skip(DEFAULT_START)
      .map(|key| layout.get_text_width(&i18n.get_runtime_text(NS, key), None))
      .max()
      .unwrap_or_default();
    let bracket_width = (DEFAULT_START..ROW_LEN)
      .filter_map(|index| self.value_key(index))
      .map(|key| layout.get_text_width(&i18n.get_runtime_text(NS, key), None))
      .max()
      .unwrap_or_default()
      .saturating_add(2);
    label_width
      .saturating_add(2)
      .saturating_add(bracket_width)
      .saturating_add(4)
  }

  fn value_key(&self, index: usize) -> Option<&'static str> {
    match index {
      3 if self.default_enabled => Some("security_settings.reset.status.on"),
      3 => Some("security_settings.reset.status.off"),
      4 if self.default_debug => Some("security_settings.reset.debug.on"),
      4 => Some("security_settings.reset.debug.off"),
      _ => None,
    }
  }

  fn hint(&self, i18n: &I18nService) -> String {
    let confirm = if self.selected_index < DEFAULT_START {
      "security_settings.action.confirm"
    } else {
      "security_settings.action.switch"
    };
    format!(
      "f%<fg:rgb(85,87,83)>{}  {}  {}  {}</fg>",
      i18n.get_runtime_text(NS, "security_settings.action.focus"),
      i18n.get_runtime_text(NS, "security_settings.action.select"),
      i18n.get_runtime_text(NS, confirm),
      i18n.get_runtime_text(NS, "security_settings.action.back"),
    )
  }
}

fn value_color(key: &str) -> &'static str {
  match key {
    "security_settings.reset.status.off" => "bright_red",
    "security_settings.reset.status.on" => "bright_green",
    "security_settings.reset.debug.on" => "bright_magenta",
    "security_settings.reset.debug.off" => "rgb(85,87,83)",
    _ => "white",
  }
}

fn action(name: &str, key: &str, description: &str) -> ActionMapEntry {
  ActionMapEntry {
    action: name.to_string(),
    description: description.to_string(),
    keys: vec![vec![key.to_string()]],
    priority: 0,
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn menu_contains_only_security_settings_actions() {
    assert_eq!(LABEL_KEYS.len(), MENU_LEN);

    let mut ui = SecuritySettingsUi::init(&HitAreaService::new());
    let expected = [
      SecuritySettingsCommand::ResetTerminal,
      SecuritySettingsCommand::ResetStatus,
      SecuritySettingsCommand::ResetDebug,
      SecuritySettingsCommand::SetDefaultStatus(false),
      SecuritySettingsCommand::SetDefaultDebug(true),
    ];

    for (index, command) in expected.into_iter().enumerate() {
      ui.selected_index = index;
      assert_eq!(ui.confirm_selected(), Some(command));
    }
  }

  #[test]
  fn default_options_are_focusable_and_emit_switch_commands() {
    let mut ui = SecuritySettingsUi::init(&HitAreaService::new());
    ui.selected_index = 3;
    assert_eq!(
      ui.confirm_selected(),
      Some(SecuritySettingsCommand::SetDefaultStatus(false))
    );
    ui.selected_index = 4;
    assert_eq!(
      ui.confirm_selected(),
      Some(SecuritySettingsCommand::SetDefaultDebug(true))
    );
  }
}
