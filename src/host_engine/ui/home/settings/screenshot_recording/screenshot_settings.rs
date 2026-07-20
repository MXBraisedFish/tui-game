use std::time::Duration;

use crate::host_engine::services::{
  ActionMapEntry, CanvasService, DrawTextParams, HitAreaEvent, HitAreaId, HitAreaOptions,
  HitAreaService, I18nService, KeyState, LayoutService, MouseButton, Rect, RenderService,
  RichTextParams, RuntimeObjectPool, RuntimeObjectPoolOwner, ScreenshotDoubleAction,
  ScreenshotProfile, ScrollBoxService, TextInputService, UiEvent, UiObjectPool, UiObjectPoolOwner,
};

use super::fonts_settings::{FontsSettingsCommand, FontsSettingsUi};

const NS: &str = "screenshot_settings";
const MENU_LEN: usize = 4;
const LABEL_KEYS: [&str; MENU_LEN] = [
  "screenshot_settings.re_hint",
  "screenshot_settings.fonts",
  "screenshot_settings.double",
  "screenshot_settings.auto_exit",
];

pub struct ScreenshotSettingsUi {
  selected_index: usize,
  profile: ScreenshotProfile,
  objects: UiObjectPool,
  runtime_objects: RuntimeObjectPool,
  back_area: HitAreaId,
  menu_areas: [HitAreaId; MENU_LEN],
  fonts_open: bool,
  fonts: FontsSettingsUi,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScreenshotSettingsCommand {
  Back,
  Changed(ScreenshotProfile),
  OpenFonts,
  StartAddFont,
  StartModifyFont,
  FinishFontEdit(String),
  CancelFontEdit,
  ScrollFonts(i32),
}

impl UiObjectPoolOwner for ScreenshotSettingsUi {
  fn objects(&self) -> &UiObjectPool {
    &self.objects
  }

  fn objects_mut(&mut self) -> &mut UiObjectPool {
    &mut self.objects
  }
}

impl RuntimeObjectPoolOwner for ScreenshotSettingsUi {
  fn runtime_objects(&self) -> &RuntimeObjectPool {
    &self.runtime_objects
  }

  fn runtime_objects_mut(&mut self) -> &mut RuntimeObjectPool {
    &mut self.runtime_objects
  }
}

impl ScreenshotSettingsUi {
  pub fn init(
    hit_area: &HitAreaService,
    text_input: &TextInputService,
    scroll_box: &ScrollBoxService,
    profile: ScreenshotProfile,
  ) -> Self {
    let mut objects = UiObjectPool::new();
    let fonts = FontsSettingsUi::create(&mut objects, text_input, scroll_box);
    Self {
      selected_index: 0,
      profile,
      back_area: hit_area.create(&mut objects, HitAreaOptions::default()),
      menu_areas: std::array::from_fn(|_| hit_area.create(&mut objects, HitAreaOptions::default())),
      fonts_open: false,
      fonts,
      objects,
      runtime_objects: RuntimeObjectPool::new(),
    }
  }

  pub fn action_map() -> Vec<ActionMapEntry> {
    let mut actions = vec![
      action(
        "screenshot_settings.focus_up",
        "up",
        "Focus previous option",
      ),
      action(
        "screenshot_settings.focus_down",
        "down",
        "Focus next option",
      ),
      action(
        "screenshot_settings.confirm",
        "enter",
        "Activate selected option",
      ),
      action("screenshot_settings.back", "esc", "Back"),
      action(
        "screenshot_settings.focus_re_hint",
        "1",
        "Focus guide reset",
      ),
      action("screenshot_settings.focus_fonts", "2", "Focus custom fonts"),
      action(
        "screenshot_settings.focus_double",
        "3",
        "Focus double action",
      ),
      action(
        "screenshot_settings.focus_auto_exit",
        "4",
        "Focus auto exit",
      ),
    ];
    actions.extend(FontsSettingsUi::action_map());
    actions
  }

