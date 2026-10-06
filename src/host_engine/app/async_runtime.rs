//! Application completion-event aggregation and conversion into service-owned Lua routing inputs.

use crate::host_engine::services::{
  AudioAsyncEvent, CommittedTextEvent, ExportAsyncEvent, FileEvent, ImageEvent, InputListenerError,
  KeyEvent, LogSource, LuaRoutableEvent, NetworkEvent, PackageAsyncEvent, RecordingAsyncEvent,
  ScreenshotAsyncEvent, SystemEvent, TimeAsyncEvent, VideoAsyncEvent,
};

/// A engine event payload queued for its owning consumer.
#[derive(Clone, Debug)]
pub enum EngineEvent {
  /// A input key notification delivered to the owning consumer.
  InputKey(KeyEvent),
  /// Text submitted through the terminal input stream.
  CommittedText(CommittedTextEvent),
  /// A system notification delivered to the owning consumer.
  System(SystemEvent),
  /// A package notification delivered to the owning consumer.
  Package(PackageAsyncEvent),
  /// A export notification delivered to the owning consumer.
  Export(ExportAsyncEvent),
  /// A screenshot notification delivered to the owning consumer.
  Screenshot(ScreenshotAsyncEvent),
  /// A recording notification delivered to the owning consumer.
  Recording(RecordingAsyncEvent),
  /// A video notification delivered to the owning consumer.
  Video(VideoAsyncEvent),
  /// A file notification delivered to the owning consumer.
  File(FileEvent),
  /// A image notification delivered to the owning consumer.
  Image(ImageEvent),
  /// A network notification delivered to the owning consumer.
  Network(NetworkEvent),
  /// A audio notification delivered to the owning consumer.
  Audio(AudioAsyncEvent),
  /// A time notification delivered to the owning consumer.
  Time(TimeAsyncEvent),
  /// A task finished notification delivered to the owning consumer.
  TaskFinished,
  /// A task failed notification delivered to the owning consumer.
  TaskFailed {
    /// The error.
    error: String,
  },
  /// A log notification delivered to the owning consumer.
  Log {
    /// The log source carried by this engine event.
    source: LogSource,
    /// The diagnostic or display message.
    message: String,
  },
}

pub use tg_service_async::TaskStatusEvent;

/// The shared type used for async runtime.
pub type AsyncRuntime = tg_service_async::AsyncRuntime<EngineEvent>;

impl From<TaskStatusEvent> for EngineEvent {
  fn from(event: TaskStatusEvent) -> Self {
    match event {
      TaskStatusEvent::Finished { .. } => Self::TaskFinished,
      TaskStatusEvent::Failed { error, .. } => Self::TaskFailed { error },
    }
  }
}

impl From<KeyEvent> for EngineEvent {
  fn from(event: KeyEvent) -> Self {
    Self::InputKey(event)
  }
}

impl From<InputListenerError> for EngineEvent {
  fn from(error: InputListenerError) -> Self {
    Self::Log {
      source: LogSource::Input,
      message: error.0,
    }
  }
}

impl EngineEvent {
  /// Return the current Lua routable.
  pub fn lua_routable(&self) -> Option<LuaRoutableEvent<'_>> {
    Some(match self {
      Self::Audio(event) => LuaRoutableEvent::Audio(event),
      Self::File(event) => LuaRoutableEvent::File(event),
      Self::Image(event) => LuaRoutableEvent::Image(event),
      Self::Network(event) => LuaRoutableEvent::Network(event),
      Self::Time(event) => LuaRoutableEvent::Time(event),
      _ => return None,
    })
  }
}

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
  CommittedTextEvent => CommittedText,
  ExportAsyncEvent => Export,
  FileEvent => File,
  ImageEvent => Image,
  NetworkEvent => Network,
  PackageAsyncEvent => Package,
  RecordingAsyncEvent => Recording,
  ScreenshotAsyncEvent => Screenshot,
  SystemEvent => System,
  TimeAsyncEvent => Time,
  VideoAsyncEvent => Video,
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::host_engine::services::{FileTask, SleepTask};
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
    runtime.submit(FileTask::ReadText {
      path: PathBuf::from("missing-file-for-async-runtime-test.txt"),
    });

    let mut found = false;
    for _ in 0..50 {
      if runtime
        .poll_events()
        .into_iter()
        .any(|event| matches!(event, EngineEvent::TaskFailed { .. }))
      {
        found = true;
        break;
      }
      thread::sleep(Duration::from_millis(2));
    }

    assert!(found);
  }
}
