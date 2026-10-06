//! Independent unicode smoke entry exercising the public API and checking its results.

use tg_core_unicode::{char_width, display_width, graphemes};

fn main() {
  assert_eq!(display_width("Hello世界"), 9);
  assert_eq!(char_width('你'), 2);
  let parts = graphemes("e\u{0301}好");
  assert_eq!(parts.len(), 2);
  assert_eq!(parts[1].display_width, 2);
  println!("unicode ok: {parts:?}");
}
