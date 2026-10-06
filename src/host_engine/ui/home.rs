//! Home page state, user commands, and terminal-cell presentation.

mod about;
mod game_list;
mod logo;
mod settings;

pub use about::{InputDemoCommand, InputDemoUi};
pub(crate) use game_list::GameListRenderContext;
pub use game_list::{GameListCommand, GameListUi};
pub(crate) use settings::PackageListRenderContext;
pub use settings::display_settings::{DisplaySettingsCommand, DisplaySettingsUi};
pub use settings::key_bindings::{
  GameKeyBindingsCommand, GameKeyBindingsUi, GlobalKeyBindingsCommand, GlobalKeyBindingsUi,
  KeyBindingsCommand, KeyBindingsUi,
};
pub use settings::language::{LanguageSelectCommand, LanguageSelectUi};
pub use settings::mods::game::{GamePackageCommand, GamePackageUi};
pub use settings::mods::screensaver::{ScreensaverPackageCommand, ScreensaverPackageUi};
pub use settings::mods::{ModsCommand, ModsUi};
pub use settings::screensaver_list::{ScreensaverListCommand, ScreensaverListUi};
pub use settings::screenshot_recording::{
  MediaListNotice, MediaRenameError, RecordingListCommand, RecordingListUi,
  RecordingSettingsCommand, RecordingSettingsUi, ScreenshotListCommand, ScreenshotListUi,
  ScreenshotRecordingCommand, ScreenshotRecordingUi, ScreenshotSettingsCommand,
  ScreenshotSettingsUi,
};
pub use settings::security::{SecuritySettingsCommand, SecuritySettingsUi};
pub use settings::storage_management::{
  StorageManagementClearCommand, StorageManagementClearUi, StorageManagementCommand,
  StorageManagementExportCommand, StorageManagementExportUi, StorageManagementUi,
  StorageManagementViewCommand, StorageManagementViewUi,
};
pub use settings::toolbar_custom::ToolbarCustomCommand;
pub use settings::{SettingsUi, SettingsUiCommand};

use std::time::Duration;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::host_engine::services::{
  ActionMapEntry, AnimationService, CanvasService, DisplayLogoMode, DrawTextParams, HOST_VERSION,
  HitAreaEvent, HitAreaId, HitAreaOptions, HitAreaService, KeyState, LayoutService, MouseButton,
  RandomService, Rect, RenderService, RichTextParams, RuntimeObjectPool, RuntimeObjectPoolOwner,
  TextColor, UiEvent, UiObjectPool, UiObjectPoolOwner,
};

use crate::host_engine::services::I18nService;
use logo::HomeLogo;

const LOGO_LINES: &[&str] = &[
  "████████╗██╗   ██╗██╗     ██████╗  █████╗ ███╗   ███╗███████╗",
  "╚══██╔══╝██║   ██║██║    ██╔════╝ ██╔══██╗████╗ ████║██╔════╝",
  "   ██║   ██║   ██║██║    ██║  ███╗███████║██╔████╔██║█████╗  ",
  "   ██║   ██║   ██║██║    ██║   ██║██╔══██║██║╚██╔╝██║██╔══╝  ",
  "   ██║   ╚██████╔╝██║    ╚██████╔╝██║  ██║██║ ╚═╝ ██║███████╗",
  "   ╚═╝    ╚═════╝ ╚═╝     ╚═════╝ ╚═╝  ╚═╝╚═╝     ╚═╝╚══════╝",
];

const HOME_MENU_LEN: usize = 5;

fn style_logo(lines: &[&str]) -> String {
  let plain = lines.join("\n");
  let mut result = String::from("f%");
  let mut in_box = false;

  for ch in plain.chars() {
    let is_box = ch != '█' && ch != ' ' && ch != '\n';

    if is_box && !in_box {
      result.push_str("<fg:white>");
      in_box = true;
    } else if !is_box && in_box {
      result.push_str("</fg>");
      in_box = false;
    }

    result.push(ch);
  }

  if in_box {
    result.push_str("</fg>");
  }

  result
}

