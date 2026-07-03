use crate::host_engine::services::{
  ActionMapEntry, CanvasService, HitAreaEvent, HitAreaId, HitAreaOptions, HitAreaService, KeyState,
  LayoutService, Overflow, Rect, RenderService, ScrollBoxEvent, ScrollBoxId, ScrollBoxOptions,
  ScrollBoxService, ScrollbarLayout, ScrollbarPolicy, ScrollbarVisibility, RuntimeObjectPool,
  RuntimeObjectPoolOwner, SliceId, SliceLength, SliceOptions, SliceRect, SliceService, SurfaceId,
  TerminalColor, TextColor, TextStyle, UiEvent, UiObjectPool, UiObjectPoolOwner,
};

/// ScrollBox v2 综合测试 UI。
pub struct InputDemoUi {
  objects: UiObjectPool,
  runtime_objects: RuntimeObjectPool,
  /// 基础纵向滚动盒子（Auto 滚动条 + 事件）。
  v_scroll: ScrollBoxId,
  /// 纯横向滚动盒子（Overlay 布局，宽内容）。
  h_scroll: ScrollBoxId,
  /// 双轴滚动盒子（同时横向和纵向均可滚动）。
  both_scroll: ScrollBoxId,
  /// ReserveSpace 布局的纵向滚动盒子。
  reserve_scroll: ScrollBoxId,
  /// Always 滚动条策略（内容未溢出时也显示）。
  always_scroll: ScrollBoxId,
  /// overflow_y = Hidden（禁止纵向滚动）。
  hidden_scroll: ScrollBoxId,
  /// 透明（opaque=false）滚动盒子。
  transparent_scroll: ScrollBoxId,
  /// 用于遮挡测试的 Slice。
  cover_slice: SliceId,
  /// Slice 上的命中区域。
  cover_hit: HitAreaId,
  /// 鼠标事件日志。
  last_event: String,
  /// 首次布局是否完成。
  first_layout: bool,
}

impl UiObjectPoolOwner for InputDemoUi {
  fn objects(&self) -> &UiObjectPool {
    &self.objects
  }
  fn objects_mut(&mut self) -> &mut UiObjectPool {
    &mut self.objects
  }
}

impl RuntimeObjectPoolOwner for InputDemoUi {
  fn runtime_objects(&self) -> &RuntimeObjectPool {
    &self.runtime_objects
  }

