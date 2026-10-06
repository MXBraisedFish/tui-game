//! Exercise strict optional parameters through independent game and screensaver sessions.
//!
//! Run `cargo run -p tg-service-lua --example lua_options_smoke --locked` with a writable
//! temporary directory. No terminal, audio device, or deployed assets are required.

use std::{
  fs,
  path::PathBuf,
  time::{Duration, SystemTime, UNIX_EPOCH},
};

use tg_service_layout::Size;
use tg_service_lua::{LuaService, LuaSessionKind, LuaSessionSpec};

/// The temporary deployment tree owned by this smoke run.
struct SmokeDeployment(PathBuf);

impl Drop for SmokeDeployment {
  fn drop(&mut self) {
    if let Err(error) = fs::remove_dir_all(&self.0) {
      eprintln!(
        "failed to remove smoke deployment {}: {error}",
        self.0.display()
      );
    }
  }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
  let root = std::env::temp_dir().join(format!(
    "tg_lua_options_smoke_{}_{nonce}",
    std::process::id()
  ));
  fs::create_dir(&root)?;
  let deployment = SmokeDeployment(root);
  fs::create_dir(deployment.0.join("scripts"))?;
  let entry = deployment.0.join("scripts/main.lua");
  fs::write(
    &entry,
    r##"
    function Init(ctx)
      debug.assert(tonumber("ff", {base = 16}) == 255)
      local payload = {pos = "data"}
      local list = {"a", "c"}
      table.insert(list, "b", {pos = 2})
      debug.assert(table.concat(list, {sep = "-"}) == "a-b-c")
      debug.assert(table.remove(list, {pos = 2}) == "b")
      table.insert(list, payload)
      debug.assert(list[3] == payload)
      local target = {}
      debug.assert(table.move(list, 1, 3, 1, {target = target}) == target)
      debug.assert(target[3] == payload)
      local packed = table.pack("a", nil, "c")
      local a, b, c = table.unpack(packed, {j = packed.n})
      debug.assert(a == "a" and b == nil and c == "c")
      debug.assert(select("#", table.unpack(packed, {j = packed.n})) == 3)
      local numbers = {1, 3, 2}
      table.sort(numbers, {comp = function(left, right) return left > right end})
      debug.assert(numbers[1] == 3 and numbers[3] == 1)
      local seen = 0
      local key, value = next(numbers)
      while key ~= nil do
        seen = seen + value
        key, value = next(numbers, {key = key})
      end
      debug.assert(seen == 6)
      seen = 0
      for _, value in pairs(numbers) do seen = seen + value end
      debug.assert(seen == 6)
      debug.assert(not select(1, debug.pcall(function() table.remove(list, 1) end)))
      debug.assert(not select(1, debug.pcall(function() tonumber("ff", {unknown = true}) end)))
    end
    function HandleEvent(event) end
    function Update(dt) end
    function UpdateFrame(dt, alpha) end
    function Render() end
    "##,
  )?;
  for session_kind in [LuaSessionKind::Game, LuaSessionKind::Screensaver] {
    let mut session = LuaService::new().create_session(LuaSessionSpec {
      package_id: "options_smoke".to_string(),
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
    })?;
    session.update()?;
    session.render()?;
  }
  println!("Lua options ok: game and screensaver calls, payloads, iteration and strict errors");
  Ok(())
}