/// Resolved geometry and positions used to display home.
pub(crate) struct HomeLayout {
  logo_x: u16,
  logo_y: u16,
  menu_item_xs: [u16; HOME_MENU_LEN],
  menu_item_rects: [Rect; HOME_MENU_LEN],
  menu_y: u16,
  version_x: u16,
  version_y: u16,
  action_hint_x: u16,
  action_hint_y: u16,
}

/// The state and owned widgets of the home view.
pub struct HomeUi {
  selected_index: usize,
  continue_game_name: Option<String>,
  objects: UiObjectPool,
  runtime_objects: RuntimeObjectPool,
  menu_areas: [HitAreaId; HOME_MENU_LEN],
  logo: HomeLogo,
}

impl UiObjectPoolOwner for HomeUi {
  fn objects(&self) -> &UiObjectPool {
    &self.objects
  }

  fn objects_mut(&mut self) -> &mut UiObjectPool {
    &mut self.objects
  }
}

impl RuntimeObjectPoolOwner for HomeUi {
  fn runtime_objects(&self) -> &RuntimeObjectPool {
    &self.runtime_objects
  }

  fn runtime_objects_mut(&mut self) -> &mut RuntimeObjectPool {
    &mut self.runtime_objects
  }
}

/// An application request produced by home UI interactions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HomeUiCommand {
  /// A request to start game.
  StartGame,
  /// The continue game setting for home UI command.
  ContinueGame,
  /// A request to open settings.
  OpenSettings,
  /// A request to open about.
  OpenAbout,
  /// The exit setting for home UI command.
  Exit,
}

impl HomeUi {
  /// Select the next logo mode using the retained sequence cursor.
  pub(crate) fn sequential_logo_mode(cursor: u64) -> DisplayLogoMode {
    logo::dynamic_mode_for_cursor(cursor)
  }

  /// Create the home view and allocate its owned UI objects.
  ///
  /// # Arguments
  ///
  /// * `hit_area` - The hit area.
  /// * `animation` - The animation.
  /// * `random` - The random.
  /// * `logo_mode` - The logo mode.
  /// * `logo_seed` - The logo seed.
  pub fn init(
    hit_area: &HitAreaService,
    animation: &AnimationService,
    random: &RandomService,
    logo_mode: DisplayLogoMode,
    logo_seed: u64,
  ) -> Self {
    let mut objects = UiObjectPool::new();
    let mut runtime_objects = RuntimeObjectPool::new();
    let logo = HomeLogo::new(
      logo_mode,
      logo_seed,
      animation,
      random,
      &mut runtime_objects,
    );
    Self {
      selected_index: 0,
      continue_game_name: None,
      menu_areas: std::array::from_fn(|_| hit_area.create(&mut objects, HitAreaOptions::default())),
      objects,
      runtime_objects,
      logo,
    }
  }

  /// Return the shortcuts currently enabled by the home view.
  pub fn action_map() -> Vec<ActionMapEntry> {
    vec![
      ActionMapEntry {
        action: "home.focus_exit".to_string(),
        description: "Focus exit option".to_string(),
        keys: vec![vec!["esc".to_string()]],
        priority: 0,
      },
      ActionMapEntry {
        action: "home.focus_start_game".to_string(),
        description: "Focus start game option".to_string(),
        keys: vec![vec!["1".to_string()]],
        priority: 0,
      },
      ActionMapEntry {
        action: "home.focus_continue_game".to_string(),
        description: "Focus continue game option".to_string(),
        keys: vec![vec!["2".to_string()]],
        priority: 0,
      },
      ActionMapEntry {
        action: "home.focus_settings".to_string(),
        description: "Focus settings option".to_string(),
        keys: vec![vec!["3".to_string()]],
        priority: 0,
      },
      ActionMapEntry {
        action: "home.focus_about".to_string(),
        description: "Focus about option".to_string(),
        keys: vec![vec!["4".to_string()]],
        priority: 0,
      },
      ActionMapEntry {
        action: "home.focus_up".to_string(),
        description: "Focus previous option".to_string(),
        keys: vec![vec!["up".to_string()]],
        priority: 0,
      },
      ActionMapEntry {
        action: "home.focus_down".to_string(),
        description: "Focus next option".to_string(),
        keys: vec![vec!["down".to_string()]],
        priority: 0,
      },
      ActionMapEntry {
        action: "home.confirm".to_string(),
        description: "Confirm selected option".to_string(),
        keys: vec![vec!["enter".to_string()]],
        priority: 0,
      },
    ]
  }

