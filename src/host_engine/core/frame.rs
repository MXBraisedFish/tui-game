//! Frame scheduling and optional target-frame-rate pacing.

use std::thread;
use std::time::{Duration, Instant};

/// Optional frame-rate pacing and frame deadline tracking.
pub struct FrameScheduler {
  current_frame: u64,
  frame_start: Instant,
  target_frame_duration: Option<Duration>,
}

impl FrameScheduler {
  /// Create a frame scheduler initialized from `target_fps`.
  pub fn new(target_fps: u16) -> Self {
    let target_fps = target_fps.max(1);
    let target_frame_duration = Duration::from_secs_f64(1.0 / target_fps as f64);

    Self {
      current_frame: 0,
      frame_start: Instant::now(),
      target_frame_duration: Some(target_frame_duration),
    }
  }

  /// Prepare per-frame state and discard submissions or observations belonging to the previous
  /// frame.
  pub fn begin_frame(&mut self) -> u64 {
    self.current_frame = self.current_frame.saturating_add(1);
    self.frame_start = Instant::now();
    self.current_frame
  }

  /// Wait for the configured frame deadline when frame-rate limiting is enabled.
  pub fn wait_for_next_frame(&self) {
    let Some(target_frame_duration) = self.target_frame_duration else {
      return;
    };
    let elapsed = self.frame_start.elapsed();

    if elapsed >= target_frame_duration {
      return;
    }

    thread::sleep(target_frame_duration - elapsed);
  }

  /// Update the target fps used by this frame scheduler.
  pub fn set_target_fps(&mut self, target_fps: Option<u16>) {
    self.target_frame_duration =
      target_fps.map(|fps| Duration::from_secs_f64(1.0 / fps.max(1) as f64));
  }
}
