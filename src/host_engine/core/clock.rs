use std::time::{Duration, Instant};

/// 引擎时钟，追踪帧时间增量与运行时长
pub struct EngineClock {
  epoch: Instant,
  last_tick: Instant,
  dt: Duration,
}

impl EngineClock {
  pub fn new() -> Self {
    let now = Instant::now();

    Self {
      epoch: now,
      last_tick: now,
      dt: Duration::ZERO,
    }
  }

  /// 记录一次时钟滴答，更新上一帧到当前帧的时间增量
  pub fn tick(&mut self) {
    let now = Instant::now();
    self.dt = now.duration_since(self.last_tick);
    self.last_tick = now;
  }

  pub fn delta_time(&self) -> Duration {
    self.dt
  }

  pub fn elapsed(&self) -> Duration {
    self.epoch.elapsed()
  }
}
