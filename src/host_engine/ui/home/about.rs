use crate::host_engine::services::{
  ActionMapEntry, BorderStyle, CanvasService, HitAreaEvent, HitAreaId, HitAreaOptions,
  HitAreaService, KeyState, LayoutService, MouseButton, Rect, RenderService, ScrollBoxId,
  ScrollBoxOptions, ScrollBoxService, SliceId, SliceLength, SliceOptions, SliceRect, SliceService,
  SurfaceId, TerminalColor, TextColor, TextInputEvent, TextInputId, TextInputMode,
  TextInputOptions, TextInputRenderParams, TextInputService, TextStyle, UiEvent, UiObjectPool,
  UiObjectPoolOwner,
};

/// 输入演示 UI：展示基础层、不透明切片、透明切片和文本输入的叠加渲染效果。
pub struct InputDemoUi {
  objects: UiObjectPool,
  opaque_slice: SliceId,
  transparent_slice: SliceId,
  empty_scroll_box: ScrollBoxId,
  text_scroll_box: ScrollBoxId,
  base_area: HitAreaId,
  opaque_area: HitAreaId,
  transparent_area: HitAreaId,
  input: TextInputId,
  transparent_visible: bool,
  transparent_in_front: bool,
  scroll_box_in_front: bool,
  click_count: usize,
  last_event: String,
}

impl UiObjectPoolOwner for InputDemoUi {
  fn objects(&self) -> &UiObjectPool {
    &self.objects
  }

  fn objects_mut(&mut self) -> &mut UiObjectPool {
    &mut self.objects
  }
}

/// 输入演示页面的命令。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputDemoCommand {
  ToggleTransparent,
  SwapLayers,
  FocusInput,
  BlurInput,
  Back,
}

impl InputDemoUi {
  /// 初始化演示页面：创建切片、命中区域和文本输入组件。
  pub fn init(
    hit_area: &HitAreaService,
    slices: &SliceService,
    scroll_box: &ScrollBoxService,
    text_input: &TextInputService,
  ) -> Self {
    let mut objects = UiObjectPool::new();
    let opaque_slice = slices
      .create(
        &mut objects,
        SliceOptions {
          rect: SliceRect {
            x: 3,
            y: 3,
            width: SliceLength::Percent(55),
            height: SliceLength::Fixed(10),
          },
          ..Default::default()
        },
      )
      .unwrap();
    let transparent_slice = slices
      .create(
        &mut objects,
        SliceOptions {
          rect: SliceRect {
            x: 15,
            y: 6,
            width: SliceLength::Percent(60),
            height: SliceLength::Fixed(10),
          },
          opaque: false,
          ..Default::default()
        },
      )
      .unwrap();
    let empty_scroll_box = scroll_box
      .create(
        &mut objects,
        ScrollBoxOptions {
          rect: Rect {
            x: 6,
            y: 13,
            width: 28,
            height: 9,
          },
          content_width: 28,
          content_height: 9,
          ..Default::default()
        },
      )
      .unwrap();
    let text_scroll_box = scroll_box
      .create(
        &mut objects,
        ScrollBoxOptions {
          rect: Rect {
            x: 38,
            y: 13,
            width: 38,
            height: 9,
          },
          content_width: 38,
          content_height: 60,
          ..Default::default()
        },
      )
      .unwrap();
    let base_area = hit_area.create(&mut objects, HitAreaOptions::default());
    let opaque_area = hit_area.create(&mut objects, HitAreaOptions::default());
    let transparent_area = hit_area.create(&mut objects, HitAreaOptions::default());
    let input = text_input.create(
      &mut objects,
      TextInputOptions {
        initial_text: "中文 / 日本語 / emoji 👨‍👩‍👧".into(),
        max_chars: Some(40),
        mode: TextInputMode::SingleLine,
        mouse: true,
      },
    );

    Self {
      objects,
      opaque_slice,
      transparent_slice,
      empty_scroll_box,
      text_scroll_box,
      base_area,
      opaque_area,
      transparent_area,
      input,
      transparent_visible: true,
      transparent_in_front: true,
      scroll_box_in_front: true,
      click_count: 0,
      last_event: "None".into(),
    }
  }

  /// 返回演示页面的按键映射定义。
  pub fn action_map() -> Vec<ActionMapEntry> {
    vec![
      ActionMapEntry {
        action: "input_demo.focus".into(),
        description: "Focus the Slice TextInput".into(),
        keys: vec![vec!["enter".into()]],
      },
      ActionMapEntry {
        action: "input_demo.visible".into(),
        description: "Toggle transparent Slice".into(),
        keys: vec![vec!["v".into()]],
      },
      ActionMapEntry {
        action: "input_demo.order".into(),
        description: "Swap Slice order".into(),
        keys: vec![vec!["f".into()]],
      },
      ActionMapEntry {
        action: "input_demo.back".into(),
        description: "Back to home".into(),
        keys: vec![vec!["esc".into()]],
      },
    ]
  }

