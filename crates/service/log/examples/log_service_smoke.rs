//! Minimal entry: logs a host message with translated labels and reads it back from memory.

use std::collections::HashMap;

use tg_core_log::{HostLogMessage, LogSource};
use tg_service_log::{LogService, format_log_entry};

fn main() {
  let mut log = LogService::new();
  let templates = HashMap::from([("log.smoke".to_string(), "hello {name}".to_string())]);
  log
    .refresh_labels(
      |key| (key == "log.service.engine").then(|| "Engine*".to_string()),
      templates,
    )
    .expect("no file output configured");
  log.info_message(
    LogSource::Engine,
    HostLogMessage::new("log.smoke", "fallback {name}").param("name", "tg"),
  );
  let entry = log.entries().back().expect("one entry");
  assert_eq!(entry.message, "hello tg");
  println!("log ok: {}", format_log_entry(entry));
}
