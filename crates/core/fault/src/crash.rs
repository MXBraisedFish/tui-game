use std::backtrace::Backtrace;
use std::io::Write;
use std::panic;

use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};

use crate::{CapturedPanic, HostFault, capture_panic, current_fault_domain, is_supervised};

/// Lifecycle phase that identifies where the host was when a panic happened.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CrashPhase {
  Boot = 0,
  Init = 1,
  Runtime = 2,
  Shutdown = 3,
  Stopped = 4,
}

static CRASH_PHASE: AtomicU8 = AtomicU8::new(CrashPhase::Boot as u8);
static CRASH_RECORDED: AtomicBool = AtomicBool::new(false);

/// Sets the current crash phase.
pub fn set_crash_phase(phase: CrashPhase) {
  CRASH_PHASE.store(phase as u8, Ordering::SeqCst);
}

/// Returns the current crash phase.
pub fn current_crash_phase() -> CrashPhase {
  match CRASH_PHASE.load(Ordering::SeqCst) {
    1 => CrashPhase::Init,
    2 => CrashPhase::Runtime,
    3 => CrashPhase::Shutdown,
    4 => CrashPhase::Stopped,
    _ => CrashPhase::Boot,
  }
}

/// Installs the custom panic hook.
///
/// A supervised panic (inside [`catch_host_fault`](crate::catch_host_fault)) during the boot,
/// init or runtime phase is only captured for the supervisor. Any other panic restores the
/// terminal through `restore_terminal` and appends a crash record with the current phase to the
/// crash log; when the record cannot be written, the phase and the panic are printed to stderr
/// and the previous hook runs.
pub fn install_panic_hook(restore_terminal: fn()) {
  CRASH_RECORDED.store(false, Ordering::SeqCst);
  let previous_hook = panic::take_hook();

  panic::set_hook(Box::new(move |panic_info| {
    let phase = current_crash_phase();

    if is_supervised()
      && matches!(
        phase,
        CrashPhase::Boot | CrashPhase::Init | CrashPhase::Runtime
      )
    {
      capture_panic(CapturedPanic {
        domain: current_fault_domain(),
        detail: panic_info
          .payload_as_str()
          .unwrap_or("panic with non-string payload")
          .to_string(),
        location: panic_info.location().map(ToString::to_string),
        backtrace: Backtrace::force_capture().to_string(),
      });
      return;
    }

    let logged = final_restore_and_log(restore_terminal, &format!(
      "phase={phase:?}\nkind=Panic\nlocation={}\ndetail={}\nbacktrace={}\n",
      panic_info
        .location()
        .map(ToString::to_string)
        .unwrap_or_else(|| "unknown".to_string()),
      panic_info
        .payload_as_str()
        .unwrap_or("panic with non-string payload"),
      Backtrace::force_capture(),
    ));

    if !logged {
      eprintln!("[Crash] {:?} phase: {}", phase, panic_info);
      previous_hook(panic_info);
    }
  }))
}

/// Writes one complete supervised fault record. Terminal restoration is left
/// to the normal shutdown path so the exception screen remains visible.
pub fn finalize_host_fault(run_id: &str, fault: &HostFault) -> bool {
  append_crash_record(&format!(
    "run_id={run_id}\nphase={:?}\ndomain={:?}\nkind={:?}\nlocation={}\ndetail={}\nbacktrace={}\n",
    fault.phase,
    fault.domain,
    fault.kind,
    fault.location.as_deref().unwrap_or("unknown"),
    fault.detail,
    fault.backtrace,
  ))
}

fn final_restore_and_log(restore_terminal: fn(), record: &str) -> bool {
  restore_terminal();
  append_crash_record(record)
}

fn append_crash_record(record: &str) -> bool {
  if CRASH_RECORDED.swap(true, Ordering::SeqCst) {
    return true;
  }
  let log_dir = crash_log_dir();
  std::fs::create_dir_all(&log_dir)
    .and_then(|_| {
      std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_dir.join("tui_crash.log"))
    })
    .and_then(|mut file| writeln!(file, "[HostFault]\n{record}---"))
    .is_ok()
}

fn crash_log_dir() -> std::path::PathBuf {
  if let Ok(current_dir) = std::env::current_dir()
    && (current_dir.join("assets").exists() || current_dir.join("Cargo.toml").exists())
  {
    return current_dir.join("data/log");
  }
  if let Ok(executable) = std::env::current_exe()
    && let Some(directory) = executable.parent()
  {
    return directory.join("data/log");
  }
  std::path::PathBuf::from("data/log")
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn crash_phase_round_trips_through_the_global_slot() {
    for phase in [
      CrashPhase::Init,
      CrashPhase::Runtime,
      CrashPhase::Shutdown,
      CrashPhase::Stopped,
      CrashPhase::Boot,
    ] {
      set_crash_phase(phase);
      assert_eq!(current_crash_phase(), phase);
    }
  }
}
