//! Exercise committed terminal text and input-method controls without starting the application.
//!
//! Run `cargo run -p tg-service-lua --example lua_ime_smoke --locked` for injected text.
//! Add `-- --native` to exercise the platform input method and restore it before returning.
//! Native control needs an installed ASCII input method and platform input-method access.
//! Injected text does not replace physical typing, paste, or terminal focus tests.

use std::{
  cell::RefCell,
  fs,
  rc::Rc,
  time::{Duration, SystemTime, UNIX_EPOCH},
};
use tg_core_input::{FocusEvent, SystemEvent};
use tg_service_input::{CommittedTextEvent, InputNotification, InputService};
use tg_service_input_method::InputMethodService;
use tg_service_layout::Size;
use tg_service_log::LogService;
use tg_service_lua::{
  LuaApiConfig, LuaEventBroker, LuaEventData, LuaEventRoute, LuaService, LuaSessionKind,
  LuaSessionSpec, LuaSessionToken,
};

struct Deployment(std::path::PathBuf);

impl Drop for Deployment {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
}

fn main() {
  let native = std::env::args().any(|argument| argument == "--native");
  let root = std::env::temp_dir().join(format!(
    "tg-ime-smoke-{}-{}",
    std::process::id(),
    SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap()
      .as_nanos()
  ));
  let deployment = Deployment(root);
  let root = &deployment.0;
  fs::create_dir_all(root.join("scripts")).unwrap();
  let entry = root.join("scripts/main.lua");
  let source = format!(
    r#"
    local texts = {{}}
    local native = {native}
    local locked, unlocked = false, false
    function Init(ctx)
      debug.assert(ime.receive_input_event())
      debug.assert(ime.receive_input_event())
      if native then
        locked = ime.lock()
        unlocked = ime.unlock({{restore=true}})
      end
    end
    function HandleEvent(event)
      if event.type == "input" then
        texts[#texts+1] = event.data.text
        if event.data.text == "reject" then
          ime.reject_input_event()
          ime.receive_input_event()
        end
      end
    end
    function Update(dt) end
    function UpdateFrame(dt, alpha) end
    function Render() end
    function SaveGame() return {{texts=texts, locked=locked, unlocked=unlocked}} end
  "#
  );
  fs::write(&entry, source).unwrap();
  let platform = Rc::new(RefCell::new(InputMethodService::new()));
  let mut session = LuaService::new()
    .create_session_with_api(
      LuaSessionSpec {
        package_id: "ime_smoke".into(),
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
      },
      LuaApiConfig {
        input_method: Some(platform.clone()),
        ..Default::default()
      },
    )
    .unwrap();
  let token = LuaSessionToken {
    kind: LuaSessionKind::Game,
    generation: 1,
  };
  let mut broker = LuaEventBroker::new();
  broker.synchronize_sessions(Some(token), None);
  let mut input = InputService::new();
  let mut log = LogService::new();
  input.begin_frame();
  for text in ["ordinary", "中文", "paste\ntext", "reject", "stale"] {
    input.queue_committed_text(CommittedTextEvent { text: text.into() });
  }
  input.poll();
  for notification in input.notifications() {
    if let InputNotification::Text { text } = notification {
      let data = LuaEventData::Input { text: text.clone() };
      let generation = session.input_generation(&data).unwrap();
      broker
        .push_owned(token, 1, data, LuaEventRoute::Input { generation })
        .unwrap();
    }
  }
  broker.clear_pending_actions(LuaSessionKind::Game);
  for event in broker.drain_frame(LuaSessionKind::Game) {
    session.dispatch_event(&event).unwrap();
  }
  let saved = session.save_game().unwrap().unwrap();
  assert_eq!(
    saved["texts"],
    serde_json::json!(["ordinary", "中文", "paste\ntext", "reject"])
  );

  input.begin_frame();
  input.queue_system_event(SystemEvent::Focus(FocusEvent { gained: false }), &mut log);
  input.queue_committed_text(CommittedTextEvent {
    text: "unfocused".into(),
  });
  input.queue_system_event(SystemEvent::Focus(FocusEvent { gained: true }), &mut log);
  input.queue_committed_text(CommittedTextEvent {
    text: "fresh".into(),
  });
  input.poll();
  assert!(session.focus_lost_input().is_empty());
  let texts = input
    .notifications()
    .iter()
    .filter_map(|notification| match notification {
      InputNotification::Text { text } => Some(text.as_str()),
      _ => None,
    })
    .collect::<Vec<_>>();
  assert_eq!(texts, ["fresh"]);
  if native {
    assert_eq!(
      saved["locked"],
      true,
      "native lock failed: {:?}",
      platform.borrow().last_error()
    );
    assert_eq!(
      saved["unlocked"],
      true,
      "native restore failed: {:?}",
      platform.borrow().last_error()
    );
    assert!(!platform.borrow().is_input_method_restricted());
    assert!(!session.input_method_locked());
  }
  drop(session);
  drop(platform);
  drop(deployment);
  println!(
    "ime ok: committed text, independent subscriptions, stale text rejection, focus order; native={native}"
  );
}
