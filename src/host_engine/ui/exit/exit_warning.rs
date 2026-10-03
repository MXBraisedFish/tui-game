//! Exit warning page state, user commands, and terminal-cell presentation.

use crate::host_engine::services::text_layout::TextWrapMode;
use crate::host_engine::services::{
  ActionMapEntry, CanvasService, DrawTextParams, HitAreaEvent, HitAreaId, HitAreaOptions,
  HitAreaService, I18nService, KeyState, LayoutService, MouseButton, ProgressBarId,
  ProgressBarOptions, ProgressBarSegmentStyle, ProgressBarService, Rect, RenderService,
  RichTextParams, TerminalColor, TextAlign, TextColor, TextStyle, UiEvent, UiObjectPool,
  UiObjectPoolOwner,
};

const NS: &str = "exit_warning";

/// The normal or pending-write form of the exit confirmation view.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExitWarningMode {
  /// The export warning setting for exit warning mode.
  ExportWarning,
  /// The waiting for exports setting for exit warning mode.
  WaitingForExports,
  /// The exception setting for exit warning mode.
  Exception {
    /// The seconds left.
    seconds_left: u8,
  },
}

/// An application request produced by exit warning interactions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExitWarningCommand {
  /// The wait for exports setting for exit warning command.
  WaitForExports,
  /// A request to back.
  Back,
  /// The exit now setting for exit warning command.
  ExitNow,
}

/// The state and owned widgets of the exit warning view.
pub struct ExitWarningUi {
  objects: UiObjectPool,
  image_bar: ProgressBarId,
  video_bar: ProgressBarId,
  wait_area: HitAreaId,
  back_area: HitAreaId,
  exit_area: HitAreaId,
  params: RichTextParams,
}

impl ExitWarningUi {
  /// Create the exit warning view and allocate its owned UI objects.
  ///
  /// # Panics
  ///
  /// Panic if an internal invariant is violated: `exit image progress style must be valid`; `exit
  /// video progress style must be valid`.
  pub fn init(progress_bar: &ProgressBarService, hit_area: &HitAreaService) -> Self {
    let mut objects = UiObjectPool::new();
    let options = progress_options();
    let image_bar = progress_bar
      .create(&mut objects, options.clone())
      .expect("exit image progress style must be valid");
    let video_bar = progress_bar
      .create(&mut objects, options)
      .expect("exit video progress style must be valid");
    let wait_area = hit_area.create(&mut objects, HitAreaOptions::default());
    let back_area = hit_area.create(&mut objects, HitAreaOptions::default());
    let exit_area = hit_area.create(&mut objects, HitAreaOptions::default());
    Self {
      objects,
      image_bar,
      video_bar,
      wait_area,
      back_area,
      exit_area,
      params: RichTextParams::default(),
    }
  }

  /// Return the shortcuts currently enabled by the exit warning view.
  pub fn action_map() -> Vec<ActionMapEntry> {
    vec![
      action("exit_warning.tip_export_auto_exit", "Wait for exports", "1"),
      action("exit_warning.tip_export_back", "Back", "2"),
      action("exit_warning.tip_export_exit", "Exit now", "esc"),
    ]
  }

  /// Return the current waiting action map.
  pub fn waiting_action_map() -> Vec<ActionMapEntry> {
    vec![
      action("exit_warning.tip_export_back", "Back", "1"),
      action("exit_warning.tip_export_exit", "Exit now", "esc"),
    ]
  }

  /// Interpret a exit warning UI event and return the requested application command.
  pub fn handle_event(&self, mode: ExitWarningMode, event: &UiEvent) -> Option<ExitWarningCommand> {
    match event {
      UiEvent::Action(event) if event.state == KeyState::Pressed => {
        match (mode, event.action.as_str()) {
          (ExitWarningMode::ExportWarning, "exit_warning.tip_export_auto_exit") => {
            Some(ExitWarningCommand::WaitForExports)
          }
          (
            ExitWarningMode::ExportWarning | ExitWarningMode::WaitingForExports,
            "exit_warning.tip_export_back",
          ) => Some(ExitWarningCommand::Back),
          (_, "exit_warning.tip_export_exit") => Some(ExitWarningCommand::ExitNow),
          _ => None,
        }
      }
      UiEvent::HitArea(HitAreaEvent::Click {
        id,
        button: MouseButton::Left,
        ..
      }) if *id == self.wait_area && mode == ExitWarningMode::ExportWarning => {
        Some(ExitWarningCommand::WaitForExports)
      }
      UiEvent::HitArea(HitAreaEvent::Click {
        id,
        button: MouseButton::Left,
        ..
      }) if *id == self.back_area => Some(ExitWarningCommand::Back),
      UiEvent::HitArea(HitAreaEvent::Click {
        id,
        button: MouseButton::Left,
        ..
      }) if *id == self.exit_area => Some(ExitWarningCommand::ExitNow),
      _ => None,
    }
  }

