//! Minimal entry: advances a count-up timer and runs one async sleep job.

use std::time::{Duration, Instant};

use tg_service_async::{AsyncRuntime, TaskStatusEvent};
use tg_service_time::{TimeAsyncEvent, TimeObjects, TimeService};

#[derive(Debug)]
enum Event {
  Time(TimeAsyncEvent),
  /// Executor status; this example only looks at time events.
  Status,
}

impl From<TimeAsyncEvent> for Event {
  fn from(event: TimeAsyncEvent) -> Self {
    Self::Time(event)
  }
}

impl From<TaskStatusEvent> for Event {
  fn from(_: TaskStatusEvent) -> Self {
    Self::Status
  }
}

fn main() {
  let time = TimeService::new();
  let mut objects = TimeObjects::new();
  let timer = time.create_count_up(&mut objects);
  assert!(time.start(&mut objects, timer));
  time.update(&mut objects, Duration::from_millis(250));
  assert_eq!(time.elapsed(&objects, timer), Some(Duration::from_millis(250)));

  let runtime = AsyncRuntime::<Event>::with_worker_count(1);
  let task = time.sleep(&runtime, Duration::from_millis(10), None);
  let deadline = Instant::now() + Duration::from_secs(5);
  let mut woke = false;
  while !woke && Instant::now() < deadline {
    woke = runtime.poll_events().iter().any(|event| {
      matches!(event, Event::Time(TimeAsyncEvent::SleepFinished { task_id, .. }) if *task_id == task)
    });
    std::thread::sleep(Duration::from_millis(5));
  }
  assert!(woke, "sleep job reports completion");
  println!("time ok: timer {timer:?}, sleep {task:?}");
}
