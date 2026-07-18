use std::thread;
use std::time::{Duration, Instant};

/// 帧调度器，按目标帧率控制每帧的时长，提供帧间休眠
pub struct FrameScheduler {
  current_frame: u64,
  frame_start: Instant,
  target_frame_duration: Option<Duration>,
}

impl FrameScheduler {
  pub fn new(target_fps: u16) -> Self {
    let target_fps = target_fps.max(1);
    let target_frame_duration = Duration::from_secs_f64(1.0 / target_fps as f64);

    Self {
      current_frame: 0,
      frame_start: Instant::now(),
      target_frame_duration: Some(target_frame_duration),
    }
  }

  /// 开始新一帧，返回自调度开始以来的累计帧号
  pub fn begin_frame(&mut self) -> u64 {
    self.current_frame = self.current_frame.saturating_add(1);
    self.frame_start = Instant::now();
    self.current_frame
  }

  /// 等待直到当前帧的目标时长用完，控制帧率上限
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

  pub fn set_target_fps(&mut self, target_fps: Option<u16>) {
    self.target_frame_duration =
      target_fps.map(|fps| Duration::from_secs_f64(1.0 / fps.max(1) as f64));
  }

  pub fn current_frame(&self) -> u64 {
    self.current_frame
  }
}
