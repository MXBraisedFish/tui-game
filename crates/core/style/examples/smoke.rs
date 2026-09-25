//! Minimal entry: parses a color and applies it to a style.

use tg_core_style::{TextColor, TextStyle, parse_text_color};

fn main() {
  let color = parse_text_color("#ff8000").expect("hex color");
  assert_eq!(color, TextColor::Rgb { r: 255, g: 128, b: 0 });
  let mut style = TextStyle::default();
  style.set_foreground(color);
  assert!(style.enable_style("bold"));
  println!("style ok: {style:?}");
}
