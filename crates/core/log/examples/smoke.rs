//! Minimal entry: renders a host log message and maps a source to its phase.

use tg_core_log::{HostLogMessage, LogLevel, LogPhase, LogSource, format_log_level};

fn main() {
  let message = HostLogMessage::new("log.demo", "loaded {count} packages").param("count", "3");
  assert_eq!(message.render(None), "loaded 3 packages");
  assert_eq!(message.render(Some("{count} ok")), "3 ok");
  assert_eq!(LogSource::Boot.phase(), LogPhase::Boot);
  println!("log ok: [{}] {}", format_log_level(LogLevel::Info), message.render(None));
}
