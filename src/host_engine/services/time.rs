use std::time::Duration;

use super::widget::runtime_object::RuntimeObjectPool;
use super::widget::runtime_object::time::{
  Timer, TimerEvent, TimerId, TimerMode, TimerOptions, TimerState,
};

pub struct TimeService;

impl TimeService {
  pub fn new() -> Self {
    Self
  }

  pub fn update(&self, pool: &mut RuntimeObjectPool, dt: Duration) {
    for (id, timer) in &mut pool.timers.timers {
      if timer.state != TimerState::Running {
        continue;
      }

      timer.elapsed = timer.elapsed.saturating_add(dt);

      if let TimerMode::CountDown { duration } = timer.mode {
        if timer.elapsed >= duration {
          timer.elapsed = duration;
          timer.state = TimerState::Finished;

          if timer.options.emit_finished {
            pool
              .timers
              .events
              .push_back(TimerEvent::Finished { id: *id });
          }
        }
      }
    }
  }

  pub fn create_count_up(&self, pool: &mut RuntimeObjectPool) -> TimerId {
    self.create(pool, TimerMode::CountUp, TimerOptions::default())
  }

  pub fn create_count_down(
    &self,
    pool: &mut RuntimeObjectPool,
    duration: Duration,
    options: TimerOptions,
  ) -> Option<TimerId> {
    (duration > Duration::ZERO).then(|| self.create(pool, TimerMode::CountDown { duration }, options))
  }

  pub fn remove(&self, pool: &mut RuntimeObjectPool, id: TimerId) -> bool {
    let removed = pool.timers.timers.remove(&id).is_some();
    if removed {
      pool.clear_timer_events(id);
    }
    removed
  }

  pub fn start(&self, pool: &mut RuntimeObjectPool, id: TimerId) -> bool {
    let Some(timer) = pool.timers.timers.get_mut(&id) else {
      return false;
    };
    if matches!(timer.state, TimerState::Finished | TimerState::Stopped) {
      timer.elapsed = Duration::ZERO;
    }
    timer.state = TimerState::Running;
    true
  }

  pub fn pause(&self, pool: &mut RuntimeObjectPool, id: TimerId) -> bool {
    let Some(timer) = pool.timers.timers.get_mut(&id) else {
      return false;
    };
    if timer.state != TimerState::Running {
      return false;
    }
    timer.state = TimerState::Paused;
    true
  }

  pub fn resume(&self, pool: &mut RuntimeObjectPool, id: TimerId) -> bool {
    let Some(timer) = pool.timers.timers.get_mut(&id) else {
      return false;
    };
    if timer.state != TimerState::Paused {
      return false;
    }
    timer.state = TimerState::Running;
    true
  }

  pub fn stop(&self, pool: &mut RuntimeObjectPool, id: TimerId) -> bool {
    let Some(timer) = pool.timers.timers.get_mut(&id) else {
      return false;
    };
    timer.elapsed = Duration::ZERO;
    timer.state = TimerState::Stopped;
    true
  }

  pub fn reset(&self, pool: &mut RuntimeObjectPool, id: TimerId) -> bool {
    let Some(timer) = pool.timers.timers.get_mut(&id) else {
      return false;
    };
    timer.elapsed = Duration::ZERO;
    timer.state = TimerState::Idle;
    pool.clear_timer_events(id);
    true
  }

  pub fn state(&self, pool: &RuntimeObjectPool, id: TimerId) -> Option<TimerState> {
    Some(pool.timers.timers.get(&id)?.state)
  }

  pub fn elapsed(&self, pool: &RuntimeObjectPool, id: TimerId) -> Option<Duration> {
    Some(pool.timers.timers.get(&id)?.elapsed)
  }

  pub fn duration(&self, pool: &RuntimeObjectPool, id: TimerId) -> Option<Duration> {
    pool.timers.timers.get(&id)?.duration()
  }

  pub fn remaining(&self, pool: &RuntimeObjectPool, id: TimerId) -> Option<Duration> {
    pool.timers.timers.get(&id)?.remaining()
  }

  pub fn progress(&self, pool: &RuntimeObjectPool, id: TimerId) -> Option<f32> {
    pool.timers.timers.get(&id)?.progress()
  }

  pub fn is_running(&self, pool: &RuntimeObjectPool, id: TimerId) -> bool {
    self.state(pool, id) == Some(TimerState::Running)
  }

  pub fn is_paused(&self, pool: &RuntimeObjectPool, id: TimerId) -> bool {
    self.state(pool, id) == Some(TimerState::Paused)
  }

  pub fn is_finished(&self, pool: &RuntimeObjectPool, id: TimerId) -> bool {
    self.state(pool, id) == Some(TimerState::Finished)
  }

  pub fn take_events(&self, pool: &mut RuntimeObjectPool, id: TimerId) -> Vec<TimerEvent> {
    pool.take_timer_events(id)
  }