  /// 处理 UI 事件，包含命中区域、文本输入事件的转发。
  pub fn handle_event(&mut self, event: &UiEvent) -> Option<InputDemoCommand> {
    match event {
      UiEvent::Action(event) if event.state == KeyState::Pressed => match event.action.as_str() {
        "input_demo.focus" => Some(InputDemoCommand::FocusInput),
        "input_demo.visible" => Some(InputDemoCommand::ToggleTransparent),
        "input_demo.order" => Some(InputDemoCommand::SwapLayers),
        "input_demo.back" => Some(InputDemoCommand::Back),
        _ => None,
      },
      UiEvent::HitArea(event) => {
        self.last_event = format_hit_event(event, self);
        if matches!(
          event,
          HitAreaEvent::Click {
            button: MouseButton::Left,
            ..
          }
        ) {
          self.click_count += 1;
        }
        None
      }
      UiEvent::TextInput(event) => {
        self.last_event = format_text_event(event);
        match event {
          TextInputEvent::Pressed { id } if *id == self.input => Some(InputDemoCommand::FocusInput),
          TextInputEvent::PressedOutside { id } | TextInputEvent::Cancel { id, .. }
            if *id == self.input =>
          {
            Some(InputDemoCommand::BlurInput)
          }
          _ => None,
        }
      }
      _ => None,
    }
  }

  /// 切换透明切片的可见性。
  pub fn toggle_transparent(&mut self, slices: &SliceService) {
    self.transparent_visible = !self.transparent_visible;
    slices.set_visible(
      &mut self.objects,
      self.transparent_slice,
      self.transparent_visible,
    );
  }

  /// 交换透明切片和不透明切片的绘制顺序。
  pub fn swap_layers(&mut self, _slices: &SliceService, scroll_box: &ScrollBoxService) {
    self.scroll_box_in_front = !self.scroll_box_in_front;
    if self.scroll_box_in_front {
      scroll_box.move_above(
        &mut self.objects,
        self.text_scroll_box,
        SurfaceId::Slice(self.transparent_slice),
      );
    } else {
      scroll_box.move_below(
        &mut self.objects,
        self.text_scroll_box,
        SurfaceId::Slice(self.transparent_slice),
      );
    }
    self.transparent_in_front = !self.scroll_box_in_front;
  }

  /// 聚焦文本输入组件。
  pub fn focus_input(&mut self, text_input: &mut TextInputService) {
    text_input.focus(&mut self.objects, self.input);
  }

  /// 取消文本输入聚焦。
  pub fn blur_input(&mut self, text_input: &mut TextInputService) {
    text_input.blur(&mut self.objects);
  }

  /// 离开页面时取消输入聚焦。
  pub fn leave(&mut self, text_input: &mut TextInputService) {
    if text_input.is_focused(&self.objects, self.input) {
      text_input.blur(&mut self.objects);
    }
  }

  pub fn update(&mut self) {}

  /// 渲染基础层、不透明切片、透明切片和宿主层，返回光标物理坐标。
  pub fn render(
    &mut self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    hit_area: &HitAreaService,
    scroll_box: &ScrollBoxService,
    text_input: &TextInputService,
  ) -> Option<(u16, u16)> {
    self.draw_base(render, canvas, layout, hit_area);
    self.draw_opaque_slice(render, canvas, hit_area);
    self.draw_scroll_box(render, canvas, layout, scroll_box);
    let cursor = self.draw_transparent_slice(canvas, hit_area, text_input);
    self.draw_host(canvas, layout);
    cursor
  }

  fn draw_base(
    &mut self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    hit_area: &HitAreaService,
  ) {
    let size = layout.developer_size();
    render.draw_filled_rect(
      canvas,
      0,
      0,
      size.width,
      size.height,
      Some("·".into()),
      Some(TextColor::Terminal(TerminalColor::BrightBlack)),
      Some(TextColor::Rgb { r: 8, g: 16, b: 24 }),
    );
    render.draw_border_rect(
      canvas,
      0,
      0,
      size.width,
      size.height,
      &BorderStyle::Double,
      Some(TextColor::Terminal(TerminalColor::BrightCyan)),
      None,
      Some(TextColor::Rgb { r: 8, g: 16, b: 24 }),
      None,
    );
    canvas.styled_text(
      2,
      1,
      "Developer Base (viewport-local 0,0)",
      bright(TerminalColor::BrightCyan),
    );
    hit_area.render(
      &mut self.objects,
      self.base_area,
      Rect {
        x: 0,
        y: 0,
        width: size.width,
        height: size.height,
      },
      canvas,
    );
  }

