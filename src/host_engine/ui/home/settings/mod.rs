//! Settings page state, user commands, and terminal-cell presentation.

use std::time::Duration;

use crate::host_engine::services::{
  ActionMapEntry, CanvasService, DrawTextParams, HitAreaEvent, HitAreaId, HitAreaOptions,
  HitAreaService, I18nService, KeyState, LayoutService, MouseButton, Rect, RenderService,
  RichTextParams, RuntimeObjectPool, RuntimeObjectPoolOwner, ScrollBoxService, TextInputService,
  UiEvent, UiObjectPool, UiObjectPoolOwner,
};

pub mod display_settings;
pub mod key_bindings;
pub mod language;
pub mod mods;
pub mod screensaver_list;
pub mod screenshot_recording;
pub mod security;
pub mod storage_management;
pub mod toolbar_custom;

pub(crate) use mods::PackageListRenderContext;

use key_bindings::KeyBindingsUi;
use screenshot_recording::ScreenshotRecordingUi;

const SETTINGS_MENU_LEN: usize = 8;

const MENU_KEYS: &[&str] = &[
  "settings.language",
  "settings.key_bindings",
  "settings.mod",
  "settings.storage_management",
  "settings.security_settings",
  "settings.display_settings",
  "settings.screensaver_list",
  "settings.screenshot_recording",
];

/// Resolved geometry and positions used to display settings.
pub(crate) struct SettingsLayout {
  title_x: u16,
  title_y: u16,
  menu_item_rects: [Rect; SETTINGS_MENU_LEN],
  action_hint_x: u16,
  action_hint_y: u16,
}

/// The state and owned widgets of the settings view.
pub struct SettingsUi {
  selected_index: usize,
  objects: UiObjectPool,
  runtime_objects: RuntimeObjectPool,
  back_area: HitAreaId,
  menu_areas: [HitAreaId; SETTINGS_MENU_LEN],
  screenshot_recording: ScreenshotRecordingUi,
  key_bindings: KeyBindingsUi,
}

impl UiObjectPoolOwner for SettingsUi {
  fn objects(&self) -> &UiObjectPool {
    &self.objects
  }

  fn objects_mut(&mut self) -> &mut UiObjectPool {
    &mut self.objects
  }
}

impl RuntimeObjectPoolOwner for SettingsUi {
  fn runtime_objects(&self) -> &RuntimeObjectPool {
    &self.runtime_objects
  }

  fn runtime_objects_mut(&mut self) -> &mut RuntimeObjectPool {
    &mut self.runtime_objects
  }
}

/// An application request produced by settings UI interactions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsUiCommand {
  /// A request to back.
  Back,
  /// A request to open language select.
  OpenLanguageSelect,
  /// A request to open key bindings.
  OpenKeyBindings,
  /// A request to open mods.
  OpenMods,
  /// A request to open storage management.
  OpenStorageManagement,
  /// A request to open security settings.
  OpenSecuritySettings,
  /// A request to open display settings.
  OpenDisplaySettings,
  /// A request to open screensaver list.
  OpenScreensaverList,
  /// A request to open screenshot recording.
  OpenScreenshotRecording,
}