  /// Interpret a home UI event and return the requested application command.
  pub fn handle_event(&mut self, event: &UiEvent) -> Option<HomeUiCommand> {
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
      UiEvent::Action(event) if event.state == KeyState::Pressed => match event.action.as_str() {
        "home.focus_exit" => {
          self.selected_index = 4;
          None
        }

        "home.focus_start_game" => {
          self.selected_index = 0;
          None
        }

        "home.focus_continue_game" => {
          if self.continue_game_name.is_some() {
            self.selected_index = 1;
          }
          None
        }

        "home.focus_settings" => {
          self.selected_index = 2;
          None
        }

        "home.focus_about" => {
          self.selected_index = 3;
          None
        }

        "home.focus_up" => {
          self.focus_previous();
          None
        }

        "home.focus_down" => {
          self.focus_next();
          None
        }

        "home.confirm" => self.confirm_selected(),

        _ => None,
      },
      _ => None,
    }
  }

  /// Advance the home view's transient state for this host frame.
  ///
  /// # Arguments
  ///
  /// * `dt` - The elapsed duration applied to this update.
  /// * `animation` - The animation.
  /// * `random` - The random.
  pub fn update(
    &mut self,
    dt: Duration,
    animation: &AnimationService,
    random: &RandomService,
  ) -> Option<HomeUiCommand> {
    self
      .logo
      .update(dt, animation, random, &mut self.runtime_objects);
    None
  }

  /// Draw the home view and register interaction regions in its assigned surfaces.
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
    for (id, rect) in self.menu_areas.into_iter().zip(positions.menu_item_rects) {
      let rect = if id == self.menu_areas[1] && self.continue_game_name.is_none() {
        Rect::default()
      } else {
        rect
      };
      hit_area.render_host(&mut self.objects, id, rect, canvas);
    }
  }

  /// Update the continue game name used by this home ui.
  pub fn set_continue_game_name(&mut self, name: Option<String>) {
    self.continue_game_name = name.map(|name| truncate_visible(&name, 12));
    if self.continue_game_name.is_none() && self.selected_index == 1 {
      self.selected_index = 0;
    }
  }

  fn focus_previous(&mut self) {
    loop {
      self.selected_index = if self.selected_index == 0 {
        HOME_MENU_LEN - 1
      } else {
        self.selected_index - 1
      };
      if self.selected_index != 1 || self.continue_game_name.is_some() {
        break;
      }
    }
  }

  fn focus_next(&mut self) {
    loop {
      self.selected_index = (self.selected_index + 1) % HOME_MENU_LEN;
      if self.selected_index != 1 || self.continue_game_name.is_some() {
        break;
      }
    }
  }

  fn confirm_selected(&self) -> Option<HomeUiCommand> {
    match self.selected_index {
      0 => Some(HomeUiCommand::StartGame),
      1 if self.continue_game_name.is_some() => Some(HomeUiCommand::ContinueGame),
      1 => None,
      2 => Some(HomeUiCommand::OpenSettings),
      3 => Some(HomeUiCommand::OpenAbout),
      _ => Some(HomeUiCommand::Exit),
    }
  }

  fn menu_items(&self, i18n: &I18nService) -> [String; HOME_MENU_LEN] {
    let labels = [
      i18n.get_runtime_text("home", "home.game_list"),
      self
        .continue_game_name
        .as_ref()
        .map(|name| {
          format!(
            "{} - {}",
            i18n.get_runtime_text("home", "home.countinue"),
            name
          )
        })
        .unwrap_or_else(|| i18n.get_runtime_text("home", "home.countinue")),
      i18n.get_runtime_text("home", "home.settings"),
      i18n.get_runtime_text("home", "home.about"),
      i18n.get_runtime_text("home", "home.exit"),
    ];

    std::array::from_fn(|i| {
      if i == 1 && self.continue_game_name.is_none() {
        format!("f%<fg:rgb(85,87,83)>{}</fg>", labels[i])
      } else if i == self.selected_index {
        let fg = if i >= 4 { "bright_red" } else { "bright_cyan" };
        format!("f%<fg:{}>❯ {} ❮</fg>", fg, labels[i])
      } else {
        labels[i].to_string()
      }
    })
  }

  fn build_key_params(&self) -> RichTextParams {
    RichTextParams::from_action_map(&Self::action_map(), "home.")
  }

  /// Resolve the home view's terminal-cell layout from its available dimensions.
  pub(crate) fn compute_positions(&self, layout: &LayoutService, i18n: &I18nService) -> HomeLayout {
    let params = self.build_key_params();
    let viewport = layout.developer_viewport_rect();

    let logo = self.logo.template_text(LOGO_LINES);
    let logo_size = layout.get_text_size(&logo, None);

    let menu_items = self.menu_items(i18n);
    let menu_item_widths: [u16; HOME_MENU_LEN] =
      std::array::from_fn(|i| layout.get_text_width(&menu_items[i], None));

    let menu_item_xs: [u16; HOME_MENU_LEN] = std::array::from_fn(|i| {
      viewport.x.saturating_add(layout.resolve_x(
        LayoutService::ALIGN_CENTER,
        menu_item_widths[i],
        0,
      ))
    });
    let menu_height = HOME_MENU_LEN as u16;

    let version_width = layout.get_text_width(
      &format!("f%<fg:rgb(85,87,83)>v{}</fg>", HOST_VERSION).to_string(),
      None,
    );
    let action_hint = format!(
      "f%<fg:bright_black>{}  {}  {}</fg>",
      i18n.get_runtime_text("home", "home.action.focus"),
      i18n.get_runtime_text("home", "home.action.select"),
      i18n.get_runtime_text("home", "home.action.confirm"),
    );
    let action_hint_w = layout.get_text_width(&action_hint, Some(&params));
    let total_height = logo_size
      .height
      .saturating_add(2)
      .saturating_add(menu_height)
      .saturating_add(2)
      .saturating_add(1)
      .saturating_add(1);

    let start_y =
      viewport
        .y
        .saturating_add(layout.resolve_y(LayoutService::ALIGN_MIDDLE, total_height, 0));

    let logo_x =
      viewport
        .x
        .saturating_add(layout.resolve_x(LayoutService::ALIGN_CENTER, logo_size.width, 0));
    let logo_y = start_y;

    let menu_y = logo_y.saturating_add(logo_size.height).saturating_add(2);

    let menu_item_rects: [Rect; HOME_MENU_LEN] = std::array::from_fn(|i| Rect {
      x: menu_item_xs[i],
      y: menu_y.saturating_add(i as u16),
      width: menu_item_widths[i],
      height: 1,
    });

    let version_x =
      viewport
        .x
        .saturating_add(layout.resolve_x(LayoutService::ALIGN_CENTER, version_width, 0));
    let version_y = menu_y.saturating_add(menu_height).saturating_add(2);

    let action_hint_x =
      viewport
        .x
        .saturating_add(layout.resolve_x(LayoutService::ALIGN_CENTER, action_hint_w, 0));
    let action_hint_y = version_y.saturating_add(2);

    HomeLayout {
      logo_x,
      logo_y,
      menu_item_xs,
      menu_item_rects,
      menu_y,
      version_x,
      version_y,
      action_hint_x,
      action_hint_y,
    }
  }

  fn draw_content(
    &self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    positions: &HomeLayout,
    i18n: &I18nService,
  ) {
    let logo = self.logo.render_text(|| style_logo(LOGO_LINES));
    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: positions.logo_x,
        y: self.logo.render_y(positions.logo_y),
        text: logo,
        fg: (self.logo.mode() == DisplayLogoMode::Classic).then_some(TextColor::Rgb {
          r: 255,
          g: 165,
          b: 0,
        }),
        ..Default::default()
      },
    );

    let menu_items = self.menu_items(i18n);
    for (i, item) in menu_items.iter().enumerate() {
      render.draw_host_text(
        canvas,
        &DrawTextParams {
          x: positions.menu_item_xs[i],
          y: positions.menu_y.saturating_add(i as u16),
          text: item.clone(),
          ..Default::default()
        },
      );
    }

    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: positions.version_x,
        y: positions.version_y,
        text: format!("f%<fg:rgb(85,87,83)>v{}</fg>", HOST_VERSION),
        ..Default::default()
      },
    );
    let params = self.build_key_params();
    let action_hint = format!(
      "f%<fg:rgb(85,87,83)>{}  {}  {}</fg>",
      i18n.get_runtime_text("home", "home.action.focus"),
      i18n.get_runtime_text("home", "home.action.select"),
      i18n.get_runtime_text("home", "home.action.confirm"),
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

fn truncate_visible(text: &str, max_width: usize) -> String {
  if UnicodeWidthStr::width(text) <= max_width {
    return text.to_string();
  }

  const ELLIPSIS: &str = "...";
  let ellipsis_width = UnicodeWidthStr::width(ELLIPSIS);
  if max_width <= ellipsis_width {
    return ELLIPSIS.chars().take(max_width).collect();
  }

  let mut result = String::new();
  let mut width = 0;
  for grapheme in UnicodeSegmentation::graphemes(text, true) {
    let grapheme_width = UnicodeWidthStr::width(grapheme);
    if width + grapheme_width + ellipsis_width > max_width {
      break;
    }
    result.push_str(grapheme);
    width += grapheme_width;
  }
  result.push_str(ELLIPSIS);
  result
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

  fn home() -> HomeUi {
    HomeUi::init(
      &HitAreaService::new(),
      &AnimationService::new(),
      &RandomService::new(),
      DisplayLogoMode::Classic,
      1,
    )
  }

  #[test]
  fn unavailable_continue_game_is_skipped_and_cannot_be_confirmed() {
    let mut ui = home();
    ui.handle_event(&action("home.focus_continue_game"));
    assert_eq!(ui.selected_index, 0);

    ui.handle_event(&action("home.focus_down"));
    assert_eq!(ui.selected_index, 2);
    ui.handle_event(&action("home.focus_up"));
    assert_eq!(ui.selected_index, 0);
  }

  #[test]
  fn available_continue_game_can_be_selected_and_is_width_limited() {
    let mut ui = home();
    ui.set_continue_game_name(Some("继续游戏名称".to_string()));
    assert_eq!(ui.continue_game_name.as_deref(), Some("继续游戏名称"));

    ui.handle_event(&action("home.focus_continue_game"));
    assert_eq!(ui.selected_index, 1);
    assert_eq!(
      ui.handle_event(&action("home.confirm")),
      Some(HomeUiCommand::ContinueGame)
    );

    ui.set_continue_game_name(None);
    assert_eq!(ui.selected_index, 0);
  }
}
