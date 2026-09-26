use super::{
  audio::AudioAsyncEvent,
  export::ExportAsyncEvent,
  file::FileEvent,
  image::ImageEvent,
  input::{KeyEvent, SystemEvent},
  log::LogSource,
  network::NetworkEvent,
  package::PackageAsyncEvent,
  recording::RecordingAsyncEvent,
  screenshot::ScreenshotAsyncEvent,
  time::TimeAsyncEvent,
  video::VideoAsyncEvent,
};

#[derive(Clone, Debug)]
pub enum EngineEvent {
  InputKey(KeyEvent),
  System(SystemEvent),
  Package(PackageAsyncEvent),
  Export(ExportAsyncEvent),
  Screenshot(ScreenshotAsyncEvent),
  Recording(RecordingAsyncEvent),
  Video(VideoAsyncEvent),
  File(FileEvent),
  Image(ImageEvent),
  Network(NetworkEvent),
  Audio(AudioAsyncEvent),
  Time(TimeAsyncEvent),
  TaskFinished { id: TaskId },
  TaskFailed { id: TaskId, error: String },
  Log { source: LogSource, message: String },
}

pub use tg_service_async::{ManagedThreadId, TaskCancellation, TaskId, TaskState, TaskStatusEvent};

/// 宿主的异步执行器，事件类型为 [`EngineEvent`]。
pub type AsyncRuntime = tg_service_async::AsyncRuntime<EngineEvent>;

impl From<TaskStatusEvent> for EngineEvent {
  fn from(event: TaskStatusEvent) -> Self {
    match event {
      TaskStatusEvent::Finished { id } => Self::TaskFinished { id },
      TaskStatusEvent::Failed { id, error } => Self::TaskFailed { id, error },
    }
  }
}

/// Wraps each service event type into its [`EngineEvent`] variant.
macro_rules! engine_event_from {
  ($($event:ty => $variant:ident),* $(,)?) => {
    $(
      impl From<$event> for EngineEvent {
        fn from(event: $event) -> Self {
          Self::$variant(event)
        }
      }
    )*
  };
}

engine_event_from! {
  AudioAsyncEvent => Audio,
  ExportAsyncEvent => Export,
  FileEvent => File,
  ImageEvent => Image,
  NetworkEvent => Network,
  PackageAsyncEvent => Package,
  RecordingAsyncEvent => Recording,
  ScreenshotAsyncEvent => Screenshot,
  TimeAsyncEvent => Time,
  VideoAsyncEvent => Video,
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::host_engine::services::file::FileTask;
  use crate::host_engine::services::time::SleepTask;
  use std::path::PathBuf;
  use std::thread;
  use std::time::Duration;

  #[test]
  fn async_runtime_assigns_unique_task_ids() {
    let runtime = AsyncRuntime::with_worker_count(1);
    let first = runtime.submit(SleepTask {
      duration: Duration::ZERO,
      callback: None,
    });
    let second = runtime.submit(SleepTask {
      duration: Duration::ZERO,
      callback: None,
    });

    assert_ne!(first, second);
  }

  #[test]
  fn sleep_task_returns_time_event() {
    let runtime = AsyncRuntime::with_worker_count(1);
    let task_id = runtime.submit(SleepTask {
      duration: Duration::from_millis(1),
      callback: None,
    });

    let mut found = false;
    for _ in 0..50 {
      if runtime.poll_events().into_iter().any(|event| {
        matches!(
            event,
            EngineEvent::Time(TimeAsyncEvent::SleepFinished { task_id: id, .. })
                if id == task_id
        )
      }) {
        found = true;
        break;
      }
      thread::sleep(Duration::from_millis(2));
    }

    assert!(found);
  }

  #[test]
  fn failed_file_task_emits_task_failed() {
    let runtime = AsyncRuntime::with_worker_count(1);
    let task_id = runtime.submit(FileTask::ReadText {
      path: PathBuf::from("missing-file-for-async-runtime-test.txt"),
    });

    let mut found = false;
    for _ in 0..50 {
      if runtime
        .poll_events()
        .into_iter()
        .any(|event| matches!(event, EngineEvent::TaskFailed { id, .. } if id == task_id))
      {
        found = true;
        break;
      }
      thread::sleep(Duration::from_millis(2));
    }

    assert!(found);
  }
}
