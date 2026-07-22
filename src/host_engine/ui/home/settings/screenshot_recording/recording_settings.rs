use std::time::Duration;

use crate::host_engine::services::{
  ActionMapEntry, AutoRecordingMode, AutoSplitDuration, CanvasService, DrawTextParams,
  HitAreaEvent, HitAreaId, HitAreaOptions, HitAreaService, I18nService, KeyState, LayoutService,
  MouseButton, RecordingExportFrameRate, RecordingExportQuality, RecordingFrameRate,
  RecordingPixelScale, RecordingPopupMode, RecordingProfile, Rect, RenderService, RichTextParams,
  RuntimeObjectPool, RuntimeObjectPoolOwner, ScrollBoxService, TextInputService, UiEvent,
  UiObjectPool, UiObjectPoolOwner,
};

use super::fonts_settings::{FontsSettingsCommand, FontsSettingsUi};

const NS: &str = "recording_settings";
const MENU_LEN: usize = 8;
const LABEL_KEYS: [&str; MENU_LEN] = [
  "recording_settings.fonts",
  "recording_settings.auto_recording",
  "recording_settings.popup",
  "recording_settings.auto_split",
  "recording_settings.recording_fps",
  "recording_settings.video_resolution",
  "recording_settings.video_fps",
  "recording_settings.video_bitrate",
];

pub struct RecordingSettingsUi {
  selected_index: usize,
  profile: RecordingProfile,
  objects: UiObjectPool,
  runtime_objects: RuntimeObjectPool,
  back_area: HitAreaId,
  menu_areas: [HitAreaId; MENU_LEN],
  fonts_open: bool,
  fonts: FontsSettingsUi,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecordingSettingsCommand {
  Back,
  Changed(RecordingProfile),
  ExportFontPreview(Vec<String>),
  OpenFonts,
  StartAddFont,
  StartModifyFont,
  FinishFontEdit(String),
  CancelFontEdit,
  ScrollFonts(i32),
}

impl UiObjectPoolOwner for RecordingSettingsUi {
  fn objects(&self) -> &UiObjectPool {
    &self.objects
  }

  fn objects_mut(&mut self) -> &mut UiObjectPool {
    &mut self.objects
  }
}

impl RuntimeObjectPoolOwner for RecordingSettingsUi {
  fn runtime_objects(&self) -> &RuntimeObjectPool {
    &self.runtime_objects
  }

  fn runtime_objects_mut(&mut self) -> &mut RuntimeObjectPool {
    &mut self.runtime_objects
  }
}

impl RecordingSettingsUi {
  pub fn init(
    hit_area: &HitAreaService,
    text_input: &TextInputService,
    scroll_box: &ScrollBoxService,
    profile: RecordingProfile,
  ) -> Self {
    let mut objects = UiObjectPool::new();
    let fonts = FontsSettingsUi::create_recording(&mut objects, hit_area, text_input, scroll_box);
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
      action("recording_settings.focus_up", "up", "Focus previous option"),
      action("recording_settings.focus_down", "down", "Focus next option"),
      action(
        "recording_settings.confirm",
        "enter",
        "Activate selected option",
      ),
      action("recording_settings.back", "esc", "Back"),
      action("recording_settings.focus_fonts", "1", "Focus custom fonts"),
      action(
        "recording_settings.focus_auto_recording",
        "2",
        "Focus auto recording",
      ),
      action("recording_settings.focus_popup", "3", "Focus notifications"),
      action(
        "recording_settings.focus_auto_split",
        "4",
        "Focus auto split",
      ),
      action(
        "recording_settings.focus_recording_fps",
        "5",
        "Focus capture FPS",
      ),
      action(
        "recording_settings.focus_video_resolution",
        "6",
        "Focus export resolution",
      ),
      action(
        "recording_settings.focus_video_fps",
        "7",
        "Focus export FPS",
      ),
      action(
        "recording_settings.focus_video_bitrate",
        "8",
        "Focus export quality",
      ),
    ];
    actions.extend(FontsSettingsUi::action_map());
    actions
  }

