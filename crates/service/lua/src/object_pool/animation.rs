//! Session-owned arbitrary-value animation parameters and native schedule identities.

use mlua::Value;
use std::rc::Rc;
use tg_service_animation::ParameterTrack;
use tg_service_time::ScheduledTimerId;

/// Script configuration backed by a countdown schedule in the session time pool.
#[derive(Clone)]
pub(crate) struct LuaAnimationBinding {
  /// The native schedule advancing this animation once per host frame.
  pub timer: ScheduledTimerId,
  /// The active duration in seconds.
  pub duration: f64,
  /// The initial wait in seconds.
  pub delay: f64,
  /// The wait between cycles in seconds.
  pub interval: f64,
  /// Whether repeated cycles are enabled.
  pub looping: bool,
  /// The optional total cycle limit.
  pub repetitions: Option<u32>,
  /// The sorted, normalized parameter tracks retained across callbacks.
  pub keyframes: Rc<ParameterTrack<Value>>,
  /// The validated easing selector, copied independently of the caller's tables.
  pub easing: Value,
  /// The validated fallback selector.
  pub callback_easing: Value,
  /// The configured playback direction.
  pub direction: String,
  /// The configured value retained between repeated cycles.
  pub fill_mode: String,
}
