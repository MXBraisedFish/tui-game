use std::time::Duration;

use crate::host_engine::services::{
  ActionMapEntry, BorderStyle, CanvasService, DrawTextParams, I18nService, KeyState, LayoutService,
  MouseButton, MouseEvent, MouseEventKind, Rect, RenderService, RichTextParams, RuntimeObjectPool,
  RuntimeObjectPoolOwner, StorageService, TextColor, TextStyle, UiEvent, UiObjectPool,
  UiObjectPoolOwner,
};

const STEP_UNICODE: usize = 0;

const STEP_COLOR: usize = 1;

const STEP_MOUSE: usize = 2;

const UNICODE_OPTIONS: usize = 2;

const COLOR_OPTIONS: usize = 3;

const UNICODE_SAMPLE: &str = "你好 World \u{1F30D} ȧb عربى";

const BAND_LEFT_PCT: u16 = 20;

const BAND_RIGHT_PCT: u16 = 80;

const BAND_ROWS: u16 = 3;

const MOUSE_BOX_HEIGHT: u16 = 5;

const MOUSE_BOX_PADDING: u16 = 5;

const RAINBOW: &[(u8, u8, u8)] = &[
  (255, 0, 0),
  (255, 165, 0),
  (255, 255, 0),
  (0, 255, 0),
  (0, 255, 255),
  (0, 0, 255),
  (128, 0, 255),
];

/// 终端检测页面的布局信息。
pub(crate) struct TerminalCheckLayout {
  title_x: u16,
  title_y: u16,
  tip_x: u16,
  tip_y: u16,
  sample_x: u16,
  sample_y: u16,
  option_rects: Vec<Rect>,
  option_xs: Vec<u16>,
  hint_x: u16,
  hint_y: u16,
}

/// 终端能力检测 UI：分步检测 Unicode 支持、真彩色支持和鼠标支持。
pub struct TerminalCheckUi {
  step: usize,

  selected_index: usize,
  objects: UiObjectPool,
  runtime_objects: RuntimeObjectPool,
}

impl UiObjectPoolOwner for TerminalCheckUi {
  fn objects(&self) -> &UiObjectPool {
    &self.objects
  }

  fn objects_mut(&mut self) -> &mut UiObjectPool {
    &mut self.objects
  }
}

impl RuntimeObjectPoolOwner for TerminalCheckUi {
  fn runtime_objects(&self) -> &RuntimeObjectPool {
    &self.runtime_objects
  }

  fn runtime_objects_mut(&mut self) -> &mut RuntimeObjectPool {
    &mut self.runtime_objects
  }
}

/// 终端检测页面的命令。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TerminalCheckCommand {
  Next,

  Exit,

  Done { mouse: bool },
}

impl TerminalCheckUi {
  /// 初始化终端检测 UI。
  pub fn init() -> Self {
    Self {
      step: STEP_UNICODE,
      selected_index: 0,
      objects: UiObjectPool::new(),
      runtime_objects: RuntimeObjectPool::new(),
    }
  }

  fn apply_detection(&mut self) {
    self.selected_index = match self.step {
      STEP_UNICODE => 0,
      STEP_COLOR => 0,
      STEP_MOUSE => 1,
      _ => 0,
    };
  }

  /// 返回终端检测页面的按键映射定义。
  pub fn action_map() -> Vec<ActionMapEntry> {
    vec![
      ActionMapEntry {
        action: "terminal.focus_up".to_string(),
        description: "Focus previous option".to_string(),
        keys: vec![vec!["up".to_string()]],
      },
      ActionMapEntry {
        action: "terminal.focus_down".to_string(),
        description: "Focus next option".to_string(),
        keys: vec![vec!["down".to_string()]],
      },
      ActionMapEntry {
        action: "terminal.confirm".to_string(),
        description: "Confirm selection".to_string(),
        keys: vec![vec!["enter".to_string()]],
      },
      ActionMapEntry {
        action: "terminal.exit".to_string(),
        description: "Exit program".to_string(),
        keys: vec![vec!["esc".to_string()]],
      },
    ]
  }