  pub fn handle_event(&mut self, event: &UiEvent) -> Option<RecordingSettingsCommand> {
    if self.fonts_open {
      return self.fonts.handle_event(event).map(|command| match command {
        FontsSettingsCommand::Back(fonts) => {
          self.profile.fonts = fonts;
          self.fonts_open = false;
          RecordingSettingsCommand::Changed(self.profile.clone())
        }
        FontsSettingsCommand::ExportPreview(fonts) => {
          RecordingSettingsCommand::ExportFontPreview(fonts)
        }
        FontsSettingsCommand::StartAdd => RecordingSettingsCommand::StartAddFont,
        FontsSettingsCommand::StartModify => RecordingSettingsCommand::StartModifyFont,
        FontsSettingsCommand::FinishEdit(value) => RecordingSettingsCommand::FinishFontEdit(value),
        FontsSettingsCommand::CancelEdit => RecordingSettingsCommand::CancelFontEdit,
        FontsSettingsCommand::Scroll(dy) => RecordingSettingsCommand::ScrollFonts(dy),
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
      }) => Some(RecordingSettingsCommand::Back),
      UiEvent::Action(event) if event.state == KeyState::Pressed => match event.action.as_str() {
        "recording_settings.focus_up" => {
          self.selected_index = (self.selected_index + MENU_LEN - 1) % MENU_LEN;
          None
        }
        "recording_settings.focus_down" => {
          self.selected_index = (self.selected_index + 1) % MENU_LEN;
          None
        }
        "recording_settings.confirm" => self.activate_selected(),
        "recording_settings.back" => Some(RecordingSettingsCommand::Back),
        "recording_settings.focus_fonts" => self.focus(0),
        "recording_settings.focus_auto_recording" => self.focus(1),
        "recording_settings.focus_popup" => self.focus(2),
        "recording_settings.focus_auto_split" => self.focus(3),
        "recording_settings.focus_recording_fps" => self.focus(4),
        "recording_settings.focus_video_resolution" => self.focus(5),
        "recording_settings.focus_video_fps" => self.focus(6),
        "recording_settings.focus_video_bitrate" => self.focus(7),
        _ => None,
      },
      _ => None,
    }
  }

  pub fn update(&mut self, _dt: Duration) {}

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

  pub fn prepare_surfaces(
    &mut self,
    scroll_box: &ScrollBoxService,
    layout: &LayoutService,
    i18n: &I18nService,
  ) {
    if self.fonts_open {
      self
        .fonts
        .prepare(&mut self.objects, scroll_box, layout, i18n);
    }
  }

  pub fn scroll_fonts(&mut self, scroll_box: &ScrollBoxService, layout: &LayoutService, dy: i32) {
    if self.fonts_open {
      self.fonts.scroll(&mut self.objects, scroll_box, layout, dy);
    }
  }

  #[allow(clippy::too_many_arguments)]
  pub fn render(
    &mut self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    i18n: &I18nService,
    hit_area: &HitAreaService,
    text_input: &TextInputService,
    scroll_box: &ScrollBoxService,
  ) -> Option<(u16, u16)> {
    if self.fonts_open {
      return self.fonts.render(
        &mut self.objects,
        render,
        canvas,
        layout,
        i18n,
        hit_area,
        text_input,
        scroll_box,
      );
    }
    let viewport = layout.developer_viewport_rect();
    let title = i18n.get_runtime_text(NS, "recording_settings.title");
    let title_y = viewport.y.saturating_add(1);
    let hint = self.hint(i18n);
    let params = RichTextParams::from_action_map(&Self::action_map(), "recording_settings.");
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

  fn activate_selected(&mut self) -> Option<RecordingSettingsCommand> {
    match self.selected_index {
      0 => return Some(RecordingSettingsCommand::OpenFonts),
      1 => self.profile.auto_recording = self.profile.auto_recording.next(),
      2 => self.profile.popup = self.profile.popup.next(),
      3 => self.profile.auto_split = self.profile.auto_split.next(),
      4 => self.profile.capture_frame_rate = self.profile.capture_frame_rate.next(),
      5 => self.profile.pixel_scale = self.profile.pixel_scale.next(),
      6 => self.profile.export_frame_rate = self.profile.export_frame_rate.next(),
      7 => self.profile.quality = self.profile.quality.next(),
      _ => return None,
    }
    Some(RecordingSettingsCommand::Changed(self.profile.clone()))
  }

  fn focus(&mut self, index: usize) -> Option<RecordingSettingsCommand> {
    self.selected_index = index;
    None
  }

  fn rows(&self, i18n: &I18nService, layout: &LayoutService) -> [String; MENU_LEN] {
    let labels: [String; MENU_LEN] =
      std::array::from_fn(|index| i18n.get_runtime_text(NS, LABEL_KEYS[index]));
    let values = [
      String::new(),
      i18n.get_runtime_text(NS, auto_recording_key(self.profile.auto_recording)),
      i18n.get_runtime_text(NS, popup_key(self.profile.popup)),
      i18n.get_runtime_text(NS, auto_split_key(self.profile.auto_split)),
      i18n.get_runtime_text(NS, recording_fps_key(self.profile.capture_frame_rate)),
      i18n.get_runtime_text(NS, resolution_key(self.profile.pixel_scale)),
      i18n.get_runtime_text(NS, video_fps_key(self.profile.export_frame_rate)),
      i18n.get_runtime_text(NS, quality_key(self.profile.quality)),
    ];
    let label_width = labels[1..]
      .iter()
      .map(|label| layout.get_text_width(label, None))
      .max()
      .unwrap_or_default();
    let value_width = values[1..]
      .iter()
      .map(|value| layout.get_text_width(value, None))
      .max()
      .unwrap_or_default();
    std::array::from_fn(|index| {
      let focused = index == self.selected_index;
      let color = if focused { "bright_cyan" } else { "white" };
      let prefix = if focused { "❯ " } else { "  " };
      let suffix = if focused { " ❮" } else { "  " };
      if index == 0 {
        return format!("f%<fg:{color}>{prefix}{}{suffix}</fg>", labels[index]);
      }
      let padding = " ".repeat(
        label_width
          .saturating_sub(layout.get_text_width(&labels[index], None))
          .saturating_add(value_width.saturating_sub(layout.get_text_width(&values[index], None)))
          as usize,
      );
      let value_color = if self.is_disabled(index) {
        "rgb(85,87,83)"
      } else {
        "bright_yellow"
      };
      format!(
        "f%<fg:{color}>{prefix}{}{padding}  </fg><fg:white>[</fg><fg:{value_color}>{}</fg><fg:white>]</fg><fg:{color}>{suffix}</fg>",
        labels[index], values[index]
      )
    })
  }

  fn is_disabled(&self, index: usize) -> bool {
    match index {
      1 => self.profile.auto_recording == AutoRecordingMode::Off,
      2 => self.profile.popup == RecordingPopupMode::Off,
      3 => self.profile.auto_split == AutoSplitDuration::Off,
      _ => false,
    }
  }

  fn hint(&self, i18n: &I18nService) -> String {
    let action = if self.selected_index == 0 {
      "recording_settings.action.confirm"
    } else {
      "recording_settings.action.switch"
    };
    format!(
      "f%<fg:rgb(85,87,83)>{}  {}  {}  {}</fg>",
      i18n.get_runtime_text(NS, "recording_settings.action.focus"),
      i18n.get_runtime_text(NS, "recording_settings.action.select"),
      i18n.get_runtime_text(NS, action),
      i18n.get_runtime_text(NS, "recording_settings.action.back"),
    )
  }
}

fn centered_x(viewport: Rect, width: u16) -> u16 {
  viewport
    .x
    .saturating_add(viewport.width.saturating_sub(width) / 2)
}

fn auto_recording_key(value: AutoRecordingMode) -> &'static str {
  match value {
    AutoRecordingMode::Off => "recording_settings.auto_recording.nope",
    AutoRecordingMode::Host => "recording_settings.auto_recording.host",
    AutoRecordingMode::Game => "recording_settings.auto_recording.game",
  }
}

