use super::*;
use crate::host_engine::services::{EngineEvent, ExportAsyncEvent};

pub(super) struct RuntimeEngineEvents {
  pub package: Vec<PackageEvent>,
  pub export: Vec<ExportAsyncEvent>,
}

pub(super) fn drain_engine_events(services: &mut EngineServices) -> RuntimeEngineEvents {
  let mut package_events = Vec::new();
  let mut export_events = Vec::new();

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
  }
}
