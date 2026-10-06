//! Countdown schedules with initial delay, repeat gaps, and invalidatable completion events.

use std::collections::{BTreeMap, VecDeque};
use std::fmt;
use std::time::Duration;

use crate::{TimeObjects, TimeService, TimerState};

/// The identity of a countdown schedule within its owning pool.
///
/// # Fields
///
/// * `0` - The pool-local monotonic identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ScheduledTimerId(
  /// The pool-local monotonic identifier.
  pub u64,
);

/// The timing rules of a countdown schedule.
///
/// # Fields
///
/// * `duration` - The active countdown duration.
/// * `delay` - The initial wait.
/// * `gap` - The wait between executions.
/// * `repetitions` - The optional total execution limit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScheduledTimerOptions {
  /// The active countdown duration of each execution.
  pub duration: Duration,
  /// The wait before the first execution starts.
  pub delay: Duration,
  /// The wait between executions, including any recurring delay.
  pub gap: Duration,
  /// The total number of executions, or `None` for unlimited repetition.
  pub repetitions: Option<u32>,
}

/// A snapshot of one schedule's timing and lifecycle state.
///
/// # Fields
///
/// * `options` - The timing rules.
/// * `state` - The lifecycle state.
/// * `elapsed` - The current cycle's accumulated time.
/// * `remaining` - The time until the next execution completes.
/// * `executed_count` - The completed executions.
/// * `revision` - The revision invalidating stale events.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScheduledTimerInfo {
  /// The configured timing rules.
  pub options: ScheduledTimerOptions,
  /// The current lifecycle state.
  pub state: TimerState,
  /// The elapsed time in the current wait and countdown, including deferred overrun.
  pub elapsed: Duration,
  /// The time until the next execution completes.
  pub remaining: Duration,
  /// The number of completed executions.
  pub executed_count: u32,
  /// The revision used to reject events invalidated by lifecycle changes.
  pub revision: u64,
}

/// One completed execution of a schedule.
///
/// # Fields
///
/// * `id` - The owning schedule.
/// * `revision` - The producing revision.
/// * `executed_count` - The completed executions.
/// * `finished` - Whether the schedule reached its execution limit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScheduledTimerEvent {
  /// The schedule that completed an execution.
  pub id: ScheduledTimerId,
  /// The revision that produced this event.
  pub revision: u64,
  /// The number of completed executions.
  pub executed_count: u32,
  /// Whether this execution ended the schedule.
  pub finished: bool,
}

/// Failures reported while configuring a countdown schedule.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScheduleError {
  /// A finite execution count was zero.
  ZeroRepetitions,
  /// A wait plus its countdown exceeded the duration range.
  DurationOverflow,
  /// The pool exhausted its monotonically increasing identifiers.
  IdExhausted,
}

impl fmt::Display for ScheduleError {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter.write_str(match self {
      Self::ZeroRepetitions => "repeat count must be positive",
      Self::DurationOverflow => "delay or gap plus duration exceeds the supported range",
      Self::IdExhausted => "timer identifier space exhausted",
    })
  }
}

impl std::error::Error for ScheduleError {}

struct Schedule {
  info: ScheduledTimerInfo,
}

#[derive(Default)]
pub(crate) struct ScheduledTimers {
  next_id: u64,
  timers: BTreeMap<ScheduledTimerId, Schedule>,
  events: VecDeque<ScheduledTimerEvent>,
}

impl ScheduledTimerOptions {
  fn validate(self) -> Result<(), ScheduleError> {
    if self.repetitions == Some(0) {
      return Err(ScheduleError::ZeroRepetitions);
    }
    self
      .delay
      .checked_add(self.duration)
      .ok_or(ScheduleError::DurationOverflow)?;
    self
      .gap
      .checked_add(self.duration)
      .ok_or(ScheduleError::DurationOverflow)?;
    Ok(())
  }
}

impl Schedule {
  fn new(options: ScheduledTimerOptions, revision: u64) -> Self {
    Self {
      info: ScheduledTimerInfo {
        options,
        revision,
        state: TimerState::Idle,
        elapsed: Duration::ZERO,
        remaining: options.delay + options.duration,
        executed_count: 0,
      },
    }
  }

  fn threshold(&self) -> Duration {
    self.info.options.duration
      + if self.info.executed_count == 0 {
        self.info.options.delay
      } else {
        self.info.options.gap
      }
  }
}

