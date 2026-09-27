//! Minimal entry: inserts, removes and reuses one arena slot.

use tg_core_arena::Arena;

fn main() {
  let mut arena = Arena::new();
  let handle = arena.insert(42);
  assert_eq!(arena.remove(handle.0, handle.1), Some(42));
  let reused = arena.insert(7);
  assert_eq!(reused.0, handle.0);
  assert_ne!(reused.1, handle.1);
  println!("arena ok: slot {} generation {}", reused.0, reused.1);
}
