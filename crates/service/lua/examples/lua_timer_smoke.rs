//! Exercise timer bindings, ownership, callbacks, and broker limits without the application.
//!
//! Run `cargo run -p tg-service-lua --example lua_timer_smoke --locked`.
//! The entry creates temporary deployment roots; no terminal or wall-clock waits are required.

use std::{
  fs,
  path::PathBuf,
  time::{Duration, SystemTime, UNIX_EPOCH},
};
use tg_service_layout::Size;
use tg_service_lua::{
  LuaEventBroker, LuaEventRoute, LuaService, LuaSessionKind, LuaSessionSpec, LuaSessionToken,
};
use tg_service_time::TimeService;

struct Deployment(PathBuf);

impl Drop for Deployment {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
}

fn main() {
  let nonce = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .expect("system clock")
    .as_nanos();
  let root =
    Deployment(std::env::temp_dir().join(format!("tg_timer_smoke_{}_{nonce}", std::process::id())));
  fs::create_dir_all(root.0.join("scripts")).expect("create deployment");
  let entry = root.0.join("scripts/main.lua");
  fs::write(&entry, r#"
    function Init(ctx)
      count, handled, frame = 0, 0, 0
      for i = 1, 130 do
        local id = timer.create(0, {tip = "burst", callback = function(event)
          debug.assert(event.type == "timer" and type(event.data.id) == "string")
          debug.assert(event.data.kind == "finished" and event.data.tip == "burst")
          debug.assert(event.sequence > 0 and event.frame > 0)
          count = count + 1
        end})
        timer.start(id)
      end
      regular = timer.create(0.1, {delay = 0.05, interval = -0.05, loop = true, ["repeat"] = 2})
      timer.start(regular)
    end
    function HandleEvent(event)
      debug.assert(event.data.id == regular and event.data.executed_count == handled + 1)
      handled = handled + 1
      debug.assert(event.data.kind == (handled == 2 and "finished" or "tick"))
    end
    function Update(dt) end
    function UpdateFrame(dt, alpha)
      frame = frame + 1
      if frame == 1 then debug.assert(count == 128 and handled == 0) end
      if frame == 2 then debug.assert(count == 130 and handled == 1) end
      if frame == 3 then
        debug.assert(count == 130 and handled == 2 and timer.get_info(regular).state == "finished")
        debug.assert(timer.set(regular, {loop = false, tip = "new"}))
        debug.assert(timer.get_info(regular).state == "idle" and timer.get_info(regular).executed_count == 0)
        debug.assert(timer.clear() and timer.count() == 0)
      end
    end
    function Render() end
  "#).expect("write timer script");

  let game = LuaSessionToken {
    kind: LuaSessionKind::Game,
    generation: 1,
  };
  let saver = LuaSessionToken {
    kind: LuaSessionKind::Screensaver,
    generation: 1,
  };
  let mut broker = LuaEventBroker::new();
  broker.synchronize_sessions(Some(game), Some(saver));
  for token in [game, saver] {
    let mut session = LuaService::new()
      .create_session(LuaSessionSpec {
        package_id: "timer_smoke".into(),
        session_kind: token.kind,
        entry_path: entry.clone(),
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
      .expect("initialize timer session");
    for frame in 1..=3 {
      let events = session
        .with_objects_mut(|objects| {
          TimeService::new().update(&mut objects.runtime_mut().time, Duration::from_millis(100));
          objects.take_timer_events()
        })
        .expect("session pool");
      for event in events {
        broker
          .push_owned(token, frame, event, LuaEventRoute::HandleEvent)
          .expect("queue timer event");
      }
      for delivery in broker.drain_frame(token.kind) {
        session
          .dispatch_event(&delivery)
          .expect("dispatch budgeted timer event");
      }
      session.update().expect("fixed update");
      session
        .update_frame(Duration::from_millis(100), 0.0)
        .expect("verify timer state");
      session.render().expect("render");
    }
    session.stop();
    assert!(!session.has_objects());
  }
  assert!(broker.take_overflowed_sessions().is_empty());
  println!(
    "timer Lua ok: game/screensaver ownership, strict options, callback delivery, 128-event frame budget and cleanup"
  );
}
