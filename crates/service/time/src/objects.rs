//! Objects support for the time service.

use std::collections::{HashMap, HashSet, VecDeque};
use std::time::Duration;

/// The identity of timer within its owning pool or session.
///
/// # Fields
///
/// * `0` - The wrapped u64 value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TimerId(
  /// The wrapped u64 value.
  pub u64,
);

/// The identity of delay timer within its owning pool or session.
///
/// # Fields
///
/// * `0` - The wrapped u64 value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DelayTimerId(
  /// The wrapped u64 value.
  pub u64,
);

/// The identity of repeat timer within its owning pool or session.
///
/// # Fields
///
/// * `0` - The wrapped u64 value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RepeatTimerId(
  /// The wrapped u64 value.
  pub u64,
);

/// The identity of time callback within its owning pool or session.
///
/// # Fields
///
/// * `0` - The wrapped u64 value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TimeCallbackId(
  /// The wrapped u64 value.
  pub u64,
);

/// The count-up or count-down behavior of an owned timer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimerMode {
  /// The count up setting for timer mode.
  CountUp,
  /// The count down setting for timer mode.
  CountDown {
    /// The duration represented as a duration.
    duration: Duration,
  },
}

/// The stopped, running, paused, or finished timer lifecycle state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimerState {
  /// The operation is idle.
  Idle,
  /// The operation is running.
  Running,
  /// The operation is paused.
  Paused,
  /// The operation is finished.
  Finished,
  /// The operation is stopped.
  Stopped,
}

/// Configuration values controlling timer behavior.
///
/// # Fields
///
/// * `emit_finished` - Whether finishing this timer emits a completion event.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TimerOptions {
  /// Whether finishing this timer emits a completion event.
  pub emit_finished: bool,
}

/// A timer event payload queued for its owning consumer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimerEvent {
  /// The operation is finished.
  Finished {
    /// The identifier of the owned object.
    id: TimerId,
  },
}

/// The finite or indefinite policy used by repeating timers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RepeatMode {
  /// The forever setting for repeat mode.
  Forever,
  /// The count setting for repeat mode.
  Count(u32),
}

/// Configuration values controlling delay timer behavior.
///
/// # Fields
///
/// * `delay` - The delay represented as a duration.
/// * `report_event_queue` - Whether timer events are retained for polling.
/// * `callback` - The callback.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DelayTimerOptions {
  /// The delay represented as a duration.
  pub delay: Duration,
  /// Whether timer events are retained for polling.
  pub report_event_queue: bool,
  /// The callback.
  pub callback: Option<TimeCallbackId>,
}

/// Configuration values controlling repeat timer behavior.
///
/// # Fields
///
/// * `interval` - The interval represented as a duration.
/// * `repeat_mode` - The repeat mode.
/// * `report_event_queue` - Whether timer events are retained for polling.
/// * `callback` - The callback.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RepeatTimerOptions {
  /// The interval represented as a duration.
  pub interval: Duration,
  /// The repeat mode.
  pub repeat_mode: RepeatMode,
  /// Whether timer events are retained for polling.
  pub report_event_queue: bool,
  /// The callback.
  pub callback: Option<TimeCallbackId>,
}

/// A delay timer event payload queued for its owning consumer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DelayTimerEvent {
  /// The operation is finished.
  Finished {
    /// The identifier of the owned object.
    id: DelayTimerId,
  },
}

impl DelayTimerEvent {
  /// Return the current id.
  pub(crate) fn id(&self) -> DelayTimerId {
    match self {
      Self::Finished { id } => *id,
    }
  }
}

/// A repeat timer event payload queued for its owning consumer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RepeatTimerEvent {
  /// A tick notification delivered to the owning consumer.
  Tick {
    /// The identifier of the owned object.
    id: RepeatTimerId,
    /// The executed count.
    executed_count: u32,
  },
  /// The operation is finished.
  Finished {
    /// The identifier of the owned object.
    id: RepeatTimerId,
    /// The executed count.
    executed_count: u32,
  },
}

impl RepeatTimerEvent {
  /// Return the current id.
  pub(crate) fn id(&self) -> RepeatTimerId {
    match self {
      Self::Tick { id, .. } | Self::Finished { id, .. } => *id,
    }
  }
}

/// The time callback request representation used by this module.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimeCallbackRequest {
  /// The delay finished setting for time callback request.
  DelayFinished {
    /// The identifier of the owned object.
    id: DelayTimerId,
    /// The callback.
    callback: TimeCallbackId,
  },
  /// The repeat tick setting for time callback request.
  RepeatTick {
    /// The identifier of the owned object.
    id: RepeatTimerId,
    /// The callback.
    callback: TimeCallbackId,
    /// The executed count.
    executed_count: u32,
  },
  /// The repeat finished setting for time callback request.
  RepeatFinished {
    /// The identifier of the owned object.
    id: RepeatTimerId,
    /// The callback.
    callback: TimeCallbackId,
    /// The executed count.
    executed_count: u32,
  },
}