fn popup_key(value: RecordingPopupMode) -> &'static str {
  match value {
    RecordingPopupMode::Off => "recording_settings.popup.no",
    RecordingPopupMode::All => "recording_settings.popup.all",
    RecordingPopupMode::SplitOnly => "recording_settings.popup.only.split",
    RecordingPopupMode::StateOnly => "recording_settings.popup.only.state",
    RecordingPopupMode::StartStopOnly => "recording_settings.popup.only.start_stop",
  }
}

fn auto_split_key(value: AutoSplitDuration) -> &'static str {
  match value {
    AutoSplitDuration::Off => "recording_settings.auto_split.nope",
    AutoSplitDuration::Minutes5 => "recording_settings.auto_split.5",
    AutoSplitDuration::Minutes10 => "recording_settings.auto_split.10",
  }
}

fn recording_fps_key(value: RecordingFrameRate) -> &'static str {
  match value {
    RecordingFrameRate::Fps30 => "recording_settings.recording_fps.30",
    RecordingFrameRate::Fps60 => "recording_settings.recording_fps.60",
    RecordingFrameRate::Fps120 => "recording_settings.recording_fps.120",
  }
}

fn video_fps_key(value: RecordingExportFrameRate) -> &'static str {
  match value {
    RecordingExportFrameRate::Recorded => "recording_settings.video_fps.vidio",
    RecordingExportFrameRate::Fps30 => "recording_settings.video_fps.30",
    RecordingExportFrameRate::Fps60 => "recording_settings.video_fps.60",
    RecordingExportFrameRate::Fps120 => "recording_settings.video_fps.120",
  }
}

fn resolution_key(value: RecordingPixelScale) -> &'static str {
  match value {
    RecordingPixelScale::Half => "recording_settings.video_resolution.half",
    RecordingPixelScale::Original => "recording_settings.video_resolution.original",
    RecordingPixelScale::Double => "recording_settings.video_resolution.double",
  }
}

fn quality_key(value: RecordingExportQuality) -> &'static str {
  match value {
    RecordingExportQuality::Compact => "recording_settings.video_bitrate.compact",
    RecordingExportQuality::Balanced => "recording_settings.video_bitrate.balanced",
    RecordingExportQuality::High => "recording_settings.video_bitrate.high",
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
  fn defaults_match_requested_recording_behavior() {
    let profile = RecordingProfile::default();
    assert_eq!(profile.popup, RecordingPopupMode::All);
    assert_eq!(profile.capture_frame_rate, RecordingFrameRate::Fps60);
    assert_eq!(profile.auto_recording, AutoRecordingMode::Off);
    assert_eq!(profile.auto_split, AutoSplitDuration::Minutes10);
  }
}