  /// 处理键盘事件。
  pub fn handle_event(&mut self, event: &UiEvent) -> Option<TerminalCheckCommand> {
    let UiEvent::Action(event) = event else {
      return None;
    };
    if event.state != KeyState::Pressed {
      return None;
    }

    match event.action.as_str() {
      "terminal.focus_up" => {
        if self.step == STEP_MOUSE {
          return None;
        }
        if self.selected_index > 0 {
          self.selected_index -= 1;
        }
        None
      }
      "terminal.focus_down" => {
        if self.step == STEP_MOUSE {
          return None;
        }
        let max = self.option_count().saturating_sub(1);
        if self.selected_index < max {
          self.selected_index += 1;
        }
        None
      }
      "terminal.confirm" => self.confirm_current(),
      "terminal.exit" => Some(TerminalCheckCommand::Exit),
      _ => None,
    }
  }

  /// 处理鼠标事件（移动、点击、右键返回）。
  pub fn handle_mouse_event(
    &mut self,
    event: &MouseEvent,
    positions: &TerminalCheckLayout,
  ) -> Option<TerminalCheckCommand> {
    match event.kind {
      MouseEventKind::Move | MouseEventKind::Hold => {
        if let Some(index) = Self::hit_test(positions, event.x, event.y) {
          self.selected_index = index;
        }
        None
      }
      MouseEventKind::Press => {
        if event.button == Some(MouseButton::Right) {
          return Some(TerminalCheckCommand::Exit);
        }
        if event.button == Some(MouseButton::Left) {
          if Self::hit_test(positions, event.x, event.y).is_some() {
            return self.confirm_current();
          }
        }
        None
      }
      _ => None,
    }
  }

  pub fn update(&mut self, dt: Duration) -> Option<TerminalCheckCommand> {
    let _ = dt;
    None
  }

  /// 根据当前步骤渲染对应内容。
  pub fn render(
    &self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    i18n: &I18nService,
  ) {
    match self.step {
      STEP_UNICODE => self.render_unicode_step(render, canvas, layout, i18n),
      STEP_COLOR => self.render_color_step(render, canvas, layout, i18n),
      STEP_MOUSE => self.render_mouse_step(render, canvas, layout, i18n),
      _ => self.render_placeholder(render, canvas, layout),
    }
  }

  /// 根据当前步骤计算布局信息。
  pub fn compute_positions(
    &self,
    layout: &LayoutService,
    i18n: &I18nService,
  ) -> TerminalCheckLayout {
    match self.step {
      STEP_UNICODE => self.compute_unicode_positions(layout, i18n),
      STEP_COLOR => self.compute_color_positions(layout, i18n),
      STEP_MOUSE => self.compute_mouse_positions(layout, i18n),
      _ => self.compute_placeholder_positions(layout),
    }
  }