impl TimeCallbackRequest {
  /// Return the current delay id.
  pub(crate) fn delay_id(&self) -> Option<DelayTimerId> {
    match self {
      Self::DelayFinished { id, .. } => Some(*id),
      _ => None,
    }
  }

  /// Return the current repeat id.
  pub(crate) fn repeat_id(&self) -> Option<RepeatTimerId> {
    match self {
      Self::RepeatTick { id, .. } | Self::RepeatFinished { id, .. } => Some(*id),
      _ => None,
    }
  }
}

impl TimerEvent {
  /// Return the current id.
  pub(crate) fn id(&self) -> TimerId {
    match self {
      Self::Finished { id } => *id,
    }
  }
}

/// The collection of owned timer instances and their queued events.
///
/// # Fields
///
/// * `next_id` - The identifier of the next.
/// * `timers` - The timers indexed by their declared keys.
/// * `events` - Pending events retained in delivery order.
/// * `composition_owned` - The composition owned.
pub(crate) struct TimerObjects {
  /// The identifier of the next.
  pub(crate) next_id: u64,
  /// The timers indexed by their declared keys.
  pub(crate) timers: HashMap<TimerId, Timer>,
  /// Pending events retained in delivery order.
  pub(crate) events: VecDeque<TimerEvent>,
  /// The composition owned.
  pub(crate) composition_owned: HashSet<TimerId>,
}

impl TimerObjects {
  /// Create a timer objects with its initial state.
  pub(crate) fn new() -> Self {
    Self {
      next_id: 1,
      timers: HashMap::new(),
      events: VecDeque::new(),
      composition_owned: HashSet::new(),
    }
  }
}

/// The collection of owned delay timer instances and their queued events.
///
/// # Fields
///
/// * `next_id` - The identifier of the next.
/// * `timers` - The timers indexed by their declared keys.
/// * `events` - Pending events retained in delivery order.
pub(crate) struct DelayTimerObjects {
  /// The identifier of the next.
  pub(crate) next_id: u64,
  /// The timers indexed by their declared keys.
  pub(crate) timers: HashMap<DelayTimerId, DelayTimer>,
  /// Pending events retained in delivery order.
  pub(crate) events: VecDeque<DelayTimerEvent>,
}

impl DelayTimerObjects {
  /// Create a delay timer objects with its initial state.
  pub(crate) fn new() -> Self {
    Self {
      next_id: 1,
      timers: HashMap::new(),
      events: VecDeque::new(),
    }
  }
}

/// The collection of owned repeat timer instances and their queued events.
///
/// # Fields
///
/// * `next_id` - The identifier of the next.
/// * `timers` - The timers indexed by their declared keys.
/// * `events` - Pending events retained in delivery order.
pub(crate) struct RepeatTimerObjects {
  /// The identifier of the next.
  pub(crate) next_id: u64,
  /// The timers indexed by their declared keys.
  pub(crate) timers: HashMap<RepeatTimerId, RepeatTimer>,
  /// Pending events retained in delivery order.
  pub(crate) events: VecDeque<RepeatTimerEvent>,
}

impl RepeatTimerObjects {
  /// Create a repeat timer objects with its initial state.
  pub(crate) fn new() -> Self {
    Self {
      next_id: 1,
      timers: HashMap::new(),
      events: VecDeque::new(),
    }
  }
}

/// The timer representation used by this module.
///
/// # Fields
///
/// * `mode` - The timer mode carried by this timer.
/// * `options` - The timer options carried by this timer.
/// * `state` - The timer state carried by this timer.
/// * `elapsed` - The elapsed represented as a duration.
pub(crate) struct Timer {
  /// The timer mode carried by this timer.
  pub(crate) mode: TimerMode,
  /// The timer options carried by this timer.
  pub(crate) options: TimerOptions,
  /// The timer state carried by this timer.
  pub(crate) state: TimerState,
  /// The elapsed represented as a duration.
  pub(crate) elapsed: Duration,
}

/// The delay timer representation used by this module.
///
/// # Fields
///
/// * `timer_id` - The identifier of the timer.
/// * `report_event_queue` - Whether timer events are retained for polling.
/// * `callback` - The callback.
pub(crate) struct DelayTimer {
  /// The identifier of the timer.
  pub(crate) timer_id: TimerId,
  /// Whether timer events are retained for polling.
  pub(crate) report_event_queue: bool,
  /// The callback.
  pub(crate) callback: Option<TimeCallbackId>,
}

