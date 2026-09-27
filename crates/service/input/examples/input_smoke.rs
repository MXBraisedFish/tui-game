//! Minimal entry: feeds one key press through the input queue and reads the key state back.

use tg_core_input::{Key, KeyEvent, KeyEventKind};
use tg_service_input::InputService;
use tg_service_log::LogService;

fn main() {
  let mut input = InputService::new();
  let mut log = LogService::new();

  input.queue_key_event(
    KeyEvent {
      key: Key::A,
      kind: KeyEventKind::Press,
    },
    &mut log,
  );
  input.poll();

  assert!(input.is_down(Key::A));
  assert!(input.was_pressed(Key::A));
  println!("input ok: key A pressed");
}
