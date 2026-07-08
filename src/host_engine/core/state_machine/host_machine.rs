use super::{MainHostState, OverlayKind, OverlayState, RuntimeState, UiNodeKind, UiNodeState};
use crate::host_engine::core::CrashPhase;

/// 主机状态机枚举，管理引擎的完整生命周期：引导 -> 初始化 -> 运行时 -> 关闭 -> 停止
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HostMachineState {
  Boot,
  Init,
  Runtime(RuntimeState),
  Shutdown,
  Stopped,
}

impl HostMachineState {
  pub fn new() -> Self {
    HostMachineState::Boot
  }

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

  pub fn crash_phase(&self) -> CrashPhase {
    match self {
      HostMachineState::Boot => CrashPhase::Boot,
      HostMachineState::Init => CrashPhase::Init,
      HostMachineState::Runtime(_) => CrashPhase::Runtime,
      HostMachineState::Shutdown => CrashPhase::Shutdown,
      HostMachineState::Stopped => CrashPhase::Stopped,
    }
  }

  /// 切换到初始化状态
  pub fn enter_init(&mut self) {
    *self = HostMachineState::Init;
  }

  /// 切换到运行时状态，以 Host 模式启动
  pub fn enter_runtime(&mut self) {
    *self = HostMachineState::Runtime(RuntimeState::new_host_runtime());
  }

  /// 切换到关闭状态
  pub fn enter_shutdown(&mut self) {
    *self = HostMachineState::Shutdown;
  }

  /// 切换到停止状态
  pub fn enter_stopped(&mut self) {
    *self = HostMachineState::Stopped;
  }

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

  pub fn current_ui_kind(&self) -> Option<UiNodeKind> {
    let runtime = self.runtime()?;
    let MainHostState::Host(host) = runtime.main_host() else {
      return None;
    };
    let node = host.ui_tree().current()?;
    Some(node.kind)
  }

  pub fn current_ui_path_kinds(&self) -> Vec<UiNodeKind> {
    let Some(runtime) = self.runtime() else {
      return Vec::new();
    };
    let MainHostState::Host(host) = runtime.main_host() else {
      return Vec::new();
    };
    host.ui_tree().path().iter().map(|node| node.kind).collect()
  }

  /// 进入指定的 UI 节点，将其压入 UI 导航栈
  pub fn enter_ui_node(&mut self, node: UiNodeState) {
    if let Some(runtime) = self.runtime_mut() {
      if let Some(host) = runtime.main_host_mut().host_mut() {
        host.ui_tree_mut().enter(node);
      }
    }
  }

  /// 弹出当前 UI 节点，返回上一级
  pub fn pop_ui_node(&mut self) -> Option<UiNodeState> {
    self
      .runtime_mut()?
      .main_host_mut()
      .host_mut()?
      .ui_tree_mut()
      .back()
  }

  pub fn current_overlay_kind(&self) -> Option<OverlayKind> {
    self.runtime()?.overlays().current_kind()
  }

  /// 压入一个窗口尺寸过小的警告覆盖层
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

  pub fn push_language_loading_overlay(&mut self) {
    if let Some(runtime) = self.runtime_mut() {
      runtime.overlays_mut().push(OverlayState {
        kind: OverlayKind::LanguageLoading,
        logic: super::OverlayLogicState,
        render: super::OverlayRenderState {
          required_width: 0,
          required_height: 0,
        },
      });
    }
  }

  pub fn push_safe_mode_warning_overlay(&mut self) {
    if let Some(runtime) = self.runtime_mut() {
      runtime.overlays_mut().push(OverlayState {
        kind: OverlayKind::SafeModeWarning,
        logic: super::OverlayLogicState,
        render: super::OverlayRenderState {
          required_width: 0,
          required_height: 0,
        },
      });
    }
  }

  pub fn push_clear_warning_overlay(&mut self) {
    if let Some(runtime) = self.runtime_mut() {
      runtime.overlays_mut().push(OverlayState {
        kind: OverlayKind::ClearWarning,
        logic: super::OverlayLogicState,
        render: super::OverlayRenderState {
          required_width: 0,
          required_height: 0,
        },
      });
    }
  }

  pub fn push_export_settings_overlay(&mut self) {
    if let Some(runtime) = self.runtime_mut() {
      runtime.overlays_mut().push(OverlayState {
        kind: OverlayKind::ExportSettings,
        logic: super::OverlayLogicState,
        render: super::OverlayRenderState {
          required_width: 0,
          required_height: 0,
        },
      });
    }
  }

  pub fn push_export_loading_overlay(&mut self) {
    if let Some(runtime) = self.runtime_mut() {
      runtime.overlays_mut().push(OverlayState {
        kind: OverlayKind::ExportLoading,
        logic: super::OverlayLogicState,
        render: super::OverlayRenderState {
          required_width: 0,
          required_height: 0,
        },
      });
    }
  }

  /// 弹出当前覆盖层的顶部项
  pub fn pop_overlay(&mut self) -> Option<OverlayState> {
    self.runtime_mut()?.overlays_mut().pop()
  }

  pub fn remove_overlay_kind(&mut self, kind: OverlayKind) -> Option<OverlayState> {
    self.runtime_mut()?.overlays_mut().remove_kind(kind)
  }

  pub fn is_host_mode(&self) -> bool {
    self
      .runtime()
      .map(|r| r.main_host().is_host())
      .unwrap_or(true)
  }
}
