//! Independent file smoke entry exercising the public API and checking its results.

use std::{
  path::PathBuf,
  time::{Duration, Instant},
};

use tg_service_async::{AsyncRuntime, TaskStatusEvent};
use tg_service_file::{FileEvent, FileService, FileTask};

#[derive(Debug)]
enum Event {
  File(FileEvent),

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
  let root = create_temp_dir("tg_file_smoke");
  let path = root.join("smoke.txt");
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

  let request = runtime.submit(FileTask::LuaLoadI18n {
    assets_root: root.clone(),
    language_code: "zh_cn".into(),
    callback_language_code: "en_us".into(),
  });
  match next_file_event(&runtime) {
    FileEvent::LuaI18nFinished {
      task_id,
      language_code,
      namespaces,
      warning,
      ..
    } => {
      assert_eq!(task_id, request);
      assert_eq!(language_code, "zh_cn");
      assert!(namespaces.is_empty());
      let warning = warning.expect("missing-language warning");
      assert!(
        warning.contains("primary language 'zh_cn'")
          && warning.contains("fallback language 'en_us'")
      );
    }
    other => panic!("missing translations unexpectedly failed: {other:?}"),
  }

  std::fs::remove_dir_all(&root).expect("clean up temporary directory");
  println!("file ok: wrote and read back {}", path.display());
}

fn create_temp_dir(prefix: &str) -> PathBuf {
  let nonce = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap_or_default()
    .as_nanos();
  let root = std::env::temp_dir().join(format!("{prefix}_{}_{nonce}", std::process::id()));
  std::fs::create_dir(&root)
    .unwrap_or_else(|error| panic!("create temporary directory {}: {error}", root.display()));
  root
}
