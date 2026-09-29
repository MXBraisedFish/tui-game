//! Minimal entry: extracts plain text from the built-in font preview frame.

use tg_service_screenshot::{ScreenshotService, TerminalFrameRasterizer};
use tg_service_storage::RecordingPixelScale;

fn main() {
  let frame = ScreenshotService::font_preview_frame();
  let rect = ScreenshotService::whole_frame_rect(&frame).expect("preview frame is not empty");
  let text = ScreenshotService::plain_text(&frame, rect);
  assert!(!text.trim().is_empty(), "preview frame has visible text");
  println!(
    "screenshot ok: {} lines of plain text",
    text.lines().count()
  );
  // Optional output path makes the same production renderer available for visual checks.
  if let Some(output) = std::env::args_os().nth(1) {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let start = std::time::Instant::now();
    let renderer = TerminalFrameRasterizer::load(&[], &root).expect("load deployment fonts");
    let loaded = start.elapsed();
    let image = renderer
      .render(&frame, rect, RecordingPixelScale::Original, |_, _| true)
      .expect("render preview");
    let rendered = start.elapsed() - loaded;
    image.save(output).expect("save preview PNG");
    println!(
      "{}x{}; font loading {loaded:?}; render {rendered:?}",
      image.width(),
      image.height()
    );
  }
}
