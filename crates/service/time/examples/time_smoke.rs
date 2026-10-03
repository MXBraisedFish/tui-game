//! Independent time smoke entry exercising the public API and checking its results.

use std::time::{Duration, Instant};

use tg_service_async::{AsyncRuntime, TaskStatusEvent};
use tg_service_time::{
  ScheduledTimerOptions, TimeAsyncEvent, TimeObjects, TimeService, TimerState,
};

#[derive(Debug)]
enum Event {
  Time(TimeAsyncEvent),

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
  assert_eq!(
    time.elapsed(&objects, timer),
    Some(Duration::from_millis(250))
  );

  let scheduled = time
    .create_scheduled_timer(
      &mut objects,
      ScheduledTimerOptions {
        duration: Duration::from_millis(10),
        delay: Duration::from_millis(5),
        gap: Duration::from_millis(3),
        repetitions: Some(2),
      },
    )
    .expect("valid countdown schedule");
  assert!(time.start_scheduled_timer(&mut objects, scheduled));
  time.update(&mut objects, Duration::from_millis(15));
  assert!(!time.take_scheduled_timer_events(&mut objects)[0].finished);
  assert!(time.pause_scheduled_timer(&mut objects, scheduled));
  time.update(&mut objects, Duration::from_secs(1));
  assert!(time.take_scheduled_timer_events(&mut objects).is_empty());
  assert!(time.start_scheduled_timer(&mut objects, scheduled));
  time.update(&mut objects, Duration::from_millis(13));
  assert!(time.take_scheduled_timer_events(&mut objects)[0].finished);
  assert_eq!(
    time
      .scheduled_timer_info(&objects, scheduled)
      .unwrap()
      .state,
    TimerState::Finished
  );
  assert!(time.reset_scheduled_timer(&mut objects, scheduled));
  assert!(time.remove_scheduled_timer(&mut objects, scheduled));

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
