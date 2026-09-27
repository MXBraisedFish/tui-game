//! Minimal entry: loads a checked-in screensaver script into a sandboxed Lua session.

use std::{path::PathBuf, time::Duration};

use tg_service_layout::Size;
use tg_service_lua::{LuaService, LuaSessionKind, LuaSessionSpec};

fn main() {
  let package_root =
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../test_package/screensaver/layer_waves");
  let session = LuaService::new()
    .create_session(LuaSessionSpec {
      package_id: "layer_waves".to_string(),
      session_kind: LuaSessionKind::Screensaver,
      entry_path: package_root.join("scripts").join("main.lua"),
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
    .expect("load the layer_waves screensaver");

  assert_eq!(session.session_kind(), LuaSessionKind::Screensaver);
  println!("lua ok: layer_waves screensaver loaded");
}
