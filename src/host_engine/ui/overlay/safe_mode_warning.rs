use std::time::Duration;

use unicode_width::UnicodeWidthChar;

use crate::host_engine::services::{
  ActionMapEntry, CanvasService, DrawTextParams, HitAreaEvent, HitAreaId, HitAreaOptions,
  HitAreaService, I18nService, InputService, Key, KeyEventKind, LayoutService, MouseButton, Rect,
  RenderService, RichTextParams, RuntimeObjectPool, RuntimeObjectPoolOwner, UiEvent, UiObjectPool,
  UiObjectPoolOwner,
};

const TEMPORARY_DELAY: Duration = Duration::from_secs(3);
const PERMANENT_DELAY: Duration = Duration::from_secs(5);

pub struct SafeModeWarningUi {
  objects: UiObjectPool,
  runtime_objects: RuntimeObjectPool,
  no_area: HitAreaId,
  temporary_area: HitAreaId,
  permanent_area: HitAreaId,
  elapsed: Duration,
}

impl SafeModeWarningUi {
  pub fn init(hit_area: &HitAreaService) -> Self {
    let mut objects = UiObjectPool::new();
    let no_area = hit_area.create(&mut objects, HitAreaOptions::default());
    let temporary_area = hit_area.create(&mut objects, HitAreaOptions::default());
    let permanent_area = hit_area.create(&mut objects, HitAreaOptions::default());
    Self {
      objects,
      runtime_objects: RuntimeObjectPool::new(),
      no_area,
      temporary_area,
      permanent_area,
      elapsed: Duration::ZERO,
    }
  }

  pub fn action_map() -> Vec<ActionMapEntry> {
    vec![
      ActionMapEntry {
        action: "safe_mode_warning.yes.temporary".to_string(),
        description: "Disable safe mode temporarily".to_string(),
        keys: vec![vec!["1".to_string()]],
      },
      ActionMapEntry {
        action: "safe_mode_warning.yes.permanent".to_string(),
        description: "Disable safe mode permanently".to_string(),
        keys: vec![vec!["2".to_string()]],
      },
      ActionMapEntry {
        action: "safe_mode_warning.no".to_string(),
        description: "Cancel".to_string(),
        keys: vec![vec!["esc".to_string()]],
      },
    ]
  }

  pub fn start(&mut self) {
    self.elapsed = Duration::ZERO;
  }

  pub fn update(&mut self, dt: Duration) {
    self.elapsed = self.elapsed.saturating_add(dt);
  }

  pub fn handle_raw_key_events(&self, input: &mut InputService) -> Option<SafeModeWarningCommand> {
    for event in input.take_raw_key_events() {
      if event.kind != KeyEventKind::Press {
        continue;
      }
      return match event.key {
        Key::Num(1) | Key::Numpad(1) if self.temporary_ready() => {
          Some(SafeModeWarningCommand::DisableTemporary)
        }
        Key::Num(1) | Key::Numpad(1) => None,
        Key::Num(2) | Key::Numpad(2) if self.permanent_ready() => {
          Some(SafeModeWarningCommand::DisablePermanent)
        }
        Key::Num(2) | Key::Numpad(2) => None,
        Key::Esc => Some(SafeModeWarningCommand::Cancel),
        _ => Some(SafeModeWarningCommand::Cancel),
      };
    }
    None
  }

