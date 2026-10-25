//! Verify editing configuration and original input geometry independently of Lua and the host.

use super::types::TextAlign;
use super::*;
use crate::{HitAreaService, UiEvent, UiObjectPool};
use tg_core_input::{MouseButton, MouseEvent, MouseEventKind, TerminalKeyCode, TerminalKeyEvent};
use tg_service_canvas::CanvasService;
use tg_service_clipboard::ClipboardService;
use tg_service_layout::{LayoutService, Rect};

fn setup() -> (UiObjectPool, TextInputService, CanvasService, LayoutService) {
  let mut layout = LayoutService::new();
  layout.resize_physical(20, 10);
  let pool = UiObjectPool::new();
  let mut canvas = CanvasService::new();
  pool.prepare_canvas(&mut canvas, &layout);
  (pool, TextInputService::new(), canvas, layout)
}
fn press(x: u16, y: u16) -> MouseEvent {
  MouseEvent {
    kind: MouseEventKind::Press,
    button: Some(MouseButton::Left),
    scroll: None,
    x,
    y,
  }
}
fn key(code: TerminalKeyCode, ctrl: bool) -> TerminalKeyEvent {
  TerminalKeyEvent {
    code,
    ctrl,
    shift: false,
  }
}

#[test]
fn configure_preserves_selection_and_applies_submit_mode_and_character_limit() {
  let (mut pool, mut input, _, _) = setup();
  let options = TextInputOptions {
    initial_text: "ab".into(),
    mouse: true,
    ..Default::default()
  };
  let id = input.create(&mut pool, options.clone());
  assert!(input.focus(&mut pool, id));
  let mut clipboard = ClipboardService::new();
  input.route_terminal_key(&mut pool, &mut clipboard, key(TerminalKeyCode::Home, false));
  assert_eq!(input.cursor(&pool, id), Some(0));
  input.route_terminal_key(&mut pool, &mut clipboard, key(TerminalKeyCode::End, false));
  assert_eq!(input.cursor(&pool, id), Some(2));
  input.route_terminal_key(
    &mut pool,
    &mut clipboard,
    key(TerminalKeyCode::Char('a'), true),
  );
  assert_eq!(input.selection(&pool, id), Some(0..2));
  assert!(input.configure(
    &mut pool,
    id,
    TextInputOptions {
      mode: TextInputMode::MultiLine,
      ..options
    },
    false
  ));
  assert!(input.is_focused(&pool, id));
  assert_eq!(input.selection(&pool, id), Some(0..2));
  input.route_terminal_key(
    &mut pool,
    &mut clipboard,
    key(TerminalKeyCode::Enter, false),
  );
  assert_eq!(input.get_text(&pool, id), Some("\n"));
  input.route_terminal_key(&mut pool, &mut clipboard, key(TerminalKeyCode::Enter, true));
  let mut submitted = false;
  while let Some(event) = pool.pop_event() {
    submitted |=
      matches!(event,UiEvent::TextInput(TextInputEvent::Submit{value,..}) if value=="\n");
  }
  assert!(submitted);
  assert!(input.configure(
    &mut pool,
    id,
    TextInputOptions {
      initial_text: "e\u{301}\n👨‍👩abc".into(),
      max_chars: Some(2),
      ..Default::default()
    },
    true
  ));
  assert_eq!(input.get_text(&pool, id), Some("e\u{301}👨‍👩"));
  input.insert_text(&mut pool, "ignored");
  assert_eq!(input.get_text(&pool, id), Some("e\u{301}👨‍👩"));
  assert!(!input.configure(
    &mut pool,
    TextInputId(999),
    TextInputOptions::default(),
    true
  ));
}

#[test]
fn negative_coordinates_clip_without_reflow_and_pointer_maps_to_original_text() {
  let (mut pool, mut input, mut canvas, _) = setup();
  let id = input.create(
    &mut pool,
    TextInputOptions {
      initial_text: "abcd\nef".into(),
      mode: TextInputMode::MultiLine,
      mouse: true,
      ..Default::default()
    },
  );
  let params = TextInputRenderParams {
    rect: Rect {
      x: 0,
      y: 0,
      width: 4,
      height: 2,
    },
    ..Default::default()
  };
  input.render_at(&mut pool, id, None, (-1, -1), &params, &mut canvas);
  assert_eq!(canvas.base_buffer().get(0, 0).unwrap().text, "f");
  assert!(HitAreaService::new().route_mouse_event(&mut pool, &mut input, &canvas, press(0, 0)));
  assert!(input.focus(&mut pool, id));
  assert_eq!(input.cursor(&pool, id), Some(6));
  for position in [(i32::MIN, 0), (i32::MAX, i32::MAX)] {
    assert!(
      input
        .render_at(&mut pool, id, None, position, &params, &mut canvas)
        .is_none()
    );
    assert!(pool.text_inputs.inputs[&id].hit.is_none());
  }
}

#[test]
fn aligned_multiline_pointer_matches_rendered_text_and_custom_cursor_can_be_hidden() {
  let (mut pool, mut input, mut canvas, layout) = setup();
  let id = input.create(
    &mut pool,
    TextInputOptions {
      initial_text: "abcd\nxy".into(),
      mode: TextInputMode::MultiLine,
      mouse: true,
      ..Default::default()
    },
  );
  let params = TextInputRenderParams {
    rect: Rect {
      x: 0,
      y: 0,
      width: 8,
      height: 5,
    },
    vertical_align: VerticalAlign::Center,
    text_align: TextAlign::Right,
    ..Default::default()
  };
  input.render_at(&mut pool, id, None, (0, 0), &params, &mut canvas);
  assert_eq!(canvas.base_buffer().get(2, 2).unwrap().text, "x");
  assert_eq!(canvas.base_buffer().get(3, 2).unwrap().text, "y");
  HitAreaService::new().route_mouse_event(&mut pool, &mut input, &canvas, press(3, 2));
  assert!(input.focus(&mut pool, id));
  assert_eq!(input.cursor(&pool, id), Some(6));
  input.blur(&mut pool);
  let empty = input.create(&mut pool, TextInputOptions::default());
  input.focus(&mut pool, empty);
  pool.prepare_canvas(&mut canvas, &layout);
  let mut params = TextInputRenderParams {
    rect: Rect {
      x: 0,
      y: 0,
      width: 4,
      height: 1,
    },
    cursor_character: Some("|".into()),
    cursor_blink: false,
    ..Default::default()
  };
  input.render(&mut pool, empty, &params, &mut canvas);
  assert_eq!(canvas.base_buffer().get(0, 0).unwrap().text, "|");
  params.cursor_character = None;
  params.cursor_shape = Some(TextInputCursorShape::None);
  pool.prepare_canvas(&mut canvas, &layout);
  input.render(&mut pool, empty, &params, &mut canvas);
  assert_ne!(canvas.base_buffer().get(0, 0).unwrap().text, "|");
}