  fn compute_unicode_positions(
    &self,
    layout: &LayoutService,
    i18n: &I18nService,
  ) -> TerminalCheckLayout {
    let title = i18n.get_runtime_text("terminal", "terminal.unicode.title");
    let tip = i18n.get_runtime_text("terminal", "terminal.unicode.tip");
    let yes_text = i18n.get_runtime_text("terminal", "terminal.unicode.yes");
    let no_text = i18n.get_runtime_text("terminal", "terminal.unicode.no");

    let key_params = self.build_key_params();
    let hint = format!(
      "{}  {}  {}",
      i18n.get_runtime_text("terminal", "terminal.action.select"),
      i18n.get_runtime_text("terminal", "terminal.action.confirm"),
      i18n.get_runtime_text("terminal", "terminal.action.exit"),
    );

    let term_h = layout.physical_size().height;

    let title_y: u16 = 1;
    let hint_y = term_h.saturating_sub(1);

    let title_x = centered_x(layout, &title);

    let hint_w = layout.get_text_width(&hint, Some(&key_params));
    let hint_x = layout.resolve_host_x(LayoutService::ALIGN_CENTER, hint_w, 0);

    let option_names: [&str; UNICODE_OPTIONS] = [&yes_text, &no_text];
    let option_texts: Vec<String> = (0..UNICODE_OPTIONS)
      .map(|i| self.option_display_name(&option_names, i))
      .collect();
    let option_xs: Vec<u16> = option_texts.iter().map(|t| centered_x(layout, t)).collect();
    let options_height = UNICODE_OPTIONS as u16;
    let available = hint_y.saturating_sub(title_y).saturating_sub(1);
    let option_start_y = if available > options_height {
      title_y
        .saturating_add(1)
        .saturating_add((available - options_height) / 2)
    } else {
      title_y.saturating_add(1)
    };
    let sample_y = option_start_y.saturating_sub(2);
    let tip_y = option_start_y.saturating_sub(4);

    let tip_x = centered_x(layout, &tip);
    let sample_x = centered_x(layout, UNICODE_SAMPLE);

    let option_widths: Vec<u16> = option_texts
      .iter()
      .map(|t| layout.get_text_width(t, None))
      .collect();
    let option_rects: Vec<Rect> = (0..UNICODE_OPTIONS)
      .map(|i| Rect {
        x: option_xs[i],
        y: option_start_y.saturating_add(i as u16),
        width: option_widths[i],
        height: 1,
      })
      .collect();

    TerminalCheckLayout {
      title_x,
      title_y,
      tip_x,
      tip_y,
      sample_x,
      sample_y,
      hint_x,
      hint_y,
      option_rects,
      option_xs,
    }
  }

