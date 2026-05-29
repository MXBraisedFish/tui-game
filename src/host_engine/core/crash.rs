// panic模块
use std::panic;
// 线程原子化和内存顺序
use std::sync::atomic::{AtomicU8, Ordering};

use crate::host_engine::services::TerminalService;

// 阶段枚举
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CrashPhase {
  Initial = 0, // 初始化
  Preparing = 1, // 准备阶段
  Runtime = 2, // 运行阶段
  Shutdown = 3 // 关闭阶段
}

// 全局静态且唯一，每个线程都用这个变量
// 内存顺序安全确保读值干净
static CRASH_PHASE: AtomicU8 = AtomicU8::new(CrashPhase::Initial as u8);

// 设置运行阶段
pub fn set_crash_phase(phase: CrashPhase) {
  CRASH_PHASE.store(phase as u8, Ordering::SeqCst);
}

// 获取当前运行阶段
pub fn current_crash_phase() -> CrashPhase {
  match CRASH_PHASE.load(Ordering::SeqCst) {
    1 => CrashPhase::Preparing,
    2 => CrashPhase::Runtime,
    3 => CrashPhase::Shutdown,
    _ => CrashPhase::Initial
  }
}

pub fn install_panic_hook() {
  let previous_hook = panic::take_hook();

  panic::set_hook(Box::new(move |panic_info| {
    let phase = current_crash_phase();

    TerminalService::force_restore();

    eprintln!("[Crash] Panic occurred during {:?} phase.", phase);

    previous_hook(panic_info);
  }))
}