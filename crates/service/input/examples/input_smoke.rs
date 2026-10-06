//! Exercise shared actions and edge-only keyboard states without a terminal device.
//!
//! Run `cargo run -p tg-service-input --example input_smoke --locked`.
//! Injected input verifies matching and cleanup; physical terminal focus remains a separate test.

use tg_core_input::{
  FocusEvent, Key, KeyBinding, KeyEvent, KeyEventKind, KeyPattern, KeyState, SystemEvent,
};
use tg_service_input::InputService;
use tg_service_log::LogService;

fn main() {
  let mut input = InputService::new();
  let mut log = LogService::new();
  input.load_key_bindings(vec![
    KeyBinding {
      action: "pause".into(),
      pattern: KeyPattern::Single(Key::A),
      priority: 0,
    },
    KeyBinding {
      action: "exit".into(),
      pattern: KeyPattern::Single(Key::A),
      priority: 10,
    },
  ]);
  input.begin_frame();

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
  let events = input.collect_action_events();
  assert_eq!(
    events
      .iter()
      .map(|event| event.action.as_str())
      .collect::<Vec<_>>(),
    ["exit", "pause"]
  );
  assert!(events.iter().all(|event| event.state == KeyState::Pressed));
  input.begin_frame();
  input.poll();
  assert!(
    input
      .collect_action_events()
      .iter()
      .all(|event| event.state == KeyState::Held)
  );
  input.begin_frame();
  input.poll();
  assert!(input.notifications().is_empty());
  input.queue_system_event(SystemEvent::Focus(FocusEvent { gained: false }), &mut log);
  input.poll();
  assert_eq!(input.collect_action_events().len(), 2);
  assert!(
    input
      .collect_action_events()
      .iter()
      .all(|event| event.state == KeyState::Released)
  );
  assert!(!input.is_down(Key::A));
  input.begin_frame();
  input.queue_system_event(SystemEvent::Focus(FocusEvent { gained: true }), &mut log);
  input.queue_key_event(
    KeyEvent {
      key: Key::A,
      kind: KeyEventKind::Press,
    },
    &mut log,
  );
  input.poll();
  assert!(!input.is_down(Key::A));
  input.queue_key_event(
    KeyEvent {
      key: Key::A,
      kind: KeyEventKind::Release,
    },
    &mut log,
  );
  input.queue_key_event(
    KeyEvent {
      key: Key::A,
      kind: KeyEventKind::Press,
    },
    &mut log,
  );
  input.poll();
  assert_eq!(input.collect_action_events().len(), 2);
  println!("input ok: shared priority ordering, held once, focus release and fresh-press recovery");
}
