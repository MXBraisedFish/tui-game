//! Minimal entry: parses a color, applies it to a style and places a styled cell in a composed frame.

use tg_core_style::{CanvasCell, ComposedCell, ComposedFrame, TextColor, TextStyle, parse_text_color};

fn main() {
  let color = parse_text_color("#ff8000").expect("hex color");
  assert_eq!(color, TextColor::Rgb { r: 255, g: 128, b: 0 });
  let mut style = TextStyle::default();
  style.set_foreground(color);
  assert!(style.enable_style("bold"));
  let mut frame = ComposedFrame::new(2, 1);
  frame.set(1, 0, ComposedCell::Text(CanvasCell::styled("A", style.clone())));
  assert_eq!(frame.get(0, 0), Some(&ComposedCell::Empty));
  println!("style ok: {:?}", frame.get(1, 0));
}
