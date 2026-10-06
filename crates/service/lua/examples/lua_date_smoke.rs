//! Exercise the date library through public Lua sessions without running the host.
//!
//! Run `cargo run -p tg-service-lua --example lua_date_smoke --locked` with the system clock
//! and local time-zone configuration available. No game assets or terminal device are required.

use std::{
  fs,
  time::{Duration, SystemTime, UNIX_EPOCH},
};
use tg_service_layout::Size;
use tg_service_lua::{LuaService, LuaSessionKind, LuaSessionSpec};

fn main() {
  let nonce = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .expect("system clock")
    .as_nanos();
  let root = std::env::temp_dir().join(format!("tg_date_lua_smoke_{}_{nonce}", std::process::id()));
  fs::create_dir_all(root.join("scripts")).expect("create temporary deployment root");
  let entry = root.join("scripts/main.lua");
  fs::write(
    &entry,
    r#"
    function Init(ctx)
      local t = date.timestamp_to_date(123, {timezone = date.UTC_PLUS_8})
      debug.assert(t.year == 1970 and t.hour == 8 and t.millisecond == 123)
      debug.assert(date.date_to_timestamp(t.year, t.month, t.day, t.hour, t.minute, t.second, t.millisecond, {timezone = date.UTC_PLUS_8}) == 123)
      debug.assert(date.timestamp_diff(-500, 1000) == 1500)
      debug.assert(date.timestamp_diff(1000, 1000) == 0)
      local early = date.timestamp_to_date(-1, {timezone = date.UTC})
      local later = date.timestamp_to_date(123, {timezone = date.UTC})
      debug.assert(date.timestamp_diff(early, later, {timezone = date.UTC}) == 124)
      debug.assert(date.timestamp_diff(early, 123, {timezone = date.UTC}) == 124)
      debug.assert(date.timestamp_diff(-1, later, {timezone = date.UTC}) == 124)
      debug.assert(date.timestamp_diff(early, -1, {timezone = date.UTC}) == 0)
      debug.assert(not select(1, debug.pcall(function()
        date.timestamp_diff(later, -1, {timezone = date.UTC})
      end)))
      debug.assert(not select(1, debug.pcall(function()
        date.timestamp_diff(1000, 999)
      end)))
      debug.assert(type(date.now()) == 'number')
      debug.assert(type(date.now({time_type = date.DATE})) == 'table')
      debug.assert(not select(1, debug.pcall(function() date.now({unexpected = true}) end)))
    end
    function HandleEvent(event) end
    function Update(dt) end
    function UpdateFrame(dt, alpha) end
    function Render() end
  "#,
  )
  .expect("write date smoke script");
  for session_kind in [LuaSessionKind::Game, LuaSessionKind::Screensaver] {
    let mut session = LuaService::new()
      .create_session(LuaSessionSpec {
        package_id: "date_smoke".to_string(),
        session_kind,
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
      .expect("load date script and run Init assertions");
    session.update().expect("update date session");
    session.render().expect("render date session");
  }
  fs::remove_dir_all(root).expect("remove date smoke deployment");
  println!("date Lua ok: game and screensaver sessions, millisecond round trip and strict fields");
}
