// 引入时间的间隔和正计时库
use std::time::{Duration, Instant};

// 引擎时钟结构体
pub struct EngineClock {
  epoch: Instant,       // 引擎启动的绝对时间点
  frame_start: Instant, // 帧开始时间点
  dt: Duration,         // 上一帧的持续时间
}

// 引擎时钟实现块
impl EngineClock {
  pub fn new() -> Self {
    let now = Instant::now();

    Self {
      epoch: now,
      frame_start: now,
      dt: Duration::ZERO,
    }
  }

  // 更新帧时间
  pub fn tick(&mut self) {
    let now = Instant::now();

    // 计算当前和上一帧的时间差
    self.dt = now.duration_since(self.frame_start);
    // 更新帧开始时间
    self.frame_start = now;
  }

  // 计算引擎总运行时间
  pub fn elapsed_since_epoch(&self) -> Duration {
    // 等价于 Instant::now() - self.epoch
    self.epoch.elapsed()
  }

  // 获取帧间隔时间
  pub fn delta_time(&self) -> Duration {
    self.dt
  }

  // 基于dt算fps（波动）
  pub fn fps(&self) -> f64 {
    if self.dt.as_secs_f64() > 0.0 {
      1.0 / self.dt.as_secs_f64()
    } else {
      0.0
    }
  }

  // 获取平滑后的fps（避免波动）
  pub fn smooth_fps(&self, previous_fps: f64, alpha: f64) -> f64 {
    // 当前fps
    let current_fps = self.fps();
    // 根据平滑参数和上一个平滑fps以及当前fps计算结果
    previous_fps * (1.0 - alpha) + current_fps * alpha
  }
}
