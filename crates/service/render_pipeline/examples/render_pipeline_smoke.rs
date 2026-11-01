//! Independent render pipeline smoke entry exercising the public API and checking its results.

use tg_core_style::{ComposedCell, TextStyle};
use tg_service_canvas::CanvasService;
use tg_service_render_pipeline::FrameCompositor;

pub(crate) fn main() {
  let mut canvas = CanvasService::new();
  canvas.styled_text(1, 2, "a", TextStyle::default());

  let frame = FrameCompositor::new().compose(&canvas);

  assert!(matches!(
    frame.get(1, 2),
    Some(ComposedCell::Text(cell)) if cell.text == "a"
  ));
  let compositor = FrameCompositor::new();
  let mut processed = compositor.compose_canvas(&canvas);
  processed.set(1, 2, tg_core_style::CanvasCell::new("b"));
  assert!(canvas.set_composed_canvas(processed));
  let frame = compositor.compose(&canvas);
  assert!(matches!(frame.get(1, 2), Some(ComposedCell::Text(cell)) if cell.text == "b"));
  assert_eq!(canvas.cell_at(1, 2).unwrap().text, "a");
  source_views_preserve_occlusion_and_project_canvas_changes();
  embedded_sources_survive_later_parent_drawing();
  println!("render_pipeline ok: composed cell (1, 2) = \"a\"");
}

fn region(x: i32, width: u16) -> tg_service_canvas::CellEffectRegion {
  tg_service_canvas::CellEffectRegion {
    x,
    y: 0,
    width,
    height: 1,
    include_continuation: false,
  }
}

fn white() -> TextStyle {
  TextStyle {
    foreground: Some(tg_core_style::TextColor::Terminal(
      tg_core_style::TerminalColor::BrightWhite,
    )),
    ..Default::default()
  }
}

fn green() -> tg_core_style::TextColor {
  tg_core_style::TextColor::Terminal(tg_core_style::TerminalColor::Green)
}

fn source_views_preserve_occlusion_and_project_canvas_changes() {
  use tg_service_canvas::{CellSurface, SliceFrame, SliceId, SurfaceFrame};
  use tg_service_layout::{LayoutService, Rect};
  let mut layout = LayoutService::new();
  layout.resize_physical(8, 2);
  let mut canvas = CanvasService::new();
  canvas.begin_frame(&layout);
  canvas.prepare(
    1,
    [(1, 1), (2, 3)]
      .into_iter()
      .map(|(id, x)| {
        SurfaceFrame::Slice(SliceFrame {
          id: SliceId(id),
          rect: Rect {
            x,
            y: 0,
            width: 3,
            height: 1,
          },
          source_x: 0,
          source_y: 0,
          visible: true,
          opaque: false,
          background: None,
          event_passthrough: false,
        })
      })
      .collect(),
    &layout,
  );
  canvas.begin_cell_source_tracking();
  canvas.styled_text(0, 0, "888", white());
  canvas.styled_text_on(SliceId(1), 0, 0, "aaa", white());
  canvas.styled_text_on(SliceId(2), 0, 0, "bbb", white());
  let mut composed = FrameCompositor::new().compose_canvas(&canvas);
  let changes = composed
    .transform_region_with_sources(region(0, 6), |mut cell| {
      cell.cell.style.foreground = Some(green());
      Ok::<_, ()>(cell)
    })
    .unwrap();
  canvas.project_cell_changes(&changes, true);
  let snapshot = canvas.snapshot_cells(&composed);
  assert_eq!(
    snapshot
      .cells(None, region(0, 6))
      .iter()
      .map(|cell| cell.cell.text.as_str())
      .collect::<String>(),
    "8aabbb"
  );
  for (surface, chars, green_indices) in [
    (CellSurface::Base, "888", vec![0]),
    (CellSurface::Slice(SliceId(1)), "aaa", vec![0, 1]),
    (CellSurface::Slice(SliceId(2)), "bbb", vec![0, 1, 2]),
  ] {
    let cells = snapshot.cells(Some(surface), region(0, 3));
    assert_eq!(
      cells
        .iter()
        .map(|cell| cell.cell.text.as_str())
        .collect::<String>(),
      chars
    );
    for (index, cell) in cells.iter().enumerate() {
      assert_eq!(
        cell.cell.style.foreground == Some(green()),
        green_indices.contains(&index)
      );
    }
  }
  let changes = composed
    .transform_region_with_sources(region(1, 1), |mut cell| {
      cell.x = 2;
      cell.cell.text = "Z".into();
      Ok::<_, ()>(cell)
    })
    .unwrap();
  canvas.project_cell_changes(&changes, true);
  let moved = canvas.snapshot_cells(&composed);
  let layer = moved.cells(Some(CellSurface::Slice(SliceId(1))), region(0, 3));
  assert_eq!(
    layer
      .iter()
      .map(|cell| cell.cell.text.as_str())
      .collect::<String>(),
    "Za"
  );
  assert_eq!(layer[0].cell.text, "");
  assert_eq!(layer[1].cell.text, "Z");
  let changes = composed
    .transform_region_with_sources(region(2, 1), |mut cell| {
      cell.x = 7;
      Ok::<_, ()>(cell)
    })
    .unwrap();
  canvas.project_cell_changes(&changes, true);
  let clipped = canvas.snapshot_cells(&composed);
  assert_eq!(
    clipped.cells(Some(CellSurface::Slice(SliceId(1))), region(0, 3))[1]
      .cell
      .text,
    ""
  );
  assert_eq!(clipped.cells(None, region(7, 1))[0].cell.text, "Z");
  assert!(
    clipped
      .cells(Some(CellSurface::Slice(SliceId(1))), region(3, 1))
      .is_empty()
  );
  assert_eq!(
    snapshot.cells(Some(CellSurface::Base), region(0, 3))[0]
      .cell
      .text,
    "8"
  );
}