  pub fn handle_event(&mut self, event: &UiEvent) -> Option<ScreenshotSettingsCommand> {
    if self.fonts_open {
      return self.fonts.handle_event(event).map(|command| match command {
        FontsSettingsCommand::Back(fonts) => {
          self.profile.fonts = fonts;
          self.fonts_open = false;
          ScreenshotSettingsCommand::Changed(self.profile.clone())
        }
        FontsSettingsCommand::StartAdd => ScreenshotSettingsCommand::StartAddFont,
        FontsSettingsCommand::StartModify => ScreenshotSettingsCommand::StartModifyFont,
        FontsSettingsCommand::FinishEdit(value) => ScreenshotSettingsCommand::FinishFontEdit(value),
        FontsSettingsCommand::CancelEdit => ScreenshotSettingsCommand::CancelFontEdit,
        FontsSettingsCommand::Scroll(dy) => ScreenshotSettingsCommand::ScrollFonts(dy),
      });
    }
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
      }) => Some(ScreenshotSettingsCommand::Back),
      UiEvent::Action(event) if event.state == KeyState::Pressed => match event.action.as_str() {
        "screenshot_settings.focus_up" => {
          self.selected_index = (self.selected_index + MENU_LEN - 1) % MENU_LEN;
          None
        }
        "screenshot_settings.focus_down" => {
          self.selected_index = (self.selected_index + 1) % MENU_LEN;
          None
        }
        "screenshot_settings.confirm" => self.activate_selected(),
        "screenshot_settings.back" => Some(ScreenshotSettingsCommand::Back),
        "screenshot_settings.focus_re_hint" => self.focus(0),
        "screenshot_settings.focus_fonts" => self.focus(1),
        "screenshot_settings.focus_double" => self.focus(2),
        "screenshot_settings.focus_auto_exit" => self.focus(3),
        _ => None,
      },
      _ => None,
    }
  }

  pub fn update(&mut self, dt: Duration) {
    let _ = dt;
  }

  pub fn open_fonts(&mut self) {
    self.fonts_open = true;
    self.fonts.enter(self.profile.fonts.clone());
  }

  pub fn start_add_font(&mut self, text_input: &mut TextInputService) {
    self.fonts.start_add(&mut self.objects, text_input);
  }

  pub fn start_modify_font(&mut self, text_input: &mut TextInputService) {
    self.fonts.start_modify(&mut self.objects, text_input);
  }

  pub fn finish_font_edit(&mut self, text_input: &mut TextInputService, value: String) {
    self.fonts.finish_edit(&mut self.objects, text_input, value);
  }

  pub fn cancel_font_edit(&mut self, text_input: &mut TextInputService) {
    self.fonts.cancel_edit(&mut self.objects, text_input);
  }

  pub fn prepare_surfaces(&mut self, scroll_box: &ScrollBoxService, layout: &LayoutService) {
    if self.fonts_open {
      self.fonts.prepare(&mut self.objects, scroll_box, layout);
    }
  }

  pub fn scroll_fonts(&mut self, scroll_box: &ScrollBoxService, layout: &LayoutService, dy: i32) {
    if self.fonts_open {
      self.fonts.scroll(&mut self.objects, scroll_box, layout, dy);
    }
  }

  pub fn render(
    &mut self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    i18n: &I18nService,
    hit_area: &HitAreaService,
    text_input: &TextInputService,
  ) -> Option<(u16, u16)> {
    if self.fonts_open {
      return self
        .fonts
        .render(&mut self.objects, render, canvas, layout, i18n, text_input);
    }
    let viewport = layout.developer_viewport_rect();
    let title = i18n.get_runtime_text(NS, "screenshot_settings.title");
    let title_y = viewport.y.saturating_add(1);
    let hint = self.hint(i18n);
    let params = RichTextParams::from_action_map(&Self::action_map(), "screenshot_settings.");
    let hint_y = viewport.y.saturating_add(viewport.height.saturating_sub(1));
    let rows = self.rows(i18n, layout);
    let available = hint_y.saturating_sub(title_y).saturating_sub(1);
    let menu_y = title_y
      .saturating_add(1)
      .saturating_add(available.saturating_sub(MENU_LEN as u16) / 2);

    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: centered_x(viewport, layout.get_text_width(&title, None)),
        y: title_y,
        text: format!("f%<fg:bright_magenta>{title}</fg>"),
        bold: true,
        ..Default::default()
      },
    );

    hit_area.render_host(&mut self.objects, self.back_area, viewport, canvas);
    for (index, row) in rows.iter().enumerate() {
      let width = layout.get_text_width(row, None);
      let rect = Rect {
        x: centered_x(viewport, width),
        y: menu_y.saturating_add(index as u16),
        width,
        height: 1,
      };
      render.draw_host_text(
        canvas,
        &DrawTextParams {
          x: rect.x,
          y: rect.y,
          text: row.clone(),
          ..Default::default()
        },
      );
      hit_area.render_host(&mut self.objects, self.menu_areas[index], rect, canvas);
    }

    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: centered_x(viewport, layout.get_text_width(&hint, Some(&params))),
        y: hint_y,
        text: hint,
        params: Some(params),
        ..Default::default()
      },
    );
    None
  }

  fn activate_selected(&mut self) -> Option<ScreenshotSettingsCommand> {
    match self.selected_index {
      0 => self.profile.guide_seen = false,
      1 => return Some(ScreenshotSettingsCommand::OpenFonts),
      2 => self.profile.double_action = self.profile.double_action.next(),
      3 => self.profile.auto_exit = !self.profile.auto_exit,
      _ => return None,
    }
    Some(ScreenshotSettingsCommand::Changed(self.profile.clone()))
  }

  fn focus(&mut self, index: usize) -> Option<ScreenshotSettingsCommand> {
    self.selected_index = index;
    None
  }

  fn rows(&self, i18n: &I18nService, layout: &LayoutService) -> [String; MENU_LEN] {
    let labels: [String; MENU_LEN] =
      std::array::from_fn(|index| i18n.get_runtime_text(NS, LABEL_KEYS[index]));
    let values = [
      String::new(),
      String::new(),
      i18n.get_runtime_text(NS, double_action_key(self.profile.double_action)),
      i18n.get_runtime_text(
        NS,
        if self.profile.auto_exit {
          "screenshot_settings.auto_exit.action"
        } else {
          "screenshot_settings.auto_exit.no"
        },
      ),
    ];
    let label_width = labels[2..]
      .iter()
      .map(|label| layout.get_text_width(label, None))
      .max()
      .unwrap_or_default();
    let value_width = values[2..]
      .iter()
      .map(|value| layout.get_text_width(value, None))
      .max()
      .unwrap_or_default();

    std::array::from_fn(|index| {
      let focused = index == self.selected_index;
      let color = if focused { "bright_cyan" } else { "white" };
      let prefix = if focused { "❯ " } else { "  " };
      let suffix = if focused { " ❮" } else { "  " };
      if index < 2 {
        return format!("f%<fg:{color}>{prefix}{}{suffix}</fg>", labels[index]);
      }
      let padding = " ".repeat(
        label_width
          .saturating_sub(layout.get_text_width(&labels[index], None))
          .saturating_add(value_width.saturating_sub(layout.get_text_width(&values[index], None)))
          as usize,
      );
      format!(
        "f%<fg:{color}>{prefix}{}{padding}  </fg><fg:white>[</fg><fg:{}>{}</fg><fg:white>]</fg><fg:{color}>{suffix}</fg>",
        labels[index],
        if index == 3 && !self.profile.auto_exit {
          "rgb(85,87,83)"
        } else {
          "bright_yellow"
        },
        values[index],
      )
    })
  }

  fn hint(&self, i18n: &I18nService) -> String {
    format!(
      "f%<fg:rgb(85,87,83)>{}  {}  {}  {}</fg>",
      i18n.get_runtime_text(NS, "screenshot_settings.action.focus"),
      i18n.get_runtime_text(NS, "screenshot_settings.action.select"),
      i18n.get_runtime_text(NS, "screenshot_settings.action.type.select"),
      i18n.get_runtime_text(NS, "screenshot_settings.action.back"),
    )
  }
}