  /// Draw the exit warning view and register interaction regions in its assigned surfaces.
  ///
  /// # Arguments
  ///
  /// * `render` - The drawing service used to render terminal cells.
  /// * `canvas` - The clipped canvas used for drawing.
  /// * `layout` - The service resolving terminal sizes and positions.
  /// * `i18n` - The service resolving localized text.
  /// * `progress_bar` - The progress bar.
  /// * `hit_area` - The hit area.
  /// * `mode` - The mode.
  /// * `image` - The image.
  /// * `video` - The video.
  #[allow(clippy::too_many_arguments)]
  pub fn render(
    &mut self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    i18n: &I18nService,
    progress_bar: &ProgressBarService,
    hit_area: &HitAreaService,
    mode: ExitWarningMode,
    image: Option<(usize, f32)>,
    video: Option<(usize, f32)>,
  ) {
    let actions = if mode == ExitWarningMode::WaitingForExports {
      Self::waiting_action_map()
    } else {
      Self::action_map()
    };
    self.params = RichTextParams::from_action_map(&actions, "exit_warning.");
    match mode {
      ExitWarningMode::ExportWarning => {
        self.draw_export_warning(render, canvas, layout, i18n, hit_area)
      }
      ExitWarningMode::WaitingForExports => self.draw_export_waiting(
        render,
        canvas,
        layout,
        i18n,
        progress_bar,
        hit_area,
        image,
        video,
      ),
      ExitWarningMode::Exception { seconds_left } => {
        self.draw_exception(render, canvas, layout, i18n, seconds_left)
      }
    }
  }

  fn draw_title(
    &self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    text: String,
    color: TerminalColor,
    base: bool,
  ) {
    let width = layout.get_text_width(&text, Some(&self.params));
    let params = DrawTextParams {
      x: if base {
        layout.resolve_x(LayoutService::ALIGN_CENTER, width, 0)
      } else {
        layout.resolve_host_x(LayoutService::ALIGN_CENTER, width, 0)
      },
      y: 1,
      text: format!("f%<fg:{}><b>{text}</b></fg>", terminal_color_name(color)),
      params: Some(self.params.clone()),
      ..Default::default()
    };
    if base {
      render.draw_text(canvas, &params);
    } else {
      render.draw_host_text(canvas, &params);
    }
  }

  fn draw_export_warning(
    &mut self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    i18n: &I18nService,
    hit_area: &HitAreaService,
  ) {
    self.draw_title(
      render,
      canvas,
      layout,
      i18n.get_runtime_text(NS, "exit_warning.title.stop"),
      TerminalColor::BrightYellow,
      true,
    );
    let tip = i18n.get_runtime_text(NS, "exit_warning.stop.tip.export");
    let auto = color_text(
      i18n.get_runtime_text(NS, "exit_warning.stop.tip.export.auto_exit"),
      TerminalColor::BrightGreen,
    );
    let back = color_text(
      i18n.get_runtime_text(NS, "exit_warning.stop.tip.export.back"),
      TerminalColor::BrightGreen,
    );
    let exit = color_text(
      i18n.get_runtime_text(NS, "exit_warning.stop.tip.export.exit"),
      TerminalColor::BrightRed,
    );
    let max_width = content_width(layout, 100);
    let width = [&tip, &auto, &back, &exit]
      .into_iter()
      .map(|line| layout.get_text_width(line, Some(&self.params)))
      .max()
      .unwrap_or(1)
      .min(max_width)
      .max(1);
    let tip_height = self.wrapped_height(layout, &tip, width);
    let option_heights = [&auto, &back, &exit].map(|text| self.wrapped_height(layout, text, width));
    let block_height = tip_height
      .saturating_add(1)
      .saturating_add(option_heights.into_iter().fold(0u16, u16::saturating_add));
    let x = layout.resolve_x(LayoutService::ALIGN_CENTER, width, 0);
    let start_y = layout.developer_height().saturating_sub(block_height) / 2;
    self.draw_wrapped_line(render, canvas, x, start_y, width, &tip);

    let auto_y = start_y.saturating_add(tip_height).saturating_add(1);
    let back_y = auto_y.saturating_add(option_heights[0]);
    let exit_y = back_y.saturating_add(option_heights[1]);
    for (id, y, height, text) in [
      (self.wait_area, auto_y, option_heights[0], auto.as_str()),
      (self.back_area, back_y, option_heights[1], back.as_str()),
      (self.exit_area, exit_y, option_heights[2], exit.as_str()),
    ] {
      self.draw_wrapped_line(render, canvas, x, y, width, text);
      self.register_area(
        hit_area,
        canvas,
        id,
        Rect {
          x,
          y,
          width,
          height,
        },
      );
    }
  }

