//! Verify official Tetris event handling through an independent Lua session.
//!
//! Run `cargo run -p tg-service-lua --example lua_official_tetris_smoke --locked`.
//! Require the repository's official Tetris package and a writable temporary directory.
//! No interactive terminal, keyboard device, or external tool is required.

use std::{
  fs,
  path::{Path, PathBuf},
  time::{Duration, SystemTime, UNIX_EPOCH},
};
use tg_service_layout::Size;
use tg_service_lua::{LuaService, LuaSessionKind, LuaSessionSpec};

struct Deployment(PathBuf);
impl Drop for Deployment {
  fn drop(&mut self) {
    fs::remove_dir_all(&self.0).expect("remove smoke deployment");
  }
}

fn copy_directory(source: &Path, destination: &Path) {
  fs::create_dir_all(destination).unwrap();
  for entry in fs::read_dir(source).unwrap() {
    let entry = entry.unwrap();
    let destination = destination.join(entry.file_name());
    if entry.file_type().unwrap().is_dir() {
      copy_directory(&entry.path(), &destination);
    } else {
      fs::copy(entry.path(), destination).unwrap();
    }
  }
}

pub fn main() {
  let nonce = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_nanos();
  let root = Deployment(
    std::env::temp_dir().join(format!("tg-official-tetris-{}-{nonce}", std::process::id())),
  );
  let package = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../scripts/game/official_tetris");
  copy_directory(&package.join("scripts"), &root.0.join("scripts"));
  copy_directory(&package.join("assets"), &root.0.join("assets"));
  let entry = root.0.join("scripts/main.lua");
  let mut source = fs::read_to_string(&entry).unwrap();
  source.push_str(
    r#"
    local initialize = Init
    function Init(ctx)
      initialize(ctx)
      -- Supply the asynchronous completion explicitly without starting the host runtime.
      HandleEvent({type="i18n", data={request_id=session.i18n_request_id, kind="created", ok=true}})
      debug.assert(session.i18n_ready and session.phase == "stop")
      local function action(name, state)
        HandleEvent({type="action", data={action=name, state=state}})
      end
      action("start-resume", "pressed")
      debug.assert(session.phase == "ready")
      Update(2)
      debug.assert(session.phase == "running")
      local origin = session.x
      action("move_left", "pressed")
      debug.assert(session.move_held[-1] and session.x == origin - 1)
      action("move_left", "held")
      debug.assert(session.x == origin - 1)
      action("move_left", "released")
      debug.assert(not session.move_held[-1] and session.move_direction == 0)
      action("soft_drop", "pressed")
      debug.assert(session.soft_drop)
      action("soft_drop", "held")
      action("soft_drop", "released")
      debug.assert(not session.soft_drop)
      action("move_right", "pressed")
      HandleEvent({type="focus", data={gained=false}})
      debug.assert(session.phase == "pause" and not session.move_held[1])
      action("start-resume", "pressed")
      debug.assert(session.phase == "running")
      action("move_left", "pressed")
      action("soft_drop", "pressed")
      HandleEvent({type="overlay_started", data={}})
      debug.assert(not session.move_held[-1] and not session.soft_drop)
      action("pause", "pressed")
      debug.assert(session.phase == "pause")
    end
  "#,
  );
  fs::write(&entry, source).unwrap();
  let mut session = LuaService::new()
    .create_session(LuaSessionSpec {
      package_id: "official_tetris".into(),
      session_kind: LuaSessionKind::Game,
      entry_path: entry,
      fixed_delta: Duration::from_secs_f64(1.0 / 60.0),
      base_size: Size {
        width: 100,
        height: 40,
      },
      continue_data: None,
      best_data: None,
      save_game_enabled: false,
      save_best_enabled: true,
    })
    .expect("official Tetris event handling");
  session.update().unwrap();
  session.stop();
  assert!(!session.has_objects());
  println!(
    "official Tetris ok: i18n completion, action press/hold/release, focus loss and overlay input cleanup"
  );
}
