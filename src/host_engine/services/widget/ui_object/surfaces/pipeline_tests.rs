//! Frame composition of widget-created slices and scroll boxes (widget + canvas + render pipeline).

use crate::host_engine::services::{
  CanvasService, ComposedCell, ComposedFrame, FrameCompositor, LayoutService, Overflow,
  ScrollBoxOptions, ScrollBoxService, ScrollbarPolicy, ScrollbarVisibility, SliceLength,
  SliceOptions, SliceRect, SliceService, SurfaceId, TerminalColor, TextColor, TextStyle,
  UiObjectPool,
};

fn text(frame: &ComposedFrame, x: u16, y: u16) -> &str {
  match frame.get(x, y).unwrap() {
    ComposedCell::Text(cell) => &cell.text,
    ComposedCell::Empty => "",
  }
}

#[test]
fn viewport_slices_and_host_compose_in_order() {
  let mut layout = LayoutService::new();
  layout.resize_physical(12, 6);
  layout.set_developer_viewport(tg_service_layout::Rect {
    x: 2,
    y: 1,
    width: 8,
    height: 4,
  });
  let slices = SliceService::new();
  let mut pool = UiObjectPool::new();
  let opaque = slices
    .create(
      &mut pool,
      SliceOptions {
        rect: SliceRect {
          x: 2,
          y: 1,
          width: SliceLength::Fixed(3),
          height: SliceLength::Fixed(2),
        },
        ..Default::default()
      },
    )
    .unwrap();
  let transparent = slices
    .create(
      &mut pool,
      SliceOptions {
        rect: SliceRect {
          x: 2,
          y: 1,
          width: SliceLength::Fixed(3),
          height: SliceLength::Fixed(2),
        },
        opaque: false,
        ..Default::default()
      },
    )
    .unwrap();
  let mut canvas = CanvasService::new();
  canvas.begin_frame(&layout);
  pool.prepare_canvas(&mut canvas, &layout);
  canvas.styled_text(2, 1, "B", TextStyle::default());
  canvas.styled_text_on(transparent, 1, 0, "T", TextStyle::default());
  canvas.host_styled_text(5, 2, "H", TextStyle::default());

  let frame = FrameCompositor::new().compose(&canvas);
  assert_eq!(text(&frame, 4, 2), " ");
  assert_eq!(text(&frame, 5, 2), "H");
  assert_eq!(text(&frame, 6, 2), " ");
  assert!(canvas.prepared_slice_rect(opaque).is_some());
}

#[test]
fn later_slice_wins_and_wide_grapheme_is_not_split() {
  let mut layout = LayoutService::new();
  layout.resize_physical(5, 2);
  let service = SliceService::new();
  let mut pool = UiObjectPool::new();
  let options = SliceOptions {
    rect: SliceRect {
      x: 0,
      y: 0,
      width: SliceLength::Fixed(1),
      height: SliceLength::Fixed(1),
    },
    opaque: false,
    ..Default::default()
  };
  let a = service.create(&mut pool, options.clone()).unwrap();
  let b = service.create(&mut pool, options).unwrap();
  let mut canvas = CanvasService::new();
  canvas.begin_frame(&layout);
  pool.prepare_canvas(&mut canvas, &layout);
  canvas.styled_text_on(a, 0, 0, "A", TextStyle::default());
  canvas.styled_text_on(b, 0, 0, "B", TextStyle::default());
  canvas.styled_text_on(b, 0, 0, "我", TextStyle::default());
  let frame = FrameCompositor::new().compose(&canvas);
  assert_eq!(text(&frame, 0, 0), "B");
}

#[test]
fn transparent_slice_explicit_space_inherits_lower_background() {
  let mut layout = LayoutService::new();
  layout.resize_physical(3, 1);
  let slices = SliceService::new();
  let mut pool = UiObjectPool::new();
  let slice = slices
    .create(
      &mut pool,
      SliceOptions {
        opaque: false,
        ..Default::default()
      },
    )
    .unwrap();
  let mut canvas = CanvasService::new();
  canvas.begin_frame(&layout);
  pool.prepare_canvas(&mut canvas, &layout);
  let background = TextColor::Terminal(TerminalColor::Blue);
  canvas.styled_text(
    0,
    0,
    "B",
    TextStyle {
      background: Some(background.clone()),
      ..Default::default()
    },
  );
  canvas.styled_text_on(
    slice,
    0,
    0,
    " ",
    TextStyle {
      background: Some(TextColor::Transparent),
      ..Default::default()
    },
  );

  let frame = FrameCompositor::new().compose(&canvas);
  let ComposedCell::Text(cell) = frame.get(0, 0).unwrap() else {
    panic!("expected text cell")
  };
  assert_eq!(cell.text, " ");
  assert_eq!(cell.style.background, Some(background));
}