  #[allow(clippy::too_many_arguments)]
  fn draw_export_waiting(
    &mut self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    i18n: &I18nService,
    progress_bar: &ProgressBarService,
    hit_area: &HitAreaService,
    image: Option<(usize, f32)>,
    video: Option<(usize, f32)>,
  ) {
    self.draw_title(
      render,
      canvas,
      layout,
      i18n.get_runtime_text(NS, "exit_warning.title.stop"),
      TerminalColor::BrightYellow,
      true,
    );
    let tip = i18n.get_runtime_text(NS, "exit_warning.stop.tip.export.auto_exit.tip");
    let back = color_text(
      i18n.get_runtime_text(NS, "exit_warning.stop.tip.export.back"),
      TerminalColor::BrightGreen,
    );
    let exit = color_text(
      i18n.get_runtime_text(NS, "exit_warning.stop.tip.export.exit"),
      TerminalColor::BrightRed,
    );
    let content_width = content_width(layout, 100);
    let x = layout.resolve_x(LayoutService::ALIGN_CENTER, content_width, 0);
    let tip_params = DrawTextParams {
      text: tip.clone(),
      params: Some(self.params.clone()),
      wrap_mode: TextWrapMode::Auto,
      max_width: Some(content_width),
      ..Default::default()
    };
    let tip_height = layout.get_draw_text_size(&tip_params).height.max(1);
    let back_height = self.wrapped_height(layout, &back, content_width);
    let exit_height = self.wrapped_height(layout, &exit, content_width);
    let block_height = tip_height
      .saturating_add(4)
      .saturating_add(back_height)
      .saturating_add(exit_height);
    let start_y = layout.developer_height().saturating_sub(block_height) / 2;
    render.draw_text(
      canvas,
      &DrawTextParams {
        x,
        y: start_y,
        ..tip_params
      },
    );
    let first_queue_y = start_y.saturating_add(tip_height).saturating_add(1);
    self.draw_queue_row(
      render,
      canvas,
      layout,
      i18n,
      progress_bar,
      x,
      first_queue_y,
      content_width,
      true,
      image,
    );
    self.draw_queue_row(
      render,
      canvas,
      layout,
      i18n,
      progress_bar,
      x,
      first_queue_y.saturating_add(1),
      content_width,
      false,
      video,
    );
    let first_action_y = first_queue_y.saturating_add(3);
    let exit_y = first_action_y.saturating_add(back_height);
    render.draw_text(
      canvas,
      &DrawTextParams {
        x,
        y: first_action_y,
        text: back,
        params: Some(self.params.clone()),
        max_width: Some(content_width),
        ..Default::default()
      },
    );
    render.draw_text(
      canvas,
      &DrawTextParams {
        x,
        y: exit_y,
        text: exit,
        params: Some(self.params.clone()),
        max_width: Some(content_width),
        ..Default::default()
      },
    );
    self.register_area(
      hit_area,
      canvas,
      self.back_area,
      Rect {
        x,
        y: first_action_y,
        width: content_width,
        height: back_height,
      },
    );
    self.register_area(
      hit_area,
      canvas,
      self.exit_area,
      Rect {
        x,
        y: exit_y,
        width: content_width,
        height: exit_height,
      },
    );
  }