/// The repeat timer representation used by this module.
///
/// # Fields
///
/// * `timer_id` - The identifier of the timer.
/// * `repeat_mode` - The repeat mode.
/// * `executed_count` - The executed count.
/// * `report_event_queue` - Whether timer events are retained for polling.
/// * `callback` - The callback.
pub(crate) struct RepeatTimer {
  /// The identifier of the timer.
  pub(crate) timer_id: TimerId,
  /// The repeat mode.
  pub(crate) repeat_mode: RepeatMode,
  /// The executed count.
  pub(crate) executed_count: u32,
  /// Whether timer events are retained for polling.
  pub(crate) report_event_queue: bool,
  /// The callback.
  pub(crate) callback: Option<TimeCallbackId>,
}

impl Timer {
  /// Create a timer initialized from `mode`, `options`.
  pub(crate) fn new(mode: TimerMode, options: TimerOptions) -> Self {
    Self {
      mode,
      options,
      state: TimerState::Idle,
      elapsed: Duration::ZERO,
    }
  }

  /// Return the duration for the addressed object when it is available.
  pub(crate) fn duration(&self) -> Option<Duration> {
    match self.mode {
      TimerMode::CountUp => None,
      TimerMode::CountDown { duration } => Some(duration),
    }
  }

  /// Return the remaining for the addressed object when it is available.
  pub(crate) fn remaining(&self) -> Option<Duration> {
    Some(self.duration()?.saturating_sub(self.elapsed))
  }

  /// Return the progress for the addressed object when it is available.
  pub(crate) fn progress(&self) -> Option<f32> {
    let duration = self.duration()?;
    Some((self.elapsed.as_secs_f32() / duration.as_secs_f32()).clamp(0.0, 1.0))
  }
}

/// The collection of owned time instances and their queued events.
///
/// # Fields
///
/// * `timers` - The timers.
/// * `delay_timers` - The delay timers.
/// * `repeat_timers` - The repeat timers.
/// * `time_callback_requests` - The ordered time callback requests retained by this owner.
/// * `scheduled` - The countdown schedules and their completion events.
pub struct TimeObjects {
  /// The countdown schedules and their pending completion events.
  pub(crate) scheduled: crate::schedule::ScheduledTimers,
  /// The timers.
  pub(crate) timers: TimerObjects,
  /// The delay timers.
  pub(crate) delay_timers: DelayTimerObjects,
  /// The repeat timers.
  pub(crate) repeat_timers: RepeatTimerObjects,
  /// The ordered time callback requests retained by this owner.
  pub(crate) time_callback_requests: Vec<TimeCallbackRequest>,
}

impl TimeObjects {
  /// Create a time objects with its initial state.
  pub fn new() -> Self {
    Self {
      timers: TimerObjects::new(),
      delay_timers: DelayTimerObjects::new(),
      repeat_timers: RepeatTimerObjects::new(),
      time_callback_requests: Vec::new(),
      scheduled: crate::schedule::ScheduledTimers::default(),
    }
  }

  /// Clear the timer events retained by this time objects.
  pub(crate) fn clear_timer_events(&mut self, id: TimerId) {
    self.timers.events.retain(|event| event.id() != id);
  }

  /// Drain and return the queued timer events.
  pub(crate) fn take_timer_events(&mut self, id: TimerId) -> Vec<TimerEvent> {
    let mut events = Vec::new();
    self.timers.events.retain(|event| {
      if event.id() == id {
        events.push(*event);
        false
      } else {
        true
      }
    });
    events
  }

  /// Clear the delay timer events retained by this time objects.
  pub(crate) fn clear_delay_timer_events(&mut self, id: DelayTimerId) {
    self.delay_timers.events.retain(|event| event.id() != id);
    self
      .time_callback_requests
      .retain(|request| request.delay_id() != Some(id));
  }

  /// Drain and return the queued delay timer events.
  pub(crate) fn take_delay_timer_events(&mut self, id: DelayTimerId) -> Vec<DelayTimerEvent> {
    let mut events = Vec::new();
    self.delay_timers.events.retain(|event| {
      if event.id() == id {
        events.push(*event);
        false
      } else {
        true
      }
    });
    events
  }

  /// Clear the repeat timer events retained by this time objects.
  pub(crate) fn clear_repeat_timer_events(&mut self, id: RepeatTimerId) {
    self.repeat_timers.events.retain(|event| event.id() != id);
    self
      .time_callback_requests
      .retain(|request| request.repeat_id() != Some(id));
  }

  /// Drain and return the queued repeat timer events.
  pub(crate) fn take_repeat_timer_events(&mut self, id: RepeatTimerId) -> Vec<RepeatTimerEvent> {
    let mut events = Vec::new();
    self.repeat_timers.events.retain(|event| {
      if event.id() == id {
        events.push(*event);
        false
      } else {
        true
      }
    });
    events
  }

  /// Drain deferred timer callbacks produced during time-service updates.
  pub(crate) fn take_time_callback_requests(&mut self) -> Vec<TimeCallbackRequest> {
    self.time_callback_requests.drain(..).collect()
  }
}

impl Default for TimeObjects {
  fn default() -> Self {
    Self::new()
  }
}
