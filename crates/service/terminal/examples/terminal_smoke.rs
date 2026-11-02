//! Independent terminal smoke entry exercising the public API and checking its results.
//!
//! Run `cargo run -p tg-service-terminal --example terminal_smoke --locked` for passive checks.
//! Add `-- --interactive` to exercise raw mode, title output, and restoration on a real terminal.

use tg_service_terminal::TerminalService;

fn main() {
  let mut terminal = TerminalService::new();
  terminal.apply_capability_profile(Some(true), Some("truecolor"), Some(true));
  assert!(terminal.capabilities().truecolor);
  assert!(!terminal.is_active());
  assert_eq!(
    terminal.set_title("Tui Game").unwrap_err().kind(),
    std::io::ErrorKind::NotConnected
  );
  if std::env::args().any(|argument| argument == "--interactive") {
    terminal.enter().expect("enter terminal");
    terminal.set_title("Tui Game").expect("set title");
    terminal.exit();
    assert!(!terminal.is_active());
  }
  println!("terminal ok: {:?}", terminal.capabilities());
}