  fn draw_opaque_slice(
    &mut self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    hit_area: &HitAreaService,
  ) {
    let Some(rect) = canvas.prepared_slice_rect(self.opaque_slice) else {
      return;
    };
    render.draw_filled_rect_on(
      canvas,
      self.opaque_slice,
      0,
      0,
      rect.width,
      rect.height,
      Some(" ".into()),
      None,
      Some(TextColor::Rgb {
        r: 16,
        g: 48,
        b: 96,
      }),
    );
    render.draw_border_rect_on(
      canvas,
      self.opaque_slice,
      0,
      0,
      rect.width,
      rect.height,
      &BorderStyle::Line,
      Some(TextColor::Terminal(TerminalColor::BrightBlue)),
      None,
      Some(TextColor::Rgb {
        r: 16,
        g: 48,
        b: 96,
      }),
      None,
    );
    canvas.styled_text_on(
      self.opaque_slice,
      2,
      1,
      "Opaque Slice",
      bright(TerminalColor::BrightWhite),
    );
    canvas.styled_text_on(
      self.opaque_slice,
      2,
      3,
      "Unwritten cells cover Base",
      bright(TerminalColor::BrightBlue),
    );
    hit_area.render_on(
      &mut self.objects,
      self.opaque_area,
      self.opaque_slice,
      Rect {
        x: 0,
        y: 0,
        width: rect.width,
        height: rect.height,
      },
      canvas,
    );
  }

  fn draw_scroll_box(
    &mut self,
    _render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    scroll_box: &ScrollBoxService,
  ) {
    let Some(rect) = canvas.prepared_scroll_box_rect(self.text_scroll_box) else {
      return;
    };
    let _ = canvas.prepared_scroll_box_rect(self.empty_scroll_box);
    for line in 0..56u16 {
      canvas.styled_text_in_scroll_box(
        self.text_scroll_box,
        0,
        line,
        &format!("Line {line:02}  only text / 中文 / emoji 😀"),
        bright(TerminalColor::BrightWhite),
      );
    }
    let scroll_y = scroll_box
      .scroll_y(&self.objects, self.text_scroll_box)
      .unwrap_or(0);
    let max_y = scroll_box
      .max_scroll_y(&self.objects, self.text_scroll_box, layout)
      .unwrap_or(0);
    canvas.styled_text(
      rect.x,
      rect.y.saturating_sub(1),
      &format!("Scroll y={scroll_y}/{max_y}"),
      bright(TerminalColor::BrightYellow),
    );
  }

  fn draw_transparent_slice(
    &mut self,
    canvas: &mut CanvasService,
    hit_area: &HitAreaService,
    text_input: &TextInputService,
  ) -> Option<(u16, u16)> {
    let rect = canvas.prepared_slice_rect(self.transparent_slice)?;
    draw_slice_frame(
      canvas,
      self.transparent_slice,
      rect.width,
      rect.height,
      bright(TerminalColor::BrightMagenta),
    );
    canvas.styled_text_on(
      self.transparent_slice,
      2,
      1,
      "Transparent Slice",
      bright(TerminalColor::BrightMagenta),
    );
    canvas.styled_text_on(
      self.transparent_slice,
      2,
      2,
      "Unwritten cells show lower surfaces",
      bright(TerminalColor::BrightWhite),
    );
    canvas.styled_text_on(
      self.transparent_slice,
      2,
      3,
      " explicit spaces ",
      TextStyle {
        foreground: Some(TextColor::Terminal(TerminalColor::Black)),
        background: Some(TextColor::Terminal(TerminalColor::BrightYellow)),
        ..Default::default()
      },
    );
    hit_area.render_on(
      &mut self.objects,
      self.transparent_area,
      self.transparent_slice,
      Rect {
        x: 0,
        y: 0,
        width: rect.width,
        height: rect.height,
      },
      canvas,
    );
    text_input.render_on(
      &mut self.objects,
      self.input,
      self.transparent_slice,
      &TextInputRenderParams {
        rect: Rect {
          x: 2,
          y: 5,
          width: rect.width.saturating_sub(4),
          height: 1,
        },
        placeholder: "type here".into(),
        fg: Some(TextColor::Terminal(TerminalColor::BrightWhite)),
        bg: Some(TextColor::Rgb {
          r: 64,
          g: 16,
          b: 72,
        }),
        placeholder_fg: Some(TextColor::Terminal(TerminalColor::BrightBlack)),
        cursor_blink: true,
        ..Default::default()
      },
      canvas,
    )
  }

