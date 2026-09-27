//! Time service: count-up/down, delay and repeat timers plus the async sleep job.

mod objects;
mod service;

pub use objects::{
  DelayTimerEvent, DelayTimerId, DelayTimerOptions, RepeatMode, RepeatTimerEvent, RepeatTimerId,
  RepeatTimerOptions, TimeCallbackId, TimeCallbackRequest, TimeObjects, TimerEvent, TimerId,
  TimerMode, TimerOptions, TimerState,
};
pub use service::{SleepTask, TimeAsyncEvent, TimeService};
