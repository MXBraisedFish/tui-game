//! Independent terminal smoke entry exercising the public API and checking its results.

use tg_service_terminal::TerminalService;

fn main() {
  let mut terminal = TerminalService::new();
  terminal.apply_capability_profile(Some(true), Some("truecolor"), Some(true));
  assert!(terminal.capabilities().truecolor);
  assert!(!terminal.is_active());
  println!("terminal ok: {:?}", terminal.capabilities());
}
