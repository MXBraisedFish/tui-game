use std::{path::PathBuf, time::Duration};

use crate::host_engine::services::text_layout::TextWrapMode;
use crate::host_engine::services::{
  ActionMapEntry, CanvasService, DrawTextParams, HitAreaEvent, HitAreaId, HitAreaOptions,
  HitAreaService, I18nService, InputService, Key, KeyEventKind, LayoutService, MouseButton, Rect,
  RenderService, RichTextParams, RuntimeObjectPool, RuntimeObjectPoolOwner, UiEvent, UiObjectPool,
  UiObjectPoolOwner,
};

const CONFIRM_DELAY: Duration = Duration::from_secs(3);
const NS: &str = "clear_warning";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClearWarningTarget {
  Cache,
  Log,
  Mod,
  Profile,
  Data,
}

impl ClearWarningTarget {
  pub fn description_key(self) -> &'static str {
    match self {
      Self::Cache => "clear_warning.description.cache",
      Self::Log => "clear_warning.description.log",
      Self::Mod => "clear_warning.description.mod",
      Self::Profile => "clear_warning.description.profile",
      Self::Data => "clear_warning.description.data",
    }
  }
}

pub struct ClearWarningUi {
  objects: UiObjectPool,
  runtime_objects: RuntimeObjectPool,
  no_area: HitAreaId,
  yes_area: HitAreaId,
  elapsed: Duration,
  target: Option<ClearWarningTarget>,
  path: PathBuf,
}

impl ClearWarningUi {
  pub fn init(hit_area: &HitAreaService) -> Self {
    let mut objects = UiObjectPool::new();
    Self {
      no_area: hit_area.create(&mut objects, HitAreaOptions::default()),
      yes_area: hit_area.create(&mut objects, HitAreaOptions::default()),
      objects,
      runtime_objects: RuntimeObjectPool::new(),
      elapsed: Duration::ZERO,
      target: None,
      path: PathBuf::new(),
    }
  }

  pub fn action_map() -> Vec<ActionMapEntry> {
    vec![
      ActionMapEntry {
        action: "clear_warning.yes.temporary".to_string(),
        description: "Confirm clear data".to_string(),
        keys: vec![vec!["1".to_string()]],
      },
      ActionMapEntry {
        action: "clear_warning.no".to_string(),
        description: "Cancel clear data".to_string(),
        keys: vec![vec!["esc".to_string()]],
      },
    ]
  }

  pub fn start(&mut self, target: ClearWarningTarget, path: PathBuf) {
    self.target = Some(target);
    self.path = path;
    self.elapsed = Duration::ZERO;
  }

  pub fn target(&self) -> Option<ClearWarningTarget> {
    self.target
  }

  pub fn update(&mut self, dt: Duration) {
    self.elapsed = self.elapsed.saturating_add(dt);
  }

  pub fn handle_raw_key_events(&self, input: &mut InputService) -> Option<ClearWarningCommand> {
    for event in input.take_raw_key_events() {
      if event.kind != KeyEventKind::Press {
        continue;
      }
      return match event.key {
        Key::Num(1) | Key::Numpad(1) if self.confirm_ready() => Some(ClearWarningCommand::Confirm),
        Key::Num(1) | Key::Numpad(1) => None,
        _ => Some(ClearWarningCommand::Cancel),
      };
    }
    None
  }

  pub fn handle_event(&self, event: &UiEvent) -> Option<ClearWarningCommand> {
    match event {
      UiEvent::HitArea(HitAreaEvent::Click {
        id,
        button: MouseButton::Left,
        ..
      }) if *id == self.no_area => Some(ClearWarningCommand::Cancel),
      UiEvent::HitArea(HitAreaEvent::Click {
        id,
        button: MouseButton::Left,
        ..
      }) if *id == self.yes_area && self.confirm_ready() => Some(ClearWarningCommand::Confirm),
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
    let Some(target) = self.target else { return };
    let size = layout.physical_size();
    let params = Self::key_params();
    let title = i18n.get_runtime_text(NS, "clear_warning.title");
    let title_w = layout.get_text_width(&title, None);
    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: layout.resolve_host_x(LayoutService::ALIGN_CENTER, title_w, 0),
        y: 1,
        text: format!("f%<fg:bright_yellow><b>{}</b></fg>", title),
        ..Default::default()
      },
    );

