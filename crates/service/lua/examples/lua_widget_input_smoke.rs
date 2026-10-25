//! Exercise Lua input APIs, ordered editing, clipping, focus events, and cleanup without the host.
//!
//! Run `cargo run -p tg-service-lua --example lua_widget_input_smoke --locked`.
//! Require a writable temporary deployment directory. Input is injected through public APIs;
//! a physical terminal, IME, and system clipboard contents are not exercised.

use std::{
  fs,
  path::PathBuf,
  time::{Duration, SystemTime, UNIX_EPOCH},
};
use tg_core_input::{MouseButton, MouseEvent, MouseEventKind, TerminalKeyCode, TerminalKeyEvent};
use tg_service_canvas::CanvasService;
use tg_service_clipboard::ClipboardService;
use tg_service_layout::{LayoutService, Rect, Size};
use tg_service_lua::{
  LuaDrawCommand, LuaEventBroker, LuaEventData, LuaEventRoute, LuaService, LuaSessionKind,
  LuaSessionSpec, LuaSessionToken,
};

struct Deployment(PathBuf);
impl Drop for Deployment {
  fn drop(&mut self) {
    fs::remove_dir_all(&self.0).expect("remove input deployment");
  }
}

pub(crate) fn main() {
  let nonce = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_nanos();
  let root = Deployment(
    std::env::temp_dir().join(format!("tg-widget-input-{}-{nonce}", std::process::id())),
  );
  fs::create_dir_all(root.0.join("scripts")).unwrap();
  let entry = root.0.join("scripts/main.lua");
  fs::write(&entry, r##"
    function Init(ctx)
      local input = widget.input
      debug.assert(input.SINGLELINE == "singleline" and input.MULTILINE == "multiline")
      debug.assert(input.BLOCK == "█" and input.UNDERLINE == "_" and input.LINE == "▏")
      first = input.create(6, 3, {mouse=true, line_mode=input.MULTILINE, cursor_blink=false})
      debug.assert(first == "input_001")
      local info = input.get_info(first)
      debug.assert(info.enter_submit == false and info.max_chars == nil and info.cursor_shape == "█")
      debug.assert(info.vertical_align == align.AUTO and info.horizontal_align == align.AUTO)
      debug.assert(info.fg == color.NONE and info.bg == color.NONE)
      debug.assert(input.count() == 1 and input.list().n == 1)
      for _, options in ipairs({{max_chars=0}, {max_chars=65536}, {width=0}, {height=1.5},
        {mouse=1}, {line_mode="multi"}, {cursor_shape="ab"}, {cursor_shape=true},
        {horizontal_align=align.TOP}, {vertical_align=align.LEFT}, {unknown=true}}) do
        debug.assert(not debug.pcall(input.set, first, options))
      end
      debug.assert(input.get_info(first).width == 6)
      debug.assert(not debug.pcall(function() input.BLOCK = "X" end))
      debug.assert(not debug.pcall(function() widget.input = {} end))
      debug.assert(input.set(first, {text="你好abc", max_chars=3, fg="#112233", bg=color.BLUE,
        placeholder="type", placeholder_fg=color.GRAY, cursor_shape="|", bold=true, dim=true}))
      info = input.get_info(first)
      debug.assert(info.text == "你好a" and info.max_chars == 3 and info.cursor_shape == "|")
      debug.assert(info.fg == "#112233" and info.bg == color.BLUE and info.bold and info.dim)
      debug.assert(input.set(first, {max_chars=false, cursor_shape=false, text=""}))
      debug.assert(input.get_info(first).max_chars == nil and input.get_info(first).cursor_shape == nil)
      debug.assert(input.set(first, {cursor_shape=input.LINE}))
      second = input.create(2, 1)
      debug.assert(input.get_info(second).enter_submit == true)
      debug.assert(input.delete(second) and not input.exists(second) and input.get_info(second) == nil)
      debug.assert(not input.delete(second) and not input.focus(second) and not input.set(second, {text="x"}))
      debug.assert(not debug.pcall(input.draw, first, 2147483648, 0))
      debug.assert(not debug.pcall(input.draw, first, 0, 0, {slice_layer="slice_999"}))
      owner = slice.create(5, 3)
      overlay = slice.create(1, 1)
      phase, seen, changes = 0, {}, {}
    end
    function Render()
      slice.draw(owner, 0, 0)
      slice.draw(overlay, 0, 0)
      widget.input.draw(first, -1, 0, {slice_layer=owner})
    end
    function HandleEvent(event)
      debug.assert(event.type == "input" and event.data.id == first)
      local data = event.data
      seen[data.kind] = (seen[data.kind] or 0) + 1
      if data.kind == "focus" then debug.assert(widget.input.focus(first)) end
      if data.kind == "change" then table.insert(changes, data.text) end
      if data.kind == "submit" then debug.assert(data.text == "a\nb" or data.text == "a\nb\n") end
    end
    function Update(dt)
      phase = phase + 1
      if phase == 1 then
        slice.set(overlay, {event_passthrough=true})
      elseif phase == 2 then
        debug.assert(seen.focus == 1 and seen.focused == 1)
        debug.assert(changes[#changes] == "a\nb")
        debug.assert(widget.input.get_info(first).text == "a\nb")
        debug.assert(seen.submit == 1 and seen.cancel == 1)
        debug.assert(widget.input.set(first, {enter_submit=true}))
      elseif phase == 3 then
        debug.assert(widget.input.get_info(first).text == "a\nb\n")
        debug.assert(seen.submit == 2)
        debug.assert(widget.input.set(first, {line_mode=widget.input.SINGLELINE, max_chars=2}))
        debug.assert(widget.input.get_info(first).text == "ab")
        debug.assert(widget.input.blur())
      elseif phase == 4 then
        debug.assert(seen.blurred == 1)
        debug.assert(widget.input.focus(first))
        debug.assert(widget.input.delete(first))
        debug.assert(widget.input.count() == 0)
        debug.assert(widget.input.clear())
        debug.assert(widget.input.list().n == 0)
      end
    end
    function UpdateFrame(dt, alpha) end
  "##).unwrap();
  let mut session = LuaService::new()
    .create_session(LuaSessionSpec {
      package_id: "widget_input_smoke".into(),
      session_kind: LuaSessionKind::Game,
      entry_path: entry,
      fixed_delta: Duration::from_secs_f64(1.0 / 60.0),
      base_size: Size {
        width: 12,
        height: 6,
      },
      continue_data: None,
      best_data: None,
      save_game_enabled: false,
      save_best_enabled: false,
    })
    .unwrap();
  let token = LuaSessionToken {
    kind: LuaSessionKind::Game,
    generation: 1,
  };
  let mut broker = LuaEventBroker::new();
  broker.synchronize_sessions(Some(token), None);
  // Discard setup-time changes; later assertions concern interactive edits.
  session
    .with_objects_mut(|objects| objects.take_widget_events())
    .unwrap();
  let mut canvas = CanvasService::new();
  let mut layout = LayoutService::new();
  layout.resize_physical(12, 8);
  layout.set_developer_viewport(Rect {
    x: 0,
    y: 2,
    width: 12,
    height: 6,
  });
  let mut clipboard = ClipboardService::new();
  for phase in 0..=1 {
    session
      .with_objects_mut(|objects| objects.begin_frame())
      .unwrap();
    session.render().unwrap();
    let commands = session.take_draw_commands();
    canvas.begin_frame(&layout);
    session
      .with_objects_mut(|objects| {
        objects.ui_mut().begin_render();
        objects.ui().prepare_canvas(&mut canvas, &layout);
        for command in commands {
          if let LuaDrawCommand::Input { id, target, x, y } = command {
            objects.draw_input(id, target, (x, y), &mut canvas);
          }
        }
      })
      .unwrap();
    let events = session
      .with_objects_mut(|objects| {
        objects.route_widget_mouse(
          &canvas,
          MouseEvent {
            kind: MouseEventKind::Press,
            button: Some(MouseButton::Left),
            scroll: None,
            x: 0,
            y: 2,
          },
        )
      })
      .unwrap();
    assert_eq!(
      events
        .iter()
        .filter(|event| matches!(event,LuaEventData::TextInput(data) if data.kind=="focus"))
        .count(),
      phase
    );
    deliver(&mut session, &mut broker, token, events);
    let events = session
      .with_objects_mut(|objects| objects.take_widget_events())
      .unwrap();
    deliver(&mut session, &mut broker, token, events);
    if phase == 0 {
      session.update().unwrap();
    }
  }
  assert!(!session.input_method_locked());
  assert!(
    session
      .input_generation(&LuaEventData::Input {
        text: "ignored".into()
      })
      .is_none()
  );
  session
    .with_objects_mut(|objects| {
      objects.route_input_text("a");
      objects.route_input_key(&mut clipboard, key(TerminalKeyCode::Char('a'), false));
      objects.route_input_key(&mut clipboard, key(TerminalKeyCode::Enter, false));
      objects.route_input_text("bc");
      objects.route_input_key(&mut clipboard, key(TerminalKeyCode::Backspace, false));
      objects.route_input_key(&mut clipboard, key(TerminalKeyCode::Enter, true));
      objects.route_input_key(&mut clipboard, key(TerminalKeyCode::Esc, false));
    })
    .unwrap();
  flush(&mut session, &mut broker, token);
  session.update().unwrap();
  session
    .with_objects_mut(|objects| {
      objects.route_input_key(&mut clipboard, key(TerminalKeyCode::Enter, true));
      objects.route_input_key(&mut clipboard, key(TerminalKeyCode::Enter, false));
    })
    .unwrap();
  flush(&mut session, &mut broker, token);
  session.update().unwrap();
  flush(&mut session, &mut broker, token);
  session.update().unwrap();
  let stale = LuaEventData::TextInput(tg_service_lua::LuaTextInputEvent {
    id: 1,
    kind: "change",
    value: Some("stale".into()),
  });
  session.update().unwrap();
  deliver(&mut session, &mut broker, token, vec![stale]);
  assert!(
    !session
      .with_objects(|objects| objects.input_is_focused())
      .unwrap()
  );
  flush(&mut session, &mut broker, token);
  println!(
    "widget.input ok: lifecycle, strict configuration, sliced pointer focus, ordered editing, events and cleanup"
  );
}
fn key(code: TerminalKeyCode, ctrl: bool) -> TerminalKeyEvent {
  TerminalKeyEvent {
    code,
    ctrl,
    shift: false,
  }
}
fn flush(
  session: &mut tg_service_lua::LuaSession,
  broker: &mut LuaEventBroker,
  token: LuaSessionToken,
) {
  let events = session
    .with_objects_mut(|objects| objects.take_widget_events())
    .unwrap();
  deliver(session, broker, token, events);
}
fn deliver(
  session: &mut tg_service_lua::LuaSession,
  broker: &mut LuaEventBroker,
  token: LuaSessionToken,
  events: Vec<LuaEventData>,
) {
  for data in events {
    broker
      .push_owned(token, 1, data, LuaEventRoute::HandleEvent)
      .unwrap();
  }
  for event in broker.drain_frame(LuaSessionKind::Game) {
    session.dispatch_event(&event).unwrap();
  }
}
