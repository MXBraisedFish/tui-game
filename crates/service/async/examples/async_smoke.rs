//! Minimal entry: runs one job on a worker thread and collects its event and completion status.

use std::time::{Duration, Instant};

use crossbeam_channel::Sender;
use tg_service_async::{
  AsyncJob, AsyncRuntime, TaskCancellation, TaskId, TaskState, TaskStatusEvent,
};

#[derive(Debug, PartialEq)]
enum Event {
  Answer(u32),
  Status(TaskStatusEvent),
}

impl From<TaskStatusEvent> for Event {
  fn from(event: TaskStatusEvent) -> Self {
    Self::Status(event)
  }
}

struct Answer;

impl AsyncJob<Event> for Answer {
  fn run(
    self: Box<Self>,
    _id: TaskId,
    events: &Sender<Event>,
    _cancellation: &TaskCancellation,
  ) -> Result<(), String> {
    let _ = events.send(Event::Answer(42));
    Ok(())
  }
}

fn main() {
  let runtime = AsyncRuntime::<Event>::with_worker_count(1);
  let id = runtime.submit(Answer);
  let deadline = Instant::now() + Duration::from_secs(5);
  let mut events = Vec::new();
  while events.len() < 2 && Instant::now() < deadline {
    events.extend(runtime.poll_events());
    std::thread::sleep(Duration::from_millis(5));
  }
  assert_eq!(events, [Event::Answer(42), Event::Status(TaskStatusEvent::Finished { id })]);
  assert_eq!(runtime.task_state(id), Some(TaskState::Finished));
  println!("async ok: {events:?}");
}
