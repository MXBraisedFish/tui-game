//! Independent input smoke entry exercising the public API and checking its results.

use tg_core_input::{
  ActionMapEntry, Key, KeyPattern, key_token, parse_key_token, translate_action_map,
};

fn main() {
  assert_eq!(parse_key_token(&key_token(Key::A)), Some(Key::A));
  let entries = [ActionMapEntry {
    action: "jump".to_string(),
    description: "Jump".to_string(),
    keys: vec![vec!["space".to_string()]],
    priority: 0,
  }];
  let bindings = translate_action_map(&entries).expect("valid action map");
  assert_eq!(bindings[0].pattern, KeyPattern::Single(Key::Space));
  println!("input ok: {bindings:?}");
}
