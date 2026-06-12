use std::time::Duration;

use crate::host_engine::services::{
  ActionMapEntry, CanvasService, DetectionResult, DrawImageParams, DrawTextParams,
  I18nService, ImageFit, ImageProtocol, InputActionEvent, KeyState, LayoutService,
  MouseButton, MouseEvent, MouseEventKind, Rect, RenderService, RichTextParams, TextColor,
  TextStyle,
};

/// 检测步骤
const STEP_UNICODE: usize = 0;
const STEP_COLOR: usize = 1;
const STEP_IMAGE: usize = 2;
const STEP_MOUSE: usize = 3;


/// Unicode 检测的选项数
const UNICODE_OPTIONS: usize = 2;
/// 色彩检测的选项数
const COLOR_OPTIONS: usize = 3;
/// 图片协议检测的选项数
const IMAGE_OPTIONS: usize = 4;

/// Unicode 示例文本
const UNICODE_SAMPLE: &str = "你好 World \u{1F30D} ȧb عربى";

/// 色带占据终端宽度的比例范围
const BAND_LEFT_PCT: u16 = 20;
const BAND_RIGHT_PCT: u16 = 80;
/// 色带行数
const BAND_ROWS: u16 = 3;

/// 彩虹色带关键色 (红→橙→黄→绿→青→蓝→紫)
const RAINBOW: &[(u8, u8, u8)] = &[
  (255, 0, 0),
  (255, 165, 0),
  (255, 255, 0),
  (0, 255, 0),
  (0, 255, 255),
  (0, 0, 255),
  (128, 0, 255),
];

/// 布局计算结果
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