impl TimeService {
  /// Create an idle countdown schedule and return its pool-local identity.
  ///
  /// # Errors
  ///
  /// Return [`ScheduleError`] for zero finite repetitions, overflowing timing sums, or exhausted IDs.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use std::time::Duration;
  /// use tg_service_time::{ScheduledTimerOptions, TimeObjects, TimeService};
  /// let service = TimeService::new();
  /// let mut pool = TimeObjects::new();
  /// let id = service.create_scheduled_timer(&mut pool, ScheduledTimerOptions {
  ///   duration: Duration::from_secs(1), delay: Duration::ZERO,
  ///   gap: Duration::ZERO, repetitions: Some(1),
  /// }).unwrap();
  /// assert!(service.start_scheduled_timer(&mut pool, id));
  /// service.update(&mut pool, Duration::from_secs(1));
  /// assert!(service.take_scheduled_timer_events(&mut pool)[0].finished);
  /// ```
  pub fn create_scheduled_timer(
    &self,
    pool: &mut TimeObjects,
    options: ScheduledTimerOptions,
  ) -> Result<ScheduledTimerId, ScheduleError> {
    options.validate()?;
    pool.scheduled.next_id = pool
      .scheduled
      .next_id
      .checked_add(1)
      .ok_or(ScheduleError::IdExhausted)?;
    let id = ScheduledTimerId(pool.scheduled.next_id);
    pool.scheduled.timers.insert(id, Schedule::new(options, 1));
    Ok(id)
  }

  /// Return a snapshot of an existing countdown schedule.
  pub fn scheduled_timer_info(
    &self,
    pool: &TimeObjects,
    id: ScheduledTimerId,
  ) -> Option<ScheduledTimerInfo> {
    pool.scheduled.timers.get(&id).map(|timer| timer.info)
  }

  /// Replace timing rules and reset an existing schedule to its idle state.
  ///
  /// # Arguments
  ///
  /// * `pool` - The pool owning the schedule.
  /// * `id` - The schedule to reconfigure.
  /// * `options` - The replacement timing rules.
  ///
  /// # Errors
  ///
  /// Return [`ScheduleError`] for zero finite repetitions or overflowing timing sums, without
  /// changing the original schedule. Return `Ok(false)` for an unknown identity.
  pub fn set_scheduled_timer(
    &self,
    pool: &mut TimeObjects,
    id: ScheduledTimerId,
    options: ScheduledTimerOptions,
  ) -> Result<bool, ScheduleError> {
    options.validate()?;
    let Some(timer) = pool.scheduled.timers.get_mut(&id) else {
      return Ok(false);
    };
    *timer = Schedule::new(options, timer.info.revision.saturating_add(1));
    pool.scheduled.events.retain(|event| event.id != id);
    Ok(true)
  }

  /// Start an idle schedule, resume a paused one, or reset and start a finished one.
  ///
  /// Return `false` for a missing or already running schedule.
  pub fn start_scheduled_timer(&self, pool: &mut TimeObjects, id: ScheduledTimerId) -> bool {
    let Some(timer) = pool.scheduled.timers.get_mut(&id) else {
      return false;
    };
    if timer.info.state == TimerState::Running {
      return false;
    }
    if matches!(timer.info.state, TimerState::Finished | TimerState::Stopped) {
      *timer = Schedule::new(timer.info.options, timer.info.revision.saturating_add(1));
      pool.scheduled.events.retain(|event| event.id != id);
    }
    timer.info.state = TimerState::Running;
    true
  }

  /// Pause a running schedule without discarding its elapsed time.
  ///
  /// Invalidate undelivered events. Return `false` for missing or non-running schedules.
  pub fn pause_scheduled_timer(&self, pool: &mut TimeObjects, id: ScheduledTimerId) -> bool {
    let Some(timer) = pool.scheduled.timers.get_mut(&id) else {
      return false;
    };
    if timer.info.state != TimerState::Running {
      return false;
    }
    timer.info.state = TimerState::Paused;
    timer.info.revision = timer.info.revision.saturating_add(1);
    pool.scheduled.events.retain(|event| event.id != id);
    true
  }

  /// Reset a schedule to idle and invalidate its undelivered events.
  pub fn reset_scheduled_timer(&self, pool: &mut TimeObjects, id: ScheduledTimerId) -> bool {
    let Some(timer) = pool.scheduled.timers.get_mut(&id) else {
      return false;
    };
    *timer = Schedule::new(timer.info.options, timer.info.revision.saturating_add(1));
    pool.scheduled.events.retain(|event| event.id != id);
    true
  }

  /// Remove a schedule and all events still retained in its pool.
  pub fn remove_scheduled_timer(&self, pool: &mut TimeObjects, id: ScheduledTimerId) -> bool {
    pool.scheduled.events.retain(|event| event.id != id);
    pool.scheduled.timers.remove(&id).is_some()
  }

  /// Drain completed executions in schedule-creation order for each update.
  pub fn take_scheduled_timer_events(&self, pool: &mut TimeObjects) -> Vec<ScheduledTimerEvent> {
    pool.scheduled.events.drain(..).collect()
  }

