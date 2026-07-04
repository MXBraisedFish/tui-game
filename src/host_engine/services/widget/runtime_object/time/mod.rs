use std::collections::{HashMap, HashSet, VecDeque};
use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TimerId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DelayTimerId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RepeatTimerId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TimeCallbackId(pub u64);

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RepeatMode {
  Forever,
  Count(u32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DelayTimerOptions {
  pub delay: Duration,
  pub report_event_queue: bool,
  pub callback: Option<TimeCallbackId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RepeatTimerOptions {
  pub interval: Duration,
  pub repeat_mode: RepeatMode,
  pub report_event_queue: bool,
  pub callback: Option<TimeCallbackId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DelayTimerEvent {
  Finished { id: DelayTimerId },
}

impl DelayTimerEvent {
  pub(crate) fn id(&self) -> DelayTimerId {
    match self {
      Self::Finished { id } => *id,
    }
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RepeatTimerEvent {
  Tick {
    id: RepeatTimerId,
    executed_count: u32,
  },
  Finished {
    id: RepeatTimerId,
    executed_count: u32,
  },
}

impl RepeatTimerEvent {
  pub(crate) fn id(&self) -> RepeatTimerId {
    match self {
      Self::Tick { id, .. } | Self::Finished { id, .. } => *id,
    }
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimeCallbackRequest {
  DelayFinished {
    id: DelayTimerId,
    callback: TimeCallbackId,
  },
  RepeatTick {
    id: RepeatTimerId,
    callback: TimeCallbackId,
    executed_count: u32,
  },
  RepeatFinished {
    id: RepeatTimerId,
    callback: TimeCallbackId,
    executed_count: u32,
  },
}

impl TimeCallbackRequest {
  pub(crate) fn delay_id(&self) -> Option<DelayTimerId> {
    match self {
      Self::DelayFinished { id, .. } => Some(*id),
      _ => None,
    }
  }

  pub(crate) fn repeat_id(&self) -> Option<RepeatTimerId> {
    match self {
      Self::RepeatTick { id, .. } | Self::RepeatFinished { id, .. } => Some(*id),
      _ => None,
    }
  }
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
  pub(crate) composition_owned: HashSet<TimerId>,
}

impl TimerObjects {
  pub(crate) fn new() -> Self {
    Self {
      next_id: 1,
      timers: HashMap::new(),
      events: VecDeque::new(),
      composition_owned: HashSet::new(),
    }
  }
}

pub(crate) struct DelayTimerObjects {
  pub(crate) next_id: u64,
  pub(crate) timers: HashMap<DelayTimerId, DelayTimer>,
  pub(crate) events: VecDeque<DelayTimerEvent>,
}

impl DelayTimerObjects {
  pub(crate) fn new() -> Self {
    Self {
      next_id: 1,
      timers: HashMap::new(),
      events: VecDeque::new(),
    }
  }
}

pub(crate) struct RepeatTimerObjects {
  pub(crate) next_id: u64,
  pub(crate) timers: HashMap<RepeatTimerId, RepeatTimer>,
  pub(crate) events: VecDeque<RepeatTimerEvent>,
}

impl RepeatTimerObjects {
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

pub(crate) struct DelayTimer {
  pub(crate) timer_id: TimerId,
  pub(crate) report_event_queue: bool,
  pub(crate) callback: Option<TimeCallbackId>,
}

pub(crate) struct RepeatTimer {
  pub(crate) timer_id: TimerId,
  pub(crate) repeat_mode: RepeatMode,
  pub(crate) executed_count: u32,
  pub(crate) report_event_queue: bool,
  pub(crate) callback: Option<TimeCallbackId>,
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
