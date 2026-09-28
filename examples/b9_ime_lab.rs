use std::error::Error;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use tg_service_canvas::CanvasService;
use tg_service_clipboard::ClipboardService;
use tg_service_input::{TerminalKeyCode, TerminalKeyEvent};
use tg_service_layout::{LayoutService, Rect};
use tg_service_render_pipeline::{FrameCompositor, FramePresenter};
use tg_service_terminal::TerminalService;
use tg_service_text_layout::DrawTextParams;
use tg_service_widget::{
  TextInputEvent, TextInputMode, TextInputOptions, TextInputRenderParams, TextInputService,
  UiEvent, UiObjectPool,
};

const FRAME_INTERVAL: Duration = Duration::from_millis(16);

#[derive(Default)]
struct LabState {
  page: usize,
  overlay: bool,
  moving: bool,
  movement_step: usize,
  last_movement: Option<Instant>,
  submit_count: usize,
  cancel_count: usize,
  window_active: bool,
  exit: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
  run()
}

fn run() -> Result<(), Box<dyn Error>> {
  let mut terminal = TerminalService::new();
  terminal.enter()?;

  let mut layout = LayoutService::new();
  let mut canvas = CanvasService::new();
  let mut compositor = FrameCompositor::new();
  let mut presenter = FramePresenter::new();
  let mut render_clock = TextInputService::new();
  let mut pool = UiObjectPool::new();
  let mut clipboard = ClipboardService::new();
  let input_id = render_clock.create(
    &mut pool,
    TextInputOptions {
      initial_text: "prefix-".to_string(),
      mode: TextInputMode::SingleLine,
      mouse: false,
      ..Default::default()
    },
  );
  render_clock.focus(&mut pool, input_id);

  let mut state = LabState {
    window_active: true,
    ..Default::default()
  };
  let started = Instant::now();

  while !state.exit {
    if event::poll(FRAME_INTERVAL)? {
      handle_event(
        event::read()?,
        &mut state,
        &mut render_clock,
        &mut pool,
        &mut clipboard,
        input_id,
      );
      while event::poll(Duration::ZERO)? {
        handle_event(
          event::read()?,
          &mut state,
          &mut render_clock,
          &mut pool,
          &mut clipboard,
          input_id,
        );
      }
    }

    advance_movement(&mut state);
    render_frame(
      &mut terminal,
      &mut layout,
      &mut canvas,
      &mut compositor,
      &mut presenter,
      &render_clock,
      &mut pool,
      input_id,
      &state,
      started.elapsed(),
    )?;
  }

  terminal.exit();
  Ok(())
}

fn handle_event(
  event: Event,
  state: &mut LabState,
  text_input: &mut TextInputService,
  pool: &mut UiObjectPool,
  clipboard: &mut ClipboardService,
  input_id: tg_service_widget::TextInputId,
) {
  match event {
    Event::FocusGained => state.window_active = true,
    Event::FocusLost => state.window_active = false,
    Event::Resize(_, _) => {}
    Event::Key(key) if key.kind != KeyEventKind::Release => {
      if handle_control_key(key, state, text_input, pool, input_id) {
        return;
      }
      if let Some(key) = terminal_key(key) {
        text_input.route_terminal_key(pool, clipboard, key);
      }
      collect_input_events(state, pool);
    }
    Event::Key(_) | Event::Mouse(_) | Event::Paste(_) => {}
  }
}

fn handle_control_key(
  key: KeyEvent,
  state: &mut LabState,
  text_input: &mut TextInputService,
  pool: &mut UiObjectPool,
  input_id: tg_service_widget::TextInputId,
) -> bool {
  let command_chord = KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SHIFT;
  let action = if key.modifiers.contains(command_chord) {
    match key.code {
      KeyCode::Char('p' | 'P') => Some('p'),
      KeyCode::Char('o' | 'O') => Some('o'),
      KeyCode::Char('m' | 'M') => Some('m'),
      KeyCode::Char('l' | 'L') => Some('l'),
      KeyCode::Char('q' | 'Q') => Some('q'),
      _ => None,
    }
  } else if key.code == KeyCode::F(10) {
    Some('q')
  } else {
    None
  };

  match action {
    Some('p') => {
      state.page = (state.page + 1) % 3;
      state.moving = false;
    }
    Some('o') => state.overlay = !state.overlay,
    Some('m') => {
      state.moving = !state.moving;
      state.last_movement = Some(Instant::now());
    }
    Some('l') => {
      text_input.clear(pool, input_id);
      state.submit_count = 0;
      state.cancel_count = 0;
    }
    Some('q') => state.exit = true,
    _ => return false,
  }
  true
}

fn terminal_key(key: KeyEvent) -> Option<TerminalKeyEvent> {
  let code = match key.code {
    KeyCode::Char(ch) => TerminalKeyCode::Char(ch),
    KeyCode::Enter => TerminalKeyCode::Enter,
    KeyCode::Esc => TerminalKeyCode::Esc,
    KeyCode::Backspace => TerminalKeyCode::Backspace,
    KeyCode::Delete => TerminalKeyCode::Delete,
    KeyCode::Left => TerminalKeyCode::Left,
    KeyCode::Right => TerminalKeyCode::Right,
    KeyCode::Up => TerminalKeyCode::Up,
    KeyCode::Down => TerminalKeyCode::Down,
    KeyCode::Home => TerminalKeyCode::Home,
    KeyCode::End => TerminalKeyCode::End,
    _ => return None,
  };
  Some(TerminalKeyEvent {
    code,
    ctrl: key.modifiers.contains(KeyModifiers::CONTROL),
    shift: key.modifiers.contains(KeyModifiers::SHIFT),
  })
}