  #[allow(clippy::too_many_arguments)]
  fn draw_queue_row(
    &mut self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    i18n: &I18nService,
    progress_bar: &ProgressBarService,
    x: u16,
    y: u16,
    width: u16,
    image: bool,
    queue: Option<(usize, f32)>,
  ) {
    let key = if image {
      "exit_warning.stop.tip.export.queue.image"
    } else {
      "exit_warning.stop.tip.export.queue.video"
    };
    let done_key = if image {
      "exit_warning.stop.tip.export.queue.image.get"
    } else {
      "exit_warning.stop.tip.export.queue.video.get"
    };
    let Some((count, ratio)) = queue else {
      render.draw_text(
        canvas,
        &DrawTextParams {
          x,
          y,
          text: color_text(
            i18n.get_runtime_text(NS, done_key),
            TerminalColor::BrightGreen,
          ),
          params: Some(self.params.clone()),
          max_width: Some(width),
          ..Default::default()
        },
      );
      return;
    };
    let label = i18n
      .get_runtime_text(NS, key)
      .replace("{value:image_queue}", &count.to_string())
      .replace("{value:video_queue}", &count.to_string());
    let percent = format!("{:>5.1}%", ratio.clamp(0.0, 1.0) * 100.0);
    let label_width = layout
      .get_text_width(&label, Some(&self.params))
      .min(width / 3);
    let percent_width = layout.get_text_width(&percent, Some(&self.params));
    let bar_x = x.saturating_add(label_width).saturating_add(2);
    let bar_width = width
      .saturating_sub(label_width)
      .saturating_sub(percent_width)
      .saturating_sub(4);
    render.draw_text(
      canvas,
      &DrawTextParams {
        x,
        y,
        text: label,
        params: Some(self.params.clone()),
        max_width: Some(label_width),
        ..Default::default()
      },
    );
    render.draw_text(
      canvas,
      &DrawTextParams {
        x: x.saturating_add(width.saturating_sub(percent_width)),
        y,
        text: percent,
        params: Some(self.params.clone()),
        ..Default::default()
      },
    );
    let bar = if image {
      self.image_bar
    } else {
      self.video_bar
    };
    let _ = progress_bar.set_progress(&mut self.objects, bar, ratio, ratio);
    let _ = progress_bar.render(
      &self.objects,
      bar,
      Rect {
        x: bar_x,
        y,
        width: bar_width,
        height: 1,
      },
      canvas,
    );
  }

  fn draw_exception(
    &self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    i18n: &I18nService,
    seconds_left: u8,
  ) {
    self.draw_title(
      render,
      canvas,
      layout,
      i18n.get_runtime_text(NS, "exit_warning.title.exception"),
      TerminalColor::BrightRed,
      false,
    );
    let tip = i18n.get_runtime_text(NS, "exit_warning.exception.tip");
    let countdown = color_text(
      i18n
        .get_runtime_text(NS, "exit_warning.exception.countdown")
        .replace("{value:second}", &seconds_left.to_string()),
      TerminalColor::BrightRed,
    );
    let exit = color_text(
      i18n.get_runtime_text(NS, "exit_warning.exception.exit"),
      TerminalColor::BrightRed,
    );
    self.draw_centered_text_block(render, canvas, layout, &[tip, countdown, exit], 1);
  }

  fn draw_wrapped_line(
    &self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    x: u16,
    y: u16,
    width: u16,
    text: &str,
  ) {
    render.draw_text(
      canvas,
      &DrawTextParams {
        x,
        y,
        text: text.to_string(),
        params: Some(self.params.clone()),
        wrap_mode: TextWrapMode::Auto,
        max_width: Some(width),
        ..Default::default()
      },
    );
  }

  fn register_area(
    &mut self,
    hit_area: &HitAreaService,
    canvas: &CanvasService,
    id: HitAreaId,
    rect: Rect,
  ) {
    hit_area.render(
      &mut self.objects,
      id,
      Rect {
        width: rect.width.max(1),
        height: rect.height.max(1),
        ..rect
      },
      canvas,
    );
  }

  fn draw_centered_text_block(
    &self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    lines: &[String],
    gap: u16,
  ) {
    let width = content_width(layout, 100);
    let heights = lines
      .iter()
      .map(|line| self.wrapped_height(layout, line, width))
      .collect::<Vec<_>>();
    let height = heights
      .iter()
      .copied()
      .fold(0u16, u16::saturating_add)
      .saturating_add(gap.saturating_mul(lines.len().saturating_sub(1) as u16));
    let mut y = layout.physical_height().saturating_sub(height) / 2;
    for (line, line_height) in lines.iter().zip(heights) {
      let line_width = layout
        .get_draw_text_size(&DrawTextParams {
          text: line.clone(),
          params: Some(self.params.clone()),
          wrap_mode: TextWrapMode::Auto,
          max_width: Some(width),
          ..Default::default()
        })
        .width
        .max(1);
      render.draw_host_text(
        canvas,
        &DrawTextParams {
          x: layout.resolve_host_x(LayoutService::ALIGN_CENTER, line_width, 0),
          y,
          text: line.clone(),
          params: Some(self.params.clone()),
          line_align: TextAlign::Center,
          wrap_mode: TextWrapMode::Auto,
          max_width: Some(width),
          ..Default::default()
        },
      );
      y = y.saturating_add(line_height).saturating_add(gap);
    }
  }

  fn wrapped_height(&self, layout: &LayoutService, text: &str, width: u16) -> u16 {
    layout
      .get_draw_text_size(&DrawTextParams {
        text: text.to_string(),
        params: Some(self.params.clone()),
        wrap_mode: TextWrapMode::Auto,
        max_width: Some(width),
        ..Default::default()
      })
      .height
      .max(1)
  }
}