#[test]
fn slice_background_fills_blank_cells_and_is_overridden_by_cell_style() {
  let mut layout = LayoutService::new();
  layout.resize_physical(3, 1);
  let slices = SliceService::new();
  let mut pool = UiObjectPool::new();
  let slice = slices
    .create(
      &mut pool,
      SliceOptions {
        rect: SliceRect {
          x: 0,
          y: 0,
          width: SliceLength::Fixed(2),
          height: SliceLength::Fixed(1),
        },
        background: Some(TextColor::Terminal(TerminalColor::Blue)),
        ..Default::default()
      },
    )
    .unwrap();
  let mut canvas = CanvasService::new();
  canvas.begin_frame(&layout);
  pool.prepare_canvas(&mut canvas, &layout);
  canvas.styled_text_on(
    slice,
    1,
    0,
    "X",
    TextStyle {
      background: Some(TextColor::Terminal(TerminalColor::Red)),
      ..Default::default()
    },
  );

  let frame = FrameCompositor::new().compose(&canvas);
  let ComposedCell::Text(blank) = frame.get(0, 0).unwrap() else {
    panic!("expected text cell")
  };
  let ComposedCell::Text(written) = frame.get(1, 0).unwrap() else {
    panic!("expected text cell")
  };
  assert_eq!(
    blank.style.background,
    Some(TextColor::Terminal(TerminalColor::Blue))
  );
  assert_eq!(
    written.style.background,
    Some(TextColor::Terminal(TerminalColor::Red))
  );
}

#[test]
fn slice_state_changes_apply_on_next_prepare() {
  let mut layout = LayoutService::new();
  layout.resize_physical(3, 1);
  let slices = SliceService::new();
  let mut pool = UiObjectPool::new();
  let slice = slices.create(&mut pool, SliceOptions::default()).unwrap();
  let mut canvas = CanvasService::new();
  canvas.begin_frame(&layout);
  pool.prepare_canvas(&mut canvas, &layout);
  slices.set_visible(&mut pool, slice, false);
  assert!(canvas.styled_text_on(slice, 0, 0, "A", TextStyle::default()));
  assert_eq!(text(&FrameCompositor::new().compose(&canvas), 0, 0), "A");

  canvas.begin_frame(&layout);
  pool.prepare_canvas(&mut canvas, &layout);
  assert!(!canvas.styled_text_on(slice, 0, 0, "B", TextStyle::default()));
  assert_eq!(text(&FrameCompositor::new().compose(&canvas), 0, 0), " ");
}

#[test]
fn scroll_box_clips_content_by_scroll_y_and_draws_scrollbar() {
  let mut layout = LayoutService::new();
  layout.resize_physical(8, 4);
  let service = ScrollBoxService::new();
  let mut pool = UiObjectPool::new();
  let id = service
    .create(
      &mut pool,
      ScrollBoxOptions {
        rect: tg_service_layout::Rect {
          x: 0,
          y: 0,
          width: 6,
          height: 2,
        },
        content_width: 6,
        content_height: 4,
        ..Default::default()
      },
    )
    .unwrap();
  service.scroll_to(&mut pool, id, 0, 1, &layout);
  let mut canvas = CanvasService::new();
  canvas.begin_frame(&layout);
  pool.prepare_canvas(&mut canvas, &layout);
  canvas.styled_text_in_scroll_box(id, 0, 0, "row0", TextStyle::default());
  canvas.styled_text_in_scroll_box(id, 0, 1, "row1", TextStyle::default());
  canvas.styled_text_in_scroll_box(id, 0, 2, "row2", TextStyle::default());

  let frame = FrameCompositor::new().compose(&canvas);

  assert_eq!(text(&frame, 0, 0), "r");
  assert_eq!(text(&frame, 3, 0), "1");
  assert_eq!(text(&frame, 3, 1), "2");
  assert_eq!(text(&frame, 5, 0), "█");
}

