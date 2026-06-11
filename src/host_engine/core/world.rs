// 引入结构体
use super::{EngineClock, HostMachineState};

// 运行时世界结构体
pub struct RuntimeWorld {
  pub clock: EngineClock,
  pub state: HostMachineState,
}

// 运行时世界实现块
impl RuntimeWorld {
  pub fn new() -> Self {
    Self {
      clock: EngineClock::new(),
      state: HostMachineState::new(),
    }
  }

  /// 检查世界是否已停止（循环退出条件）
  pub fn is_stopped(&self) -> bool {
    self.state.is_stopped()
  }
}
