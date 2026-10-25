//! Execute the same independent input scenario used by the runnable smoke entry.

#[path = "../examples/lua_widget_input_smoke.rs"]
mod scenario;

#[test]
fn input_lifecycle_editing_events_and_slice_ownership() {
  scenario::main();
}

#[test]
fn independent_sessions_enforce_input_limits_and_release_focused_objects() {
  use std::{
    fs,
    time::{Duration, SystemTime, UNIX_EPOCH},
  };
  use tg_service_layout::Size;
  use tg_service_lua::{LuaService, LuaSessionKind, LuaSessionSpec};
  let nonce = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_nanos();
  struct Deployment(std::path::PathBuf);
  impl Drop for Deployment {
    fn drop(&mut self) {
      let _ = fs::remove_dir_all(&self.0);
    }
  }
  let root =
    Deployment(std::env::temp_dir().join(format!("tg-input-limit-{}-{nonce}", std::process::id())));
  fs::create_dir_all(root.0.join("scripts")).unwrap();
  let entry = root.0.join("scripts/main.lua");
  fs::write(
    &entry,
    r#"
    function Init(ctx)
      debug.assert(widget.input.create(1,1) == "input_001")
      for index=2,1024 do widget.input.create(1,1) end
      debug.assert(widget.input.count() == 1024)
      debug.assert(not debug.pcall(widget.input.create,1,1))
      debug.assert(widget.input.focus("input_001"))
      debug.assert(widget.input.focus("input_002"))
      debug.assert(widget.input.delete("input_002"))
      debug.assert(widget.input.clear() and widget.input.count() == 0)
      debug.assert(widget.input.create(1,1) == "input_1025")
      debug.assert(widget.input.get_info("input_001") == nil)
    end
    function Update(dt) end
    function UpdateFrame(dt,alpha) end
    function Render() end
    function HandleEvent(event) end
  "#,
  )
  .unwrap();
  let make_session = |kind| {
    LuaService::new()
      .create_session(LuaSessionSpec {
        package_id: "input_limits".into(),
        session_kind: kind,
        entry_path: entry.clone(),
        fixed_delta: Duration::from_secs_f64(1.0 / 60.0),
        base_size: Size {
          width: 12,
          height: 6,
        },
        continue_data: None,
        best_data: None,
        save_game_enabled: false,
        save_best_enabled: false,
      })
      .unwrap()
  };
  let mut first = make_session(LuaSessionKind::Game);
  let second = make_session(LuaSessionKind::Game);
  let saver = make_session(LuaSessionKind::Screensaver);
  assert!(
    !first
      .with_objects(|objects| objects.input_is_focused())
      .unwrap()
  );
  first.stop();
  assert!(first.with_objects(|_| ()).is_none());
  assert!(second.with_objects(|_| ()).is_some());
  assert!(saver.with_objects(|_| ()).is_some());
}
