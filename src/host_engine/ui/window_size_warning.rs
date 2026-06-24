use crate::host_engine::services::{
  ActionMapEntry, CanvasService, DrawTextParams, HitAreaEvent, HitAreaId, HitAreaService,
  I18nService, KeyState, LayoutService, MouseButton, Rect, RenderService, RichTextParams, UiEvent,
  UiObjectPool, UiObjectPoolOwner,
};

pub struct WindowSizeWarningUi {
  objects: UiObjectPool,
  area: HitAreaId,
}

impl WindowSizeWarningUi {
  pub fn init(hit_area: &HitAreaService) -> Self {
    let mut objects = UiObjectPool::new();
    let area = hit_area.create(&mut objects);
    Self { objects, area }
  }

  pub fn action_map() -> Vec<ActionMapEntry> {
    vec![ActionMapEntry {
      action: "window_size.exit".to_string(),
      description: "Exit / Back".to_string(),
      keys: vec![vec!["esc".to_string()]],
    }]
  }

  pub fn handle_event(&mut self, event: &UiEvent) -> Option<WindowSizeWarningCommand> {
    match event {
      UiEvent::Action(event)
        if event.state == KeyState::Pressed && event.action == "window_size.exit" =>
      {
        Some(WindowSizeWarningCommand::Exit)
      }
      UiEvent::HitArea(HitAreaEvent::Press {
        button: MouseButton::Right,
        ..
      }) => Some(WindowSizeWarningCommand::Exit),
      _ => None,
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
    required_width: u32,
    required_height: u32,
    current_width: u16,
    current_height: u16,
    is_host_mode: bool,
  ) {
    draw_content(
      render,
      canvas,
      layout,
      i18n,
      required_width,
      required_height,
      current_width,
      current_height,
      is_host_mode,
    );
    hit_area.render(
      &mut self.objects,
      self.area,
      Rect {
        x: 0,
        y: 0,
        width: current_width,
        height: current_height,
      },
    );
  }
}

impl UiObjectPoolOwner for WindowSizeWarningUi {
  fn objects(&self) -> &UiObjectPool {
    &self.objects
  }

  fn objects_mut(&mut self) -> &mut UiObjectPool {
    &mut self.objects
  }
}

// ── 布局 ──

pub(crate) struct WindowSizeWarningLayout {
  pub title_x: u16,
  pub title_y: u16,
  pub tip_x: u16,
  pub tip_y: u16,
  pub required_x: u16,
  pub required_y: u16,
  pub current_x: u16,
  pub current_y: u16,
  pub hint_x: u16,
  pub hint_y: u16,
}

// ── 命令 ──

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowSizeWarningCommand {
  Exit,
}

// ── 布局计算 ──

/// 计算所有文本的像素位置。无副作用。
pub fn compute_positions(
  layout: &LayoutService,
  i18n: &I18nService,
  required_width: u32,
  required_height: u32,
  current_width: u16,
  current_height: u16,
) -> WindowSizeWarningLayout {
  let title = i18n.get_runtime_text("window_size", "window_size.title");
  let tip = i18n.get_runtime_text("window_size", "window_size.tip");
  let required_prefix = i18n.get_runtime_text("window_size", "window_size.required");
  let current_prefix = i18n.get_runtime_text("window_size", "window_size.current");

  let required_line = format!("{} {}×{}", required_prefix, required_width, required_height);
  let current_line = format!("{} {}×{}", current_prefix, current_width, current_height);

  let term_h = layout.get_terminal_size().height;

  let title_y: u16 = 1;
  let hint_y = term_h.saturating_sub(1);

  let title_x = centered_x(layout, &title);

  // 内容块：tip + required + current，共 3 行，在 title 和 hint 之间垂直居中
  let content_lines: u16 = 3;
  let available = hint_y.saturating_sub(title_y).saturating_sub(1);
  let content_start_y = if available > content_lines {
    title_y
      .saturating_add(1)
      .saturating_add((available - content_lines) / 2)
  } else {
    title_y.saturating_add(1)
  };

  let tip_x = centered_x(layout, &tip);
  let tip_y = content_start_y;
  let required_x = centered_x(layout, &required_line);
  let required_y = content_start_y + 1;
  let current_x = centered_x(layout, &current_line);
  let current_y = content_start_y + 2;

  // hint 需要用 params 展开 {key:} 后量宽度
  let key_params = build_key_params();
  let hint_host = i18n.get_runtime_text("window_size", "window_size.action.exit.host");
  let hint_game = i18n.get_runtime_text("window_size", "window_size.action.exit.game");
  // 取两者中较宽的用于布局（确保切换语言时不偏移）
  let hint_w_host = layout.get_text_width(&hint_host, Some(&key_params));
  let hint_w_game = layout.get_text_width(&hint_game, Some(&key_params));
  let hint_w = hint_w_host.max(hint_w_game);

  let hint_x = layout.resolve_x(LayoutService::ALIGN_CENTER, hint_w, 0);

  WindowSizeWarningLayout {
    title_x,
    title_y,
    tip_x,
    tip_y,
    required_x,
    required_y,
    current_x,
    current_y,
    hint_x,
    hint_y,
  }
}

// ── 渲染 ──

/// 绘制窗口尺寸警告界面。
#[allow(clippy::too_many_arguments)]
fn draw_content(
  render: &mut RenderService,
  canvas: &mut CanvasService,
  layout: &LayoutService,
  i18n: &I18nService,
  required_width: u32,
  required_height: u32,
  current_width: u16,
  current_height: u16,
  is_host_mode: bool,
) {
  let positions = compute_positions(
    layout,
    i18n,
    required_width,
    required_height,
    current_width,
    current_height,
  );
  let key_params = build_key_params();

  let title = i18n.get_runtime_text("window_size", "window_size.title");
  let tip = i18n.get_runtime_text("window_size", "window_size.tip");
  let required_prefix = i18n.get_runtime_text("window_size", "window_size.required");
  let current_prefix = i18n.get_runtime_text("window_size", "window_size.current");
  let hint_key = if is_host_mode {
    "window_size.action.exit.host"
  } else {
    "window_size.action.exit.game"
  };
  let hint = i18n.get_runtime_text("window_size", hint_key);

  let required_line = format!("{}{}×{}", required_prefix, required_width, required_height);
  let current_line = format!("{}{}×{}", current_prefix, current_width, current_height);

  // title — bright_magenta + bold
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
      text: tip,
      ..Default::default()
    },
  );

  // required — yellow
  render.draw_text(
    canvas,
    &DrawTextParams {
      x: positions.required_x,
      y: positions.required_y,
      text: format!("f%<fg:bright_yellow>{}</fg>", required_line),
      ..Default::default()
    },
  );

  // current — red
  render.draw_text(
    canvas,
    &DrawTextParams {
      x: positions.current_x,
      y: positions.current_y,
      text: format!("f%<fg:bright_red>{}</fg>", current_line),
      ..Default::default()
    },
  );

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

// ── 内部辅助 ──

fn build_key_params() -> RichTextParams {
  RichTextParams::from_action_map(&WindowSizeWarningUi::action_map(), "window_size.")
}

fn centered_x(layout: &LayoutService, text: &str) -> u16 {
  let w = layout.get_text_width(text, None);
  layout.resolve_x(LayoutService::ALIGN_CENTER, w, 0)
}
