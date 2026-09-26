use std::path::PathBuf;

use crossbeam_channel::Sender;
use tg_core_atomic_fs::temporary_path;
use tg_service_async::AsyncJob;

use super::{
  audio::AudioAsyncEvent,
  export::{self, ExportAsyncEvent, ExportTask},
  file::FileEvent,
  image::{ImageConvertParams, ImageService},
  input::{KeyEvent, SystemEvent},
  log::LogSource,
  network::{NetworkEvent, NetworkTask},
  package::{self, PackageAsyncEvent, PackageTask},
  recording::{self, RecordingAsyncEvent, RecordingTask},
  screenshot::{self, ScreenshotAsyncEvent, ScreenshotTask},
  time::TimeAsyncEvent,
  video::{self, VideoAsyncEvent, VideoExportTask},
};

#[derive(Clone, Debug)]
pub enum ImageTask {
  Convert {
    params: ImageConvertParams,
    cache_dir: Option<PathBuf>,
  },
}

#[derive(Clone, Debug)]
pub enum ImageEvent {
  ConvertFinished { task_id: TaskId, output: String },
  Failed { task_id: TaskId, error: String },
}

#[derive(Clone, Debug)]
pub enum EngineTask {
  Package(PackageTask),
  Export(ExportTask),
  Screenshot(ScreenshotTask),
  Recording(RecordingTask),
  Video(VideoExportTask),
  Image(ImageTask),
  Network(NetworkTask),
}

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

impl From<FileEvent> for EngineEvent {
  fn from(event: FileEvent) -> Self {
    Self::File(event)
  }
}

impl From<TimeAsyncEvent> for EngineEvent {
  fn from(event: TimeAsyncEvent) -> Self {
    Self::Time(event)
  }
}

impl AsyncJob<EngineEvent> for EngineTask {
  fn run(
    self: Box<Self>,
    id: TaskId,
    events: &Sender<EngineEvent>,
    cancellation: &TaskCancellation,
  ) -> Result<(), String> {
    run_task(id, *self, events, cancellation)
  }

  fn write_target(&self, id: TaskId) -> Option<(PathBuf, PathBuf)> {
    let target = write_target(self)?;
    let temporary = temporary_target(self, &target, id);
    Some((target, temporary))
  }

  fn cancelled_before_start(&self, id: TaskId, events: &Sender<EngineEvent>) {
    if let EngineTask::Network(task) = self {
      super::network::emit_cancelled(id, task, events);
    }
  }

  fn reports_own_cancellation(&self) -> bool {
    matches!(self, EngineTask::Network(_))
  }
}

fn write_target(task: &EngineTask) -> Option<PathBuf> {
  match task {
    EngineTask::Package(_) | EngineTask::Image(_) | EngineTask::Network(_) => None,
    EngineTask::Export(task) => Some(task.output_dir.join(format!(
      "{}.{}",
      task.file_stem,
      task.format.extension()
    ))),
    EngineTask::Screenshot(task) => Some(task.png_path.clone()),
    EngineTask::Recording(task) => Some(task.path().to_path_buf()),
    EngineTask::Video(task) => Some(task.output_path.clone()),
  }
}

fn temporary_target(task: &EngineTask, target: &std::path::Path, task_id: TaskId) -> PathBuf {
  if matches!(task, EngineTask::Video(_)) {
    return target.with_extension(format!("mp4.task-{}.part", task_id.0));
  }
  temporary_path(target)
}

fn run_task(
  id: TaskId,
  task: EngineTask,
  event_tx: &Sender<EngineEvent>,
  cancellation: &TaskCancellation,
) -> Result<(), String> {
  match task {
    EngineTask::Package(task) => package::run_package_task(id, task, event_tx),
    EngineTask::Export(task) => export::run_export_task(id, task, event_tx, cancellation),
    EngineTask::Screenshot(task) => {
      screenshot::run_screenshot_task(id, task, event_tx, cancellation)
    }
    EngineTask::Recording(task) => recording::run_recording_task(id, task, event_tx),
    EngineTask::Video(task) => video::run_video_task(id, task, event_tx, cancellation),
    EngineTask::Image(task) => run_image_task(id, task, event_tx),
    EngineTask::Network(task) => super::network::run_network_task(id, task, event_tx, cancellation),
  }
}

fn run_image_task(
  task_id: TaskId,
  task: ImageTask,
  event_tx: &Sender<EngineEvent>,
) -> Result<(), String> {
  match task {
    ImageTask::Convert { params, cache_dir } => {
      match ImageService::new(cache_dir).convert(params) {
        Ok(output) => {
          let _ = event_tx.send(EngineEvent::Image(ImageEvent::ConvertFinished {
            task_id,
            output,
          }));
          Ok(())
        }
        Err(error) => {
          let _ = event_tx.send(EngineEvent::Image(ImageEvent::Failed {
            task_id,
            error: error.clone(),
          }));
          Err(error)
        }
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::host_engine::services::file::FileTask;
  use crate::host_engine::services::time::SleepTask;
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
