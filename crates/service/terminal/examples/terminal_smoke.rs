//! Minimal entry: applies a capability profile without entering raw mode.

use tg_service_terminal::TerminalService;

fn main() {
  let mut terminal = TerminalService::new();
  terminal.apply_capability_profile(Some(true), Some("truecolor"), Some(true));
  assert!(terminal.capabilities().truecolor);
  assert!(!terminal.is_active());
  println!("terminal ok: {:?}", terminal.capabilities());
}
