use crate::host_engine::services::{
  ActionMapEntry, CanvasService, HitAreaEvent, HitAreaId, HitAreaOptions, HitAreaService,
  InputService, KeyState, LayoutService, MouseButton, Rect, RenderService, TerminalColor,
  TextColor, TextStyle, UiEvent, UiObjectPool, UiObjectPoolOwner,
};

// ── 拖拽窗口常量 ──

const WIN_MIN_WIDTH: u16 = 24;
const WIN_MIN_HEIGHT: u16 = 7;
const WIN_DEFAULT_WIDTH: u16 = 46;
const WIN_DEFAULT_HEIGHT: u16 = 14;

// ── 窗口子区域枚举 ──

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WindowZone {
  TitleBar,
  CloseButton,
  ResizeHandle,
  Body,
}

pub struct InputDemoUi {
  objects: UiObjectPool,
  /// 窗口位置（左上角）
  window_x: u16,
  window_y: u16,
  /// 窗口尺寸
  window_width: u16,
  window_height: u16,
  /// 窗口子区域 → HitAreaId
  areas: [HitAreaId; 4],
  /// 拖拽状态（用于在 drag 和 click 之间区分）
  drag_active: bool,
  /// 原始输入事件（保留原有 demo 功能）
  raw_events: Vec<String>,
  raw_event_count: usize,
  action_event_count: usize,
  hit_event_count: usize,
  click_count: usize,
  last_action: String,
  last_hit_event: String,
}

impl UiObjectPoolOwner for InputDemoUi {
  fn objects(&self) -> &UiObjectPool {
    &self.objects
  }

