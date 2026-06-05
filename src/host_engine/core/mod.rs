// 模块统一导出
pub mod boot_output;
pub mod clock;
pub mod crash;
pub mod exit_state;
pub mod frame;
pub mod runtime_session;
pub mod world;

// 引用结构体
pub use boot_output::BootOutput;
pub use clock::EngineClock;
pub use crash::{CrashPhase, current_crash_phase, install_panic_hook, set_crash_phase};
pub use exit_state::ExitState;
pub use frame::FrameScheduler;
pub use runtime_session::{
  ExecutionContext, FocusState, HostSurface, OverlayKind, OverlayStack, RuntimeAction,
  RuntimeSession, RuntimeState, UiNode, UiTree,
};
pub use world::RuntimeWorld;
