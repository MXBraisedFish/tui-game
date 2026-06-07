use super::RuntimeState;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HostMachineState {
  Boot,
  Init,
  Runtime(RuntimeState),
  Shutdown,
  Stopped,
}

// 顶层状态机切换逻辑
impl HostMachineState {
  // 阶段查询方法
  pub fn is_boot(&self) -> bool {
    matches!(self, HostMachineState::Boot)
  }

  pub fn is_init(&self) -> bool {
    matches!(self, HostMachineState::Init)
  }

  pub fn is_runtime(&self) -> bool {
    matches!(self, HostMachineState::Runtime(_))
  }

  pub fn is_shutdown(&self) -> bool {
    matches!(self, HostMachineState::Shutdown)
  }

  pub fn is_stopped(&self) -> bool {
    matches!(self, HostMachineState::Stopped)
  }

  // Runtime状态访问方法
  pub fn runtime(&self) -> Option<&RuntimeState> {
    match self {
      HostMachineState::Runtime(runtime) => Some(runtime),
      _ => None,
    }
  }

  // Runtime状态访问方法（可变）
  pub fn runtime_mut(&mut self) -> Option<&mut RuntimeState> {
    match self {
      HostMachineState::Runtime(runtime) => Some(runtime),
      _ => None,
    }
  }

  // 切换为boot
  pub fn set_boot(&mut self) {
    *self = HostMachineState::Boot;
  }

  //切换为init
  pub fn set_init(&mut self) {
    *self = HostMachineState::Init;
  }

  // 切换为runtime
  pub fn set_runtime(&mut self, runtime: RuntimeState) {
    *self = HostMachineState::Runtime(runtime);
  }

  // 切换为shutdown
  pub fn set_shutdown(&mut self) {
    *self = HostMachineState::Shutdown;
  }

  // 切换为stopped
  pub fn set_stopped(&mut self) {
    *self = HostMachineState::Stopped;
  }
}
