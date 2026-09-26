//! Minimal entry: writes a text file through the async executor and reads it back.

use std::time::{Duration, Instant};

use tg_service_async::{AsyncRuntime, TaskStatusEvent};
use tg_service_file::{FileEvent, FileService};

#[derive(Debug)]
enum Event {
  File(FileEvent),
  /// Executor status; this example only looks at file events.
  Status,
}

impl From<FileEvent> for Event {
  fn from(event: FileEvent) -> Self {
    Self::File(event)
  }
}

impl From<TaskStatusEvent> for Event {
  fn from(_: TaskStatusEvent) -> Self {
    Self::Status
  }
}

/// Only one task is in flight at a time, so the first file event belongs to it.
fn next_file_event(runtime: &AsyncRuntime<Event>) -> FileEvent {
  let deadline = Instant::now() + Duration::from_secs(5);
  while Instant::now() < deadline {
    for event in runtime.poll_events() {
      if let Event::File(event) = event {
        return event;
      }
    }
    std::thread::sleep(Duration::from_millis(5));
  }
  panic!("no file event within 5 seconds");
}

fn main() {
  let path = std::env::temp_dir().join(format!("tg_file_smoke_{}.txt", std::process::id()));
  let file = FileService::new();
  let runtime = AsyncRuntime::<Event>::with_worker_count(1);

  let write = file.write_text(&runtime, path.clone(), "hello".to_string());
  match next_file_event(&runtime) {
    FileEvent::WriteTextFinished { task_id, .. } => assert_eq!(task_id, write),
    other => panic!("unexpected event after write: {other:?}"),
  }

  let read = file.read_text(&runtime, path.clone());
  match next_file_event(&runtime) {
    FileEvent::ReadTextFinished { task_id, text, .. } => {
      assert_eq!(task_id, read);
      assert_eq!(text, "hello");
    }
    other => panic!("unexpected event after read: {other:?}"),
  }

  let _ = std::fs::remove_file(&path);
  println!("file ok: wrote and read back {}", path.display());
}
