use std::time::Duration;

use crate::host_engine::services::text_layout::TextWrapMode;
use crate::host_engine::services::{
  ActionMapEntry, CanvasCell, CanvasService, ComposedCell, ComposedFrame, DrawTextParams,
  I18nService, InputService, Key, KeyEventKind, LayoutService, LogService, MouseButton,
  MouseEventKind, RenderService, RichTextParams, ScreenshotRect, ScreenshotService, StorageService,
  SystemEvent, TerminalColor, TextColor,
};

const DOUBLE_F1_WINDOW: Duration = Duration::from_millis(300);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScreenshotCaptureCommand {
  Exit,
  Copy,
  CopyRichText,
  SavePng,
  All,
  FullFrameSave,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct MenuState {
  x: u16,
  y: u16,
  width: u16,
  height: u16,
}

pub struct ScreenshotCaptureUi {
  frame: Option<ComposedFrame>,
  selection: Option<ScreenshotRect>,
  drag_anchor: Option<(u16, u16)>,
  drag_cursor: Option<(u16, u16)>,
  menu: Option<MenuState>,
  guide_visible: bool,
  opened_elapsed: Duration,
  user_touched: bool,
  mode_toast_dismiss_requested: bool,
  operation_toast_dismiss_requested: bool,
}

impl ScreenshotCaptureUi {
  pub fn init() -> Self {
    Self {
      frame: None,
      selection: None,
      drag_anchor: None,
      drag_cursor: None,
      menu: None,
      guide_visible: false,
      opened_elapsed: Duration::ZERO,
      user_touched: false,
      mode_toast_dismiss_requested: false,
      operation_toast_dismiss_requested: false,
    }
  }

  pub fn action_map() -> Vec<ActionMapEntry> {
    vec![
      entry("screenshot.copy", "Copy screenshot", "c"),
      entry(
        "screenshot.copy_rich_text",
        "Copy screenshot rich text",
        "r",
      ),
      entry("screenshot.save_png", "Save screenshot PNG", "s"),
      entry("screenshot.all", "Copy and save screenshot", "a"),
    ]
  }

  pub fn start(&mut self, frame: ComposedFrame, show_guide: bool) {
    self.frame = Some(frame);
    self.selection = None;
    self.drag_anchor = None;
    self.drag_cursor = None;
    self.menu = None;
    self.guide_visible = show_guide;
    self.opened_elapsed = Duration::ZERO;
    self.user_touched = false;
    self.mode_toast_dismiss_requested = false;
    self.operation_toast_dismiss_requested = false;
  }

  pub fn update(&mut self, dt: Duration) {
    self.opened_elapsed = self.opened_elapsed.saturating_add(dt);
  }

  pub fn can_full_save_by_double_f1(&self) -> bool {
    !self.user_touched && self.opened_elapsed <= DOUBLE_F1_WINDOW
  }

  pub fn is_guide_visible(&self) -> bool {
    self.guide_visible
  }

  pub fn take_mode_toast_dismiss_requested(&mut self) -> bool {
    let requested = self.mode_toast_dismiss_requested;
    self.mode_toast_dismiss_requested = false;
    requested
  }

  pub fn take_operation_toast_dismiss_requested(&mut self) -> bool {
    let requested = self.operation_toast_dismiss_requested;
    self.operation_toast_dismiss_requested = false;
    requested
  }

  pub fn handle_input(
    &mut self,
    input: &mut InputService,
    layout: &LayoutService,
    i18n: &I18nService,
    storage: &StorageService,
    log: &mut LogService,
  ) -> Option<ScreenshotCaptureCommand> {
    let raw_keys = input.take_raw_key_events();
    let system_events = input.drain_system_events();
    if self.guide_visible
      && (guide_direct_key_should_close(input, self.opened_elapsed)
        || guide_should_close(self.opened_elapsed, &raw_keys, &system_events))
    {
      self.close_guide(storage, log);
      self.mode_toast_dismiss_requested = true;
      return None;
    }
    if self.guide_visible {
      return None;
    }

    if mode_toast_should_close(&raw_keys, &system_events) {
      self.mode_toast_dismiss_requested = true;
    }
    if operation_toast_should_close(&raw_keys, &system_events) {
      self.operation_toast_dismiss_requested = true;
    }

    if let Some(command) = direct_key_command(input) {
      self.user_touched = true;
      self.mode_toast_dismiss_requested = false;
      self.operation_toast_dismiss_requested = false;
      return Some(command);
    }

    for event in raw_keys {
      if event.kind != KeyEventKind::Press {
        continue;
      }
      self.user_touched = true;
      return match event.key {
        Key::Fn(1) => Some(if self.can_full_save_by_double_f1() {
          ScreenshotCaptureCommand::FullFrameSave
        } else {
          ScreenshotCaptureCommand::Exit
        }),
        Key::A => Some(ScreenshotCaptureCommand::All),
        Key::S => Some(ScreenshotCaptureCommand::SavePng),
        Key::C => Some(ScreenshotCaptureCommand::Copy),
        Key::R => Some(ScreenshotCaptureCommand::CopyRichText),
        _ => None,
      };
    }

    let frame_origin = self.frame_origin(layout);
    for event in system_events {
      let SystemEvent::Mouse(mouse) = event else {
        continue;
      };
      self.user_touched = true;
      let local = self.to_frame_pos(mouse.x, mouse.y, frame_origin);
      match mouse.kind {
        MouseEventKind::Press if mouse.button == Some(MouseButton::Left) => {
          if let Some(cmd) = self.menu_command_at(mouse.x, mouse.y) {
            self.mode_toast_dismiss_requested = false;
            self.operation_toast_dismiss_requested = false;
            return Some(cmd);
          }
          self.menu = None;
          if let Some(pos) = local {
            self.drag_anchor = Some(pos);
            self.drag_cursor = Some(pos);
            self.selection = self.selection_from_drag();
          }
        }
        MouseEventKind::Drag if mouse.button == Some(MouseButton::Left) => {
          if let Some(pos) = local {
            self.drag_cursor = Some(pos);
            self.selection = self.selection_from_drag();
          }
        }
        MouseEventKind::Release if mouse.button == Some(MouseButton::Left) => {
          if let Some(pos) = local {
            self.drag_cursor = Some(pos);
            self.selection = self.selection_from_drag();
          }
          self.drag_anchor = None;
          self.drag_cursor = None;
        }
        MouseEventKind::Press if mouse.button == Some(MouseButton::Right) => {
          if self.selection.is_some() {
            self.open_menu(mouse.x, mouse.y, layout, i18n);
          }
        }
        _ => {}
      }
    }
    None
  }

  pub fn current_selection(&self) -> Option<(ComposedFrame, ScreenshotRect)> {
    let frame = self.frame.clone()?;
    let rect = ScreenshotService::normalize_selection(&frame, self.selection?)?;
    Some((frame, rect))
  }

  pub fn whole_frame(&self) -> Option<(ComposedFrame, ScreenshotRect)> {
    let frame = self.frame.clone()?;
    let rect = ScreenshotService::whole_frame_rect(&frame)?;
    Some((frame, rect))
  }

  pub fn clear_selection(&mut self) {
    self.selection = None;
    self.drag_anchor = None;
    self.drag_cursor = None;
    self.menu = None;
  }

  pub fn render(
    &self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    i18n: &I18nService,
  ) {
    let Some(frame) = &self.frame else {
      return;
    };
    let origin = self.frame_origin(layout);
    let normalized = self
      .selection
      .and_then(|rect| ScreenshotService::normalize_selection(frame, rect));
    self.draw_frame(canvas, frame, origin, normalized);
    if self.guide_visible {
      self.draw_guide(render, canvas, layout, i18n);
    }
    if let Some(menu) = self.menu {
      self.draw_menu(render, canvas, i18n, menu);
    }
  }

  fn close_guide(&mut self, storage: &StorageService, log: &mut LogService) {
    self.guide_visible = false;
    storage.mark_screenshot_guide_seen(log);
  }

  fn selection_from_drag(&self) -> Option<ScreenshotRect> {
    let (ax, ay) = self.drag_anchor?;
    let (cx, cy) = self.drag_cursor?;
    Some(ScreenshotRect {
      x: ax.min(cx),
      y: ay.min(cy),
      width: ax.max(cx).saturating_sub(ax.min(cx)).saturating_add(1),
      height: ay.max(cy).saturating_sub(ay.min(cy)).saturating_add(1),
    })
  }

  fn frame_origin(&self, layout: &LayoutService) -> (u16, u16) {
    let size = layout.physical_size();
    let Some(frame) = &self.frame else {
      return (0, 0);
    };
    (
      size.width.saturating_sub(frame.width()) / 2,
      size.height.saturating_sub(frame.height()) / 2,
    )
  }

  fn to_frame_pos(&self, x: u16, y: u16, origin: (u16, u16)) -> Option<(u16, u16)> {
    let frame = self.frame.as_ref()?;
    let fx = x.checked_sub(origin.0)?;
    let fy = y.checked_sub(origin.1)?;
    (fx < frame.width() && fy < frame.height()).then_some((fx, fy))
  }

  fn open_menu(&mut self, x: u16, y: u16, layout: &LayoutService, i18n: &I18nService) {
    let params = RichTextParams::from_action_map(&Self::action_map(), "screenshot.");
    let labels = menu_labels(i18n);
    let width = labels
      .iter()
      .map(|label| layout.get_text_width(label, Some(&params)))
      .max()
      .unwrap_or(1)
      .saturating_add(2);
    let height = labels.len() as u16;
    let size = layout.physical_size();
    let mx = if x.saturating_add(width) <= size.width {
      x
    } else {
      x.saturating_sub(width.saturating_sub(1))
    };
    let my = if y.saturating_add(height) <= size.height {
      y
    } else {
      y.saturating_sub(height.saturating_sub(1))
    };
    self.menu = Some(MenuState {
      x: mx,
      y: my,
      width,
      height,
    });
  }

  fn menu_command_at(&self, x: u16, y: u16) -> Option<ScreenshotCaptureCommand> {
    let menu = self.menu?;
    if x < menu.x || x >= menu.x.saturating_add(menu.width) || y < menu.y {
      return None;
    }
    match y.saturating_sub(menu.y) {
      0 => Some(ScreenshotCaptureCommand::Copy),
      1 => Some(ScreenshotCaptureCommand::CopyRichText),
      2 => Some(ScreenshotCaptureCommand::SavePng),
      3 => Some(ScreenshotCaptureCommand::All),
      _ => None,
    }
  }

  fn draw_frame(
    &self,
    canvas: &mut CanvasService,
    frame: &ComposedFrame,
    origin: (u16, u16),
    selection: Option<ScreenshotRect>,
  ) {
    for y in 0..frame.height() {
      for x in 0..frame.width() {
        let tx = origin.0.saturating_add(x);
        let ty = origin.1.saturating_add(y);
        let mut cell = match frame.get(x, y) {
          Some(ComposedCell::Text(cell)) => cell.clone(),
          _ => CanvasCell::blank(),
        };
        if selection.map_or(false, |r| contains(r, x, y)) {
          cell.style.reverse = !cell.style.reverse;
        }
        canvas.host_cell(tx, ty, cell);
      }
    }
  }

  fn draw_guide(
    &self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    i18n: &I18nService,
  ) {
    let bg = TextColor::Rgb {
      r: 40,
      g: 40,
      b: 40,
    };
    let params = guide_rich_text_params();
    let text = i18n.get_runtime_text("screenshot", "screenshot.guide");
    let max_width = 72.min(layout.physical_width().saturating_sub(2));
    let text_params = DrawTextParams {
      x: 1,
      y: 1,
      text,
      params: Some(params),
      bg: Some(bg.clone()),
      max_width: Some(max_width),
      wrap_mode: TextWrapMode::Auto,
      ..Default::default()
    };
    let size = layout.get_draw_text_size(&text_params);
    render.draw_host_filled_rect(
      canvas,
      0,
      0,
      size.width.saturating_add(2).min(layout.physical_width()),
      size.height.saturating_add(2).min(layout.physical_height()),
      Some(" ".to_string()),
      None,
      Some(bg),
    );
    render.draw_host_text(canvas, &text_params);
  }

  fn draw_menu(
    &self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    i18n: &I18nService,
    menu: MenuState,
  ) {
    render.draw_host_filled_rect(
      canvas,
      menu.x,
      menu.y,
      menu.width,
      menu.height,
      Some(" ".to_string()),
      Some(TextColor::Terminal(TerminalColor::White)),
      Some(TextColor::Terminal(TerminalColor::Blue)),
    );
    let params = RichTextParams::from_action_map(&Self::action_map(), "screenshot.");
    for (index, label) in menu_labels(i18n).into_iter().enumerate() {
      render.draw_host_text(
        canvas,
        &DrawTextParams {
          x: menu.x.saturating_add(1),
          y: menu.y.saturating_add(index as u16),
          text: label,
          params: Some(params.clone()),
          fg: Some(TextColor::Terminal(TerminalColor::White)),
          bg: Some(TextColor::Terminal(TerminalColor::Blue)),
          max_width: Some(menu.width.saturating_sub(2)),
          ..Default::default()
        },
      );
    }
  }
}

fn entry(action: &str, description: &str, key: &str) -> ActionMapEntry {
  ActionMapEntry {
    action: action.to_string(),
    description: description.to_string(),
    keys: vec![vec![key.to_string()]],
  }
}

fn guide_rich_text_params() -> RichTextParams {
  let mut entries = ScreenshotCaptureUi::action_map();
  entries.push(entry("host_key.screenshot", "Screenshot", "f1"));
  RichTextParams::from_action_map(&entries, "screenshot.")
}

fn menu_labels(i18n: &I18nService) -> [String; 4] {
  [
    i18n.get_runtime_text("screenshot", "screenshot.menu.copy"),
    i18n.get_runtime_text("screenshot", "screenshot.menu.copy_rich_text"),
    i18n.get_runtime_text("screenshot", "screenshot.menu.save_png"),
    i18n.get_runtime_text("screenshot", "screenshot.menu.all"),
  ]
}

fn direct_key_command(input: &InputService) -> Option<ScreenshotCaptureCommand> {
  if input.was_pressed(Key::A) {
    Some(ScreenshotCaptureCommand::All)
  } else if input.was_pressed(Key::S) {
    Some(ScreenshotCaptureCommand::SavePng)
  } else if input.was_pressed(Key::C) {
    Some(ScreenshotCaptureCommand::Copy)
  } else if input.was_pressed(Key::R) {
    Some(ScreenshotCaptureCommand::CopyRichText)
  } else {
    None
  }
}

fn guide_direct_key_should_close(input: &InputService, opened_elapsed: Duration) -> bool {
  direct_key_command(input).is_some()
    || (input.was_pressed(Key::Fn(1)) && opened_elapsed >= Duration::from_millis(1000))
}

fn contains(rect: ScreenshotRect, x: u16, y: u16) -> bool {
  x >= rect.x
    && x < rect.x.saturating_add(rect.width)
    && y >= rect.y
    && y < rect.y.saturating_add(rect.height)
}

fn guide_should_close(
  opened_elapsed: Duration,
  raw_keys: &[crate::host_engine::services::RawKeyEvent],
  system_events: &[SystemEvent],
) -> bool {
  raw_keys.iter().any(|event| {
    event.kind == KeyEventKind::Press && matches!(event.key, Key::A | Key::S | Key::C | Key::R)
      || event.kind == KeyEventKind::Press
        && event.key == Key::Fn(1)
        && opened_elapsed >= Duration::from_millis(1000)
  }) || system_events.iter().any(|event| {
    matches!(
      event,
      SystemEvent::Mouse(mouse)
        if matches!(
          mouse.kind,
          MouseEventKind::Press
            | MouseEventKind::Drag
        )
        && matches!(mouse.button, Some(MouseButton::Left | MouseButton::Right))
    )
  })
}

fn mode_toast_should_close(
  raw_keys: &[crate::host_engine::services::RawKeyEvent],
  system_events: &[SystemEvent],
) -> bool {
  raw_keys
    .iter()
    .any(|event| event.kind == KeyEventKind::Press)
    || system_events.iter().any(|event| {
      matches!(
        event,
        SystemEvent::Mouse(mouse)
          if !matches!(mouse.kind, MouseEventKind::Move | MouseEventKind::Hold)
      )
    })
}

fn operation_toast_should_close(
  raw_keys: &[crate::host_engine::services::RawKeyEvent],
  system_events: &[SystemEvent],
) -> bool {
  raw_keys
    .iter()
    .any(|event| event.kind == KeyEventKind::Press)
    || system_events
      .iter()
      .any(|event| matches!(event, SystemEvent::Mouse(_)))
}
