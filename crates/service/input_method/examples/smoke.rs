//! Minimal entry: detects the ASCII input method without switching anything.

use tg_service_input_method::{ImPolicy, InputMethodService};

fn main() {
  let service = InputMethodService::new();
  assert_eq!(service.policy(), ImPolicy::Free);
  assert!(!service.is_input_method_restricted());
  println!(
    "input_method ok: ascii={:?} error={:?}",
    service.ascii_input_method(),
    service.last_error()
  );
}
