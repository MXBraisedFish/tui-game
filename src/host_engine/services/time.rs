use std::time::Duration;

use super::widget::runtime_object::RuntimeObjectPool;
use super::widget::runtime_object::time::{
  DelayTimer, DelayTimerEvent, DelayTimerId, DelayTimerOptions, RepeatMode, RepeatTimer,
  RepeatTimerEvent, RepeatTimerId, RepeatTimerOptions, TimeCallbackId, TimeCallbackRequest, Timer,
  TimerEvent, TimerId, TimerMode, TimerOptions, TimerState,
};

use super::async_runtime::{AsyncRuntime, EngineTask, SleepTask, TaskId};

pub struct TimeService;

impl TimeService {
  pub fn new() -> Self {
    Self
  }

  pub fn update(&self, pool: &mut RuntimeObjectPool, dt: Duration) {
    self.update_standalone_timers(pool, dt);
    self.update_delay_timers(pool, dt);
    self.update_repeat_timers(pool, dt);
  }

  pub fn sleep(
    &self,
    async_runtime: &AsyncRuntime,
    duration: Duration,
    callback: Option<TimeCallbackId>,
  ) -> TaskId {
    async_runtime.submit(EngineTask::Sleep(SleepTask { duration, callback }))
  }

  fn update_standalone_timers(&self, pool: &mut RuntimeObjectPool, dt: Duration) {
    let ids = pool.timers.timers.keys().copied().collect::<Vec<_>>();
    for id in ids {
      if pool.timers.composition_owned.contains(&id) {
        continue;
      }

      let Some(timer) = pool.timers.timers.get_mut(&id) else {
        continue;
      };

      if Self::advance_timer(timer, dt) && timer.options.emit_finished {
        pool.timers.events.push_back(TimerEvent::Finished { id });
      }
    }
  }

  fn update_delay_timers(&self, pool: &mut RuntimeObjectPool, dt: Duration) {
    let ids = pool.delay_timers.timers.keys().copied().collect::<Vec<_>>();
    for id in ids {
      let Some(delay) = pool.delay_timers.timers.get(&id) else {
        continue;
      };
      let timer_id = delay.timer_id;
      let report_event_queue = delay.report_event_queue;
      let callback = delay.callback;

      let Some(timer) = pool.timers.timers.get_mut(&timer_id) else {
        continue;
      };
      if !Self::advance_timer(timer, dt) {
        continue;
      }

      if report_event_queue {
        pool
          .delay_timers
          .events
          .push_back(DelayTimerEvent::Finished { id });
      }
      if let Some(callback) = callback {
        pool
          .time_callback_requests
          .push(TimeCallbackRequest::DelayFinished { id, callback });
      }
    }
  }