fn collect_input_events(state: &mut LabState, pool: &mut UiObjectPool) {
  while let Some(event) = pool.pop_event() {
    match event {
      UiEvent::TextInput(TextInputEvent::Submit { .. }) => state.submit_count += 1,
      UiEvent::TextInput(TextInputEvent::Cancel { .. }) => state.cancel_count += 1,
      _ => {}
    }
  }
}

fn advance_movement(state: &mut LabState) {
  if !state.moving {
    return;
  }
  let now = Instant::now();
  let should_advance = state
    .last_movement
    .is_none_or(|last| now.duration_since(last) >= Duration::from_millis(700));
  if should_advance {
    state.movement_step = (state.movement_step + 1) % 10;
    state.last_movement = Some(now);
  }
}

#[allow(clippy::too_many_arguments)]
fn render_frame(
  terminal: &mut TerminalService,
  layout: &mut LayoutService,
  canvas: &mut CanvasService,
  compositor: &mut FrameCompositor,
  presenter: &mut FramePresenter,
  text_input: &TextInputService,
  pool: &mut UiObjectPool,
  input_id: tg_service_widget::TextInputId,
  state: &LabState,
  elapsed: Duration,
) -> std::io::Result<()> {
  let (width, height) = crossterm::terminal::size()?;
  layout.resize_physical(width, height);
  canvas.begin_frame(layout);
  pool.begin_render();

  let safe_width = width.max(1);
  let safe_height = height.max(1);
  draw_line(
    canvas,
    safe_width,
    0,
    "B9 IME anchor lab — live host TextInputService / FramePresenter",
  );
  draw_line(
    canvas,
    safe_width,
    1,
    "Type Chinese Pinyin or Japanese Romaji; use arrows, Enter to commit, Esc to cancel.",
  );
  draw_line(
    canvas,
    safe_width,
    2,
    "Ctrl+Alt+Shift+P page  O overlay  M scroll  L clear  Q exit (F10 also exits)",
  );

  let input_y = (if state.moving {
    let scroll_offset = if state.movement_step <= 5 {
      state.movement_step
    } else {
      10 - state.movement_step
    };
    12u16.saturating_sub(scroll_offset as u16)
  } else {
    match state.page {
      0 => 6,
      1 => safe_height / 2,
      _ => safe_height.saturating_sub(5),
    }
  })
  .min(safe_height.saturating_sub(2));

  if state.moving {
    for row in 0..safe_height.saturating_sub(8).min(12) {
      let y = 6 + row;
      draw_line(
        canvas,
        safe_width,
        y,
        &format!(
          "scroll item {:02} — live page content",
          state.movement_step + usize::from(row)
        ),
      );
    }
  }

  if state.overlay {
    let overlay_y = if input_y <= 6 {
      input_y.saturating_add(2).min(safe_height.saturating_sub(1))
    } else {
      input_y.saturating_sub(2)
    };
    draw_line(
      canvas,
      safe_width,
      overlay_y,
      "[ overlay active — the same input remains focused ]",
    );
  }

  let tick = elapsed.as_millis() / 250;
  draw_line(
    canvas,
    safe_width,
    4,
    &format!(
      "Page {} | overlay {} | moving {} | tick {} | focus {} | submit {} | cancel {}",
      state.page + 1,
      on_off(state.overlay),
      on_off(state.moving),
      tick,
      on_off(state.window_active),
      state.submit_count,
      state.cancel_count,
    ),
  );

  let input_x = 2.min(safe_width.saturating_sub(1));
  let input_width = safe_width.saturating_sub(input_x).saturating_sub(2).max(1);
  draw_line(
    canvas,
    safe_width,
    input_y.saturating_sub(1),
    "Input caret follows the moving row:",
  );
  let cursor = text_input.render_host(
    pool,
    input_id,
    &TextInputRenderParams {
      rect: Rect {
        x: input_x,
        y: input_y,
        width: input_width,
        height: 1,
      },
      placeholder: "type here…".to_string(),
      ..Default::default()
    },
    canvas,
  );

  let frame = compositor.compose(canvas);
  let force_redraw = canvas.take_render_requested();
  presenter.present(
    &frame,
    terminal,
    force_redraw,
    state.window_active.then_some(cursor).flatten(),
  )
}

fn draw_line(canvas: &mut CanvasService, width: u16, y: u16, text: &str) {
  canvas.host_text(&DrawTextParams {
    x: 1,
    y,
    text: text.to_string(),
    max_width: Some(width.saturating_sub(2).max(1)),
    max_height: Some(1),
    ..Default::default()
  });
}

fn on_off(value: bool) -> &'static str {
  if value { "on" } else { "off" }
}