fn centered_x(viewport: Rect, width: u16) -> u16 {
  viewport
    .x
    .saturating_add(viewport.width.saturating_sub(width) / 2)
}

fn double_action_key(action: ScreenshotDoubleAction) -> &'static str {
  match action {
    ScreenshotDoubleAction::Copy => "screenshot_settings.double.copy",
    ScreenshotDoubleAction::CopyRichText => "screenshot_settings.double.copy_rich_text",
    ScreenshotDoubleAction::SavePng => "screenshot_settings.double.save_png",
    ScreenshotDoubleAction::All => "screenshot_settings.double.all",
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
  fn defaults_match_requested_screenshot_behavior() {
    let profile = ScreenshotProfile::default();
    assert_eq!(profile.double_action, ScreenshotDoubleAction::SavePng);
    assert!(!profile.auto_exit);
  }

  #[test]
  fn selectable_settings_update_the_profile_snapshot() {
    let hit_area = HitAreaService::new();
    let mut ui = ScreenshotSettingsUi::init(
      &hit_area,
      &TextInputService::new(),
      &ScrollBoxService::new(),
      ScreenshotProfile::default(),
    );
    ui.selected_index = 2;
    assert_eq!(
      ui.activate_selected(),
      Some(ScreenshotSettingsCommand::Changed(ScreenshotProfile {
        double_action: ScreenshotDoubleAction::All,
        ..Default::default()
      }))
    );
    ui.selected_index = 3;
    assert_eq!(
      ui.activate_selected(),
      Some(ScreenshotSettingsCommand::Changed(ScreenshotProfile {
        double_action: ScreenshotDoubleAction::All,
        auto_exit: true,
        ..Default::default()
      }))
    );
  }
}