#[test]
fn scrollbars_share_the_uncovered_viewport_without_hiding_overflow() {
  let mut layout = LayoutService::new();
  layout.resize_physical(4, 3);
  let service = ScrollBoxService::new();
  let mut pool = UiObjectPool::new();
  let id = service
    .create(
      &mut pool,
      ScrollBoxOptions {
        rect: tg_service_layout::Rect {
          x: 0,
          y: 0,
          width: 4,
          height: 3,
        },
        content_width: 4,
        content_height: 4,
        overflow_x: Overflow::Auto,
        scrollbar: ScrollbarPolicy {
          vertical: ScrollbarVisibility::Auto,
          horizontal: ScrollbarVisibility::Auto,
        },
        ..Default::default()
      },
    )
    .unwrap();
  let mut canvas = CanvasService::new();
  canvas.begin_frame(&layout);
  pool.prepare_canvas(&mut canvas, &layout);
  canvas.styled_text_in_scroll_box(id, 0, 0, "ABCD", TextStyle::default());

  let frame = FrameCompositor::new().compose(&canvas);

  assert_eq!(text(&frame, 0, 0), "A");
  assert_eq!(text(&frame, 1, 0), "B");
  assert_eq!(text(&frame, 2, 0), "C");
  assert_eq!(text(&frame, 3, 0), "█");
  assert_eq!(text(&frame, 3, 1), "│");
  assert_eq!(text(&frame, 0, 2), "█");
  assert_eq!(text(&frame, 1, 2), "█");
  assert_eq!(text(&frame, 2, 2), "─");
  assert_eq!(text(&frame, 3, 2), " ");
}

#[test]
fn scroll_box_preserves_wide_character_continuations() {
  let mut layout = LayoutService::new();
  layout.resize_physical(4, 1);
  let service = ScrollBoxService::new();
  let mut pool = UiObjectPool::new();
  let id = service
    .create(
      &mut pool,
      ScrollBoxOptions {
        rect: tg_service_layout::Rect {
          x: 0,
          y: 0,
          width: 4,
          height: 1,
        },
        content_width: 4,
        content_height: 1,
        ..Default::default()
      },
    )
    .unwrap();
  let mut canvas = CanvasService::new();
  canvas.begin_frame(&layout);
  pool.prepare_canvas(&mut canvas, &layout);
  canvas.styled_text_in_scroll_box(id, 0, 0, "中文", TextStyle::default());

  let frame = FrameCompositor::new().compose(&canvas);
  assert_eq!(text(&frame, 0, 0), "中");
  assert!(matches!(
    frame.get(1, 0),
    Some(ComposedCell::Text(cell)) if cell.is_continuation()
  ));
  assert_eq!(text(&frame, 2, 0), "文");
  assert!(matches!(
    frame.get(3, 0),
    Some(ComposedCell::Text(cell)) if cell.is_continuation()
  ));
}

#[test]
fn scroll_box_and_slice_share_surface_order() {
  let mut layout = LayoutService::new();
  layout.resize_physical(4, 2);
  let slices = SliceService::new();
  let scroll = ScrollBoxService::new();
  let mut pool = UiObjectPool::new();
  let slice = slices.create(&mut pool, SliceOptions::default()).unwrap();
  let box_id = scroll
    .create(
      &mut pool,
      ScrollBoxOptions {
        rect: tg_service_layout::Rect {
          x: 0,
          y: 0,
          width: 1,
          height: 1,
        },
        content_width: 1,
        content_height: 1,
        ..Default::default()
      },
    )
    .unwrap();
  scroll.move_below(&mut pool, box_id, SurfaceId::Slice(slice));
  let mut canvas = CanvasService::new();
  canvas.begin_frame(&layout);
  pool.prepare_canvas(&mut canvas, &layout);
  canvas.styled_text_on(slice, 0, 0, "S", TextStyle::default());
  canvas.styled_text_in_scroll_box(box_id, 0, 0, "B", TextStyle::default());

  assert_eq!(text(&FrameCompositor::new().compose(&canvas), 0, 0), "S");

  scroll.move_above(&mut pool, box_id, SurfaceId::Slice(slice));
  canvas.begin_frame(&layout);
  pool.prepare_canvas(&mut canvas, &layout);
  canvas.styled_text_on(slice, 0, 0, "S", TextStyle::default());
  canvas.styled_text_in_scroll_box(box_id, 0, 0, "B", TextStyle::default());

  assert_eq!(text(&FrameCompositor::new().compose(&canvas), 0, 0), "B");
}
