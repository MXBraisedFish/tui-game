use std::collections::{HashMap, VecDeque};
use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TimerId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimerMode {
  CountUp,
  CountDown { duration: Duration },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimerState {
  Idle,
  Running,
  Paused,
  Finished,
  Stopped,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TimerOptions {
  pub emit_finished: bool,
}

impl Default for TimerOptions {
  fn default() -> Self {
    Self {
      emit_finished: false,
    }
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimerEvent {
  Finished { id: TimerId },
}

impl TimerEvent {
  pub(crate) fn id(&self) -> TimerId {
    match self {
      Self::Finished { id } => *id,
    }
  }
}

pub(crate) struct TimerObjects {
  pub(crate) next_id: u64,
  pub(crate) timers: HashMap<TimerId, Timer>,
  pub(crate) events: VecDeque<TimerEvent>,
}

impl TimerObjects {
  pub(crate) fn new() -> Self {
    Self {
      next_id: 1,
      timers: HashMap::new(),
      events: VecDeque::new(),
    }
  }
}

pub(crate) struct Timer {
  pub(crate) mode: TimerMode,
  pub(crate) options: TimerOptions,
  pub(crate) state: TimerState,
  pub(crate) elapsed: Duration,
}

impl Timer {
  pub(crate) fn new(mode: TimerMode, options: TimerOptions) -> Self {
    Self {
      mode,
      options,
      state: TimerState::Idle,
      elapsed: Duration::ZERO,
    }
  }

  pub(crate) fn duration(&self) -> Option<Duration> {
    match self.mode {
      TimerMode::CountUp => None,
      TimerMode::CountDown { duration } => Some(duration),
    }
  }

  pub(crate) fn remaining(&self) -> Option<Duration> {
    Some(self.duration()?.saturating_sub(self.elapsed))
  }

  pub(crate) fn progress(&self) -> Option<f32> {
    let duration = self.duration()?;
    Some((self.elapsed.as_secs_f32() / duration.as_secs_f32()).clamp(0.0, 1.0))
  }
}
