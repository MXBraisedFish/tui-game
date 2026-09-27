//! Minimal entry: wraps a CJK/ASCII string to a fixed width and measures it.

use tg_service_text_layout::{DrawTextParams, TextWrapMode, layout_text_lines, measure_draw_text};

fn main() {
  let params = DrawTextParams {
    text: "Hello 世界 wrap".to_string(),
    max_width: Some(6),
    wrap_mode: TextWrapMode::Auto,
    ..DrawTextParams::default()
  };
  let lines = layout_text_lines(&params);
  assert!(lines.len() > 1, "text wraps at width 6");
  assert!(lines.iter().all(|line| line.width <= 6));
  let (width, height) = measure_draw_text(&params);
  assert_eq!(height as usize, lines.len());
  println!("text_layout ok: {width}x{height}");
}
