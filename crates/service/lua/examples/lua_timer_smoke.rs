//! Exercise timer bindings, ownership, event delivery, and broker limits without the application.
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

pub(crate) fn main() {
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
      local function check_missing(id)
        debug.assert(not timer.exists(id))
        local count = timer.count()
        for name, method in pairs({get_info=timer.get_info,delete=timer.delete,start=timer.start,pause=timer.pause,
          reset=timer.reset,restart=timer.restart,
          set=function(id) return timer.set(id,{duration=2}) end}) do
          local ok, message = debug.pcall(method,id)
          debug.assert(not ok and string.find(message,"timer."..name,{plain=true}) ~= nil
            and string.find(message,id,{plain=true}) ~= nil)
          debug.assert(timer.count()==count)
        end
      end
      check_missing("timer_999")
      local probe = timer.create(1)
      debug.assert(timer.get_info(probe)["repeat"]==math.POSITIVE_INFINITE)
      debug.assert(type(timer.get_info(probe)["repeat"])==type(math.POSITIVE_INFINITE))
      for _, options in ipairs({{tip="removed"},{tip=false},{callback=function() end},{callback=false}}) do
        debug.assert(not debug.pcall(timer.create,1,options))
        debug.assert(not debug.pcall(timer.set,probe,options))
        debug.assert(timer.count()==1 and timer.get_info(probe).state=="idle")
      end
      debug.assert(timer.set(probe,{["repeat"]=3}))
      debug.assert(timer.get_info(probe)["repeat"]==3 and math.type(timer.get_info(probe)["repeat"])=="integer")
      debug.assert(timer.set(probe,{["repeat"]=nil}) and timer.get_info(probe)["repeat"]==3)
      debug.assert(timer.delete(probe))
      check_missing(probe)
      local cleared = timer.create(1)
      debug.assert(timer.clear())
      check_missing(cleared)
      count, handled, frame = 0, 0, 0
      burst = {}
      for i = 1, 130 do
        local id = timer.create(0)
        burst[id] = true
        timer.start(id)
      end
      regular = timer.create(0.1, {delay = 0.05, interval = 0, loop = true, ["repeat"] = 2})
      debug.assert(not debug.pcall(timer.create, 1, {["repeat"] = false}))
      debug.assert(not debug.pcall(timer.set, regular, {duration = 2, loop = false, ["repeat"] = false}))
      local unchanged = timer.get_info(regular)
      debug.assert(unchanged.duration == 0.1 and unchanged.loop and unchanged["repeat"] == 2)
      timer.start(regular)
      phased = timer.create(0.05, {delay = 0.05, interval = 0.05, loop = true, ["repeat"] = 2})
      debug.assert(timer.start(phased) and timer.get_info(phased).state == "delay")
      debug.assert(not timer.start(phased))
      debug.assert(timer.pause(phased) and timer.get_info(phased).state == "paused")
      debug.assert(timer.start(phased) and timer.get_info(phased).state == "delay")
    end
    function HandleEvent(event)
      debug.assert(event.type == "timer" and event.data.tip == "" and event.data.callback == "")
      debug.assert(event.sequence > 0 and event.frame > 0)
      if burst[event.data.id] then
        debug.assert(event.data.kind == "finished")
        count = count + 1
      elseif event.data.id == regular then
        debug.assert(event.data.executed_count == handled + 1)
        handled = handled + 1
        debug.assert(event.data.kind == (handled == 2 and "finished" or "ticked"))
      elseif infinite_ids and infinite_ids[event.data.id] then
        local index = infinite_ids[event.data.id]
        debug.assert(event.data.kind == "ticked")
        infinite_count[index] = infinite_count[index] + 1
        debug.assert(event.data.executed_count == infinite_count[index])
      else debug.assert(event.data.id == phased) end
    end
    function Update(dt) end
    function UpdateFrame(dt, alpha)
      frame = frame + 1
      if frame <= 4 then
        local expected = ({"running", "interval", "running", "finished"})[frame]
        debug.assert(timer.get_info(phased).state == expected)
        for _, info in ipairs(timer.list()) do
          if info.id == phased then debug.assert(info.state == expected) end
        end
        if frame == 2 then
          debug.assert(not timer.start(phased))
          debug.assert(timer.pause(phased) and timer.get_info(phased).state == "paused")
          debug.assert(timer.start(phased) and timer.get_info(phased).state == "interval")
        end
      end
      if frame == 1 then debug.assert(count == 128 and handled == 0) end
      if frame == 2 then debug.assert(count == 130 and handled == 0) end
      if frame == 3 or frame == 4 then debug.assert(count == 130 and handled == 1) end
      if frame == 5 then
        debug.assert(count == 130 and handled == 2 and timer.get_info(regular).state == "finished")
        debug.assert(timer.set(regular, {loop = false}))
        debug.assert(timer.get_info(regular).state == "idle" and timer.get_info(regular).executed_count == 0)
        debug.assert(timer.clear() and timer.count() == 0)
        infinite_count = {0, 0}
        infinite_ids = {}
        local created = timer.create(0, {loop = true})
        local changed = timer.create(0, {loop = true, ["repeat"] = 1})
        infinite_ids[created], infinite_ids[changed] = 1, 2
        debug.assert(timer.set(changed, {["repeat"] = math.INFINITE}))
        debug.assert(timer.start(created) and timer.start(changed))
      end
      if frame >= 6 then
        debug.assert(infinite_count[1] == frame - 5 and infinite_count[2] == frame - 5)
        for _, info in ipairs(timer.list()) do debug.assert(info.state == "running" and info["repeat"] == math.POSITIVE_INFINITE) end
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
    for frame in 1..=10 {
      let events = session
        .with_objects_mut(|objects| {
          TimeService::new().update(&mut objects.runtime_mut().time, Duration::from_millis(50));
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
        .update_frame(Duration::from_millis(50), 0.0)
        .expect("verify timer state");
      session.render().expect("render");
    }
    session.stop();
    assert!(!session.has_objects());
  }
  assert!(broker.take_overflowed_sessions().is_empty());
  println!(
    "timer Lua ok: game/screensaver ownership, strict options, unlimited repetitions, HandleEvent delivery, 128-event frame budget and cleanup"
  );
}
