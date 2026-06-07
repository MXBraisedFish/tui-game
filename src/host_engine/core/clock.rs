use std::time::{Duration, Instant};

/// 世界运行时时钟。
///
/// 职责：
/// - 记录世界总运行时间
/// - 记录帧间隔时间
/// - 向 update(dt) 提供 dt
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

  /// 记录自上次 tick 以来经过的时间
  pub fn tick(&mut self) {
    let now = Instant::now();
    self.dt = now.duration_since(self.last_tick);
    self.last_tick = now;
  }

  /// 获取当前帧的时间增量
  pub fn delta_time(&self) -> Duration {
    self.dt
  }

  /// 获取自时钟创建以来经过的总时间
  pub fn elapsed(&self) -> Duration {
    self.epoch.elapsed()
  }
}
