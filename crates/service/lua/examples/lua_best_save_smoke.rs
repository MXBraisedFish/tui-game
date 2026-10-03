//! Exercise localized best-score callbacks and raw restoration without running the host.
//!
//! Run `cargo run -p tg-service-lua --example lua_best_save_smoke --locked`.
//! Only a writable temporary directory is required; no terminal or audio device is used.

use std::{
  fs,
  time::{Duration, SystemTime, UNIX_EPOCH},
};
use tg_service_layout::Size;
use tg_service_lua::{LuaService, LuaSessionKind, LuaSessionSpec};

fn main() {
  let nonce = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_nanos();
  let root = std::env::temp_dir().join(format!("tg_best_lua_smoke_{}_{nonce}", std::process::id()));
  fs::create_dir_all(root.join("scripts")).expect("create deployment root");
  let entry = root.join("scripts/main.lua");
  fs::write(
    &entry,
    r#"
    function Init(ctx)
      if ctx.best_data then
        debug.assert(ctx.best_data.best_string.key == "score")
        debug.assert(ctx.best_data.value.rank.callback == "Gold")
        debug.assert(ctx.best_data.score == 42)
      end
    end
    function HandleEvent(event) end
    function Update(dt) end
    function UpdateFrame(dt, alpha) end
    function Render() end
    function SaveBest()
      return {
        best_string = {type="i18n", key="score", callback="f%Best: {value:score}"},
        value = {score="42", rank={type="i18n", key="rank", callback="Gold"}},
        score = 42
      }
    end
  "#,
  )
  .expect("write script");
  let mut spec = LuaSessionSpec {
    package_id: "best_smoke".into(),
    session_kind: LuaSessionKind::Game,
    entry_path: entry,
    fixed_delta: Duration::from_secs_f64(1.0 / 60.0),
    base_size: Size {
      width: 80,
      height: 24,
    },
    continue_data: None,
    best_data: None,
    save_game_enabled: false,
    save_best_enabled: true,
  };
  let mut first = LuaService::new()
    .create_session(spec.clone())
    .expect("create game session");
  let best = first
    .save_best()
    .expect("save callback")
    .expect("saved value");
  assert_eq!(best["value"]["score"], "42");
  spec.best_data = Some(best.clone());
  let mut restored = LuaService::new()
    .create_session(spec)
    .expect("restore and run Init assertions");
  assert_eq!(restored.save_best().unwrap(), Some(best));
  fs::remove_dir_all(root).expect("clean deployment root");
  println!("best-save Lua ok: i18n templates, named substitutions and raw Init restoration");
}
