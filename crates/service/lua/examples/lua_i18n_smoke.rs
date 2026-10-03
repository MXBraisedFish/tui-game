//! Exercise asynchronous i18n loading, warnings, and custom missing-key text through public APIs.
//!
//! Run `cargo run -p tg-service-lua --example lua_i18n_smoke --locked`.
//! Temporary deployment roots supply test translations; no terminal device is required.

use std::{
  fs,
  path::PathBuf,
  time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tg_service_async::{AsyncRuntime, TaskStatusEvent};
use tg_service_file::FileEvent;
use tg_service_layout::Size;
use tg_service_lua::{
  LuaEventBroker, LuaEventRoute, LuaHostCommand, LuaI18nEventKind, LuaRoutableEvent, LuaService,
  LuaSessionKind, LuaSessionSpec, LuaSessionToken, LuaTaskOperation,
};

struct Deployment(PathBuf);
impl Drop for Deployment {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
}

enum Event {
  File(FileEvent),
  Status,
}
impl From<FileEvent> for Event {
  fn from(event: FileEvent) -> Self {
    Self::File(event)
  }
}
impl From<TaskStatusEvent> for Event {
  fn from(_: TaskStatusEvent) -> Self {
    Self::Status
  }
}

fn main() {
  let nonce = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .expect("system clock")
    .as_nanos();
  for kind in [LuaSessionKind::Game, LuaSessionKind::Screensaver] {
    let root = Deployment(std::env::temp_dir().join(format!(
      "tg_i18n_smoke_{}_{nonce}_{}",
      std::process::id(),
      kind.as_str()
    )));
    fs::create_dir_all(root.0.join("scripts")).expect("create deployment");
    let entry = root.0.join("scripts/main.lua");
    fs::write(&entry, r#"
      function Init(ctx)
        completed = 0
        request_id = i18n.create({language_code = "zh_cn", callback_language_code = "en_us"})
      end
      function HandleEvent(event)
        debug.assert(event.type == "i18n" and event.data.request_id == request_id)
        completed = completed + 1
        local data = event.data
        debug.assert(data.language_code == "zh_cn" and data.callback_language_code == "en_us")
        if completed == 1 or completed == 5 then
          debug.assert(data.ok and string.find(data.warning, "primary") and string.find(data.warning, "fallback"))
          debug.assert(i18n.get_value("menu", "title", {callback = "自定义"}) == "自定义")
          debug.assert(i18n.get_value("menu", "title") == "[Missing i18n Key: menu.title]")
        elseif completed == 2 then
          debug.assert(data.ok and string.find(data.warning, "primary"))
          debug.assert(i18n.get_value("menu", "title", {callback = "自定义"}) == "Title")
          debug.assert(i18n.get_value("menu", "fallback") == "Fallback")
        elseif completed == 3 then
          debug.assert(data.ok and string.find(data.warning, "fallback"))
          debug.assert(i18n.get_value("menu", "title", {callback = "自定义"}) == "标题")
          debug.assert(i18n.get_value("menu", "fallback", {callback = ""}) == "")
        elseif completed == 4 then
          debug.assert(not data.ok and data.warning == nil)
          debug.assert(i18n.get_value("menu", "title") == "标题")
        elseif completed == 6 then
          debug.assert(data.ok and data.warning == nil)
          debug.assert(i18n.get_value("menu", "title", {callback = "自定义"}) == "标题")
          debug.assert(i18n.get_value("menu", "fallback") == "Fallback")
        end
        if completed < 6 then
          request_id = i18n.reload({language_code = "zh_cn", callback_language_code = "en_us"})
        end
      end
      function Update(dt) end
      function UpdateFrame(dt, alpha) end
      function Render() end
    "#).expect("write script");
    let mut session = LuaService::new()
      .create_session(LuaSessionSpec {
        package_id: "i18n_smoke".into(),
        session_kind: kind,
        entry_path: entry,
        fixed_delta: Duration::from_secs_f64(1.0 / 60.0),
        base_size: Size {
          width: 80,
          height: 24,
        },
        continue_data: None,
        best_data: None,
        save_game_enabled: false,
        save_best_enabled: false,
      })
      .expect("create session");
    let owner = LuaSessionToken {
      kind,
      generation: 1,
    };
    let mut broker = LuaEventBroker::new();
    broker.synchronize_sessions(
      (kind == LuaSessionKind::Game).then_some(owner),
      (kind == LuaSessionKind::Screensaver).then_some(owner),
    );
    let runtime = AsyncRuntime::<Event>::with_worker_count(1);
    let primary = root.0.join("assets/language/zh_cn/menu.json");
    let fallback = root.0.join("assets/language/en_us/menu.json");
    for round in 1..=6 {
      match round {
        2 => {
          fs::create_dir_all(fallback.parent().unwrap()).expect("fallback directory");
          fs::write(&fallback, r#"{"title":"Title","fallback":"Fallback"}"#).unwrap();
        }
        3 => {
          fs::remove_file(&fallback).unwrap();
          fs::create_dir_all(primary.parent().unwrap()).expect("primary directory");
          fs::write(&primary, r#"{"title":"标题"}"#).unwrap();
        }
        4 => fs::write(&primary, "not JSON").unwrap(),
        5 => fs::remove_file(&primary).unwrap(),
        6 => {
          fs::write(&primary, r#"{"title":"标题"}"#).unwrap();
          fs::write(&fallback, r#"{"title":"Title","fallback":"Fallback"}"#).unwrap();
        }
        _ => {}
      }
      let commands = session.take_host_commands();
      assert_eq!(commands.len(), 1, "one i18n request per round");
      for command in commands {
        let LuaHostCommand::I18nRequest {
          request_id,
          task,
          kind,
          language_code,
          callback_language_code,
        } = command
        else {
          panic!("unexpected host command");
        };
        assert_eq!(
          kind,
          if round == 1 {
            LuaI18nEventKind::Created
          } else {
            LuaI18nEventKind::Reloaded
          }
        );
        let task_id = runtime.submit(task);
        broker
          .register_task(
            task_id,
            owner,
            LuaTaskOperation::I18n {
              request_id,
              kind,
              language_code,
              callback_language_code,
            },
            LuaEventRoute::HandleEvent,
          )
          .expect("register i18n request");
      }
      let deadline = Instant::now() + Duration::from_secs(5);
      let mut delivered = false;
      while !delivered && Instant::now() < deadline {
        for event in runtime.poll_events() {
          if let Event::File(event) = event {
            broker
              .route_service_event(round, LuaRoutableEvent::File(&event))
              .expect("route result");
          }
        }
        for delivery in broker.drain_frame(owner.kind) {
          session
            .dispatch_event(&delivery)
            .expect("verify i18n event and committed translations");
          delivered = true;
        }
        if !delivered {
          std::thread::sleep(Duration::from_millis(5));
        }
      }
      assert!(delivered, "i18n request timed out");
      session.update().expect("fixed update");
      session.render().expect("render");
    }
    assert!(session.take_host_commands().is_empty());
    session.stop();
  }
  println!(
    "i18n Lua ok: warnings, fallback precedence, custom missing-key text, reload replacement and genuine failures in game/screensaver sessions"
  );
}