  pub fn handle_event(&self, event: &UiEvent) -> Option<SafeModeWarningCommand> {
    match event {
      UiEvent::HitArea(HitAreaEvent::Click {
        id,
        button: MouseButton::Left,
        ..
      }) if *id == self.no_area => Some(SafeModeWarningCommand::Cancel),
      UiEvent::HitArea(HitAreaEvent::Click {
        id,
        button: MouseButton::Left,
        ..
      }) if *id == self.temporary_area && self.temporary_ready() => {
        Some(SafeModeWarningCommand::DisableTemporary)
      }
      UiEvent::HitArea(HitAreaEvent::Click {
        id,
        button: MouseButton::Left,
        ..
      }) if *id == self.permanent_area && self.permanent_ready() => {
        Some(SafeModeWarningCommand::DisablePermanent)
      }
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
    let params = Self::key_params();
    let title = i18n.get_runtime_text("safe_mode_warning", "safe_mode_warning.title");
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

    let content_w = size.width.saturating_sub(32).max(1);
    let desc = i18n.get_runtime_text("safe_mode_warning", "safe_mode_warning.description.one");
    let desc_lines = wrap_plain_lines(&desc, content_w);
    let no = i18n.get_runtime_text("safe_mode_warning", "safe_mode_warning.no");
    let temporary = i18n.get_runtime_text("safe_mode_warning", "safe_mode_warning.yes.temporary");
    let permanent = i18n.get_runtime_text("safe_mode_warning", "safe_mode_warning.yes.permanent");
    let second = i18n.get_runtime_text("safe_mode_warning", "safe_mode_warning.second");
    let no_text = format!("f%<fg:bright_green>{}</fg>", no);
    let temporary_text =
      self.option_text(&temporary, TEMPORARY_DELAY, self.temporary_ready(), &second);
    let permanent_text =
      self.option_text(&permanent, PERMANENT_DELAY, self.permanent_ready(), &second);
    let block_w = desc_lines
      .iter()
      .map(|line| layout.get_text_width(line, None))
      .chain([
        layout.get_text_width(&no_text, Some(&params)),
        layout.get_text_width(&temporary_text, Some(&params)),
        layout.get_text_width(&permanent_text, Some(&params)),
      ])
      .max()
      .unwrap_or(1)
      .min(content_w)
      .max(1);
    let content_x = size.width.saturating_sub(block_w) / 2;
    let block_h = desc_lines.len() as u16 + 4;
    let start_y = size.height.saturating_sub(block_h) / 2;

    for (row, line) in desc_lines.iter().enumerate() {
      render.draw_host_text(
        canvas,
        &DrawTextParams {
          x: content_x,
          y: start_y.saturating_add(row as u16),
          text: line.clone(),
          max_width: Some(block_w),
          ..Default::default()
        },
      );
    }

    let no_y = start_y.saturating_add(desc_lines.len() as u16 + 1);
    let temporary_y = no_y.saturating_add(1);
    let permanent_y = no_y.saturating_add(2);

    self.draw_option(render, canvas, content_x, no_y, &no_text, &params);
    self.draw_option(
      render,
      canvas,
      content_x,
      temporary_y,
      &temporary_text,
      &params,
    );
    self.draw_option(
      render,
      canvas,
      content_x,
      permanent_y,
      &permanent_text,
      &params,
    );

    self.register_area(
      hit_area,
      canvas,
      layout,
      self.no_area,
      content_x,
      no_y,
      &no,
      &params,
    );
    self.register_area(
      hit_area,
      canvas,
      layout,
      self.temporary_area,
      content_x,
      temporary_y,
      &temporary_text,
      &params,
    );
    self.register_area(
      hit_area,
      canvas,
      layout,
      self.permanent_area,
      content_x,
      permanent_y,
      &permanent_text,
      &params,
    );
  }

  fn option_text(&self, base: &str, delay: Duration, ready: bool, second: &str) -> String {
    if ready {
      return format!("f%<fg:bright_red>{}</fg>", base);
    }
    let left = delay
      .as_secs()
      .saturating_sub(self.elapsed.as_secs())
      .max(1);
    format!(
      "f%<fg:rgb(85,87,83)>{}</fg>[<fg:bright_red>{}{}</fg>]",
      base, left, second
    )
  }

  fn draw_option(
    &self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    x: u16,
    y: u16,
    text: &str,
    params: &RichTextParams,
  ) {
    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x,
        y,
        text: text.to_string(),
        params: Some(params.clone()),
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
    let width = layout.get_text_width(text, Some(params)).max(1);
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

  fn key_params() -> RichTextParams {
    RichTextParams::from_action_map(&Self::action_map(), "safe_mode_warning.")
  }

  fn temporary_ready(&self) -> bool {
    self.elapsed >= TEMPORARY_DELAY
  }

  fn permanent_ready(&self) -> bool {
    self.elapsed >= PERMANENT_DELAY
  }
}

impl UiObjectPoolOwner for SafeModeWarningUi {
  fn objects(&self) -> &UiObjectPool {
    &self.objects
  }

  fn objects_mut(&mut self) -> &mut UiObjectPool {
    &mut self.objects
  }
}

impl RuntimeObjectPoolOwner for SafeModeWarningUi {
  fn runtime_objects(&self) -> &RuntimeObjectPool {
    &self.runtime_objects
  }

  fn runtime_objects_mut(&mut self) -> &mut RuntimeObjectPool {
    &mut self.runtime_objects
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SafeModeWarningCommand {
  Cancel,
  DisableTemporary,
  DisablePermanent,
}

fn wrap_plain_lines(text: &str, width: u16) -> Vec<String> {
  let limit = width as usize;
  if limit == 0 {
    return vec![String::new()];
  }
  let mut lines = Vec::new();
  for source in text.split('\n') {
    let mut line = String::new();
    let mut used = 0usize;
    for ch in source.chars() {
      let w = ch.width().unwrap_or(0);
      if used > 0 && used + w > limit {
        lines.push(line);
        line = String::new();
        used = 0;
      }
      line.push(ch);
      used += w.min(limit);
    }
    lines.push(line);
  }
  if lines.is_empty() {
    lines.push(String::new());
  }
  lines
}