  fn create(
    &self,
    pool: &mut RuntimeObjectPool,
    mode: TimerMode,
    options: TimerOptions,
  ) -> TimerId {
    let id = TimerId(pool.timers.next_id);
    pool.timers.next_id += 1;
    pool.timers.timers.insert(id, Timer::new(mode, options));
    id
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn service_pool() -> (TimeService, RuntimeObjectPool) {
    (TimeService::new(), RuntimeObjectPool::new())
  }

  #[test]
  fn timer_ids_are_unique() {
    let (service, mut pool) = service_pool();

    let first = service.create_count_up(&mut pool);
    let second = service.create_count_up(&mut pool);

    assert_ne!(first, second);
  }

  #[test]
  fn count_up_accumulates_only_while_running() {
    let (service, mut pool) = service_pool();
    let id = service.create_count_up(&mut pool);

    service.update(&mut pool, Duration::from_secs(1));
    assert_eq!(service.elapsed(&pool, id), Some(Duration::ZERO));

    assert!(service.start(&mut pool, id));
    service.update(&mut pool, Duration::from_secs(2));
    assert_eq!(service.elapsed(&pool, id), Some(Duration::from_secs(2)));

    assert!(service.pause(&mut pool, id));
    service.update(&mut pool, Duration::from_secs(3));
    assert_eq!(service.elapsed(&pool, id), Some(Duration::from_secs(2)));
  }

  #[test]
  fn count_down_finishes_and_clamps_elapsed() {
    let (service, mut pool) = service_pool();
    let id = service
      .create_count_down(&mut pool, Duration::from_secs(5), TimerOptions::default())
      .unwrap();

    assert!(service.start(&mut pool, id));
    service.update(&mut pool, Duration::from_secs(7));

    assert_eq!(service.state(&pool, id), Some(TimerState::Finished));
    assert_eq!(service.elapsed(&pool, id), Some(Duration::from_secs(5)));
    assert_eq!(service.remaining(&pool, id), Some(Duration::ZERO));
    assert_eq!(service.progress(&pool, id), Some(1.0));
  }

  #[test]
  fn finished_event_requires_subscription_and_is_emitted_once() {
    let (service, mut pool) = service_pool();
    let silent = service
      .create_count_down(&mut pool, Duration::from_secs(1), TimerOptions::default())
      .unwrap();
    let subscribed = service
      .create_count_down(
        &mut pool,
        Duration::from_secs(1),
        TimerOptions {
          emit_finished: true,
        },
      )
      .unwrap();

    assert!(service.start(&mut pool, silent));
    assert!(service.start(&mut pool, subscribed));
    service.update(&mut pool, Duration::from_secs(1));
    service.update(&mut pool, Duration::from_secs(1));

    assert!(service.take_events(&mut pool, silent).is_empty());
    assert_eq!(
      service.take_events(&mut pool, subscribed),
      vec![TimerEvent::Finished { id: subscribed }]
    );
    assert!(service.take_events(&mut pool, subscribed).is_empty());
  }

  #[test]
  fn stop_and_reset_clear_elapsed_state_and_events() {
    let (service, mut pool) = service_pool();
    let id = service
      .create_count_down(
        &mut pool,
        Duration::from_secs(1),
        TimerOptions {
          emit_finished: true,
        },
      )
      .unwrap();

    assert!(service.start(&mut pool, id));
    service.update(&mut pool, Duration::from_secs(1));
    assert!(!service.take_events(&mut pool, id).is_empty());

    assert!(service.start(&mut pool, id));
    service.update(&mut pool, Duration::from_secs(1));
    assert!(service.reset(&mut pool, id));

    assert_eq!(service.state(&pool, id), Some(TimerState::Idle));
    assert_eq!(service.elapsed(&pool, id), Some(Duration::ZERO));
    assert!(service.take_events(&mut pool, id).is_empty());

    assert!(service.start(&mut pool, id));
    service.update(&mut pool, Duration::from_millis(500));
    assert!(service.stop(&mut pool, id));
    assert_eq!(service.state(&pool, id), Some(TimerState::Stopped));
    assert_eq!(service.elapsed(&pool, id), Some(Duration::ZERO));
  }

  #[test]
  fn remove_clears_timer_and_events() {
    let (service, mut pool) = service_pool();
    let id = service
      .create_count_down(
        &mut pool,
        Duration::from_secs(1),
        TimerOptions {
          emit_finished: true,
        },
      )
      .unwrap();

    assert!(service.start(&mut pool, id));
    service.update(&mut pool, Duration::from_secs(1));
    assert!(service.remove(&mut pool, id));

    assert_eq!(service.state(&pool, id), None);
    assert!(service.take_events(&mut pool, id).is_empty());
  }

  #[test]
  fn count_up_has_no_count_down_queries() {
    let (service, mut pool) = service_pool();
    let id = service.create_count_up(&mut pool);

    assert_eq!(service.duration(&pool, id), None);
    assert_eq!(service.remaining(&pool, id), None);
    assert_eq!(service.progress(&pool, id), None);
  }
}
