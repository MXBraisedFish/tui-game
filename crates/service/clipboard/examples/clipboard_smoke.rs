//! Independent clipboard smoke entry exercising the public API and checking its results.

use tg_service_clipboard::ClipboardService;

fn main() {
  let mut clipboard = ClipboardService::new();
  let Some(previous) = clipboard.read_text() else {
    println!("clipboard ok: no restorable text clipboard");
    return;
  };
  if clipboard.write_text("tg clipboard smoke") {
    let read_back = clipboard.read_text();
    assert!(
      clipboard.write_text(&previous),
      "restore previous clipboard text"
    );
    assert_eq!(read_back.as_deref(), Some("tg clipboard smoke"));
    println!("clipboard ok: round trip");
  } else {
    println!("clipboard ok: no system clipboard in this environment");
  }
}