  pub(crate) fn update_scheduled_timers(&self, pool: &mut TimeObjects, delta: Duration) {
    for (&id, timer) in &mut pool.scheduled.timers {
      if timer.info.state != TimerState::Running {
        continue;
      }
      timer.info.elapsed = timer.info.elapsed.saturating_add(delta);
      let threshold = timer.threshold();
      if timer.info.elapsed < threshold {
        timer.info.remaining = threshold - timer.info.elapsed;
        continue;
      }
      // Emit at most one execution per update, retaining overrun for subsequent updates.
      timer.info.elapsed -= threshold;
      timer.info.executed_count = timer.info.executed_count.saturating_add(1);
      let finished = timer
        .info
        .options
        .repetitions
        .is_some_and(|count| timer.info.executed_count >= count);
      timer.info.remaining = if finished {
        Duration::ZERO
      } else {
        timer.threshold().saturating_sub(timer.info.elapsed)
      };
      if finished {
        timer.info.state = TimerState::Finished;
        timer.info.elapsed = threshold;
      }
      pool.scheduled.events.push_back(ScheduledTimerEvent {
        id,
        revision: timer.info.revision,
        executed_count: timer.info.executed_count,
        finished,
      });
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn options() -> ScheduledTimerOptions {
    ScheduledTimerOptions {
      duration: Duration::from_secs(1),
      delay: Duration::from_secs(2),
      gap: Duration::from_secs(4),
      repetitions: Some(2),
    }
  }

  #[test]
  fn initial_delay_repeat_gap_pause_and_finish_are_distinct() {
    let service = TimeService::new();
    let mut pool = TimeObjects::new();
    let id = service
      .create_scheduled_timer(&mut pool, options())
      .unwrap();
    assert!(service.start_scheduled_timer(&mut pool, id));
    service.update(&mut pool, Duration::from_secs(2));
    assert!(service.take_scheduled_timer_events(&mut pool).is_empty());
    assert!(service.pause_scheduled_timer(&mut pool, id));
    service.update(&mut pool, Duration::from_secs(100));
    assert!(service.start_scheduled_timer(&mut pool, id));
    service.update(&mut pool, Duration::from_secs(1));
    assert!(!service.take_scheduled_timer_events(&mut pool)[0].finished);
    service.update(&mut pool, Duration::from_secs(4));
    assert!(service.take_scheduled_timer_events(&mut pool).is_empty());
    service.update(&mut pool, Duration::from_secs(1));
    let event = service.take_scheduled_timer_events(&mut pool)[0];
    assert!(event.finished);
    assert_eq!(event.executed_count, 2);
    assert_eq!(
      service.scheduled_timer_info(&pool, id).unwrap().state,
      TimerState::Finished
    );
  }

  #[test]
  fn invalid_changes_are_atomic_and_reset_invalidates_events() {
    let service = TimeService::new();
    let mut pool = TimeObjects::new();
    let id = service
      .create_scheduled_timer(&mut pool, options())
      .unwrap();
    service.start_scheduled_timer(&mut pool, id);
    service.update(&mut pool, Duration::from_secs(3));
    let before = service.scheduled_timer_info(&pool, id).unwrap();
    assert_eq!(
      service.set_scheduled_timer(
        &mut pool,
        id,
        ScheduledTimerOptions {
          repetitions: Some(0),
          ..options()
        }
      ),
      Err(ScheduleError::ZeroRepetitions)
    );
    assert_eq!(service.scheduled_timer_info(&pool, id), Some(before));
    assert!(service.reset_scheduled_timer(&mut pool, id));
    assert!(service.take_scheduled_timer_events(&mut pool).is_empty());
    assert!(service.scheduled_timer_info(&pool, id).unwrap().revision > before.revision);
    assert!(service.remove_scheduled_timer(&mut pool, id));
    assert_eq!(service.scheduled_timer_info(&pool, id), None);
  }

  #[test]
  fn zero_duration_and_long_frames_never_run_unbounded_loops() {
    let service = TimeService::new();
    let mut pool = TimeObjects::new();
    let zero = service
      .create_scheduled_timer(
        &mut pool,
        ScheduledTimerOptions {
          duration: Duration::ZERO,
          delay: Duration::ZERO,
          gap: Duration::ZERO,
          repetitions: None,
        },
      )
      .unwrap();
    service.start_scheduled_timer(&mut pool, zero);
    service.update(&mut pool, Duration::from_secs(100));
    assert_eq!(service.take_scheduled_timer_events(&mut pool).len(), 1);
    service.update(&mut pool, Duration::ZERO);
    assert_eq!(service.take_scheduled_timer_events(&mut pool).len(), 1);
    let finite = service
      .create_scheduled_timer(
        &mut pool,
        ScheduledTimerOptions {
          delay: Duration::ZERO,
          gap: Duration::ZERO,
          ..options()
        },
      )
      .unwrap();
    service.start_scheduled_timer(&mut pool, finite);
    service.update(&mut pool, Duration::from_secs(2));
    service.take_scheduled_timer_events(&mut pool);
    service.update(&mut pool, Duration::ZERO);
    assert!(
      service
        .take_scheduled_timer_events(&mut pool)
        .iter()
        .any(|event| event.id == finite && event.finished)
    );
  }
}
