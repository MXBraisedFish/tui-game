//! Minimal entry: parses a tagged string into styled segments.

use tg_service_rich_text::{RichTextService, TextMode};

fn main() {
  let service = RichTextService::new();
  let rich = service.parse_mode("plain <b>bold</b>", None, TextMode::Rich);
  let bold = rich
    .segments
    .iter()
    .find(|segment| segment.text == "bold")
    .expect("bold segment");
  assert!(bold.style.bold);
  println!("rich_text ok: {} segments", rich.segments.len());
}
