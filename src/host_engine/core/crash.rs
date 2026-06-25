use std::panic;

use std::sync::atomic::{AtomicU8, Ordering};

use crate::host_engine::services::TerminalService;

/// 崩溃阶段枚举，用于在 panic 时标识当前所处的生命周期阶段
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CrashPhase {
  Boot = 0,
  Init = 1,
  Runtime = 2,
  Shutdown = 3,
  Stopped = 4,
}

static CRASH_PHASE: AtomicU8 = AtomicU8::new(CrashPhase::Boot as u8);

/// 设置当前崩溃阶段的值
pub fn set_crash_phase(phase: CrashPhase) {
  CRASH_PHASE.store(phase as u8, Ordering::SeqCst);
}

/// 读取当前崩溃阶段
pub fn current_crash_phase() -> CrashPhase {
  match CRASH_PHASE.load(Ordering::SeqCst) {
    1 => CrashPhase::Init,
    2 => CrashPhase::Runtime,
    3 => CrashPhase::Shutdown,
    4 => CrashPhase::Stopped,
    _ => CrashPhase::Boot,
  }
}

/// 安装自定义 panic 钩子，在崩溃时恢复终端状态并打印当前阶段
pub fn install_panic_hook() {
  let previous_hook = panic::take_hook();

  panic::set_hook(Box::new(move |panic_info| {
    let phase = current_crash_phase();

    TerminalService::force_restore();

    eprintln!("[Crash] Panic occurred during {:?} phase.", phase);

    previous_hook(panic_info);
  }))
}
