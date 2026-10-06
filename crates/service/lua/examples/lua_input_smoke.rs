//! Exercise ordered keyboard input, Lua subscriptions, and balanced focus releases.
//!
//! Run `cargo run -p tg-service-lua --example lua_input_smoke --locked`.
//! A temporary deployment supplies the script; input is injected through the public input
//! service without requiring a terminal device. Pass an optional parent deployment directory
//! after `--` to place the temporary deployment there. This does not replace physical keyboard tests.

use std::{
  fs,
  path::PathBuf,
  time::{Duration, SystemTime, UNIX_EPOCH},
};
use tg_core_input::{FocusEvent, Key, KeyBinding, KeyEvent, KeyEventKind, SystemEvent, key_token};
use tg_service_input::{InputNotification, InputService};
use tg_service_layout::Size;
use tg_service_log::LogService;
use tg_service_lua::{
  LuaEventBroker, LuaEventData, LuaEventRoute, LuaService, LuaSession, LuaSessionKind,
  LuaSessionSpec, LuaSessionToken,
};

struct Deployment(PathBuf);
impl Drop for Deployment {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
}

fn main() {
  let nonce = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_nanos();
  let parent = std::env::args_os()
    .nth(1)
    .map(PathBuf::from)
    .unwrap_or_else(std::env::temp_dir);
  let root = Deployment(parent.join(format!("tg_input_lua_{}_{nonce}", std::process::id())));
  fs::create_dir_all(root.0.join("scripts")).unwrap();
  let entry = root.0.join("scripts/main.lua");
  fs::write(
    &entry,
    r#"
    local seen = {}
    function Init(ctx)
      debug.assert(keyboard.receive_action_event())
      debug.assert(keyboard.receive_key_event())
    end
    function HandleEvent(e)
      if e.type == "action" or e.type == "key" then
        seen[#seen+1] = e.type .. ":" .. (e.data.action or e.data.key) .. ":" .. e.data.state
      elseif e.type == "focus" then
        seen[#seen+1] = "focus:" .. tostring(e.data.gained)
      elseif e.type == "overlay_started" or e.type == "overlay_stopped" then
        seen[#seen+1] = e.type
      elseif e.type == "resize" then
        local fn = e.data.width == 1 and events.disable_focus_release or events.enable_focus_release
        debug.assert(fn())
        debug.assert(fn())
      end
    end
    function SaveGame() return {seen = seen} end
    function Update(dt) end
    function UpdateFrame(dt, alpha) end
    function Render() end
  "#,
  )
  .unwrap();
  let mut session = LuaService::new()
    .create_session(LuaSessionSpec {
      package_id: "input_smoke".into(),
      session_kind: LuaSessionKind::Game,
      entry_path: entry,
      fixed_delta: Duration::from_secs_f64(1.0 / 60.0),
      base_size: Size {
        width: 80,
        height: 24,
      },
      continue_data: None,
      best_data: None,
      save_game_enabled: true,
      save_best_enabled: false,
    })
    .unwrap();
  let token = LuaSessionToken {
    kind: LuaSessionKind::Game,
    generation: 1,
  };
  let mut broker = LuaEventBroker::new();
  broker.synchronize_sessions(Some(token), None);
  let mut input = InputService::new();
  let mut log = LogService::new();
  let binding = |action: &str, key, priority| KeyBinding {
    action: action.into(),
    pattern: tg_core_input::KeyPattern::Single(key),
    priority,
  };
  input.load_key_bindings(vec![
    binding("pause", Key::Esc, 0),
    binding("exit", Key::Esc, 10),
  ]);
  input.load_system_key_bindings(vec![binding("host", Key::Esc, 0)]);
  input.begin_frame();
  input.queue_key_event(
    KeyEvent {
      key: Key::Esc,
      kind: KeyEventKind::Press,
    },
    &mut log,
  );
  input.poll();
  route(&input, &mut session, &mut broker, token, 1);
  deliver(&mut session, &mut broker);
  input.begin_frame();
  input.poll();
  route(&input, &mut session, &mut broker, token, 2);
  deliver(&mut session, &mut broker);
  input.begin_frame();
  input.poll();
  assert!(input.notifications().is_empty());
  input.queue_system_event(SystemEvent::Focus(FocusEvent { gained: false }), &mut log);
  input.poll();
  route(&input, &mut session, &mut broker, token, 3);
  deliver(&mut session, &mut broker);
  assert_eq!(
    session.save_game().unwrap().unwrap()["seen"],
    serde_json::json!([
      "key:esc:pressed",
      "action:exit:pressed",
      "action:pause:pressed",
      "key:esc:held",
      "action:exit:held",
      "action:pause:held",
      "key:esc:released",
      "action:exit:released",
      "action:pause:released",
      "focus:false"
    ])
  );
  input.begin_frame();
  input.queue_system_event(SystemEvent::Focus(FocusEvent { gained: true }), &mut log);
  input.queue_key_event(
    KeyEvent {
      key: Key::Esc,
      kind: KeyEventKind::Press,
    },
    &mut log,
  );
  input.poll();
  assert_eq!(
    input.notifications(),
    [InputNotification::Focus { gained: true }]
  );
  input.queue_key_event(
    KeyEvent {
      key: Key::Esc,
      kind: KeyEventKind::Release,
    },
    &mut log,
  );
  input.poll();
  input.load_key_bindings(vec![
    binding("exit", Key::Esc, 10),
    binding("pause", Key::Space, 0),
  ]);
  input.begin_frame();
  input.queue_key_event(
    KeyEvent {
      key: Key::Space,
      kind: KeyEventKind::Press,
    },
    &mut log,
  );
  input.poll();
  assert_eq!(
    input
      .collect_action_events()
      .iter()
      .map(|e| e.action.as_str())
      .collect::<Vec<_>>(),
    ["pause"]
  );
  route(&input, &mut session, &mut broker, token, 4);
  // An undelivered press must not create an orphan release at an overlay boundary.
  assert!(session.focus_lost_input().is_empty());
  broker.clear_pending_interactive(LuaSessionKind::Game);
  deliver(&mut session, &mut broker);
  assert!(session.close_input(true, true).is_empty());
  broker
    .push_system(
      5,
      LuaEventData::Resize {
        width: 1,
        height: 24,
      },
    )
    .unwrap();
  deliver(&mut session, &mut broker);
  input.begin_frame();
  input.queue_key_event(
    KeyEvent {
      key: Key::Space,
      kind: KeyEventKind::Release,
    },
    &mut log,
  );
  input.queue_key_event(
    KeyEvent {
      key: Key::Esc,
      kind: KeyEventKind::Press,
    },
    &mut log,
  );
  input.poll();
  route(&input, &mut session, &mut broker, token, 5);
  deliver(&mut session, &mut broker);
  let before_focus = session.save_game().unwrap().unwrap()["seen"]
    .as_array()
    .unwrap()
    .len();
  input.begin_frame();
  input.queue_system_event(SystemEvent::Focus(FocusEvent { gained: false }), &mut log);
  input.poll();
  route(&input, &mut session, &mut broker, token, 6);
  deliver(&mut session, &mut broker);
  let saved = session.save_game().unwrap().unwrap();
  assert_eq!(
    &saved["seen"].as_array().unwrap()[before_focus..],
    &[serde_json::json!("focus:false")]
  );
  input.begin_frame();
  input.queue_system_event(SystemEvent::Focus(FocusEvent { gained: false }), &mut log);
  input.poll();
  assert!(input.notifications().is_empty());
  input.queue_system_event(SystemEvent::Focus(FocusEvent { gained: true }), &mut log);
  input.queue_key_event(
    KeyEvent {
      key: Key::Esc,
      kind: KeyEventKind::Press,
    },
    &mut log,
  );
  input.poll();
  assert_eq!(
    input.notifications(),
    [InputNotification::Focus { gained: true }]
  );
  route(&input, &mut session, &mut broker, token, 7);
  deliver(&mut session, &mut broker);
  broker
    .push_system(
      8,
      LuaEventData::Resize {
        width: 80,
        height: 24,
      },
    )
    .unwrap();
  deliver(&mut session, &mut broker);
  input.begin_frame();
  input.queue_key_event(
    KeyEvent {
      key: Key::Esc,
      kind: KeyEventKind::Release,
    },
    &mut log,
  );
  input.queue_key_event(
    KeyEvent {
      key: Key::Esc,
      kind: KeyEventKind::Press,
    },
    &mut log,
  );
  input.poll();
  route(&input, &mut session, &mut broker, token, 8);
  deliver(&mut session, &mut broker);
  let before_focus = session.save_game().unwrap().unwrap()["seen"]
    .as_array()
    .unwrap()
    .len();
  input.begin_frame();
  input.queue_system_event(SystemEvent::Focus(FocusEvent { gained: false }), &mut log);
  input.poll();
  route(&input, &mut session, &mut broker, token, 9);
  deliver(&mut session, &mut broker);
  let saved = session.save_game().unwrap().unwrap();
  assert_eq!(
    &saved["seen"].as_array().unwrap()[before_focus..],
    &[
      serde_json::json!("key:esc:released"),
      serde_json::json!("action:exit:released"),
      serde_json::json!("focus:false"),
    ]
  );
  for (index, enabled) in [true, false].into_iter().enumerate() {
    let frame = 10 + index as u64 * 3;
    broker
      .push_system(
        frame,
        LuaEventData::Resize {
          width: if enabled { 80 } else { 1 },
          height: 24,
        },
      )
      .unwrap();
    deliver(&mut session, &mut broker);
    input.begin_frame();
    input.queue_system_event(SystemEvent::Focus(FocusEvent { gained: true }), &mut log);
    input.queue_key_event(
      KeyEvent {
        key: Key::Esc,
        kind: KeyEventKind::Release,
      },
      &mut log,
    );
    input.queue_key_event(
      KeyEvent {
        key: Key::Esc,
        kind: KeyEventKind::Press,
      },
      &mut log,
    );
    input.poll();
    route(&input, &mut session, &mut broker, token, frame);
    deliver(&mut session, &mut broker);
    let before_overlay = session.save_game().unwrap().unwrap()["seen"]
      .as_array()
      .unwrap()
      .len();
    for data in session.focus_lost_input() {
      broker
        .push_owned(token, frame + 1, data, LuaEventRoute::InputRelease)
        .unwrap();
    }
    broker.clear_pending_interactive(LuaSessionKind::Game);
    broker
      .push_system(frame + 1, LuaEventData::OverlayStarted)
      .unwrap();
    assert!(session.focus_lost_input().is_empty());
    deliver(&mut session, &mut broker);
    let saved = session.save_game().unwrap().unwrap();
    let expected = if enabled {
      vec![
        serde_json::json!("key:esc:released"),
        serde_json::json!("action:exit:released"),
        serde_json::json!("overlay_started"),
      ]
    } else {
      vec![serde_json::json!("overlay_started")]
    };
    assert_eq!(
      &saved["seen"].as_array().unwrap()[before_overlay..],
      expected.as_slice()
    );
    input.begin_frame();
    input.poll();
    broker
      .push_system(frame + 2, LuaEventData::OverlayStopped)
      .unwrap();
    deliver(&mut session, &mut broker);
    input.begin_frame();
    input.poll();
    assert!(input.notifications().is_empty());
  }
  println!(
    "Lua input ok: shared bindings, idempotent subscriptions, optional focus and overlay releases, fresh input after recovery"
  );
}

/// Queue keyboard transitions under the current session subscriptions.
///
/// # Arguments
///
/// * `input` - The ordered physical input journal.
/// * `session` - The script receiving game input.
/// * `broker` - The bounded session event queue.
/// * `token` - The owner of this input batch.
/// * `frame` - The host frame capturing the transitions.
fn route(
  input: &InputService,
  session: &mut LuaSession,
  broker: &mut LuaEventBroker,
  token: LuaSessionToken,
  frame: u64,
) {
  for notification in input.notifications() {
    let data = match notification {
      InputNotification::Text { text } => LuaEventData::Input { text: text.clone() },
      InputNotification::Key { key, state } => LuaEventData::Key {
        key: key_token(*key),
        state: (*state).into(),
      },
      InputNotification::Action {
        event,
        system: false,
      } => LuaEventData::Action {
        action: event.action.clone(),
        state: event.state.into(),
      },
      InputNotification::Action { system: true, .. } => continue,
      InputNotification::Focus { gained } => {
        if !gained {
          for data in session.focus_lost_input() {
            broker
              .push_owned(token, frame, data, LuaEventRoute::InputRelease)
              .unwrap();
          }
          broker.clear_pending_interactive(LuaSessionKind::Game);
        }
        broker
          .push_system(frame, LuaEventData::Focus { gained: *gained })
          .unwrap();
        continue;
      }
    };
    let Some(generation) = session.input_generation(&data) else {
      continue;
    };
    broker
      .push_owned(token, frame, data, LuaEventRoute::Input { generation })
      .unwrap();
  }
}

fn deliver(session: &mut LuaSession, broker: &mut LuaEventBroker) {
  for delivery in broker.drain_frame(LuaSessionKind::Game) {
    session.dispatch_event(&delivery).unwrap();
  }
}
