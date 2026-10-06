//! Verify Lua clipboard writes against the system clipboard and restore its previous text.
//!
//! Run `cargo run -p tg-service-lua --example lua_clipboard_smoke --locked`.
//! This requires clipboard access and existing text that can be restored; otherwise it skips.

use std::{
  cell::RefCell,
  fs,
  path::PathBuf,
  rc::Rc,
  time::{Duration, SystemTime, UNIX_EPOCH},
};
use tg_service_clipboard::ClipboardService;
use tg_service_layout::Size;
use tg_service_lua::{LuaApiConfig, LuaService, LuaSessionKind, LuaSessionSpec};

struct Fixture {
  root: PathBuf,
  clipboard: Rc<RefCell<ClipboardService>>,
  previous: Option<String>,
}

impl Drop for Fixture {
  fn drop(&mut self) {
    if let Some(previous) = &self.previous
      && !self.clipboard.borrow_mut().write_text(previous)
    {
      eprintln!("clipboard restore failed");
    }
    let _ = fs::remove_dir_all(&self.root);
  }
}

fn main() {
  let clipboard = Rc::new(RefCell::new(ClipboardService::new()));
  let Some(previous) = clipboard.borrow_mut().read_text() else {
    println!("Lua clipboard skipped: no restorable text clipboard");
    return;
  };
  let mut fixture = Fixture {
    root: std::env::temp_dir().join(format!(
      "tg-lua-clipboard-{}-{}",
      std::process::id(),
      SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos()
    )),
    clipboard: clipboard.clone(),
    previous: Some(previous),
  };
  fs::create_dir_all(fixture.root.join("scripts")).unwrap();
  let entry = fixture.root.join("scripts/main.lua");
  fs::write(
    &entry,
    r#"
    local written = false
    function Init(ctx) written = ime.write_clipboard("Tui Game 中文\nclipboard") end
    function HandleEvent(event) end
    function Update(dt) end
    function UpdateFrame(dt, alpha) end
    function Render() end
    function SaveGame() return {written=written} end
  "#,
  )
  .unwrap();
  let mut session = LuaService::new()
    .create_session_with_api(
      LuaSessionSpec {
        package_id: "clipboard_smoke".into(),
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
        clipboard: Some(clipboard.clone()),
        ..Default::default()
      },
    )
    .unwrap();
  let saved = session.save_game().unwrap().unwrap();
  let copied = clipboard.borrow_mut().read_text();
  assert!(
    clipboard
      .borrow_mut()
      .write_text(fixture.previous.as_deref().unwrap()),
    "restore previous clipboard text"
  );
  fixture.previous = None;
  assert_eq!(saved["written"], true);
  assert_eq!(copied.as_deref(), Some("Tui Game 中文\nclipboard"));
  println!("Lua clipboard ok: UTF-8 round trip and previous text restored");
}