  fn runtime_objects_mut(&mut self) -> &mut RuntimeObjectPool {
    &mut self.runtime_objects
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputDemoCommand {
  Back,
}

impl InputDemoUi {
  pub fn init(
    hit_area: &HitAreaService,
    slices: &SliceService,
    scroll_box: &ScrollBoxService,
  ) -> Self {
    let mut objects = UiObjectPool::new();

    // ── 1. 基础纵向滚动（Overlay 模式，用于对比） ──
    let v_scroll = scroll_box
      .create(
        &mut objects,
        ScrollBoxOptions {
          rect: Rect {
            x: 1,
            y: 2,
            width: 24,
            height: 11,
          },
          content_width: 24,
          content_height: 40,
          scrollbar_layout: ScrollbarLayout::Overlay,
          emit_scroll_events: true,
          ..Default::default()
        },
      )
      .unwrap();

    // ── 2. 纯横向滚动 ──
    let h_scroll = scroll_box
      .create(
        &mut objects,
        ScrollBoxOptions {
          rect: Rect {
            x: 27,
            y: 2,
            width: 24,
            height: 11,
          },
          content_width: 80,
          content_height: 11,
          overflow_x: Overflow::Auto,
          scrollbar: ScrollbarPolicy {
            vertical: ScrollbarVisibility::Never,
            horizontal: ScrollbarVisibility::Auto,
          },
          emit_scroll_events: true,
          ..Default::default()
        },
      )
      .unwrap();

    // ── 3. 双轴滚动 ──
    let both_scroll = scroll_box
      .create(
        &mut objects,
        ScrollBoxOptions {
          rect: Rect {
            x: 53,
            y: 2,
            width: 24,
            height: 11,
          },
          content_width: 60,
          content_height: 30,
          overflow_x: Overflow::Auto,
          scrollbar: ScrollbarPolicy {
            vertical: ScrollbarVisibility::Auto,
            horizontal: ScrollbarVisibility::Auto,
          },
          emit_scroll_events: true,
          ..Default::default()
        },
      )
      .unwrap();

    // ── 4. ReserveSpace 布局 ──
    let reserve_scroll = scroll_box
      .create(
        &mut objects,
        ScrollBoxOptions {
          rect: Rect {
            x: 1,
            y: 14,
            width: 24,
            height: 6,
          },
          content_width: 24,
          content_height: 20,
          scrollbar_layout: ScrollbarLayout::ReserveSpace,
          ..Default::default()
        },
      )
      .unwrap();

    // ── 5. Always 滚动条 ──
    let always_scroll = scroll_box
      .create(
        &mut objects,
        ScrollBoxOptions {
          rect: Rect {
            x: 27,
            y: 14,
            width: 24,
            height: 6,
          },
          content_width: 24,
          content_height: 6, // 内容正好等于 viewport 高度
          scrollbar: ScrollbarPolicy {
            vertical: ScrollbarVisibility::Always,
            horizontal: ScrollbarVisibility::Never,
          },
          ..Default::default()
        },
      )
      .unwrap();

    // ── 6. overflow_y = Hidden ──
    let hidden_scroll = scroll_box
      .create(
        &mut objects,
        ScrollBoxOptions {
          rect: Rect {
            x: 1,
            y: 21,
            width: 24,
            height: 4,
          },
          content_width: 24,
          content_height: 20,
          overflow_y: Overflow::Hidden,
          scrollbar: ScrollbarPolicy {
            vertical: ScrollbarVisibility::Never,
            horizontal: ScrollbarVisibility::Never,
          },
          ..Default::default()
        },
      )
      .unwrap();

    // ── 7. 透明滚动盒子 ──
    let transparent_scroll = scroll_box
      .create(
        &mut objects,
        ScrollBoxOptions {
          rect: Rect {
            x: 27,
            y: 21,
            width: 24,
            height: 4,
          },
          content_width: 24,
          content_height: 20,
          opaque: false,
          ..Default::default()
        },
      )
      .unwrap();

    // ── 用于遮挡测试的 Slice ──
    let cover_slice = slices
      .create(
        &mut objects,
        SliceOptions {
          rect: SliceRect {
            x: 42,
            y: 16,
            width: SliceLength::Fixed(12),
            height: SliceLength::Fixed(6),
          },
          opaque: true,
          ..Default::default()
        },
      )
      .unwrap();

    let cover_hit = hit_area.create(&mut objects, HitAreaOptions::default());

    // 初始层级：slice 在 scroll boxes 之上（遮挡命中测试）。
    scroll_box.move_below(&mut objects, both_scroll, SurfaceId::Slice(cover_slice));

    Self {
      objects,
      runtime_objects: RuntimeObjectPool::new(),
      v_scroll,
      h_scroll,
      both_scroll,
      reserve_scroll,
      always_scroll,
      hidden_scroll,
      transparent_scroll,
      cover_slice,
      cover_hit,
      last_event: "—".into(),
      first_layout: false,
    }
  }

  pub fn action_map() -> Vec<ActionMapEntry> {
    vec![ActionMapEntry {
      action: "input_demo.back".into(),
      description: "Back to home".into(),
      keys: vec![vec!["esc".into()]],
    }]
  }

  pub fn handle_event(&mut self, event: &UiEvent) -> Option<InputDemoCommand> {
    match event {
      UiEvent::Action(event) if event.state == KeyState::Pressed => match event.action.as_str() {
        "input_demo.back" => Some(InputDemoCommand::Back),
        _ => None,
      },
      UiEvent::HitArea(event) => {
        self.last_event = format_hit_event(event);
        None
      }
      UiEvent::ScrollBox(event) => {
        if let ScrollBoxEvent::Scrolled { id, x, y } = event {
          let name = self.scroll_name(*id);
          self.last_event = format!("{name}:Scrolled x={x} y={y}");
        }
        None
      }
      _ => None,
    }
  }

  pub fn update(&mut self) {}

  /// 后续帧的布局微调（仅在首帧后执行）。
  pub fn apply_layout(&mut self, _layout: &LayoutService, _scroll_box: &ScrollBoxService) {
    self.first_layout = true;
  }

  pub fn leave(&mut self) {}

  pub fn render(
    &mut self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    hit_area: &HitAreaService,
    scroll_box: &ScrollBoxService,
  ) {
    self.draw_base(render, canvas, layout, hit_area);
    self.draw_cover_slice(canvas, hit_area, layout);
    self.draw_v_scroll(canvas, scroll_box, layout);
    self.draw_h_scroll(canvas, scroll_box, layout);
    self.draw_both_scroll(canvas, scroll_box, layout);
    self.draw_reserve_scroll(canvas, scroll_box, layout);
    self.draw_always_scroll(canvas, scroll_box, layout);
    self.draw_hidden_scroll(canvas);
    self.draw_transparent_scroll(canvas);
    self.draw_host(canvas, layout, scroll_box);
  }

  // ─── 绘制函数 ────────────────────────────────────────

  fn draw_base(
    &mut self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    _hit_area: &HitAreaService,
  ) {
    let size = layout.developer_size();
    render.draw_filled_rect(
      canvas,
      0,
      0,
      size.width,
      size.height,
      Some(" ".into()),
      None,
      Some(TextColor::Rgb { r: 4, g: 8, b: 16 }),
    );

    // 标签行。
    canvas.styled_text(1, 1, "V-Scroll", label());
    canvas.styled_text(27, 1, "H-Scroll", label());
    canvas.styled_text(53, 1, "Both XY", label());

    canvas.styled_text(1, 13, "ReserveSpace", label());
    canvas.styled_text(27, 13, "Always bar", label());

    canvas.styled_text(1, 20, "Hidden overflow", label());
    canvas.styled_text(27, 20, "Transparent (opaque=false)", label());
  }

  /// 基础纵向滚动 — 多行文本 + 滚轮 + 拖动 + 事件。
  fn draw_v_scroll(
    &mut self,
    canvas: &mut CanvasService,
    scroll_box: &ScrollBoxService,
    layout: &LayoutService,
  ) {
    let Some(rect) = canvas.prepared_scroll_box_rect(self.v_scroll) else {
      return;
    };
    // 绘制边框。
    draw_box_border(
      canvas,
      rect,
      self.v_scroll,
      bright(TerminalColor::BrightBlue),
    );

    // 填充内容。
    for line in 0..40u16 {
      let style = if line % 5 == 0 {
        bright(TerminalColor::BrightYellow)
      } else {
        bright(TerminalColor::BrightWhite)
      };
      canvas.styled_text_in_scroll_box(
        self.v_scroll,
        1,
        line,
        &format!("Line {line:02}  中文测试 😀 あいう",),
        style,
      );
    }

    // 滚动状态显示。
    let sy = scroll_box
      .scroll_y(&self.objects, self.v_scroll)
      .unwrap_or(0);
    let max_y = scroll_box
      .max_scroll_y(&self.objects, self.v_scroll, layout)
      .unwrap_or(0);
    let pos = scroll_box
      .scroll_position(&self.objects, self.v_scroll)
      .unwrap_or((0, 0));
    canvas.styled_text(
      rect.x,
      rect.y.saturating_sub(1),
      &format!("V y={}/{} pos=({},{})", sy, max_y, pos.0, pos.1),
      bright(TerminalColor::BrightBlue),
    );
  }

  /// 纯横向滚动 — 宽内容 + 水平滚动条。
  fn draw_h_scroll(
    &mut self,
    canvas: &mut CanvasService,
    scroll_box: &ScrollBoxService,
    layout: &LayoutService,
  ) {
    let Some(rect) = canvas.prepared_scroll_box_rect(self.h_scroll) else {
      return;
    };
    draw_box_border(
      canvas,
      rect,
      self.h_scroll,
      bright(TerminalColor::BrightCyan),
    );

    // 一行宽内容。
    for col in 0..80u16 {
      let ch = if col % 10 == 0 {
        format!("│{:02}│", col)
      } else {
        format!("{col:02}")
      };
      canvas.styled_text_in_scroll_box(
        self.h_scroll,
        col,
        1,
        &ch,
        bright(TerminalColor::BrightCyan),
      );
    }
    // 第二行显示标记。
    canvas.styled_text_in_scroll_box(
      self.h_scroll,
      0,
      3,
      "←─ scroll horizontally ─→ 水平滚动测试",
      bright(TerminalColor::BrightGreen),
    );

    let sx = scroll_box
      .scroll_x(&self.objects, self.h_scroll)
      .unwrap_or(0);
    let max_x = scroll_box
      .max_scroll_x(&self.objects, self.h_scroll, layout)
      .unwrap_or(0);
    canvas.styled_text(
      rect.x,
      rect.y.saturating_sub(1),
      &format!("H x={}/{}", sx, max_x),
      bright(TerminalColor::BrightCyan),
    );
  }

  /// 双轴滚动 — 网格内容 + 两条滚动条。
  fn draw_both_scroll(
    &mut self,
    canvas: &mut CanvasService,
    scroll_box: &ScrollBoxService,
    layout: &LayoutService,
  ) {
    let Some(rect) = canvas.prepared_scroll_box_rect(self.both_scroll) else {
      return;
    };
    draw_box_border(
      canvas,
      rect,
      self.both_scroll,
      bright(TerminalColor::BrightMagenta),
    );

    for row in 0..30u16 {
      for col in 0..15u16 {
        // 每 4 列放置一个网格单元格。
        let cx = col * 4;
        let style = if (row + col) % 2 == 0 {
          bright(TerminalColor::BrightMagenta)
        } else {
          bright(TerminalColor::BrightWhite)
        };
        canvas.styled_text_in_scroll_box(
          self.both_scroll,
          cx,
          row,
          &format!("({},{})", col, row),
          style,
        );
      }
    }

    let sx = scroll_box
      .scroll_x(&self.objects, self.both_scroll)
      .unwrap_or(0);
    let sy = scroll_box
      .scroll_y(&self.objects, self.both_scroll)
      .unwrap_or(0);
    let max_x = scroll_box
      .max_scroll_x(&self.objects, self.both_scroll, layout)
      .unwrap_or(0);
    let max_y = scroll_box
      .max_scroll_y(&self.objects, self.both_scroll, layout)
      .unwrap_or(0);
    canvas.styled_text(
      rect.x,
      rect.y.saturating_sub(1),
      &format!("XY x={}/{} y={}/{}", sx, max_x, sy, max_y),
      bright(TerminalColor::BrightMagenta),
    );
  }

  /// ReserveSpace — 滚动条不覆盖内容。
  fn draw_reserve_scroll(
    &mut self,
    canvas: &mut CanvasService,
    scroll_box: &ScrollBoxService,
    layout: &LayoutService,
  ) {
    let Some(rect) = canvas.prepared_scroll_box_rect(self.reserve_scroll) else {
      return;
    };
    draw_box_border(
      canvas,
      rect,
      self.reserve_scroll,
      bright(TerminalColor::BrightYellow),
    );

    for line in 0..20u16 {
      let marker = if line % 3 == 0 { "█" } else { "·" };
      canvas.styled_text_in_scroll_box(
        self.reserve_scroll,
        1,
        line,
        &format!("Rsv {marker} Ln {line:02}"),
        bright(TerminalColor::BrightYellow),
      );
    }

    let sy = scroll_box
      .scroll_y(&self.objects, self.reserve_scroll)
      .unwrap_or(0);
    let max_y = scroll_box
      .max_scroll_y(&self.objects, self.reserve_scroll, layout)
      .unwrap_or(0);
    // 显示 viewport 与 content 尺寸。
    let vsz = scroll_box.viewport_size(&self.objects, self.reserve_scroll, layout);
    canvas.styled_text(
      rect.x,
      rect.y.saturating_sub(1),
      &format!("RSV y={}/{} vsz={:?}", sy, max_y, vsz),
      bright(TerminalColor::BrightYellow),
    );
  }

  /// Always 滚动条 — 内容刚好等于 viewport 高度但滚动条仍然显示。
  fn draw_always_scroll(
    &mut self,
    canvas: &mut CanvasService,
    scroll_box: &ScrollBoxService,
    layout: &LayoutService,
  ) {
    let Some(rect) = canvas.prepared_scroll_box_rect(self.always_scroll) else {
      return;
    };
    draw_box_border(
      canvas,
      rect,
      self.always_scroll,
      bright(TerminalColor::BrightGreen),
    );

    for line in 0..6u16 {
      canvas.styled_text_in_scroll_box(
        self.always_scroll,
        1,
        line,
        &format!("Always bar — line {}", line),
        bright(TerminalColor::BrightGreen),
      );
    }

    let max_y = scroll_box
      .max_scroll_y(&self.objects, self.always_scroll, layout)
      .unwrap_or(0);
    canvas.styled_text(
      rect.x,
      rect.y.saturating_sub(1),
      &format!("ALW max_y={} (should be 0)", max_y),
      bright(TerminalColor::BrightGreen),
    );
  }

  /// overflow_y = Hidden — 内容被裁剪，无法滚动。
  fn draw_hidden_scroll(&mut self, canvas: &mut CanvasService) {
    let Some(rect) = canvas.prepared_scroll_box_rect(self.hidden_scroll) else {
      return;
    };
    draw_box_border(
      canvas,
      rect,
      self.hidden_scroll,
      bright(TerminalColor::BrightRed),
    );

    for line in 0..20u16 {
      canvas.styled_text_in_scroll_box(
        self.hidden_scroll,
        1,
        line,
        &format!("HIDDEN line {} — should be clipped", line),
        bright(TerminalColor::BrightRed),
      );
    }
    canvas.styled_text(
      rect.x,
      rect.y.saturating_sub(1),
      "Hidden — wheel blocked, content clipped",
      bright(TerminalColor::BrightRed),
    );
  }

  /// 透明滚动盒子 — 未写入部分可看到底层。
  fn draw_transparent_scroll(&mut self, canvas: &mut CanvasService) {
    let Some(rect) = canvas.prepared_scroll_box_rect(self.transparent_scroll) else {
      return;
    };
    draw_box_border(
      canvas,
      rect,
      self.transparent_scroll,
      bright(TerminalColor::BrightWhite),
    );

    // 仅写入部分行——其余位置应透露底层。
    for line in [0u16, 3, 6, 9, 12, 15, 18] {
      canvas.styled_text_in_scroll_box(
        self.transparent_scroll,
        1,
        line,
        &format!("Opaque line {}", line),
        bright(TerminalColor::BrightWhite),
      );
    }
    canvas.styled_text(
      rect.x,
      rect.y.saturating_sub(1),
      "Transparent — unwritten cells show base",
      bright(TerminalColor::BrightWhite),
    );
  }

  /// 遮挡 Slice — 测试 Slice 覆盖 ScrollBox 时的事件优先级。
  fn draw_cover_slice(
    &mut self,
    canvas: &mut CanvasService,
    hit_area: &HitAreaService,
    _layout: &LayoutService,
  ) {
    let Some(rect) = canvas.prepared_slice_rect(self.cover_slice) else {
      return;
    };
    // 不透明 Slice 背景。
    canvas.styled_text_on(
      self.cover_slice,
      0,
      1,
      "  SLICE COVER  ",
      bright(TerminalColor::BrightBlack),
    );
    canvas.styled_text_on(
      self.cover_slice,
      0,
      2,
      " blocks scroll  ",
      bright(TerminalColor::BrightBlack),
    );
    canvas.styled_text_on(
      self.cover_slice,
      0,
      3,
      " events below   ",
      bright(TerminalColor::BrightBlack),
    );
    hit_area.render_on(
      &mut self.objects,
      self.cover_hit,
      self.cover_slice,
      Rect {
        x: 0,
        y: 0,
        width: rect.width,
        height: rect.height,
      },
      canvas,
    );
  }

  /// 宿主层状态栏。
  fn draw_host(
    &mut self,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    scroll_box: &ScrollBoxService,
  ) {
    let physical = layout.physical_size();
    let _ = scroll_box.drain_scroll_events(&mut self.objects);

    // 底部状态栏。
    let status = format!(
      "ScrollBox v2 Test  |  last: {}  |  esc=back  |  wheel/click/drag scrollbars to test",
      &self.last_event
    );
    let status: String = status.chars().take(physical.width as usize).collect();
    canvas.host_styled_text(
      0,
      physical.height.saturating_sub(1),
      &status,
      bright(TerminalColor::BrightGreen),
    );
  }

  fn scroll_name(&self, id: ScrollBoxId) -> &'static str {
    if id == self.v_scroll {
      "V"
    } else if id == self.h_scroll {
      "H"
    } else if id == self.both_scroll {
      "XY"
    } else if id == self.reserve_scroll {
      "RSV"
    } else if id == self.always_scroll {
      "ALW"
    } else {
      "?"
    }
  }
}

// ─── 辅助函数 ───────────────────────────────────────────

fn bright(color: TerminalColor) -> TextStyle {
  TextStyle {
    foreground: Some(TextColor::Terminal(color)),
    ..Default::default()
  }
}

fn label() -> TextStyle {
  TextStyle {
    foreground: Some(TextColor::Terminal(TerminalColor::BrightBlack)),
    background: Some(TextColor::Terminal(TerminalColor::BrightWhite)),
    ..Default::default()
  }
}

fn draw_box_border(canvas: &mut CanvasService, rect: Rect, id: ScrollBoxId, style: TextStyle) {
  if rect.width < 2 || rect.height < 2 {
    return;
  }
  let w = rect.width as usize;
  canvas.styled_text_in_scroll_box(id, 0, 0, &format!("┌{}┐", "─".repeat(w - 2)), style.clone());
  for y in 1..rect.height - 1 {
    canvas.styled_text_in_scroll_box(id, 0, y, "│", style.clone());
    canvas.styled_text_in_scroll_box(id, rect.width - 1, y, "│", style.clone());
  }
  canvas.styled_text_in_scroll_box(
    id,
    0,
    rect.height - 1,
    &format!("└{}┘", "─".repeat(w - 2)),
    style,
  );
}

fn format_hit_event(event: &HitAreaEvent) -> String {
  let (x, y) = match event {
    HitAreaEvent::HoverEnter { x, y, .. }
    | HitAreaEvent::HoverMove { x, y, .. }
    | HitAreaEvent::HoverLeave { x, y, .. }
    | HitAreaEvent::Press { x, y, .. }
    | HitAreaEvent::Release { x, y, .. }
    | HitAreaEvent::Click { x, y, .. }
    | HitAreaEvent::Drag { x, y, .. } => (*x, *y),
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
  format!("SliceHit:{kind}@{x},{y}")
}
