//! Minimal entry: rejects an unsupported URL, then runs one request whose loopback target is refused.

use std::time::{Duration, Instant};

use tg_service_async::{AsyncRuntime, TaskStatusEvent};
use tg_service_network::{
  NetworkErrorCode, NetworkEvent, NetworkRequest, NetworkRequestStatus, NetworkResponseMode,
  NetworkService,
};

#[derive(Debug)]
enum Event {
  Network(NetworkEvent),
  /// Executor status; this example only looks at network events.
  Status,
}

impl From<NetworkEvent> for Event {
  fn from(event: NetworkEvent) -> Self {
    Self::Network(event)
  }
}

impl From<TaskStatusEvent> for Event {
  fn from(_: TaskStatusEvent) -> Self {
    Self::Status
  }
}

fn main() {
  let runtime = AsyncRuntime::<Event>::with_worker_count(1);
  let mut network = NetworkService::new();

  let unsupported = NetworkRequest::get("ftp://example.com/", NetworkResponseMode::Text);
  let error = network.submit(&runtime, unsupported).unwrap_err();
  assert_eq!(error.code, NetworkErrorCode::Unsupported);

  let loopback = NetworkRequest::get("http://127.0.0.1:9/", NetworkResponseMode::Text);
  let task = network.submit(&runtime, loopback).expect("well-formed request");
  let deadline = Instant::now() + Duration::from_secs(5);
  while network.active_count() > 0 && Instant::now() < deadline {
    for event in runtime.poll_events() {
      if let Event::Network(event) = event {
        network.handle_engine_event(&event);
      }
    }
    std::thread::sleep(Duration::from_millis(5));
  }
  assert_eq!(
    network.status(task),
    Some(&NetworkRequestStatus::Failed {
      code: NetworkErrorCode::PermissionDenied
    })
  );
  println!("network ok: loopback request {task:?} refused");
}
