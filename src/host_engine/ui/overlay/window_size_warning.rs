use crate::host_engine::services::{
  ActionMapEntry, CanvasService, DrawTextParams, HitAreaEvent, HitAreaId, HitAreaOptions,
  HitAreaService, I18nService, KeyState, LayoutService, MouseButton, Rect, RenderService,
  RichTextParams, RuntimeObjectPool, RuntimeObjectPoolOwner, UiEvent, UiObjectPool,
  UiObjectPoolOwner,
};

/// 窗口尺寸警告 UI：当终端窗口小于最低要求时显示提示信息。
pub struct WindowSizeWarningUi {
  objects: UiObjectPool,
  runtime_objects: RuntimeObjectPool,
  area: HitAreaId,
}

impl WindowSizeWarningUi {
  /// 初始化窗口尺寸警告 UI。
  pub fn init(hit_area: &HitAreaService) -> Self {
    let mut objects = UiObjectPool::new();
    let area = hit_area.create(&mut objects, HitAreaOptions::default());
    Self {
      objects,
      runtime_objects: RuntimeObjectPool::new(),
      area,
    }
  }

  /// 返回警告页面的按键映射定义。
  pub fn action_map() -> Vec<ActionMapEntry> {
    vec![ActionMapEntry {
      action: "window_size.exit".to_string(),
      description: "Exit / Back".to_string(),
      keys: vec![vec!["esc".to_string()]],
    }]
  }

  /// 处理 UI 事件。
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

  /// 渲染窗口尺寸警告信息到宿主层。
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
    is_screensaver: bool,
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
      is_screensaver,
    );
    hit_area.render_host(
      &mut self.objects,
      self.area,
      Rect {
        x: 0,
        y: 0,
        width: current_width,
        height: current_height,
      },
      canvas,
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

impl RuntimeObjectPoolOwner for WindowSizeWarningUi {
  fn runtime_objects(&self) -> &RuntimeObjectPool {
    &self.runtime_objects
  }

  fn runtime_objects_mut(&mut self) -> &mut RuntimeObjectPool {
    &mut self.runtime_objects
  }
}

/// 窗口尺寸警告页面的布局信息。
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

/// 窗口尺寸警告页面的命令。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowSizeWarningCommand {
  Exit,
}

/// 计算窗口尺寸警告页面的布局信息。
pub fn compute_positions(
  layout: &LayoutService,
  i18n: &I18nService,
  required_width: u32,
  required_height: u32,
  current_width: u16,
  current_height: u16,
  is_host_mode: bool,
  is_screensaver: bool,
) -> WindowSizeWarningLayout {
  let title = i18n.get_runtime_text("window_size", "window_size.title");
  let tip = i18n.get_runtime_text("window_size", "window_size.tip");
  let required_prefix = i18n.get_runtime_text("window_size", "window_size.required");
  let current_prefix = i18n.get_runtime_text("window_size", "window_size.current");

  let required_line = format!("{} {}×{}", required_prefix, required_width, required_height);
  let current_line = format!("{} {}×{}", current_prefix, current_width, current_height);

  let term_h = layout.physical_size().height;

  let title_y: u16 = 1;
  let hint_y = term_h.saturating_sub(1);

  let title_x = centered_x(layout, &title);
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
  let key_params = build_key_params();
  let hint_key = if is_screensaver {
    "window_size.action.exit.screensaver"
  } else if is_host_mode {
    "window_size.action.exit.host"
  } else {
    "window_size.action.exit.game"
  };
  let hint = i18n.get_runtime_text("window_size", hint_key);
  let hint_w = layout.get_text_width(&hint, Some(&key_params));
  let hint_x = layout.resolve_host_x(LayoutService::ALIGN_CENTER, hint_w, 0);

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
  is_screensaver: bool,
) {
  let positions = compute_positions(
    layout,
    i18n,
    required_width,
    required_height,
    current_width,
    current_height,
    is_host_mode,
    is_screensaver,
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
  render.draw_host_text(
    canvas,
    &DrawTextParams {
      x: positions.title_x,
      y: positions.title_y,
      text: format!("f%<fg:bright_yellow><b>{}</b></fg>", title),
      ..Default::default()
    },
  );
  render.draw_host_text(
    canvas,
    &DrawTextParams {
      x: positions.tip_x,
      y: positions.tip_y,
      text: tip,
      ..Default::default()
    },
  );
  render.draw_host_text(
    canvas,
    &DrawTextParams {
      x: positions.required_x,
      y: positions.required_y,
      text: format!("f%<fg:bright_yellow>{}</fg>", required_line),
      ..Default::default()
    },
  );
  render.draw_host_text(
    canvas,
    &DrawTextParams {
      x: positions.current_x,
      y: positions.current_y,
      text: format!("f%<fg:bright_red>{}</fg>", current_line),
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

fn build_key_params() -> RichTextParams {
  RichTextParams::from_action_map(&WindowSizeWarningUi::action_map(), "window_size.")
}

fn centered_x(layout: &LayoutService, text: &str) -> u16 {
  let w = layout.get_text_width(text, None);
  layout.resolve_host_x(LayoutService::ALIGN_CENTER, w, 0)
}