impl UiObjectPoolOwner for ExitWarningUi {
  fn objects(&self) -> &UiObjectPool {
    &self.objects
  }

  fn objects_mut(&mut self) -> &mut UiObjectPool {
    &mut self.objects
  }
}

fn content_width(layout: &LayoutService, preferred: u16) -> u16 {
  let screen = layout.developer_width();
  let margin: u16 = if screen >= 40 { 16 } else { 1 };
  preferred
    .min(screen.saturating_sub(margin.saturating_mul(2)))
    .max(1)
}

fn action(action: &str, description: &str, key: &str) -> ActionMapEntry {
  ActionMapEntry {
    action: action.to_string(),
    description: description.to_string(),
    keys: vec![vec![key.to_string()]],
    priority: 0,
  }
}

fn progress_options() -> ProgressBarOptions {
  ProgressBarOptions {
    completed: segment(TerminalColor::Green),
    preview: segment(TerminalColor::Green),
    remaining: segment(TerminalColor::BrightBlack),
    ..Default::default()
  }
}

fn segment(color: TerminalColor) -> ProgressBarSegmentStyle {
  ProgressBarSegmentStyle {
    ch: '─',
    style: TextStyle {
      foreground: Some(TextColor::Terminal(color)),
      background: Some(TextColor::Transparent),
      ..Default::default()
    },
  }
}

fn color_text(text: String, color: TerminalColor) -> String {
  format!("f%<fg:{}>{text}</fg>", terminal_color_name(color))
}

fn terminal_color_name(color: TerminalColor) -> &'static str {
  match color {
    TerminalColor::BrightYellow => "bright_yellow",
    TerminalColor::BrightGreen => "bright_green",
    TerminalColor::BrightRed => "bright_red",
    _ => "white",
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::host_engine::services::{
    HitAreaService, InputActionEvent, InputEventType, RichTextService,
  };

  #[test]
  fn warning_actions_render_the_keys_used_by_the_active_mode() {
    let initial = RichTextParams::from_action_map(&ExitWarningUi::action_map(), "exit_warning.");
    let waiting =
      RichTextParams::from_action_map(&ExitWarningUi::waiting_action_map(), "exit_warning.");
    let rich = RichTextService::new();

    assert_eq!(
      rich.visible_text(
        "f%{key:exit_warning.tip_export_auto_exit} \
         {key:exit_warning.tip_export_back} {key:exit_warning.tip_export_exit}",
        Some(&initial),
      ),
      "[1] [2] [Esc]"
    );
    assert_eq!(
      rich.visible_text(
        "f%{key:exit_warning.tip_export_back} {key_default:exit_warning.tip_export_back}",
        Some(&waiting),
      ),
      "[1] [1]"
    );
  }

  #[test]
  fn warning_input_routes_wait_back_and_immediate_exit() {
    let ui = ExitWarningUi::init(&ProgressBarService::new(), &HitAreaService::new());
    let event = |action: &str| {
      UiEvent::Action(InputActionEvent {
        event_type: InputEventType::Keyboard,
        action: action.to_string(),
        state: KeyState::Pressed,
      })
    };

    assert_eq!(
      ui.handle_event(
        ExitWarningMode::ExportWarning,
        &event("exit_warning.tip_export_auto_exit"),
      ),
      Some(ExitWarningCommand::WaitForExports)
    );
    assert_eq!(
      ui.handle_event(
        ExitWarningMode::WaitingForExports,
        &event("exit_warning.tip_export_back"),
      ),
      Some(ExitWarningCommand::Back)
    );
    assert_eq!(
      ui.handle_event(
        ExitWarningMode::WaitingForExports,
        &event("exit_warning.tip_export_exit"),
      ),
      Some(ExitWarningCommand::ExitNow)
    );

    assert_eq!(
      ui.handle_event(
        ExitWarningMode::ExportWarning,
        &UiEvent::HitArea(HitAreaEvent::Click {
          id: ui.wait_area,
          button: MouseButton::Left,
          x: 0,
          y: 0,
        }),
      ),
      Some(ExitWarningCommand::WaitForExports)
    );
    assert_eq!(
      ui.handle_event(
        ExitWarningMode::WaitingForExports,
        &UiEvent::HitArea(HitAreaEvent::Click {
          id: ui.back_area,
          button: MouseButton::Left,
          x: 0,
          y: 0,
        }),
      ),
      Some(ExitWarningCommand::Back)
    );
  }
}