/// 图片步骤预留的占位高度（字符格）
const IMG_PLACEHOLDER_H: u16 = 8;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TerminalCheckUi {
  step: usize,
  /// 每个步骤内部的选择索引
  selected_index: usize,
  /// 自动检测结果
  detection: DetectionResult,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TerminalCheckCommand {
  /// 进入下一步
  Next,
  /// 退出程序（Unicode / Color 失败）
  Exit,
  /// 全部完成
  Done,
}

impl TerminalCheckUi {
  pub fn init(detection: &DetectionResult) -> Self {
    Self {
      step: STEP_UNICODE,
      selected_index: 0,
      detection: detection.clone(),
    }
  }

  /// 根据检测结果设置当前步骤的预选项。
  fn apply_detection(&mut self) {
    self.selected_index = match self.step {
      STEP_UNICODE => 0, // 总是"支持"
      STEP_COLOR => 0, // 预选"支持"（truecolor 不在自动检测范围）
      STEP_IMAGE => match self.detection.image_protocol {
        ImageProtocol::Kitty => 0,
        ImageProtocol::Sixel => 1,
        ImageProtocol::ITerm2 => 2,
        ImageProtocol::None => 3,
      },
      _ => 0,
    };
  }

  // ── 输入绑定 ──

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

  // ── 输入处理 ──

  pub fn handle_event(&mut self, event: &InputActionEvent) -> Option<TerminalCheckCommand> {
    if event.state != KeyState::Pressed {
      return None;
    }

    match event.action.as_str() {
      "terminal.focus_up" => {
        if self.selected_index > 0 {
          self.selected_index -= 1;
        }
        None
      }
      "terminal.focus_down" => {
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

  /// 鼠标事件：hover 聚焦、左键确认。
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

  // ── 渲染 ──

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
      STEP_IMAGE => self.render_image_step(render, canvas, layout, i18n),
      _ => self.render_placeholder(render, canvas, layout),
    }
  }

  pub fn compute_positions(
    &self,
    layout: &LayoutService,
    i18n: &I18nService,
  ) -> TerminalCheckLayout {
    match self.step {
      STEP_UNICODE => self.compute_unicode_positions(layout, i18n),
      STEP_COLOR => self.compute_color_positions(layout, i18n),
      STEP_IMAGE => self.compute_image_positions(layout, i18n),
      _ => self.compute_placeholder_positions(layout),
    }
  }

  // ── Unicode 步骤 ──

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

    let term_h = layout.get_terminal_size().height;

    let title_y: u16 = 1;
    let hint_y = term_h.saturating_sub(1);

    let title_x = centered_x(layout, &title);
    // hint 的 {key:} 需要用 params 展开后量宽度，否则 x 位置偏左
    let hint_w = layout.get_text_width(&hint, Some(&key_params));
    let hint_x = layout.resolve_x(LayoutService::ALIGN_CENTER, hint_w, 0);

    let option_names: [&str; UNICODE_OPTIONS] = [&yes_text, &no_text];
    let option_texts: Vec<String> = (0..UNICODE_OPTIONS)
      .map(|i| self.option_display_name(&option_names, i))
      .collect();
    let option_xs: Vec<u16> = option_texts.iter().map(|t| centered_x(layout, t)).collect();
    let options_height = UNICODE_OPTIONS as u16;

    // 选项在 title 和 hint 之间垂直居中
    let available = hint_y.saturating_sub(title_y).saturating_sub(1);
    let option_start_y = if available > options_height {
      title_y
        .saturating_add(1)
        .saturating_add((available - options_height) / 2)
    } else {
      title_y.saturating_add(1)
    };

    // tip / sample 相对选项向上 1 行和 2 行
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

    // title
    render.draw_text(
      canvas,
      &DrawTextParams {
        x: positions.title_x,
        y: positions.title_y,
        text: format!("f%<fg:bright_magenta><b>{}</b></fg>", title),
        ..Default::default()
      },
    );

    // tip
    render.draw_text(
      canvas,
      &DrawTextParams {
        x: positions.tip_x,
        y: positions.tip_y,
        text: format!("f%<fg:bright_yellow>{}</fg>", tip),
        ..Default::default()
      },
    );

    // sample
    render.draw_text(
      canvas,
      &DrawTextParams {
        x: positions.sample_x,
        y: positions.sample_y,
        text: UNICODE_SAMPLE.to_string(),
        ..Default::default()
      },
    );

    // options
    let option_names: [&str; UNICODE_OPTIONS] = [&yes_text, &no_text];
    let option_texts: Vec<String> = (0..UNICODE_OPTIONS)
      .map(|i| self.option_display_name(&option_names, i))
      .collect();
    for i in 0..UNICODE_OPTIONS {
      let text = if i == self.selected_index {
        format!("f%<fg:bright_cyan>{}</fg>", option_texts[i])
      } else {
        option_texts[i].clone()
      };
      render.draw_text(
        canvas,
        &DrawTextParams {
          x: positions.option_xs[i],
          y: positions.option_rects[i].y,
          text,
          ..Default::default()
        },
      );
    }

    // hint
    render.draw_text(
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

  // ── 色彩步骤 ──

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

    let term_h = layout.get_terminal_size().height;

    let title_y: u16 = 1;
    let hint_y = term_h.saturating_sub(1);

    let title_x = centered_x(layout, &title);
    let hint_w = layout.get_text_width(&hint, Some(&key_params));
    let hint_x = layout.resolve_x(LayoutService::ALIGN_CENTER, hint_w, 0);

    // 选项
    let option_names: [&str; COLOR_OPTIONS] = [&yes_text, &no256_text, &no_other_text];
    let option_texts: Vec<String> = (0..COLOR_OPTIONS)
      .map(|i| self.option_display_name(&option_names, i))
      .collect();
    let option_xs: Vec<u16> = option_texts.iter().map(|t| centered_x(layout, t)).collect();
    let options_height = COLOR_OPTIONS as u16;

    // tip 行
    let tip = i18n.get_runtime_text("terminal", "terminal.truecolor.tip");
    let tip_x = centered_x(layout, &tip);

    // 色带区域：tip + 空行 + BAND_ROWS + 空行 = 1 + 1 + 3 + 1 = 6 行
    let band_block_height: u16 = 1 + 1 + BAND_ROWS + 1;

    // 色带块 + 选项整体在 title 和 hint 之间垂直居中
    let total_content = band_block_height
      .saturating_add(1)
      .saturating_add(options_height); // +1 gap
    let available = hint_y.saturating_sub(title_y).saturating_sub(1);
    let content_start_y = if available > total_content {
      title_y
        .saturating_add(1)
        .saturating_add((available - total_content) / 2)
    } else {
      title_y.saturating_add(1)
    };

    let tip_y = content_start_y;
    let band_y = tip_y.saturating_add(2); // tip + 空行
    let option_start_y = band_y.saturating_add(BAND_ROWS).saturating_add(1); // band + 空行

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

    // title
    render.draw_text(
      canvas,
      &DrawTextParams {
        x: positions.title_x,
        y: positions.title_y,
        text: format!("f%<fg:bright_magenta><b>{}</b></fg>", title),
        ..Default::default()
      },
    );

    // tip — 黄色
    render.draw_text(
      canvas,
      &DrawTextParams {
        x: positions.tip_x,
        y: positions.tip_y,
        text: format!("f%<fg:bright_yellow>{}</fg>", tip),
        ..Default::default()
      },
    );

    // 色带
    let term_w = layout.get_terminal_size().width;
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
        canvas.styled_text(
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

    // 选项
    let option_names: [&str; COLOR_OPTIONS] = [&yes_text, &no256_text, &no_other_text];
    let option_texts: Vec<String> = (0..COLOR_OPTIONS)
      .map(|i| self.option_display_name(&option_names, i))
      .collect();
    for i in 0..COLOR_OPTIONS {
      let text = if i == self.selected_index {
        format!("f%<fg:bright_cyan>{}</fg>", option_texts[i])
      } else {
        option_texts[i].clone()
      };
      render.draw_text(
        canvas,
        &DrawTextParams {
          x: positions.option_xs[i],
          y: positions.option_rects[i].y,
          text,
          ..Default::default()
        },
      );
    }

    // hint
    render.draw_text(
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

  // ── 图片步骤 ──

  /// 返回当前图片步骤的绘图请求（供 runtime 提交给 ImageService）。
  /// 非图片步骤返回 `None`。
  pub fn image_request(
    &self,
    layout: &LayoutService,
    i18n: &I18nService,
    path: std::path::PathBuf,
  ) -> Option<DrawImageParams> {
    if self.step != STEP_IMAGE {
      return None;
    }
    let positions = self.compute_image_positions(layout, i18n);
    let term_w = layout.get_terminal_size().width;
    let img_w = Self::IMG_WIDTH;
    let img_x = term_w.saturating_sub(img_w) / 2;
    let img_y = positions.tip_y.saturating_add(2);

    // 使用 Exact 确保图片尺寸与 UI 占位高度一致，避免偏移
    Some(DrawImageParams {
      x: img_x,
      y: img_y,
      path,
      fit: ImageFit::Exact {
        width: img_w,
        height: IMG_PLACEHOLDER_H,
      },
      preserve_aspect_ratio: false,
    })
  }

  /// 获取当前图片步骤选中的协议。
  pub fn selected_image_protocol(&self) -> ImageProtocol {
    match self.selected_index {
      0 => ImageProtocol::Kitty,
      1 => ImageProtocol::Sixel,
      2 => ImageProtocol::ITerm2,
      _ => ImageProtocol::None,
    }
  }

  const IMG_WIDTH: u16 = 20;

  fn compute_image_positions(
    &self,
    layout: &LayoutService,
    i18n: &I18nService,
  ) -> TerminalCheckLayout {
    let title = i18n.get_runtime_text("terminal", "terminal.image.title");
    let tip = i18n.get_runtime_text("terminal", "terminal.image.tip");
    let kitty_name = i18n.get_runtime_text("terminal", "terminal.image.kitty");
    let sixel_name = i18n.get_runtime_text("terminal", "terminal.image.sixel");
    let iterm_name = i18n.get_runtime_text("terminal", "terminal.image.iterm");
    let none_name = i18n.get_runtime_text("terminal", "terminal.image.none");

    let key_params = self.build_key_params();
    let hint = format!(
      "{}  {}  {}",
      i18n.get_runtime_text("terminal", "terminal.action.select"),
      i18n.get_runtime_text("terminal", "terminal.action.confirm"),
      i18n.get_runtime_text("terminal", "terminal.action.exit"),
    );

    let term_h = layout.get_terminal_size().height;

    let title_y: u16 = 1;
    let hint_y = term_h.saturating_sub(1);

    let title_x = centered_x(layout, &title);
    let hint_w = layout.get_text_width(&hint, Some(&key_params));
    let hint_x = layout.resolve_x(LayoutService::ALIGN_CENTER, hint_w, 0);

    let tip_x = centered_x(layout, &tip);

    // 图片占位区域高度（字符格），无实际图片渲染
    let img_h = IMG_PLACEHOLDER_H;

    // 选项（直接用语言文件中的简短名称）
    let option_names: [&str; IMAGE_OPTIONS] = [&kitty_name, &sixel_name, &iterm_name, &none_name];
    let option_texts: Vec<String> = (0..IMAGE_OPTIONS)
      .map(|i| self.option_display_name(&option_names, i))
      .collect();
    let option_xs: Vec<u16> = option_texts.iter().map(|t| centered_x(layout, t)).collect();
    let options_height = IMAGE_OPTIONS as u16;

    // 内容块：tip(1) + 空行(1) + 占位(img_h) + 空行(1) + 选项(options_height)
    let content_height = 1 + 1 + img_h + 1 + options_height;
    let available = hint_y.saturating_sub(title_y).saturating_sub(1);
    let content_start_y = if available > content_height {
      title_y
        .saturating_add(1)
        .saturating_add((available - content_height) / 2)
    } else {
      title_y.saturating_add(1)
    };

    let tip_y = content_start_y;
    let img_y = tip_y.saturating_add(2); // tip + 空行
    let option_start_y = img_y.saturating_add(img_h).saturating_add(1); // 占位 + 空行

    let option_widths: Vec<u16> = option_texts
      .iter()
      .map(|t| layout.get_text_width(t, None))
      .collect();
    let option_rects: Vec<Rect> = (0..IMAGE_OPTIONS)
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

  fn render_image_step(
    &self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    i18n: &I18nService,
  ) {
    let positions = self.compute_image_positions(layout, i18n);
    let key_params = self.build_key_params();

    let title = i18n.get_runtime_text("terminal", "terminal.image.title");
    let tip = i18n.get_runtime_text("terminal", "terminal.image.tip");
    let kitty_name = i18n.get_runtime_text("terminal", "terminal.image.kitty");
    let sixel_name = i18n.get_runtime_text("terminal", "terminal.image.sixel");
    let iterm_name = i18n.get_runtime_text("terminal", "terminal.image.iterm");
    let none_name = i18n.get_runtime_text("terminal", "terminal.image.none");
    let hint = format!(
      "{}  {}  {}",
      i18n.get_runtime_text("terminal", "terminal.action.select"),
      i18n.get_runtime_text("terminal", "terminal.action.confirm"),
      i18n.get_runtime_text("terminal", "terminal.action.exit"),
    );

    // title
    render.draw_text(
      canvas,
      &DrawTextParams {
        x: positions.title_x,
        y: positions.title_y,
        text: format!("f%<fg:bright_magenta><b>{}</b></fg>", title),
        ..Default::default()
      },
    );

    // tip — 黄色
    render.draw_text(
      canvas,
      &DrawTextParams {
        x: positions.tip_x,
        y: positions.tip_y,
        text: format!("f%<fg:yellow>{}</fg>", tip),
        ..Default::default()
      },
    );

    // 图片区域留白（由 runtime 在 present 后输出图片），不绘制边框

    // 选项
    let option_names: [&str; IMAGE_OPTIONS] = [&kitty_name, &sixel_name, &iterm_name, &none_name];
    let option_texts: Vec<String> = (0..IMAGE_OPTIONS)
      .map(|i| self.option_display_name(&option_names, i))
      .collect();
    for i in 0..IMAGE_OPTIONS {
      let text = if i == self.selected_index {
        format!("f%<fg:bright_cyan>{}</fg>", option_texts[i])
      } else {
        option_texts[i].clone()
      };
      render.draw_text(
        canvas,
        &DrawTextParams {
          x: positions.option_xs[i],
          y: positions.option_rects[i].y,
          text,
          ..Default::default()
        },
      );
    }

    // hint
    render.draw_text(
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

  // ── 占位 ──

  fn compute_placeholder_positions(&self, layout: &LayoutService) -> TerminalCheckLayout {
    let title = "---";
    let title_w = layout.get_text_width(title, None);
    let title_x = layout.resolve_x(LayoutService::ALIGN_CENTER, title_w, 0);
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
      hint_y: layout.get_terminal_size().height.saturating_sub(1),

    }
  }

  fn render_placeholder(
    &self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout_svc: &LayoutService,
  ) {
    let positions = self.compute_placeholder_positions(layout_svc);
    let title = format!("{} ({}/{})", self._step_title(), self.step + 1, 4);
    let title_w = layout_svc.get_text_width(&title, None);
    let title_x = layout_svc.resolve_x(LayoutService::ALIGN_CENTER, title_w, 0);

    render.draw_text(
      canvas,
      &DrawTextParams {
        x: title_x,
        y: positions.title_y,
        text: format!("f%<b>{}</b>", title),
        ..Default::default()
      },
    );

    render.draw_text(
      canvas,
      &DrawTextParams {
        x: positions.hint_x,
        y: positions.hint_y,
        text: "f%<fg:bright_black>[↑↓] Select  [Enter] Confirm</fg>".to_string(),
        ..Default::default()
      },
    );
  }

  // ── 内部辅助 ──

  fn option_count(&self) -> usize {
    match self.step {
      STEP_UNICODE => UNICODE_OPTIONS,
      STEP_COLOR => COLOR_OPTIONS,
      STEP_IMAGE => IMAGE_OPTIONS,
      _ => 0,
    }
  }

  /// 选项显示名（选中带箭头，非选中等宽占位）。
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
          // "支持" → 通过
          Some(TerminalCheckCommand::Next)
        } else {
          // "不支持" → 退出
          Some(TerminalCheckCommand::Exit)
        }
      }
      STEP_COLOR => match self.selected_index {
        0 => Some(TerminalCheckCommand::Next),
        1 => Some(TerminalCheckCommand::Next),
        2 => Some(TerminalCheckCommand::Exit),
        _ => None,
      },
      STEP_IMAGE => Some(TerminalCheckCommand::Next), // 不强制，选啥都行
      STEP_MOUSE => Some(TerminalCheckCommand::Done),
      _ => None,
    }
  }

  /// 应用 Next 命令时步进到下一步，自动应用检测结果预选项。
  pub fn advance_step(&mut self) {
    self.step += 1;
    self.apply_detection();
  }

  fn build_key_params(&self) -> RichTextParams {
    RichTextParams::from_action_map(&Self::action_map(), "terminal.")
  }

  fn _step_title(&self) -> &str {
    match self.step {
      STEP_UNICODE => "Unicode Support",
      STEP_COLOR => "Color Support",
      STEP_IMAGE => "Image Protocol",
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

/// 便捷函数：计算文本水平居中的 x 起始坐标。
fn centered_x(layout: &LayoutService, text: &str) -> u16 {
  let w = layout.get_text_width(text, None);
  layout.resolve_x(LayoutService::ALIGN_CENTER, w, 0)
}

/// 彩虹色带插值。t ∈ [0, 1]，在红橙黄绿青蓝紫之间平滑过渡。
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