  fn draw_host(&self, canvas: &mut CanvasService, layout: &LayoutService) {
    let physical = layout.physical_size();
    let viewport = layout.developer_viewport_rect();
    canvas.host_styled_text(
      0,
      0,
      &format!(
        "HOST SURFACE  physical={}x{}  viewport=({}, {}) {}x{}",
        physical.width, physical.height, viewport.x, viewport.y, viewport.width, viewport.height
      ),
      bright(TerminalColor::BrightGreen),
    );
    let front = if self.scroll_box_in_front {
      "scrollbox"
    } else {
      "transparent slice"
    };
    let status = format!(
      "[V] visible={}  [F] top={}  wheel scrolls only when ScrollBox is top  [Enter/click] focus  [Esc] blur/back  clicks={}  last={}",
      self.transparent_visible, front, self.click_count, self.last_event
    );
    let status: String = status.chars().take(physical.width as usize).collect();
    canvas.host_styled_text(
      0,
      physical.height.saturating_sub(1),
      &status,
      bright(TerminalColor::BrightGreen),
    );
    draw_host_frame(canvas, viewport, bright(TerminalColor::BrightGreen));
  }
}

fn bright(color: TerminalColor) -> TextStyle {
  TextStyle {
    foreground: Some(TextColor::Terminal(color)),
    ..Default::default()
  }
}

fn draw_slice_frame(
  canvas: &mut CanvasService,
  slice: SliceId,
  width: u16,
  height: u16,
  style: TextStyle,
) {
  if width < 2 || height < 2 {
    return;
  }
  canvas.styled_text_on(
    slice,
    0,
    0,
    &format!("┌{}┐", "─".repeat((width - 2) as usize)),
    style.clone(),
  );
  for y in 1..height - 1 {
    canvas.styled_text_on(slice, 0, y, "│", style.clone());
    canvas.styled_text_on(slice, width - 1, y, "│", style.clone());
  }
  canvas.styled_text_on(
    slice,
    0,
    height - 1,
    &format!("└{}┘", "─".repeat((width - 2) as usize)),
    style,
  );
}

fn draw_host_frame(canvas: &mut CanvasService, rect: Rect, style: TextStyle) {
  if rect.width < 2 || rect.height < 2 {
    return;
  }
  canvas.host_styled_text(
    rect.x,
    rect.y,
    &format!("┏{}┓", "━".repeat((rect.width - 2) as usize)),
    style.clone(),
  );
  for y in rect.y + 1..rect.y + rect.height - 1 {
    canvas.host_styled_text(rect.x, y, "┃", style.clone());
    canvas.host_styled_text(rect.x + rect.width - 1, y, "┃", style.clone());
  }
  canvas.host_styled_text(
    rect.x,
    rect.y + rect.height - 1,
    &format!("┗{}┛", "━".repeat((rect.width - 2) as usize)),
    style,
  );
}

fn format_hit_event(event: &HitAreaEvent, ui: &InputDemoUi) -> String {
  let (id, x, y) = match event {
    HitAreaEvent::HoverEnter { id, x, y }
    | HitAreaEvent::HoverMove { id, x, y }
    | HitAreaEvent::HoverLeave { id, x, y }
    | HitAreaEvent::Press { id, x, y, .. }
    | HitAreaEvent::Release { id, x, y, .. }
    | HitAreaEvent::Click { id, x, y, .. }
    | HitAreaEvent::Drag { id, x, y, .. } => (*id, *x, *y),
  };
  let surface = if id == ui.base_area {
    "Base"
  } else if id == ui.opaque_area {
    "Opaque"
  } else {
    "Transparent"
  };
  let kind = match event {
    HitAreaEvent::HoverEnter { .. } => "Enter",
    HitAreaEvent::HoverMove { .. } => "Move",
    HitAreaEvent::HoverLeave { .. } => "Leave",
    HitAreaEvent::Press { .. } => "Press",
    HitAreaEvent::Release { .. } => "Release",
    HitAreaEvent::Click { .. } => "Click",
    HitAreaEvent::Drag { .. } => "Drag",
  };
  format!("{surface}:{kind}@{x},{y}")
}

fn format_text_event(event: &TextInputEvent) -> String {
  match event {
    TextInputEvent::Focused { .. } => "TextInput:Focused".into(),
    TextInputEvent::Blurred { .. } => "TextInput:Blurred".into(),
    TextInputEvent::Changed { value, .. } => format!("TextInput:Changed({value})"),
    TextInputEvent::Submit { value, .. } => format!("TextInput:Submit({value})"),
    TextInputEvent::Cancel { .. } => "TextInput:Cancel".into(),
    TextInputEvent::Pressed { .. } => "TextInput:Pressed".into(),
    TextInputEvent::PressedOutside { .. } => "TextInput:PressedOutside".into(),
  }
}
