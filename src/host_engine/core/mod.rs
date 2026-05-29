// 模块统一导出
pub mod boot_output;
pub mod exit_state;
pub mod world;
pub mod frame;
pub mod clock;
pub mod crash;

// 引用结构体
pub use boot_output::BootOutput;
pub use exit_state::ExitState;
pub use world::RuntimeWorld;
pub use frame::FrameScheduler;
pub use clock::EngineClock;
pub use crash::{
  CrashPhase,
  current_crash_phase,
  install_panic_hook,
  set_crash_phase,
};