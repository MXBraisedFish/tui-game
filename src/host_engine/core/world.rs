// 引入结构体
use super::EngineClock;

// 运行时世界结构体
pub struct RuntimeWorld {
  pub clock: EngineClock,
}

// 运行时世界实现块
impl RuntimeWorld {
  pub fn new() -> Self {
    Self {
      clock: EngineClock::new()
    }
  }
}