fn embedded_sources_survive_later_parent_drawing() {
  use tg_service_canvas::{CellSurface, SliceId};
  use tg_service_layout::{LayoutService, Rect};
  let mut layout = LayoutService::new();
  layout.resize_physical(6, 2);
  let mut canvas = CanvasService::new();
  canvas.begin_frame(&layout);
  canvas.prepare(1, vec![], &layout);
  canvas.begin_cell_source_tracking();
  canvas.styled_text(0, 0, "PPP", white());
  let child = SliceId(3);
  canvas.prepare_embedded_slice(
    child,
    Rect {
      x: 0,
      y: 0,
      width: 3,
      height: 1,
    },
    (0, 0),
    1,
    None,
    false,
  );
  canvas.styled_text_on(child, 0, 0, "aaa", white());
  canvas.composite_embedded_slice(child, None);
  canvas.styled_text(1, 0, "X", white());
  let mut composed = FrameCompositor::new().compose_canvas(&canvas);
  let changes = composed
    .transform_region_with_sources(region(0, 3), |mut cell| {
      cell.cell.style.foreground = Some(green());
      Ok::<_, ()>(cell)
    })
    .unwrap();
  canvas.project_cell_changes(&changes, true);
  let snapshot = canvas.snapshot_cells(&composed);
  let base = snapshot.cells(Some(CellSurface::Base), region(0, 3));
  assert_eq!(
    base
      .iter()
      .map(|cell| cell.cell.text.as_str())
      .collect::<String>(),
    "PXP"
  );
  assert_ne!(base[0].cell.style.foreground, Some(green()));
  assert_eq!(base[1].cell.style.foreground, Some(green()));
  let child = snapshot.cells(Some(CellSurface::Slice(child)), region(0, 3));
  assert_eq!(
    child
      .iter()
      .map(|cell| cell.cell.text.as_str())
      .collect::<String>(),
    "aaa"
  );
  assert_eq!(child[0].cell.style.foreground, Some(green()));
  assert_ne!(child[1].cell.style.foreground, Some(green()));
  assert_eq!(child[2].cell.style.foreground, Some(green()));
}
