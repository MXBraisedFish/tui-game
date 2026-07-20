use super::*;
use crate::host_engine::services::{EngineEvent, ExportAsyncEvent, ScreenshotAsyncEvent};

pub(super) struct RuntimeEngineEvents {
  pub package: Vec<PackageEvent>,
  pub export: Vec<ExportAsyncEvent>,
  pub screenshot: Vec<ScreenshotAsyncEvent>,
}

pub(super) fn drain_engine_events(services: &mut EngineServices) -> RuntimeEngineEvents {
  let mut package_events = Vec::new();
  let mut export_events = Vec::new();
  let mut screenshot_events = Vec::new();

  for event in services.engine_events.drain() {
    match event {
      EngineEvent::InputKey(event) => services.input.queue_key_event(event, &mut services.log),
      EngineEvent::System(event) => services.input.queue_system_event(event, &mut services.log),
      EngineEvent::Package(event) => {
        let event = services
          .package
          .handle_async_event(event, &mut services.log);
        if matches!(event, PackageEvent::WatchChanged { .. }) {
          let _ = services.package.request_rescan(&services.async_runtime);
        }
        package_events.push(event);
      }
      EngineEvent::Export(event) => export_events.push(event),
      EngineEvent::Screenshot(event) => match event {
        ScreenshotAsyncEvent::Progress {
          task_id,
          completed_rows,
          total_rows,
        } => screenshot_events.push(ScreenshotAsyncEvent::Progress {
          task_id,
          completed_rows,
          total_rows,
        }),
        ScreenshotAsyncEvent::Saved { task_id, png_path } => {
          services.log.info(
            LogSource::Storage,
            format!(
              "Screenshot task {task_id:?} saved PNG: {}",
              png_path.display()
            ),
          );
          screenshot_events.push(ScreenshotAsyncEvent::Saved { task_id, png_path });
        }
        ScreenshotAsyncEvent::Failed { task_id, error } => {
          services.log.warn(
            LogSource::Storage,
            format!("Screenshot task {task_id:?} failed: {error}"),
          );
          screenshot_events.push(ScreenshotAsyncEvent::Failed { task_id, error });
        }
      },
      EngineEvent::Recording(event) => {
        services.recording.handle_engine_event(&event);
        match event {
          crate::host_engine::services::RecordingAsyncEvent::Saved { task_id, path } => {
            services.log.info(
              LogSource::Storage,
              format!("Recording task {task_id:?} saved: {}", path.display()),
            );
          }
          crate::host_engine::services::RecordingAsyncEvent::Failed { task_id, error } => {
            services.log.warn(
              LogSource::Storage,
              format!("Recording task {task_id:?} failed: {error}"),
            );
          }
        }
      }
      EngineEvent::File(_)
      | EngineEvent::Image(_)
      | EngineEvent::Network(_)
      | EngineEvent::Time(_)
      | EngineEvent::TaskFinished { .. } => {}
      EngineEvent::TaskFailed { id, error } => {
        services.log.warn(
          LogSource::Engine,
          format!("Async task {id:?} failed: {error}"),
        );
      }
      EngineEvent::Log { source, message } => {
        services.log.warn(source, message);
      }
    }
  }

  RuntimeEngineEvents {
    package: package_events,
    export: export_events,
    screenshot: screenshot_events,
  }
}
