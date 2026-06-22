use super::{MainHostState, OverlayKind, OverlayState, RuntimeState, UiNodeKind, UiNodeState};
use crate::host_engine::core::CrashPhase;

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
  pub fn new() -> Self {
    HostMachineState::Boot
  }

  // ── 阶段查询方法 ──

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

  // ── Runtime状态访问方法 ──

  pub fn runtime(&self) -> Option<&RuntimeState> {
    match self {
      HostMachineState::Runtime(runtime) => Some(runtime),
      _ => None,
    }
  }

  pub fn runtime_mut(&mut self) -> Option<&mut RuntimeState> {
    match self {
      HostMachineState::Runtime(runtime) => Some(runtime),
      _ => None,
    }
  }

  // ── 崩溃阶段映射 ──

  pub fn crash_phase(&self) -> CrashPhase {
    match self {
      HostMachineState::Boot => CrashPhase::Boot,
      HostMachineState::Init => CrashPhase::Init,
      HostMachineState::Runtime(_) => CrashPhase::Runtime,
      HostMachineState::Shutdown => CrashPhase::Shutdown,
      HostMachineState::Stopped => CrashPhase::Stopped,
    }
  }

  // ── 生命周期转换方法 ──

  pub fn enter_init(&mut self) {
    *self = HostMachineState::Init;
  }

  pub fn enter_runtime(&mut self) {
    *self = HostMachineState::Runtime(RuntimeState::new_host_runtime());
  }

  pub fn enter_shutdown(&mut self) {
    *self = HostMachineState::Shutdown;
  }

  pub fn enter_stopped(&mut self) {
    *self = HostMachineState::Stopped;
  }

  // ── 原始赋值方法 ──

  pub fn set_boot(&mut self) {
    *self = HostMachineState::Boot;
  }

  pub fn set_init(&mut self) {
    *self = HostMachineState::Init;
  }

  pub fn set_runtime(&mut self, runtime: RuntimeState) {
    *self = HostMachineState::Runtime(runtime);
  }

  pub fn set_shutdown(&mut self) {
    *self = HostMachineState::Shutdown;
  }

  pub fn set_stopped(&mut self) {
    *self = HostMachineState::Stopped;
  }

  // ── UI 查询 ──

  /// 查询当前 UI 节点类型。
  /// 仅在 Runtime(Host) 状态下返回有意义的值。
  pub fn current_ui_kind(&self) -> Option<UiNodeKind> {
    let runtime = self.runtime()?;
    let MainHostState::Host(host) = runtime.main_host() else {
      return None;
    };
    let node = host.ui_tree().current()?;
    Some(node.kind)
  }

  /// 向 UI 树压入一个新节点。
  pub fn enter_ui_node(&mut self, node: UiNodeState) {
    if let Some(runtime) = self.runtime_mut() {
      if let Some(host) = runtime.main_host_mut().host_mut() {
        host.ui_tree_mut().enter(node);
      }
    }
  }

  /// 从 UI 树弹出一个节点（返回上一级）。
  pub fn pop_ui_node(&mut self) -> Option<UiNodeState> {
    self
      .runtime_mut()?
      .main_host_mut()
      .host_mut()?
      .ui_tree_mut()
      .back()
  }

  // ── 覆盖层查询 ──

  pub fn current_overlay_kind(&self) -> Option<OverlayKind> {
    self.runtime()?.overlays().current_kind()
  }

  pub fn push_window_size_overlay(&mut self, min_w: u32, min_h: u32) {
    if let Some(runtime) = self.runtime_mut() {
      runtime.overlays_mut().push(OverlayState {
        kind: OverlayKind::WindowSizeWarning,
        logic: super::OverlayLogicState,
        render: super::OverlayRenderState {
          required_width: min_w,
          required_height: min_h,
        },
      });
    }
  }

  pub fn pop_overlay(&mut self) -> Option<OverlayState> {
    self.runtime_mut()?.overlays_mut().pop()
  }

  pub fn is_host_mode(&self) -> bool {
    self
      .runtime()
      .map(|r| r.main_host().is_host())
      .unwrap_or(true)
  }
}
