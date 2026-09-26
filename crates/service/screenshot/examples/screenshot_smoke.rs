//! Minimal entry: extracts plain text from the built-in font preview frame.

use tg_service_screenshot::ScreenshotService;

fn main() {
  let frame = ScreenshotService::font_preview_frame();
  let rect = ScreenshotService::whole_frame_rect(&frame).expect("preview frame is not empty");
  let text = ScreenshotService::plain_text(&frame, rect);
  assert!(!text.trim().is_empty(), "preview frame has visible text");
  println!(
    "screenshot ok: {} lines of plain text",
    text.lines().count()
  );
}
