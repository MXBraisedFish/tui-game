use super::*;
use crate::host_engine::services::EngineEvent;

pub(super) fn drain_engine_events(services: &mut EngineServices) -> Vec<PackageEvent> {
  let mut package_events = Vec::new();

  for event in services.engine_events.drain() {
    match event {
      EngineEvent::InputKey(event) => services.input.queue_key_event(event),
      EngineEvent::System(event) => services.input.queue_system_event(event),
      EngineEvent::Package(event) => {
        let event = services
          .package
          .handle_async_event(event, &mut services.log);
        if matches!(event, PackageEvent::WatchChanged { .. }) {
          let _ = services.package.request_rescan(&services.async_runtime);
        }
        package_events.push(event);
      }
      EngineEvent::File(_)
      | EngineEvent::Image(_)
      | EngineEvent::Network(_)
      | EngineEvent::Time(_)
      | EngineEvent::TaskFinished { .. }
      | EngineEvent::TaskFailed { .. } => {}
    }
  }

  package_events
}
