//! Minimal entry: accepts a normal relative path and rejects an escaping one.

use tg_core_sandbox_path::SafeRelativePath;

fn main() {
  let path = SafeRelativePath::parse("saves/slot1.json").expect("normal relative path");
  assert_eq!(path.virtual_path(), "saves/slot1.json");
  assert_eq!(path.extension(), Some("json"));
  assert!(SafeRelativePath::parse("../outside.txt").is_err(), "parent traversal is rejected");
  println!("sandbox_path ok: {}", path.virtual_path());
}
