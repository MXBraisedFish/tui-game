//! Monotonic frame timing used by the host lifecycle.

use std::time::{Duration, Instant};

/// Monotonic timing retained between application frames.
pub struct EngineClock {
  last_tick: Instant,
  dt: Duration,
}

impl EngineClock {
  /// Create an engine clock with its initial state.
  pub fn new() -> Self {
    let now = Instant::now();

    Self {
      last_tick: now,
      dt: Duration::ZERO,
    }
  }

  /// Measure elapsed monotonic time and advance accumulated frame timing.
  pub fn tick(&mut self) {
    let now = Instant::now();
    self.dt = now.duration_since(self.last_tick);
    self.last_tick = now;
  }

  /// Return the current delta time.
  pub fn delta_time(&self) -> Duration {
    self.dt
  }
}
