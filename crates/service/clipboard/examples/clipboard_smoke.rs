//! Minimal entry: round-trips a text through the system clipboard (restoring the previous
//! content) when one is available.

use tg_service_clipboard::ClipboardService;

fn main() {
  let mut clipboard = ClipboardService::new();
  let previous = clipboard.read_text();
  if clipboard.write_text("tg clipboard smoke") {
    let read_back = clipboard.read_text();
    clipboard.write_text(previous.as_deref().unwrap_or(""));
    assert_eq!(read_back.as_deref(), Some("tg clipboard smoke"));
    println!("clipboard ok: round trip");
  } else {
    println!("clipboard ok: no system clipboard in this environment");
  }
}