    let max_w = size.width.saturating_sub(32).max(1);
    let desc = i18n.get_runtime_text(NS, target.description_key());
    let desc_size = layout.get_draw_text_size(&DrawTextParams {
      text: desc.clone(),
      params: Some(params.clone()),
      wrap_mode: TextWrapMode::Auto,
      max_width: Some(max_w),
      ..Default::default()
    });
    let path_text = format!(
      "f%<fg:bright_blue>{}</fg> {}",
      i18n.get_runtime_text(NS, "clear_warning.path"),
      display_path(&self.path)
    );
    let no_text = format!(
      "f%<fg:bright_green>{}</fg>",
      i18n.get_runtime_text(NS, "clear_warning.no")
    );
    let yes_text = self.yes_text(i18n);
    let block_w = [
      desc_size.width.max(1),
      layout.get_text_width(&path_text, Some(&params)).min(max_w),
      layout.get_text_width(&no_text, Some(&params)),
      layout.get_text_width(&yes_text, Some(&params)),
    ]
    .into_iter()
    .max()
    .unwrap_or(1)
    .min(max_w)
    .max(1);
    let content_x = size.width.saturating_sub(block_w) / 2;
    let desc_h = desc_size.height.max(1);
    let block_h = desc_h + 4;
    let start_y = size.height.saturating_sub(block_h) / 2;

    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: content_x,
        y: start_y,
        text: desc,
        params: Some(params.clone()),
        wrap_mode: TextWrapMode::Auto,
        max_width: Some(block_w),
        ..Default::default()
      },
    );

    let path_y = start_y.saturating_add(desc_h);
    let no_y = path_y.saturating_add(2);
    let yes_y = no_y.saturating_add(1);
    self.draw_line(
      render, canvas, content_x, path_y, &path_text, block_w, &params,
    );
    self.draw_line(render, canvas, content_x, no_y, &no_text, block_w, &params);
    self.draw_line(
      render, canvas, content_x, yes_y, &yes_text, block_w, &params,
    );

    self.register_area(
      hit_area,
      canvas,
      layout,
      self.no_area,
      content_x,
      no_y,
      &no_text,
      &params,
    );
    self.register_area(
      hit_area,
      canvas,
      layout,
      self.yes_area,
      content_x,
      yes_y,
      &yes_text,
      &params,
    );
  }

  fn yes_text(&self, i18n: &I18nService) -> String {
    let yes = i18n.get_runtime_text(NS, "clear_warning.yes");
    if self.confirm_ready() {
      return format!("f%<fg:bright_red>{}</fg>", yes);
    }
    let second = i18n.get_runtime_text(NS, "clear_warning.second");
    let left = CONFIRM_DELAY
      .as_secs()
      .saturating_sub(self.elapsed.as_secs())
      .max(1);
    format!(
      "f%<fg:rgb(85,87,83)>{}</fg>[<fg:bright_red>{}{}</fg>]",
      yes, left, second
    )
  }

  fn draw_line(
    &self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    x: u16,
    y: u16,
    text: &str,
    width: u16,
    params: &RichTextParams,
  ) {
    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x,
        y,
        text: text.to_string(),
        params: Some(params.clone()),
        max_width: Some(width),
        ..Default::default()
      },
    );
  }

  fn register_area(
    &mut self,
    hit_area: &HitAreaService,
    canvas: &CanvasService,
    layout: &LayoutService,
    id: HitAreaId,
    x: u16,
    y: u16,
    text: &str,
    params: &RichTextParams,
  ) {
    hit_area.render_host(
      &mut self.objects,
      id,
      Rect {
        x,
        y,
        width: layout.get_text_width(text, Some(params)).max(1),
        height: 1,
      },
      canvas,
    );
  }

  fn confirm_ready(&self) -> bool {
    self.elapsed >= CONFIRM_DELAY
  }

  fn key_params() -> RichTextParams {
    RichTextParams::from_action_map(&Self::action_map(), "clear_warning.")
  }
}

impl UiObjectPoolOwner for ClearWarningUi {
  fn objects(&self) -> &UiObjectPool {
    &self.objects
  }

  fn objects_mut(&mut self) -> &mut UiObjectPool {
    &mut self.objects
  }
}

impl RuntimeObjectPoolOwner for ClearWarningUi {
  fn runtime_objects(&self) -> &RuntimeObjectPool {
    &self.runtime_objects
  }

  fn runtime_objects_mut(&mut self) -> &mut RuntimeObjectPool {
    &mut self.runtime_objects
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClearWarningCommand {
  Cancel,
  Confirm,
}

fn display_path(path: &std::path::Path) -> String {
  path.to_string_lossy().replace('\\', "/")
}