  fn render_unicode_step(
    &self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    i18n: &I18nService,
  ) {
    let positions = self.compute_unicode_positions(layout, i18n);
    let key_params = self.build_key_params();

    let title = i18n.get_runtime_text("terminal", "terminal.unicode.title");
    let tip = i18n.get_runtime_text("terminal", "terminal.unicode.tip");
    let yes_text = i18n.get_runtime_text("terminal", "terminal.unicode.yes");
    let no_text = i18n.get_runtime_text("terminal", "terminal.unicode.no");
    let hint = format!(
      "{}  {}  {}",
      i18n.get_runtime_text("terminal", "terminal.action.select"),
      i18n.get_runtime_text("terminal", "terminal.action.confirm"),
      i18n.get_runtime_text("terminal", "terminal.action.exit"),
    );
    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: positions.title_x,
        y: positions.title_y,
        text: format!("f%<fg:bright_magenta><b>{}</b></fg>", title),
        ..Default::default()
      },
    );
    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: positions.tip_x,
        y: positions.tip_y,
        text: format!("f%<fg:bright_yellow>{}</fg>", tip),
        ..Default::default()
      },
    );
    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: positions.sample_x,
        y: positions.sample_y,
        text: UNICODE_SAMPLE.to_string(),
        ..Default::default()
      },
    );
    let option_names: [&str; UNICODE_OPTIONS] = [&yes_text, &no_text];
    let option_texts: Vec<String> = (0..UNICODE_OPTIONS)
      .map(|i| self.option_display_name(&option_names, i))
      .collect();
    for i in 0..UNICODE_OPTIONS {
      let text = if i == self.selected_index {
        let fg = if self.is_exit_option(i) {
          "bright_red"
        } else {
          "bright_cyan"
        };
        format!("f%<fg:{}>{}</fg>", fg, option_texts[i])
      } else {
        option_texts[i].clone()
      };
      render.draw_host_text(
        canvas,
        &DrawTextParams {
          x: positions.option_xs[i],
          y: positions.option_rects[i].y,
          text,
          ..Default::default()
        },
      );
    }
    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: positions.hint_x,
        y: positions.hint_y,
        text: format!("f%<fg:rgb(85,87,83)>{}</fg>", hint),
        params: Some(key_params),
        ..Default::default()
      },
    );
  }

  fn compute_color_positions(
    &self,
    layout: &LayoutService,
    i18n: &I18nService,
  ) -> TerminalCheckLayout {
    let title = i18n.get_runtime_text("terminal", "terminal.truecolor.title");
    let yes_text = i18n.get_runtime_text("terminal", "terminal.truecolor.yes");
    let no256_text = i18n.get_runtime_text("terminal", "terminal.truecolor.no-256");
    let no_other_text = i18n.get_runtime_text("terminal", "terminal.truecolor.no-other");

    let key_params = self.build_key_params();
    let hint = format!(
      "{}  {}  {}",
      i18n.get_runtime_text("terminal", "terminal.action.select"),
      i18n.get_runtime_text("terminal", "terminal.action.confirm"),
      i18n.get_runtime_text("terminal", "terminal.action.exit"),
    );

    let term_h = layout.physical_size().height;

    let title_y: u16 = 1;
    let hint_y = term_h.saturating_sub(1);

    let title_x = centered_x(layout, &title);
    let hint_w = layout.get_text_width(&hint, Some(&key_params));
    let hint_x = layout.resolve_host_x(LayoutService::ALIGN_CENTER, hint_w, 0);
    let option_names: [&str; COLOR_OPTIONS] = [&yes_text, &no256_text, &no_other_text];
    let option_texts: Vec<String> = (0..COLOR_OPTIONS)
      .map(|i| self.option_display_name(&option_names, i))
      .collect();
    let option_xs: Vec<u16> = option_texts.iter().map(|t| centered_x(layout, t)).collect();
    let options_height = COLOR_OPTIONS as u16;
    let tip = i18n.get_runtime_text("terminal", "terminal.truecolor.tip");
    let tip_x = centered_x(layout, &tip);
    let band_block_height: u16 = 1 + 1 + BAND_ROWS + 1;
    let total_content = band_block_height
      .saturating_add(1)
      .saturating_add(options_height);
    let available = hint_y.saturating_sub(title_y).saturating_sub(1);
    let content_start_y = if available > total_content {
      title_y
        .saturating_add(1)
        .saturating_add((available - total_content) / 2)
    } else {
      title_y.saturating_add(1)
    };

    let tip_y = content_start_y;
    let band_y = tip_y.saturating_add(2);
    let option_start_y = band_y.saturating_add(BAND_ROWS).saturating_add(1);

    let option_widths: Vec<u16> = option_texts
      .iter()
      .map(|t| layout.get_text_width(t, None))
      .collect();
    let option_rects: Vec<Rect> = (0..COLOR_OPTIONS)
      .map(|i| Rect {
        x: option_xs[i],
        y: option_start_y.saturating_add(i as u16),
        width: option_widths[i],
        height: 1,
      })
      .collect();

    TerminalCheckLayout {
      title_x,
      title_y,
      tip_x,
      tip_y,
      sample_x: 0,
      sample_y: 0,
      hint_x,
      hint_y,
      option_rects,
      option_xs,
    }
  }

  fn render_color_step(
    &self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    i18n: &I18nService,
  ) {
    let positions = self.compute_color_positions(layout, i18n);
    let key_params = self.build_key_params();

    let title = i18n.get_runtime_text("terminal", "terminal.truecolor.title");
    let tip = i18n.get_runtime_text("terminal", "terminal.truecolor.tip");
    let yes_text = i18n.get_runtime_text("terminal", "terminal.truecolor.yes");
    let no256_text = i18n.get_runtime_text("terminal", "terminal.truecolor.no-256");
    let no_other_text = i18n.get_runtime_text("terminal", "terminal.truecolor.no-other");
    let hint = format!(
      "{}  {}  {}",
      i18n.get_runtime_text("terminal", "terminal.action.select"),
      i18n.get_runtime_text("terminal", "terminal.action.confirm"),
      i18n.get_runtime_text("terminal", "terminal.action.exit"),
    );
    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: positions.title_x,
        y: positions.title_y,
        text: format!("f%<fg:bright_magenta><b>{}</b></fg>", title),
        ..Default::default()
      },
    );
    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: positions.tip_x,
        y: positions.tip_y,
        text: format!("f%<fg:bright_yellow>{}</fg>", tip),
        ..Default::default()
      },
    );
    let term_w = layout.physical_size().width;
    let left = term_w * BAND_LEFT_PCT / 100;
    let right = term_w * BAND_RIGHT_PCT / 100;
    let band_w = right.saturating_sub(left);
    for row in 0..BAND_ROWS {
      let y = positions.tip_y.saturating_add(2).saturating_add(row);
      for col in 0..band_w {
        let t = if band_w > 1 {
          col as f32 / (band_w - 1) as f32
        } else {
          0.0
        };
        let (r, g, b) = rainbow_at(t);
        canvas.host_styled_text(
          left + col,
          y,
          " ",
          TextStyle {
            background: Some(TextColor::Rgb { r, g, b }),
            ..Default::default()
          },
        );
      }
    }
    let option_names: [&str; COLOR_OPTIONS] = [&yes_text, &no256_text, &no_other_text];
    let option_texts: Vec<String> = (0..COLOR_OPTIONS)
      .map(|i| self.option_display_name(&option_names, i))
      .collect();
    for i in 0..COLOR_OPTIONS {
      let text = if i == self.selected_index {
        let fg = if self.is_exit_option(i) {
          "bright_red"
        } else {
          "bright_cyan"
        };
        format!("f%<fg:{}>{}</fg>", fg, option_texts[i])
      } else {
        option_texts[i].clone()
      };
      render.draw_host_text(
        canvas,
        &DrawTextParams {
          x: positions.option_xs[i],
          y: positions.option_rects[i].y,
          text,
          ..Default::default()
        },
      );
    }
    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: positions.hint_x,
        y: positions.hint_y,
        text: format!("f%<fg:rgb(85,87,83)>{}</fg>", hint),
        params: Some(key_params),
        ..Default::default()
      },
    );
  }

  fn compute_placeholder_positions(&self, layout: &LayoutService) -> TerminalCheckLayout {
    let title = "---";
    let title_w = layout.get_text_width(title, None);
    let title_x = layout.resolve_host_x(LayoutService::ALIGN_CENTER, title_w, 0);
    let title_y: u16 = 1;

    TerminalCheckLayout {
      title_x,
      title_y,
      tip_x: 0,
      tip_y: 0,
      sample_x: 0,
      sample_y: 0,
      option_rects: Vec::new(),
      option_xs: Vec::new(),
      hint_x: 0,
      hint_y: layout.physical_size().height.saturating_sub(1),
    }
  }

  fn render_placeholder(
    &self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout_svc: &LayoutService,
  ) {
    let positions = self.compute_placeholder_positions(layout_svc);
    let title = format!("{} ({}/{})", self._step_title(), self.step + 1, 3);
    let title_w = layout_svc.get_text_width(&title, None);
    let title_x = layout_svc.resolve_host_x(LayoutService::ALIGN_CENTER, title_w, 0);

    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: title_x,
        y: positions.title_y,
        text: format!("f%<b>{}</b>", title),
        ..Default::default()
      },
    );

    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: positions.hint_x,
        y: positions.hint_y,
        text: "f%<fg:bright_black>[↑↓] Select  [Enter] Confirm</fg>".to_string(),
        ..Default::default()
      },
    );
  }

  fn compute_mouse_positions(
    &self,
    layout: &LayoutService,
    i18n: &I18nService,
  ) -> TerminalCheckLayout {
    let title = i18n.get_runtime_text("terminal", "terminal.mouse.title");
    let tip = i18n.get_runtime_text("terminal", "terminal.mouse.tip");
    let no_text = i18n.get_runtime_text("terminal", "terminal.mouse.no");

    let key_params = self.build_key_params();
    let hint = format!(
      "{}  {}",
      i18n.get_runtime_text("terminal", "terminal.action.confirm"),
      i18n.get_runtime_text("terminal", "terminal.action.exit"),
    );

    let term_h = layout.physical_size().height;

    let title_y: u16 = 1;
    let hint_y = term_h.saturating_sub(1);

    let title_x = centered_x(layout, &title);
    let hint_w = layout.get_text_width(&hint, Some(&key_params));
    let hint_x = layout.resolve_host_x(LayoutService::ALIGN_CENTER, hint_w, 0);
    let tip_w = layout.get_text_width(&tip, None);
    let box_w = tip_w + MOUSE_BOX_PADDING * 2 + 2;
    let box_x = layout.resolve_host_x(LayoutService::ALIGN_CENTER, box_w, 0);
    let available = hint_y.saturating_sub(title_y).saturating_sub(5);
    let box_y = if available > MOUSE_BOX_HEIGHT {
      title_y
        .saturating_add(1)
        .saturating_add((available - MOUSE_BOX_HEIGHT) / 2)
    } else {
      title_y.saturating_add(1)
    };

    let tip_x = box_x + 1 + MOUSE_BOX_PADDING;
    let tip_y = box_y + (MOUSE_BOX_HEIGHT - 1) / 2;

    let no_display = if self.selected_index == 1 {
      format!("❯ {} ❮", no_text)
    } else {
      no_text.clone()
    };
    let no_x = centered_x(layout, &no_display);
    let no_y = box_y + MOUSE_BOX_HEIGHT + 1;
    let no_w = layout.get_text_width(&no_display, None);

    let option_rects = vec![
      Rect {
        x: box_x,
        y: box_y,
        width: box_w,
        height: MOUSE_BOX_HEIGHT,
      },
      Rect {
        x: no_x,
        y: no_y,
        width: no_w,
        height: 1,
      },
    ];

    TerminalCheckLayout {
      title_x,
      title_y,
      tip_x,
      tip_y,
      sample_x: 0,
      sample_y: 0,
      hint_x,
      hint_y,
      option_rects,
      option_xs: vec![box_x, no_x],
    }
  }

  fn render_mouse_step(
    &self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    i18n: &I18nService,
  ) {
    let positions = self.compute_mouse_positions(layout, i18n);
    let key_params = self.build_key_params();

    let title = i18n.get_runtime_text("terminal", "terminal.mouse.title");
    let tip = i18n.get_runtime_text("terminal", "terminal.mouse.tip");
    let no_text = i18n.get_runtime_text("terminal", "terminal.mouse.no");
    let hint = format!(
      "{}  {}",
      i18n.get_runtime_text("terminal", "terminal.action.confirm"),
      i18n.get_runtime_text("terminal", "terminal.action.exit"),
    );
    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: positions.title_x,
        y: positions.title_y,
        text: format!("f%<fg:bright_magenta><b>{}</b></fg>", title),
        ..Default::default()
      },
    );
    let rect_focused = self.selected_index == 0;
    let border_fg = if rect_focused {
      TextColor::Terminal(crate::host_engine::services::TerminalColor::BrightCyan)
    } else {
      TextColor::Terminal(crate::host_engine::services::TerminalColor::White)
    };
    let tip_fg = if rect_focused {
      Some(TextColor::Terminal(
        crate::host_engine::services::TerminalColor::BrightCyan,
      ))
    } else {
      None
    };
    render.draw_host_border_rect(
      canvas,
      positions.option_rects[0].x,
      positions.option_rects[0].y,
      positions.option_rects[0].width,
      positions.option_rects[0].height,
      &BorderStyle::Line,
      Some(border_fg),
      None,
      None,
      None,
    );
    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: positions.tip_x,
        y: positions.tip_y,
        text: tip,
        fg: tip_fg,
        ..Default::default()
      },
    );
    let no_display = if self.selected_index == 1 {
      format!("❯ {} ❮", no_text)
    } else {
      no_text
    };
    let no_text_final = if self.selected_index == 1 {
      format!("f%<fg:bright_cyan>{}</fg>", no_display)
    } else {
      format!("f%<fg:white>{}</fg>", no_display)
    };
    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: positions.option_xs[1],
        y: positions.option_rects[1].y,
        text: no_text_final,
        ..Default::default()
      },
    );
    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: positions.hint_x,
        y: positions.hint_y,
        text: format!("f%<fg:rgb(85,87,83)>{}</fg>", hint),
        params: Some(key_params),
        ..Default::default()
      },
    );
  }

  fn option_count(&self) -> usize {
    match self.step {
      STEP_UNICODE => UNICODE_OPTIONS,
      STEP_COLOR => COLOR_OPTIONS,
      STEP_MOUSE => 2,
      _ => 0,
    }
  }

  fn option_display_name(&self, names: &[&str], index: usize) -> String {
    let name = names[index];
    if index == self.selected_index {
      format!("❯ {} ❮", name)
    } else {
      format!("   {}   ", name)
    }
  }

  fn confirm_current(&self) -> Option<TerminalCheckCommand> {
    match self.step {
      STEP_UNICODE => {
        if self.selected_index == 0 {
          Some(TerminalCheckCommand::Next)
        } else {
          Some(TerminalCheckCommand::Exit)
        }
      }
      STEP_COLOR => match self.selected_index {
        0 => Some(TerminalCheckCommand::Next),
        1 => Some(TerminalCheckCommand::Next),
        2 => Some(TerminalCheckCommand::Exit),
        _ => None,
      },
      STEP_MOUSE => Some(TerminalCheckCommand::Done {
        mouse: self.selected_index == 0,
      }),
      _ => None,
    }
  }

  fn is_exit_option(&self, index: usize) -> bool {
    match self.step {
      STEP_UNICODE => index == 1,
      STEP_COLOR => index == 2,
      _ => false,
    }
  }

  /// 进入下一个检测步骤。
  pub fn advance_step(&mut self) {
    self.step += 1;
    self.apply_detection();
  }

  /// 将当前步骤的检测结果持久化到终端配置文件。
  pub fn persist_current_step(&self, storage: &mut StorageService) {
    match self.step {
      STEP_UNICODE => {
        let _ = storage.update_terminal_profile(|p| {
          p.unicode = Some(self.selected_index == 0);
        });
      }
      STEP_COLOR => {
        let color = if self.selected_index == 0 {
          "truecolor"
        } else {
          "256"
        };
        let _ = storage.update_terminal_profile(|p| {
          p.color = Some(color.to_string());
        });
      }
      _ => {}
    }
  }

  fn build_key_params(&self) -> RichTextParams {
    RichTextParams::from_action_map(&Self::action_map(), "terminal.")
  }

  fn _step_title(&self) -> &str {
    match self.step {
      STEP_UNICODE => "Unicode Support",
      STEP_COLOR => "Color Support",
      STEP_MOUSE => "Mouse Support",
      _ => "Done",
    }
  }

  fn hit_test(positions: &TerminalCheckLayout, x: u16, y: u16) -> Option<usize> {
    positions
      .option_rects
      .iter()
      .position(|rect| rect.contains(x, y))
  }
}

fn centered_x(layout: &LayoutService, text: &str) -> u16 {
  let w = layout.get_text_width(text, None);
  layout.resolve_host_x(LayoutService::ALIGN_CENTER, w, 0)
}

fn rainbow_at(t: f32) -> (u8, u8, u8) {
  let t = t.clamp(0.0, 1.0);
  let segments = (RAINBOW.len() - 1) as f32;
  let pos = t * segments;
  let i = (pos as usize).min(RAINBOW.len() - 2);
  let frac = pos - i as f32;
  let (r1, g1, b1) = RAINBOW[i];
  let (r2, g2, b2) = RAINBOW[i + 1];
  (lerp(r1, r2, frac), lerp(g1, g2, frac), lerp(b1, b2, frac))
}

fn lerp(a: u8, b: u8, t: f32) -> u8 {
  (a as f32 + (b as f32 - a as f32) * t).round() as u8
}
