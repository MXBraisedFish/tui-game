use std::time::Duration;

use crate::host_engine::services::{
  ActionMapEntry, CanvasService, DrawTextParams, HitAreaEvent, HitAreaId, HitAreaOptions,
  HitAreaService, I18nService, KeyState, LayoutService, MouseButton, Rect, RenderService,
  RichTextParams, RuntimeObjectPool, RuntimeObjectPoolOwner, UiEvent, UiObjectPool,
  UiObjectPoolOwner,
};

const NS: &str = "display_settings";
const MENU_LEN: usize = 6;
const LABEL_KEYS: [&str; MENU_LEN] = [
  "display_settings.logo.random",
  "display_settings.tool.top_toolbar",
  "display_settings.screensaver.source",
  "display_settings.screensaver.random",
  "display_settings.game_list.source",
  "display_settings.game_list.error",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LogoMode {
  Random,
  Order,
  Neon,
  Sign,
  Water,
  Error,
  Glitch,
  Building,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SourceMode {
  All,
  Mod,
  Official,
  No,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OrderMode {
  Random,
  Order,
}

pub struct DisplaySettingsUi {
  selected_index: usize,
  objects: UiObjectPool,
  runtime_objects: RuntimeObjectPool,
  back_area: HitAreaId,
  menu_areas: [HitAreaId; MENU_LEN],
  logo_mode: LogoMode,
  top_toolbar: bool,
  screensaver_source: SourceMode,
  screensaver_order: OrderMode,
  game_list_source: SourceMode,
  game_list_error: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisplaySettingsCommand {
  Back,
}

impl UiObjectPoolOwner for DisplaySettingsUi {
  fn objects(&self) -> &UiObjectPool {
    &self.objects
  }

  fn objects_mut(&mut self) -> &mut UiObjectPool {
    &mut self.objects
  }
}

impl RuntimeObjectPoolOwner for DisplaySettingsUi {
  fn runtime_objects(&self) -> &RuntimeObjectPool {
    &self.runtime_objects
  }

  fn runtime_objects_mut(&mut self) -> &mut RuntimeObjectPool {
    &mut self.runtime_objects
  }
}

impl DisplaySettingsUi {
  pub fn init(hit_area: &HitAreaService) -> Self {
    let mut objects = UiObjectPool::new();
    Self {
      selected_index: 0,
      back_area: hit_area.create(&mut objects, HitAreaOptions::default()),
      menu_areas: std::array::from_fn(|_| hit_area.create(&mut objects, HitAreaOptions::default())),
      objects,
      runtime_objects: RuntimeObjectPool::new(),
      logo_mode: LogoMode::Random,
      top_toolbar: true,
      screensaver_source: SourceMode::All,
      screensaver_order: OrderMode::Random,
      game_list_source: SourceMode::All,
      game_list_error: true,
    }
  }

  pub fn action_map() -> Vec<ActionMapEntry> {
    vec![
      action("display_settings.focus_up", "up", "Focus previous option"),
      action("display_settings.focus_down", "down", "Focus next option"),
      action(
        "display_settings.confirm",
        "enter",
        "Switch selected option",
      ),
      action("display_settings.back", "esc", "Back to settings"),
      action(
        "display_settings.focus_logo_random",
        "1",
        "Focus logo display",
      ),
      action(
        "display_settings.focus_tool_top_toolbar",
        "2",
        "Focus top toolbar",
      ),
      action(
        "display_settings.focus_screensaver_source",
        "3",
        "Focus screensaver source",
      ),
      action(
        "display_settings.focus_screensaver_random",
        "4",
        "Focus screensaver order",
      ),
      action(
        "display_settings.focus_game_list_source",
        "5",
        "Focus game list source",
      ),
      action(
        "display_settings.focus_game_list_error",
        "6",
        "Focus game list warnings",
      ),
    ]
  }

  pub fn handle_event(&mut self, event: &UiEvent) -> Option<DisplaySettingsCommand> {
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
        self.switch_selected();
        None
      }
      UiEvent::HitArea(HitAreaEvent::Press {
        button: MouseButton::Right,
        ..
      }) => Some(DisplaySettingsCommand::Back),
      UiEvent::Action(event) if event.state == KeyState::Pressed => match event.action.as_str() {
        "display_settings.focus_up" => {
          self.focus_previous();
          None
        }
        "display_settings.focus_down" => {
          self.focus_next();
          None
        }
        "display_settings.confirm" => {
          self.switch_selected();
          None
        }
        "display_settings.back" => Some(DisplaySettingsCommand::Back),
        "display_settings.focus_logo_random" => {
          self.selected_index = 0;
          None
        }
        "display_settings.focus_tool_top_toolbar" => {
          self.selected_index = 1;
          None
        }
        "display_settings.focus_screensaver_source" => {
          self.selected_index = 2;
          None
        }
        "display_settings.focus_screensaver_random" => {
          self.selected_index = 3;
          None
        }
        "display_settings.focus_game_list_source" => {
          self.selected_index = 4;
          None
        }
        "display_settings.focus_game_list_error" => {
          self.selected_index = 5;
          None
        }
        _ => None,
      },
      _ => None,
    }
  }

  pub fn update(&mut self, dt: Duration) -> Option<DisplaySettingsCommand> {
    let _ = dt;
    None
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
    let rows = self.rows(i18n, layout);
    let row_width = self.row_width(i18n, layout);
    let title = i18n.get_runtime_text(NS, "display_settings.title");
    let title_x = viewport.x.saturating_add(layout.resolve_x(
      LayoutService::ALIGN_CENTER,
      layout.get_text_width(&title, None),
      0,
    ));
    let title_y = viewport.y.saturating_add(1);
    let hint = self.hint(i18n);
    let params = self.build_key_params();
    let hint_x = viewport.x.saturating_add(layout.resolve_x(
      LayoutService::ALIGN_CENTER,
      layout.get_text_width(&hint, Some(&params)),
      0,
    ));
    let hint_y = viewport.y.saturating_add(viewport.height.saturating_sub(1));
    let available = hint_y.saturating_sub(title_y).saturating_sub(1);
    let start_y = if available > MENU_LEN as u16 {
      title_y
        .saturating_add(1)
        .saturating_add((available - MENU_LEN as u16) / 2)
    } else {
      title_y.saturating_add(1)
    };

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
          .saturating_add(layout.resolve_x(LayoutService::ALIGN_CENTER, row_width, 0));
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
      hit_area.render_host(
        &mut self.objects,
        self.menu_areas[index],
        Rect {
          x,
          y,
          width: row_width,
          height: 1,
        },
        canvas,
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

  fn switch_selected(&mut self) {
    match self.selected_index {
      0 => self.logo_mode = self.logo_mode.next(),
      1 => self.top_toolbar = !self.top_toolbar,
      2 => self.screensaver_source = self.screensaver_source.next(),
      3 => self.screensaver_order = self.screensaver_order.next(),
      4 => self.game_list_source = self.game_list_source.next(),
      _ => self.game_list_error = !self.game_list_error,
    }
  }

  fn focus_previous(&mut self) {
    self.selected_index = if self.selected_index == 0 {
      MENU_LEN - 1
    } else {
      self.selected_index - 1
    };
  }

  fn focus_next(&mut self) {
    self.selected_index = (self.selected_index + 1) % MENU_LEN;
  }

  fn rows(&self, i18n: &I18nService, layout: &LayoutService) -> [String; MENU_LEN] {
    let labels: [String; MENU_LEN] =
      std::array::from_fn(|index| i18n.get_runtime_text(NS, LABEL_KEYS[index]));
    let values: [String; MENU_LEN] =
      std::array::from_fn(|index| i18n.get_runtime_text(NS, self.value_key(index)));
    let label_width = labels
      .iter()
      .map(|label| layout.get_text_width(label, None))
      .max()
      .unwrap_or_default();
    let bracket_width = values
      .iter()
      .map(|value| layout.get_text_width(value, None))
      .max()
      .unwrap_or_default()
      .saturating_add(2);
    std::array::from_fn(|index| {
      let focused = index == self.selected_index;
      let label_color = if focused { "bright_cyan" } else { "white" };
      let prefix = if focused { "❯ " } else { "  " };
      let suffix = if focused { " ❮" } else { "  " };
      let value_key = self.value_key(index);
      let width = layout.get_text_width(&labels[index], None);
      let current_bracket_width = layout
        .get_text_width(&values[index], None)
        .saturating_add(2);
      let padding = " ".repeat(
        label_width
          .saturating_sub(width)
          .saturating_add(bracket_width.saturating_sub(current_bracket_width)) as usize,
      );
      format!(
        "f%<fg:{label_color}>{prefix}{}{padding}  </fg><fg:white>[</fg><fg:{}>{}</fg><fg:white>]</fg><fg:{label_color}>{suffix}</fg>",
        labels[index],
        value_color(value_key),
        values[index],
      )
    })
  }

  fn row_width(&self, i18n: &I18nService, layout: &LayoutService) -> u16 {
    let label_width = LABEL_KEYS
      .iter()
      .map(|key| layout.get_text_width(&i18n.get_runtime_text(NS, key), None))
      .max()
      .unwrap_or_default();
    let bracket_width = (0..MENU_LEN)
      .map(|index| layout.get_text_width(&i18n.get_runtime_text(NS, self.value_key(index)), None))
      .max()
      .unwrap_or_default()
      .saturating_add(2);
    label_width
      .saturating_add(2)
      .saturating_add(bracket_width)
      .saturating_add(4)
  }

  fn value_key(&self, index: usize) -> &'static str {
    match index {
      0 => self.logo_mode.key(),
      1 if self.top_toolbar => "display_settings.tool.top_toolbar.on",
      1 => "display_settings.tool.top_toolbar.off",
      2 => self.screensaver_source.screensaver_key(),
      3 => self.screensaver_order.key(),
      4 => self.game_list_source.game_list_key(),
      5 if self.game_list_error => "display_settings.game_list.error.yes",
      _ => "display_settings.game_list.error.no",
    }
  }

  fn hint(&self, i18n: &I18nService) -> String {
    format!(
      "f%<fg:rgb(85,87,83)>{}  {}  {}  {}</fg>",
      i18n.get_runtime_text(NS, "display_settings.action.focus"),
      i18n.get_runtime_text(NS, "display_settings.action.select"),
      i18n.get_runtime_text(NS, "display_settings.action.type.select"),
      i18n.get_runtime_text(NS, "display_settings.action.back"),
    )
  }

  fn build_key_params(&self) -> RichTextParams {
    RichTextParams::from_action_map(&Self::action_map(), "display_settings.")
  }
}

impl LogoMode {
  fn next(self) -> Self {
    match self {
      Self::Random => Self::Order,
      Self::Order => Self::Neon,
      Self::Neon => Self::Sign,
      Self::Sign => Self::Water,
      Self::Water => Self::Error,
      Self::Error => Self::Glitch,
      Self::Glitch => Self::Building,
      Self::Building => Self::Random,
    }
  }

  fn key(self) -> &'static str {
    match self {
      Self::Random => "display_settings.logo.random.random",
      Self::Order => "display_settings.logo.random.order",
      Self::Neon => "display_settings.logo.random.only.neon",
      Self::Sign => "display_settings.logo.random.only.sign",
      Self::Water => "display_settings.logo.random.only.water",
      Self::Error => "display_settings.logo.random.only.error",
      Self::Glitch => "display_settings.logo.random.only.glitch",
      Self::Building => "display_settings.logo.random.only.building",
    }
  }
}

impl SourceMode {
  fn next(self) -> Self {
    match self {
      Self::All => Self::Mod,
      Self::Mod => Self::Official,
      Self::Official => Self::No,
      Self::No => Self::All,
    }
  }

  fn screensaver_key(self) -> &'static str {
    match self {
      Self::All => "display_settings.screensaver.source.all",
      Self::Mod => "display_settings.screensaver.source.mod",
      Self::Official => "display_settings.screensaver.source.official",
      Self::No => "display_settings.screensaver.source.no",
    }
  }

  fn game_list_key(self) -> &'static str {
    match self {
      Self::All => "display_settings.game_list.source.all",
      Self::Mod => "display_settings.game_list.source.mod",
      Self::Official => "display_settings.game_list.source.official",
      Self::No => "display_settings.game_list.source.no",
    }
  }
}

impl OrderMode {
  fn next(self) -> Self {
    match self {
      Self::Random => Self::Order,
      Self::Order => Self::Random,
    }
  }

  fn key(self) -> &'static str {
    match self {
      Self::Random => "display_settings.screensaver.random.random",
      Self::Order => "display_settings.screensaver.random.order",
    }
  }
}

fn value_color(key: &str) -> &'static str {
  if key.ends_with(".off") || key.ends_with(".no") {
    "rgb(85,87,83)"
  } else if key.ends_with(".on") || key.ends_with(".yes") {
    "bright_green"
  } else {
    "bright_yellow"
  }
}

fn action(name: &str, key: &str, description: &str) -> ActionMapEntry {
  ActionMapEntry {
    action: name.to_string(),
    description: description.to_string(),
    keys: vec![vec![key.to_string()]],
  }
}