impl SettingsUi {
  /// Create the settings view and allocate its owned UI objects.
  ///
  /// # Arguments
  ///
  /// * `hit_area` - The hit area.
  /// * `text_input` - The text input.
  /// * `scroll_box` - The scroll box.
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
      screenshot_recording: ScreenshotRecordingUi::init(hit_area, text_input, scroll_box),
      key_bindings: KeyBindingsUi::init(hit_area, text_input, scroll_box),
    }
  }

  /// Return mutable access to the owned screenshot recording.
  pub fn screenshot_recording_mut(&mut self) -> &mut ScreenshotRecordingUi {
    &mut self.screenshot_recording
  }

  /// Return mutable access to the owned key bindings.
  pub fn key_bindings_mut(&mut self) -> &mut KeyBindingsUi {
    &mut self.key_bindings
  }

  /// Return the shortcuts currently enabled by the settings view.
  pub fn action_map() -> Vec<ActionMapEntry> {
    vec![
      ActionMapEntry {
        action: "settings.focus_up".to_string(),
        description: "Focus previous option".to_string(),
        keys: vec![vec!["up".to_string()]],
        priority: 0,
      },
      ActionMapEntry {
        action: "settings.focus_down".to_string(),
        description: "Focus next option".to_string(),
        keys: vec![vec!["down".to_string()]],
        priority: 0,
      },
      ActionMapEntry {
        action: "settings.confirm".to_string(),
        description: "Confirm selected option".to_string(),
        keys: vec![vec!["enter".to_string()]],
        priority: 0,
      },
      ActionMapEntry {
        action: "settings.back".to_string(),
        description: "Go back to home".to_string(),
        keys: vec![vec!["esc".to_string()]],
        priority: 0,
      },
      ActionMapEntry {
        action: "settings.focus_language".to_string(),
        description: "Focus language option".to_string(),
        keys: vec![vec!["1".to_string()]],
        priority: 0,
      },
      ActionMapEntry {
        action: "settings.focus_key_bindings".to_string(),
        description: "Focus key bindings option".to_string(),
        keys: vec![vec!["2".to_string()]],
        priority: 0,
      },
      ActionMapEntry {
        action: "settings.focus_mod".to_string(),
        description: "Focus mod option".to_string(),
        keys: vec![vec!["3".to_string()]],
        priority: 0,
      },
      ActionMapEntry {
        action: "settings.focus_storage_management".to_string(),
        description: "Focus storage management option".to_string(),
        keys: vec![vec!["4".to_string()]],
        priority: 0,
      },
      ActionMapEntry {
        action: "settings.focus_security_settings".to_string(),
        description: "Focus security settings option".to_string(),
        keys: vec![vec!["5".to_string()]],
        priority: 0,
      },
      ActionMapEntry {
        action: "settings.focus_display_settings".to_string(),
        description: "Focus display settings option".to_string(),
        keys: vec![vec!["6".to_string()]],
        priority: 0,
      },
      ActionMapEntry {
        action: "settings.focus_screensaver_list".to_string(),
        description: "Focus screensaver list option".to_string(),
        keys: vec![vec!["7".to_string()]],
        priority: 0,
      },
      ActionMapEntry {
        action: "settings.focus_screenshot_recording".to_string(),
        description: "Focus screenshot and recording option".to_string(),
        keys: vec![vec!["8".to_string()]],
        priority: 0,
      },
    ]
  }

  /// Interpret a settings UI event and return the requested application command.
  pub fn handle_event(&mut self, event: &UiEvent) -> Option<SettingsUiCommand> {
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
        match self.selected_index {
          0 => Some(SettingsUiCommand::OpenLanguageSelect),
          1 => Some(SettingsUiCommand::OpenKeyBindings),
          2 => Some(SettingsUiCommand::OpenMods),
          3 => Some(SettingsUiCommand::OpenStorageManagement),
          4 => Some(SettingsUiCommand::OpenSecuritySettings),
          5 => Some(SettingsUiCommand::OpenDisplaySettings),
          6 => Some(SettingsUiCommand::OpenScreensaverList),
          7 => Some(SettingsUiCommand::OpenScreenshotRecording),
          _ => None,
        }
      }
      UiEvent::HitArea(HitAreaEvent::Press {
        button: MouseButton::Right,
        ..
      }) => Some(SettingsUiCommand::Back),
      UiEvent::Action(event) if event.state == KeyState::Pressed => match event.action.as_str() {
        "settings.focus_up" => {
          self.focus_previous();
          None
        }
        "settings.focus_down" => {
          self.focus_next();
          None
        }
        "settings.confirm" => match self.selected_index {
          0 => Some(SettingsUiCommand::OpenLanguageSelect),
          1 => Some(SettingsUiCommand::OpenKeyBindings),
          2 => Some(SettingsUiCommand::OpenMods),
          3 => Some(SettingsUiCommand::OpenStorageManagement),
          4 => Some(SettingsUiCommand::OpenSecuritySettings),
          5 => Some(SettingsUiCommand::OpenDisplaySettings),
          6 => Some(SettingsUiCommand::OpenScreensaverList),
          7 => Some(SettingsUiCommand::OpenScreenshotRecording),
          _ => None,
        },
        "settings.back" => Some(SettingsUiCommand::Back),
        "settings.focus_language" => {
          self.selected_index = 0;
          None
        }
        "settings.focus_key_bindings" => {
          self.selected_index = 1;
          None
        }
        "settings.focus_mod" => {
          self.selected_index = 2;
          None
        }
        "settings.focus_storage_management" => {
          self.selected_index = 3;
          None
        }
        "settings.focus_security_settings" => {
          self.selected_index = 4;
          None
        }
        "settings.focus_display_settings" => {
          self.selected_index = 5;
          None
        }
        "settings.focus_screensaver_list" => {
          self.selected_index = 6;
          None
        }
        "settings.focus_screenshot_recording" => {
          self.selected_index = 7;
          None
        }

        _ => None,
      },
      _ => None,
    }
  }

  /// Advance the settings view's transient state for this host frame.
  pub fn update(&mut self, dt: Duration) -> Option<SettingsUiCommand> {
    let _ = dt;
    None
  }

  /// Draw the settings view and register interaction regions in its assigned surfaces.
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
    let positions = self.compute_positions(layout, i18n);
    self.draw_content(render, canvas, &positions, i18n);
    let viewport = layout.developer_viewport_rect();
    hit_area.render_host(&mut self.objects, self.back_area, viewport, canvas);
    for (id, rect) in self.menu_areas.into_iter().zip(positions.menu_item_rects) {
      hit_area.render_host(&mut self.objects, id, rect, canvas);
    }
  }

  /// Resolve the settings view's terminal-cell layout from its available dimensions.
  pub fn compute_positions(&self, layout: &LayoutService, i18n: &I18nService) -> SettingsLayout {
    let params = self.build_key_params();
    let viewport = layout.developer_viewport_rect();
    let title = i18n.get_runtime_text("settings", "settings.title");
    let title_w = layout.get_text_width(&format!("f%<b>{}<b>", title), None);
    let title_x =
      viewport
        .x
        .saturating_add(layout.resolve_x(LayoutService::ALIGN_CENTER, title_w, 0));
    let title_y = viewport.y.saturating_add(1);
    let menu_items = self.menu_items(i18n);
    let menu_item_widths: [u16; SETTINGS_MENU_LEN] =
      std::array::from_fn(|i| layout.get_text_width(&menu_items[i], None));
    let menu_item_xs: [u16; SETTINGS_MENU_LEN] = std::array::from_fn(|i| {
      viewport.x.saturating_add(layout.resolve_x(
        LayoutService::ALIGN_CENTER,
        menu_item_widths[i],
        0,
      ))
    });
    let menu_height = SETTINGS_MENU_LEN as u16;
    let action_hint = format!(
      "f%<fg:bright_black>{}  {}  {}  {}</fg>",
      i18n.get_runtime_text("settings", "settings.action.focus"),
      i18n.get_runtime_text("settings", "settings.action.select"),
      i18n.get_runtime_text("settings", "settings.action.confirm"),
      i18n.get_runtime_text("settings", "settings.action.back"),
    );
    let action_hint_w = layout.get_text_width(&action_hint, Some(&params));
    let action_hint_x =
      viewport
        .x
        .saturating_add(layout.resolve_x(LayoutService::ALIGN_CENTER, action_hint_w, 0));
    let action_hint_y = viewport
      .y
      .saturating_add(layout.developer_height().saturating_sub(1));
    let available = action_hint_y.saturating_sub(title_y).saturating_sub(1);
    let menu_y = if available > menu_height {
      title_y
        .saturating_add(1)
        .saturating_add((available - menu_height) / 2)
    } else {
      title_y.saturating_add(1)
    };

    let menu_item_rects: [Rect; SETTINGS_MENU_LEN] = std::array::from_fn(|i| Rect {
      x: menu_item_xs[i],
      y: menu_y.saturating_add(i as u16),
      width: menu_item_widths[i],
      height: 1,
    });

    SettingsLayout {
      title_x,
      title_y,
      menu_item_rects,
      action_hint_x,
      action_hint_y,
    }
  }

  fn focus_previous(&mut self) {
    if self.selected_index == 0 {
      self.selected_index = SETTINGS_MENU_LEN - 1;
    } else {
      self.selected_index -= 1;
    }
  }

  fn focus_next(&mut self) {
    self.selected_index = (self.selected_index + 1) % SETTINGS_MENU_LEN;
  }

  fn menu_items(&self, i18n: &I18nService) -> [String; SETTINGS_MENU_LEN] {
    std::array::from_fn(|i| {
      let label = i18n.get_runtime_text("settings", MENU_KEYS[i]);
      if i == self.selected_index {
        format!("f%<fg:bright_cyan>❯ {} ❮</fg>", label)
      } else {
        label
      }
    })
  }

  fn build_key_params(&self) -> RichTextParams {
    RichTextParams::from_action_map(&Self::action_map(), "settings.")
  }

  fn draw_content(
    &self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    positions: &SettingsLayout,
    i18n: &I18nService,
  ) {
    let title = i18n.get_runtime_text("settings", "settings.title");
    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: positions.title_x,
        y: positions.title_y,
        text: format!("f%<fg:bright_magenta>{}</fg>", title),
        bold: true,
        ..Default::default()
      },
    );
    let menu_items = self.menu_items(i18n);
    for (i, item) in menu_items.iter().enumerate() {
      render.draw_host_text(
        canvas,
        &DrawTextParams {
          x: positions.menu_item_rects[i].x,
          y: positions.menu_item_rects[i].y,
          text: item.clone(),
          ..Default::default()
        },
      );
    }
    let params = self.build_key_params();
    let action_hint = format!(
      "f%<fg:rgb(85,87,83)>{}  {}  {}  {}</fg>",
      i18n.get_runtime_text("settings", "settings.action.focus"),
      i18n.get_runtime_text("settings", "settings.action.select"),
      i18n.get_runtime_text("settings", "settings.action.confirm"),
      i18n.get_runtime_text("settings", "settings.action.back"),
    );
    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: positions.action_hint_x,
        y: positions.action_hint_y,
        text: action_hint,
        params: Some(params),
        ..Default::default()
      },
    );
  }
}