  fn update_repeat_timers(&self, pool: &mut RuntimeObjectPool, dt: Duration) {
    let ids = pool
      .repeat_timers
      .timers
      .keys()
      .copied()
      .collect::<Vec<_>>();
    for id in ids {
      let Some(repeat) = pool.repeat_timers.timers.get(&id) else {
        continue;
      };
      let timer_id = repeat.timer_id;

      let Some(timer) = pool.timers.timers.get_mut(&timer_id) else {
        continue;
      };
      if !Self::advance_timer(timer, dt) {
        continue;
      }

      let Some(repeat) = pool.repeat_timers.timers.get_mut(&id) else {
        continue;
      };
      repeat.executed_count = repeat.executed_count.saturating_add(1);
      let count = repeat.executed_count;
      let finished = matches!(repeat.repeat_mode, RepeatMode::Count(limit) if count >= limit);

      if repeat.report_event_queue {
        pool.repeat_timers.events.push_back(RepeatTimerEvent::Tick {
          id,
          executed_count: count,
        });
      }
      if let Some(callback) = repeat.callback {
        pool
          .time_callback_requests
          .push(TimeCallbackRequest::RepeatTick {
            id,
            callback,
            executed_count: count,
          });
      }

      if finished {
        if repeat.report_event_queue {
          pool
            .repeat_timers
            .events
            .push_back(RepeatTimerEvent::Finished {
              id,
              executed_count: count,
            });
        }
        if let Some(callback) = repeat.callback {
          pool
            .time_callback_requests
            .push(TimeCallbackRequest::RepeatFinished {
              id,
              callback,
              executed_count: count,
            });
        }
      } else if let Some(timer) = pool.timers.timers.get_mut(&timer_id) {
        timer.elapsed = Duration::ZERO;
        timer.state = TimerState::Running;
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
    (duration > Duration::ZERO)
      .then(|| self.create(pool, TimerMode::CountDown { duration }, options))
  }

  pub fn remove(&self, pool: &mut RuntimeObjectPool, id: TimerId) -> bool {
    if pool.timers.composition_owned.contains(&id) {
      return false;
    }
    let removed = pool.timers.timers.remove(&id).is_some();
    if removed {
      pool.clear_timer_events(id);
    }
    removed
  }

  pub fn start(&self, pool: &mut RuntimeObjectPool, id: TimerId) -> bool {
    if pool.timers.composition_owned.contains(&id) {
      return false;
    }
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
    if pool.timers.composition_owned.contains(&id) {
      return false;
    }
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
    if pool.timers.composition_owned.contains(&id) {
      return false;
    }
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
    if pool.timers.composition_owned.contains(&id) {
      return false;
    }
    let Some(timer) = pool.timers.timers.get_mut(&id) else {
      return false;
    };
    timer.elapsed = Duration::ZERO;
    timer.state = TimerState::Stopped;
    true
  }

  pub fn reset(&self, pool: &mut RuntimeObjectPool, id: TimerId) -> bool {
    if pool.timers.composition_owned.contains(&id) {
      return false;
    }
    let Some(timer) = pool.timers.timers.get_mut(&id) else {
      return false;
    };
    timer.elapsed = Duration::ZERO;
    timer.state = TimerState::Idle;
    pool.clear_timer_events(id);
    true
  }

  pub fn state(&self, pool: &RuntimeObjectPool, id: TimerId) -> Option<TimerState> {
    if pool.timers.composition_owned.contains(&id) {
      return None;
    }
    Some(pool.timers.timers.get(&id)?.state)
  }

  pub fn elapsed(&self, pool: &RuntimeObjectPool, id: TimerId) -> Option<Duration> {
    if pool.timers.composition_owned.contains(&id) {
      return None;
    }
    Some(pool.timers.timers.get(&id)?.elapsed)
  }

  pub fn duration(&self, pool: &RuntimeObjectPool, id: TimerId) -> Option<Duration> {
    if pool.timers.composition_owned.contains(&id) {
      return None;
    }
    pool.timers.timers.get(&id)?.duration()
  }

  pub fn remaining(&self, pool: &RuntimeObjectPool, id: TimerId) -> Option<Duration> {
    if pool.timers.composition_owned.contains(&id) {
      return None;
    }
    pool.timers.timers.get(&id)?.remaining()
  }

  pub fn progress(&self, pool: &RuntimeObjectPool, id: TimerId) -> Option<f32> {
    if pool.timers.composition_owned.contains(&id) {
      return None;
    }
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
    if pool.timers.composition_owned.contains(&id) {
      return Vec::new();
    }
    pool.take_timer_events(id)
  }

  pub fn create_delay_timer(
    &self,
    pool: &mut RuntimeObjectPool,
    options: DelayTimerOptions,
  ) -> Option<DelayTimerId> {
    if options.delay == Duration::ZERO {
      return None;
    }

    let timer_id = self.create_internal_count_down(pool, options.delay);
    let id = DelayTimerId(pool.delay_timers.next_id);
    pool.delay_timers.next_id += 1;
    pool.delay_timers.timers.insert(
      id,
      DelayTimer {
        timer_id,
        report_event_queue: options.report_event_queue,
        callback: options.callback,
      },
    );
    Some(id)
  }

  pub fn remove_delay_timer(&self, pool: &mut RuntimeObjectPool, id: DelayTimerId) -> bool {
    let Some(delay) = pool.delay_timers.timers.remove(&id) else {
      return false;
    };
    self.remove_internal_timer(pool, delay.timer_id);
    pool.clear_delay_timer_events(id);
    true
  }

  pub fn start_delay_timer(&self, pool: &mut RuntimeObjectPool, id: DelayTimerId) -> bool {
    let Some(timer_id) = self.delay_timer_id(pool, id) else {
      return false;
    };
    self.start_internal(pool, timer_id)
  }

  pub fn pause_delay_timer(&self, pool: &mut RuntimeObjectPool, id: DelayTimerId) -> bool {
    let Some(timer_id) = self.delay_timer_id(pool, id) else {
      return false;
    };
    self.pause_internal(pool, timer_id)
  }

  pub fn resume_delay_timer(&self, pool: &mut RuntimeObjectPool, id: DelayTimerId) -> bool {
    let Some(timer_id) = self.delay_timer_id(pool, id) else {
      return false;
    };
    self.resume_internal(pool, timer_id)
  }

  pub fn stop_delay_timer(&self, pool: &mut RuntimeObjectPool, id: DelayTimerId) -> bool {
    let Some(timer_id) = self.delay_timer_id(pool, id) else {
      return false;
    };
    self.stop_internal(pool, timer_id)
  }

  pub fn reset_delay_timer(&self, pool: &mut RuntimeObjectPool, id: DelayTimerId) -> bool {
    let Some(timer_id) = self.delay_timer_id(pool, id) else {
      return false;
    };
    pool.clear_delay_timer_events(id);
    self.reset_internal(pool, timer_id)
  }

  pub fn delay_timer_state(
    &self,
    pool: &RuntimeObjectPool,
    id: DelayTimerId,
  ) -> Option<TimerState> {
    Some(
      pool
        .timers
        .timers
        .get(&self.delay_timer_id(pool, id)?)?
        .state,
    )
  }

  pub fn delay_timer_elapsed(
    &self,
    pool: &RuntimeObjectPool,
    id: DelayTimerId,
  ) -> Option<Duration> {
    Some(
      pool
        .timers
        .timers
        .get(&self.delay_timer_id(pool, id)?)?
        .elapsed,
    )
  }

  pub fn delay_timer_remaining(
    &self,
    pool: &RuntimeObjectPool,
    id: DelayTimerId,
  ) -> Option<Duration> {
    pool
      .timers
      .timers
      .get(&self.delay_timer_id(pool, id)?)?
      .remaining()
  }

  pub fn delay_timer_progress(&self, pool: &RuntimeObjectPool, id: DelayTimerId) -> Option<f32> {
    pool
      .timers
      .timers
      .get(&self.delay_timer_id(pool, id)?)?
      .progress()
  }

  pub fn take_delay_timer_events(
    &self,
    pool: &mut RuntimeObjectPool,
    id: DelayTimerId,
  ) -> Vec<DelayTimerEvent> {
    pool.take_delay_timer_events(id)
  }

  pub fn create_repeat_timer(
    &self,
    pool: &mut RuntimeObjectPool,
    options: RepeatTimerOptions,
  ) -> Option<RepeatTimerId> {
    if options.interval == Duration::ZERO || options.repeat_mode == RepeatMode::Count(0) {
      return None;
    }

    let timer_id = self.create_internal_count_down(pool, options.interval);
    let id = RepeatTimerId(pool.repeat_timers.next_id);
    pool.repeat_timers.next_id += 1;
    pool.repeat_timers.timers.insert(
      id,
      RepeatTimer {
        timer_id,
        repeat_mode: options.repeat_mode,
        executed_count: 0,
        report_event_queue: options.report_event_queue,
        callback: options.callback,
      },
    );
    Some(id)
  }

  pub fn remove_repeat_timer(&self, pool: &mut RuntimeObjectPool, id: RepeatTimerId) -> bool {
    let Some(repeat) = pool.repeat_timers.timers.remove(&id) else {
      return false;
    };
    self.remove_internal_timer(pool, repeat.timer_id);
    pool.clear_repeat_timer_events(id);
    true
  }

  pub fn start_repeat_timer(&self, pool: &mut RuntimeObjectPool, id: RepeatTimerId) -> bool {
    let Some(timer_id) = self.repeat_timer_id(pool, id) else {
      return false;
    };
    if matches!(
      pool.timers.timers.get(&timer_id).map(|timer| timer.state),
      Some(TimerState::Finished | TimerState::Stopped)
    ) {
      if let Some(repeat) = pool.repeat_timers.timers.get_mut(&id) {
        repeat.executed_count = 0;
      }
    }
    self.start_internal(pool, timer_id)
  }

  pub fn pause_repeat_timer(&self, pool: &mut RuntimeObjectPool, id: RepeatTimerId) -> bool {
    let Some(timer_id) = self.repeat_timer_id(pool, id) else {
      return false;
    };
    self.pause_internal(pool, timer_id)
  }

  pub fn resume_repeat_timer(&self, pool: &mut RuntimeObjectPool, id: RepeatTimerId) -> bool {
    let Some(timer_id) = self.repeat_timer_id(pool, id) else {
      return false;
    };
    self.resume_internal(pool, timer_id)
  }

  pub fn stop_repeat_timer(&self, pool: &mut RuntimeObjectPool, id: RepeatTimerId) -> bool {
    let Some(timer_id) = self.repeat_timer_id(pool, id) else {
      return false;
    };
    if let Some(repeat) = pool.repeat_timers.timers.get_mut(&id) {
      repeat.executed_count = 0;
    }
    self.stop_internal(pool, timer_id)
  }

  pub fn reset_repeat_timer(&self, pool: &mut RuntimeObjectPool, id: RepeatTimerId) -> bool {
    let Some(timer_id) = self.repeat_timer_id(pool, id) else {
      return false;
    };
    if let Some(repeat) = pool.repeat_timers.timers.get_mut(&id) {
      repeat.executed_count = 0;
    }
    pool.clear_repeat_timer_events(id);
    self.reset_internal(pool, timer_id)
  }

  pub fn repeat_timer_state(
    &self,
    pool: &RuntimeObjectPool,
    id: RepeatTimerId,
  ) -> Option<TimerState> {
    Some(
      pool
        .timers
        .timers
        .get(&self.repeat_timer_id(pool, id)?)?
        .state,
    )
  }

  pub fn repeat_timer_elapsed(
    &self,
    pool: &RuntimeObjectPool,
    id: RepeatTimerId,
  ) -> Option<Duration> {
    Some(
      pool
        .timers
        .timers
        .get(&self.repeat_timer_id(pool, id)?)?
        .elapsed,
    )
  }

  pub fn repeat_timer_remaining(
    &self,
    pool: &RuntimeObjectPool,
    id: RepeatTimerId,
  ) -> Option<Duration> {
    pool
      .timers
      .timers
      .get(&self.repeat_timer_id(pool, id)?)?
      .remaining()
  }

  pub fn repeat_timer_progress(&self, pool: &RuntimeObjectPool, id: RepeatTimerId) -> Option<f32> {
    pool
      .timers
      .timers
      .get(&self.repeat_timer_id(pool, id)?)?
      .progress()
  }

  pub fn repeat_timer_executed_count(
    &self,
    pool: &RuntimeObjectPool,
    id: RepeatTimerId,
  ) -> Option<u32> {
    Some(pool.repeat_timers.timers.get(&id)?.executed_count)
  }

  pub fn take_repeat_timer_events(
    &self,
    pool: &mut RuntimeObjectPool,
    id: RepeatTimerId,
  ) -> Vec<RepeatTimerEvent> {
    pool.take_repeat_timer_events(id)
  }

  pub fn take_time_callback_requests(
    &self,
    pool: &mut RuntimeObjectPool,
  ) -> Vec<TimeCallbackRequest> {
    pool.take_time_callback_requests()
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

  fn create_internal_count_down(
    &self,
    pool: &mut RuntimeObjectPool,
    duration: Duration,
  ) -> TimerId {
    let id = self.create(
      pool,
      TimerMode::CountDown { duration },
      TimerOptions::default(),
    );
    pool.timers.composition_owned.insert(id);
    id
  }

  fn remove_internal_timer(&self, pool: &mut RuntimeObjectPool, id: TimerId) {
    pool.timers.timers.remove(&id);
    pool.timers.composition_owned.remove(&id);
    pool.clear_timer_events(id);
  }

  fn delay_timer_id(&self, pool: &RuntimeObjectPool, id: DelayTimerId) -> Option<TimerId> {
    Some(pool.delay_timers.timers.get(&id)?.timer_id)
  }

  fn repeat_timer_id(&self, pool: &RuntimeObjectPool, id: RepeatTimerId) -> Option<TimerId> {
    Some(pool.repeat_timers.timers.get(&id)?.timer_id)
  }

  fn start_internal(&self, pool: &mut RuntimeObjectPool, id: TimerId) -> bool {
    let Some(timer) = pool.timers.timers.get_mut(&id) else {
      return false;
    };
    if matches!(timer.state, TimerState::Finished | TimerState::Stopped) {
      timer.elapsed = Duration::ZERO;
    }
    timer.state = TimerState::Running;
    true
  }

  fn pause_internal(&self, pool: &mut RuntimeObjectPool, id: TimerId) -> bool {
    let Some(timer) = pool.timers.timers.get_mut(&id) else {
      return false;
    };
    if timer.state != TimerState::Running {
      return false;
    }
    timer.state = TimerState::Paused;
    true
  }

  fn resume_internal(&self, pool: &mut RuntimeObjectPool, id: TimerId) -> bool {
    let Some(timer) = pool.timers.timers.get_mut(&id) else {
      return false;
    };
    if timer.state != TimerState::Paused {
      return false;
    }
    timer.state = TimerState::Running;
    true
  }

  fn stop_internal(&self, pool: &mut RuntimeObjectPool, id: TimerId) -> bool {
    let Some(timer) = pool.timers.timers.get_mut(&id) else {
      return false;
    };
    timer.elapsed = Duration::ZERO;
    timer.state = TimerState::Stopped;
    true
  }

  fn reset_internal(&self, pool: &mut RuntimeObjectPool, id: TimerId) -> bool {
    let Some(timer) = pool.timers.timers.get_mut(&id) else {
      return false;
    };
    timer.elapsed = Duration::ZERO;
    timer.state = TimerState::Idle;
    pool.clear_timer_events(id);
    true
  }

  fn advance_timer(timer: &mut Timer, dt: Duration) -> bool {
    if timer.state != TimerState::Running {
      return false;
    }

    timer.elapsed = timer.elapsed.saturating_add(dt);

    if let TimerMode::CountDown { duration } = timer.mode {
      if timer.elapsed >= duration {
        timer.elapsed = duration;
        timer.state = TimerState::Finished;
        return true;
      }
    }

    false
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

  #[test]
  fn delay_timer_finishes_once_and_reports_event_and_callback() {
    let (service, mut pool) = service_pool();
    let id = service
      .create_delay_timer(
        &mut pool,
        DelayTimerOptions {
          delay: Duration::from_secs(2),
          report_event_queue: true,
          callback: Some(TimeCallbackId(7)),
        },
      )
      .unwrap();

    assert!(service.start_delay_timer(&mut pool, id));
    service.update(&mut pool, Duration::from_secs(1));
    assert_eq!(
      service.delay_timer_state(&pool, id),
      Some(TimerState::Running)
    );
    assert_eq!(
      service.delay_timer_elapsed(&pool, id),
      Some(Duration::from_secs(1))
    );

    service.update(&mut pool, Duration::from_secs(2));
    service.update(&mut pool, Duration::from_secs(2));

    assert_eq!(
      service.delay_timer_state(&pool, id),
      Some(TimerState::Finished)
    );
    assert_eq!(
      service.take_delay_timer_events(&mut pool, id),
      vec![DelayTimerEvent::Finished { id }]
    );
    assert!(service.take_delay_timer_events(&mut pool, id).is_empty());
    assert_eq!(
      service.take_time_callback_requests(&mut pool),
      vec![TimeCallbackRequest::DelayFinished {
        id,
        callback: TimeCallbackId(7),
      }]
    );
  }

  #[test]
  fn delay_timer_can_pause_resume_stop_reset_and_remove() {
    let (service, mut pool) = service_pool();
    let id = service
      .create_delay_timer(
        &mut pool,
        DelayTimerOptions {
          delay: Duration::from_secs(5),
          report_event_queue: true,
          callback: Some(TimeCallbackId(1)),
        },
      )
      .unwrap();

    assert!(service.start_delay_timer(&mut pool, id));
    service.update(&mut pool, Duration::from_secs(2));
    assert!(service.pause_delay_timer(&mut pool, id));
    service.update(&mut pool, Duration::from_secs(2));
    assert_eq!(
      service.delay_timer_elapsed(&pool, id),
      Some(Duration::from_secs(2))
    );
    assert!(service.resume_delay_timer(&mut pool, id));
    service.update(&mut pool, Duration::from_secs(3));
    assert!(!service.take_delay_timer_events(&mut pool, id).is_empty());

    assert!(service.reset_delay_timer(&mut pool, id));
    assert_eq!(service.delay_timer_state(&pool, id), Some(TimerState::Idle));
    assert_eq!(service.delay_timer_elapsed(&pool, id), Some(Duration::ZERO));
    assert!(service.take_delay_timer_events(&mut pool, id).is_empty());
    assert!(service.take_time_callback_requests(&mut pool).is_empty());

    assert!(service.start_delay_timer(&mut pool, id));
    service.update(&mut pool, Duration::from_secs(1));
    assert!(service.stop_delay_timer(&mut pool, id));
    assert_eq!(
      service.delay_timer_state(&pool, id),
      Some(TimerState::Stopped)
    );
    assert_eq!(service.delay_timer_elapsed(&pool, id), Some(Duration::ZERO));
    assert!(service.remove_delay_timer(&mut pool, id));
    assert_eq!(service.delay_timer_state(&pool, id), None);
  }

  #[test]
  fn invalid_composition_timers_are_rejected() {
    let (service, mut pool) = service_pool();

    assert!(
      service
        .create_delay_timer(
          &mut pool,
          DelayTimerOptions {
            delay: Duration::ZERO,
            report_event_queue: false,
            callback: None,
          },
        )
        .is_none()
    );
    assert!(
      service
        .create_repeat_timer(
          &mut pool,
          RepeatTimerOptions {
            interval: Duration::ZERO,
            repeat_mode: RepeatMode::Forever,
            report_event_queue: false,
            callback: None,
          },
        )
        .is_none()
    );
    assert!(
      service
        .create_repeat_timer(
          &mut pool,
          RepeatTimerOptions {
            interval: Duration::from_secs(1),
            repeat_mode: RepeatMode::Count(0),
            report_event_queue: false,
            callback: None,
          },
        )
        .is_none()
    );
  }

  #[test]
  fn composition_inner_timer_is_hidden_and_not_double_advanced() {
    let (service, mut pool) = service_pool();
    let id = service
      .create_delay_timer(
        &mut pool,
        DelayTimerOptions {
          delay: Duration::from_secs(2),
          report_event_queue: false,
          callback: None,
        },
      )
      .unwrap();
    let inner = pool.delay_timers.timers.get(&id).unwrap().timer_id;

    assert_eq!(service.state(&pool, inner), None);
    assert!(!service.start(&mut pool, inner));
    assert!(service.start_delay_timer(&mut pool, id));
    service.update(&mut pool, Duration::from_secs(1));

    assert_eq!(
      service.delay_timer_state(&pool, id),
      Some(TimerState::Running)
    );
    assert_eq!(
      service.delay_timer_elapsed(&pool, id),
      Some(Duration::from_secs(1))
    );
  }

  #[test]
  fn repeat_timer_forever_ticks_once_per_update() {
    let (service, mut pool) = service_pool();
    let id = service
      .create_repeat_timer(
        &mut pool,
        RepeatTimerOptions {
          interval: Duration::from_secs(1),
          repeat_mode: RepeatMode::Forever,
          report_event_queue: true,
          callback: None,
        },
      )
      .unwrap();

    assert!(service.start_repeat_timer(&mut pool, id));
    service.update(&mut pool, Duration::from_secs(3));

    assert_eq!(
      service.repeat_timer_state(&pool, id),
      Some(TimerState::Running)
    );
    assert_eq!(service.repeat_timer_executed_count(&pool, id), Some(1));
    assert_eq!(
      service.take_repeat_timer_events(&mut pool, id),
      vec![RepeatTimerEvent::Tick {
        id,
        executed_count: 1,
      }]
    );

    service.update(&mut pool, Duration::from_secs(1));
    assert_eq!(service.repeat_timer_executed_count(&pool, id), Some(2));
  }

  #[test]
  fn repeat_timer_count_finishes_with_tick_then_finished() {
    let (service, mut pool) = service_pool();
    let id = service
      .create_repeat_timer(
        &mut pool,
        RepeatTimerOptions {
          interval: Duration::from_secs(1),
          repeat_mode: RepeatMode::Count(2),
          report_event_queue: true,
          callback: Some(TimeCallbackId(9)),
        },
      )
      .unwrap();

    assert!(service.start_repeat_timer(&mut pool, id));
    service.update(&mut pool, Duration::from_secs(1));
    service.update(&mut pool, Duration::from_secs(1));

    assert_eq!(
      service.repeat_timer_state(&pool, id),
      Some(TimerState::Finished)
    );
    assert_eq!(service.repeat_timer_executed_count(&pool, id), Some(2));
    assert_eq!(
      service.take_repeat_timer_events(&mut pool, id),
      vec![
        RepeatTimerEvent::Tick {
          id,
          executed_count: 1,
        },
        RepeatTimerEvent::Tick {
          id,
          executed_count: 2,
        },
        RepeatTimerEvent::Finished {
          id,
          executed_count: 2,
        },
      ]
    );
    assert_eq!(
      service.take_time_callback_requests(&mut pool),
      vec![
        TimeCallbackRequest::RepeatTick {
          id,
          callback: TimeCallbackId(9),
          executed_count: 1,
        },
        TimeCallbackRequest::RepeatTick {
          id,
          callback: TimeCallbackId(9),
          executed_count: 2,
        },
        TimeCallbackRequest::RepeatFinished {
          id,
          callback: TimeCallbackId(9),
          executed_count: 2,
        },
      ]
    );
  }

  #[test]
  fn repeat_timer_pause_stop_reset_and_remove_clear_state() {
    let (service, mut pool) = service_pool();
    let id = service
      .create_repeat_timer(
        &mut pool,
        RepeatTimerOptions {
          interval: Duration::from_secs(1),
          repeat_mode: RepeatMode::Forever,
          report_event_queue: true,
          callback: Some(TimeCallbackId(3)),
        },
      )
      .unwrap();

    assert!(service.start_repeat_timer(&mut pool, id));
    service.update(&mut pool, Duration::from_millis(500));
    assert!(service.pause_repeat_timer(&mut pool, id));
    service.update(&mut pool, Duration::from_secs(1));
    assert_eq!(
      service.repeat_timer_elapsed(&pool, id),
      Some(Duration::from_millis(500))
    );
    assert!(service.resume_repeat_timer(&mut pool, id));
    service.update(&mut pool, Duration::from_millis(500));
    assert_eq!(service.repeat_timer_executed_count(&pool, id), Some(1));

    assert!(service.reset_repeat_timer(&mut pool, id));
    assert_eq!(
      service.repeat_timer_state(&pool, id),
      Some(TimerState::Idle)
    );
    assert_eq!(service.repeat_timer_executed_count(&pool, id), Some(0));
    assert!(service.take_repeat_timer_events(&mut pool, id).is_empty());
    assert!(service.take_time_callback_requests(&mut pool).is_empty());

    assert!(service.start_repeat_timer(&mut pool, id));
    service.update(&mut pool, Duration::from_secs(1));
    assert!(service.stop_repeat_timer(&mut pool, id));
    assert_eq!(
      service.repeat_timer_state(&pool, id),
      Some(TimerState::Stopped)
    );
    assert_eq!(service.repeat_timer_executed_count(&pool, id), Some(0));
    assert!(service.remove_repeat_timer(&mut pool, id));
    assert_eq!(service.repeat_timer_state(&pool, id), None);
  }
}