  fn objects_mut(&mut self) -> &mut UiObjectPool {
    &mut self.objects
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputDemoCommand {
  ToggleCapture,
  Back,
}

impl InputDemoUi {
  pub fn init(hit_area: &HitAreaService) -> Self {
    let mut objects = UiObjectPool::new();

    // title bar: drag 用于移动窗口
    let title_bar = hit_area.create(
      &mut objects,
      HitAreaOptions {
        hover_move: false,
        drag: true,
      },
    );
    // close button: 点击关闭/重置窗口
    let close_btn = hit_area.create(&mut objects, HitAreaOptions::default());
    // resize handle: drag 用于调整窗口大小
    let resize_handle = hit_area.create(
      &mut objects,
      HitAreaOptions {
        hover_move: false,
        drag: true,
      },
    );
    // body: 用于报告统计
    let body = hit_area.create(
      &mut objects,
      HitAreaOptions {
        hover_move: true,
        drag: false,
      },
    );

    Self {
      objects,
      window_x: 0,
      window_y: 0,
      window_width: WIN_DEFAULT_WIDTH,
      window_height: WIN_DEFAULT_HEIGHT,
      areas: [title_bar, close_btn, resize_handle, body],
      drag_active: false,
      raw_events: Vec::new(),
      raw_event_count: 0,
      action_event_count: 0,
      hit_event_count: 0,
      click_count: 0,
      last_action: "None".to_string(),
      last_hit_event: "None".to_string(),
    }
  }

  pub fn action_map() -> Vec<ActionMapEntry> {
    vec![
      ActionMapEntry {
        action: "input_demo.capture".to_string(),
        description: "Toggle raw rdev key capture".to_string(),
        keys: vec![vec!["enter".to_string()]],
      },
      ActionMapEntry {
        action: "input_demo.reset".to_string(),
        description: "Reset window position".to_string(),
        keys: vec![vec!["r".to_string()]],
      },
      ActionMapEntry {
        action: "input_demo.back".to_string(),
        description: "Back to home".to_string(),
        keys: vec![vec!["esc".to_string()]],
      },
    ]
  }

  // ── 事件处理 ──

  pub fn handle_event(&mut self, event: &UiEvent) -> Option<InputDemoCommand> {
    match event {
      UiEvent::Action(event) if event.state == KeyState::Pressed => {
        self.action_event_count += 1;
        self.last_action = event.action.clone();
        match event.action.as_str() {
          "input_demo.capture" => Some(InputDemoCommand::ToggleCapture),
          "input_demo.reset" => {
            self.reset_window();
            None
          }
          "input_demo.back" => Some(InputDemoCommand::Back),
          _ => None,
        }
      }
      UiEvent::HitArea(event) => {
        self.hit_event_count += 1;
        self.last_hit_event = format_hit_event(event);

        let zone = self.zone_of(event);

        match event {
          // ── 标题栏拖拽 → 移动窗口 ──
          HitAreaEvent::Drag { dx, dy, .. } if zone == Some(WindowZone::TitleBar) => {
            self.drag_active = true;
            self.move_window(*dx, *dy);
          }
          // ── 右下角拖拽 → 调整窗口大小 ──
          HitAreaEvent::Drag { dx, dy, .. } if zone == Some(WindowZone::ResizeHandle) => {
            self.drag_active = true;
            self.resize_window(*dx, *dy);
          }
          // ── 关闭按钮点击 → 重置窗口 ──
          HitAreaEvent::Click {
            button: MouseButton::Left,
            ..
          } if zone == Some(WindowZone::CloseButton) => {
            if !self.drag_active {
              self.reset_window();
            }
            self.drag_active = false;
          }
          HitAreaEvent::Release { .. } => {
            self.drag_active = false;
          }
          HitAreaEvent::Click { .. } => {
            self.click_count += 1;
          }
          _ => {}
        }
        None
      }
      _ => None,
    }
  }

  pub fn toggle_capture(&mut self, input: &mut InputService) {
    if input.is_raw_key_capture_enabled() {
      input.disable_raw_key_capture();
    } else {
      input.enable_raw_key_capture();
      self.raw_events.clear();
      self.raw_event_count = 0;
    }
  }

  pub fn leave(&mut self, input: &mut InputService) {
    if input.is_raw_key_capture_enabled() {
      input.disable_raw_key_capture();
    }
  }

  pub fn update(&mut self, input: &mut InputService) {
    for event in input.take_raw_key_events() {
      self.raw_event_count += 1;
      self.raw_events.push(format!(
        "{}  {:?}  {:?}",
        event.display, event.key, event.kind
      ));
    }
    if self.raw_events.len() > 5 {
      self.raw_events.drain(..self.raw_events.len() - 5);
    }
  }

  // ── 渲染 ──

  pub fn render(
    &mut self,
    _render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    input: &InputService,
    hit_area: &HitAreaService,
  ) {
    // 每帧 clamp 到终端范围（首次进入时 x=0,y=0 会自动居中）
    self.clamp_to_terminal(canvas.physical_width(), canvas.physical_height());

    self.draw_window(canvas, layout, hit_area);

    // ── 底部状态栏 ──
    let term_h = canvas.physical_height();
    self.draw_status_bar(canvas, input, term_h);
  }

  // ── 内部: 子区域判断 ──

  fn zone_of(&self, event: &HitAreaEvent) -> Option<WindowZone> {
    let id = match event {
      HitAreaEvent::HoverEnter { id, .. }
      | HitAreaEvent::HoverMove { id, .. }
      | HitAreaEvent::HoverLeave { id, .. }
      | HitAreaEvent::Press { id, .. }
      | HitAreaEvent::Release { id, .. }
      | HitAreaEvent::Click { id, .. }
      | HitAreaEvent::Drag { id, .. } => *id,
    };
    if id == self.areas[0] {
      Some(WindowZone::TitleBar)
    } else if id == self.areas[1] {
      Some(WindowZone::CloseButton)
    } else if id == self.areas[2] {
      Some(WindowZone::ResizeHandle)
    } else if id == self.areas[3] {
      Some(WindowZone::Body)
    } else {
      None
    }
  }

  // ── 内部: 窗口定位 ──

  fn clamp_to_terminal(&mut self, term_w: u16, term_h: u16) {
    // 首次居中
    if self.window_x == 0 && self.window_y == 0 {
      self.window_x = term_w.saturating_sub(self.window_width) / 2;
      self.window_y = term_h.saturating_sub(self.window_height) / 2;
    }
    // 尺寸限制
    self.window_width = self.window_width.clamp(WIN_MIN_WIDTH, term_w);
    self.window_height = self.window_height.clamp(WIN_MIN_HEIGHT, term_h);
    // 位置 clamp：至少留一行标题栏可见
    self.window_x = self.window_x.min(term_w.saturating_sub(WIN_MIN_WIDTH / 2));
    self.window_y = self.window_y.min(term_h.saturating_sub(WIN_MIN_HEIGHT / 2));
  }

  fn move_window(&mut self, dx: i32, dy: i32) {
    self.window_x = add_delta(self.window_x, dx);
    self.window_y = add_delta(self.window_y, dy);
  }

  fn resize_window(&mut self, dx: i32, dy: i32) {
    self.window_width = add_delta_dim(self.window_width, dx, WIN_MIN_WIDTH, 120);
    self.window_height = add_delta_dim(self.window_height, dy, WIN_MIN_HEIGHT, 60);
  }

  fn reset_window(&mut self) {
    self.window_x = 0;
    self.window_y = 0;
    self.window_width = WIN_DEFAULT_WIDTH;
    self.window_height = WIN_DEFAULT_HEIGHT;
    // 下一帧 clamp_to_terminal 会重新居中
  }

  // ── 内部: 绘制 ──

  fn draw_window(
    &mut self,
    canvas: &mut CanvasService,
    _layout: &LayoutService,
    hit_area: &HitAreaService,
  ) {
    let wx = self.window_x;
    let wy = self.window_y;
    let ww = self.window_width;
    let wh = self.window_height;

    let fg = TextColor::Terminal(TerminalColor::BrightWhite);
    let bg = TextColor::Rgb {
      r: 30,
      g: 30,
      b: 40,
    };
    let title_bg = TextColor::Rgb {
      r: 50,
      g: 50,
      b: 120,
    };
    let style = |fg: TextColor, bg: TextColor| TextStyle {
      foreground: Some(fg),
      background: Some(bg),
      ..Default::default()
    };

    // ── 填充背景 ──
    for row in 0..wh {
      canvas.host_styled_text(
        wx,
        wy + row,
        &" ".repeat(ww as usize),
        style(fg.clone(), bg.clone()),
      );
    }

    // ── 标题栏背景 ──
    canvas.host_styled_text(
      wx,
      wy,
      &" ".repeat(ww as usize),
      style(fg.clone(), title_bg.clone()),
    );

    // ── 边框 ──
    // 顶边
    canvas.host_styled_text(wx, wy, "┌", style(fg.clone(), title_bg.clone()));
    canvas.host_styled_text(wx + ww - 1, wy, "┐", style(fg.clone(), title_bg.clone()));
    // 底边
    canvas.host_styled_text(wx, wy + wh - 1, "└", style(fg.clone(), bg.clone()));
    canvas.host_styled_text(wx + ww - 1, wy + wh - 1, "┘", style(fg.clone(), bg.clone()));
    // 左右边
    for row in 1..wh - 1 {
      canvas.host_styled_text(wx, wy + row, "│", style(fg.clone(), bg.clone()));
      canvas.host_styled_text(wx + ww - 1, wy + row, "│", style(fg.clone(), bg.clone()));
    }

    // ── 标题文本 ──
    let title = " Draggable Window ";
    canvas.host_styled_text(wx + 2, wy, title, style(fg.clone(), title_bg.clone()));

    // ── 关闭按钮 [X] ──
    let close_x = wx + ww - 4;
    canvas.host_styled_text(
      close_x,
      wy,
      "[X]",
      style(
        TextColor::Terminal(TerminalColor::BrightRed),
        title_bg.clone(),
      ),
    );

    // ── 窗口内容 ──
    if wh >= 5 {
      canvas.host_styled_text(
        wx + 2,
        wy + 1,
        "┌─ Drag title bar to move",
        style(fg.clone(), bg.clone()),
      );
      canvas.host_styled_text(
        wx + 2,
        wy + 2,
        "│  Drag [::] corner to resize",
        style(fg.clone(), bg.clone()),
      );
      canvas.host_styled_text(
        wx + 2,
        wy + 3,
        "│  Click [X] or press R to reset",
        style(fg.clone(), bg.clone()),
      );
      canvas.host_styled_text(
        wx + 2,
        wy + 4,
        "└─ Esc to go back",
        style(fg.clone(), bg.clone()),
      );
    }
    // 窗口尺寸信息
    if wh >= 7 {
      let info = format!(
        "  Window: ({wx},{wy})  {ww}×{wh}",
        wx = self.window_x,
        wy = self.window_y,
        ww = self.window_width,
        wh = self.window_height,
      );
      canvas.host_styled_text(wx + 2, wy + 6, &info, style(fg.clone(), bg.clone()));
    }

    // ── 右下角 resize handle ──
    let handle_x = wx + ww - 4;
    let handle_y = wy + wh - 1;
    canvas.host_styled_text(
      handle_x,
      handle_y,
      "[::]",
      style(TextColor::Terminal(TerminalColor::BrightCyan), bg.clone()),
    );

    // ── 注册 HitArea ──
    // 标题栏（不含关闭按钮的区域）
    hit_area.render_host(
      &mut self.objects,
      self.areas[0],
      Rect {
        x: wx + 1,
        y: wy,
        width: ww.saturating_sub(6), // 留出边框和关闭按钮
        height: 1,
      },
      canvas,
    );
    // 关闭按钮
    hit_area.render_host(
      &mut self.objects,
      self.areas[1],
      Rect {
        x: close_x,
        y: wy,
        width: 3,
        height: 1,
      },
      canvas,
    );
    // resize handle
    hit_area.render_host(
      &mut self.objects,
      self.areas[2],
      Rect {
        x: handle_x,
        y: handle_y,
        width: 4,
        height: 1,
      },
      canvas,
    );
    // body
    if ww > 2 && wh > 2 {
      hit_area.render_host(
        &mut self.objects,
        self.areas[3],
        Rect {
          x: wx + 1,
          y: wy + 1,
          width: ww - 2,
          height: wh - 2,
        },
        canvas,
      );
    }
  }

  fn draw_status_bar(&self, canvas: &mut CanvasService, input: &InputService, term_h: u16) {
    let fg = TextColor::Terminal(TerminalColor::BrightBlack);
    let style = TextStyle {
      foreground: Some(fg.clone()),
      ..Default::default()
    };

    let status = format!(
      "Capture: {}  raw: {}  actions: {}  hits: {}  clicks: {}  last: {} | {}",
      input.is_raw_key_capture_enabled(),
      self.raw_event_count,
      self.action_event_count,
      self.hit_event_count,
      self.click_count,
      self.last_action,
      self.last_hit_event,
    );
    // 截断到终端宽度
    let max_w = canvas.physical_width() as usize;
    let status: String = status.chars().take(max_w).collect();
    canvas.host_styled_text(0, term_h.saturating_sub(1), &status, style.clone());

    // raw events
    for (i, event) in self.raw_events.iter().enumerate() {
      let event: String = event.chars().take(max_w).collect();
      canvas.host_styled_text(
        0,
        term_h.saturating_sub(3 + i as u16),
        &event,
        style.clone(),
      );
    }
  }
}

// ── 辅助 ──

fn add_delta(val: u16, delta: i32) -> u16 {
  if delta >= 0 {
    val.saturating_add(delta as u16)
  } else {
    val.saturating_sub((-delta) as u16)
  }
}

fn add_delta_dim(val: u16, delta: i32, min: u16, max: u16) -> u16 {
  if delta >= 0 {
    val.saturating_add(delta as u16).min(max)
  } else {
    val.saturating_sub((-delta) as u16).max(min)
  }
}

fn format_hit_event(event: &HitAreaEvent) -> String {
  match event {
    HitAreaEvent::HoverEnter { id, x, y } => format!("Enter {id:?} @({x},{y})"),
    HitAreaEvent::HoverMove { id, x, y } => format!("Move {id:?} @({x},{y})"),
    HitAreaEvent::HoverLeave { id, x, y } => format!("Leave {id:?} @({x},{y})"),
    HitAreaEvent::Press { id, button, x, y } => {
      format!("Press {id:?} {button:?} @({x},{y})")
    }
    HitAreaEvent::Release { id, button, x, y } => {
      format!("Release {id:?} {button:?} @({x},{y})")
    }
    HitAreaEvent::Click { id, button, x, y } => {
      format!("Click {id:?} {button:?} @({x},{y})")
    }
    HitAreaEvent::Drag {
      id,
      button,
      x,
      y,
      dx,
      dy,
    } => {
      format!("Drag {id:?} {button:?} @({x},{y}) d({dx},{dy})")
    }
  }
}

// ── 测试 ──

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn delta_arithmetic() {
    assert_eq!(add_delta(10, 5), 15);
    assert_eq!(add_delta(10, -5), 5);
    assert_eq!(add_delta(3, -10), 0); // saturating at 0
    assert_eq!(add_delta_dim(30, 10, 20, 100), 40);
    assert_eq!(add_delta_dim(30, -20, 20, 100), 20); // clamped to min
    assert_eq!(add_delta_dim(90, 20, 20, 100), 100); // clamped to max
  }
}
