//! Crash-phase tracking, supervised panic capture, and best-effort crash-log persistence.

use std::backtrace::Backtrace;
use std::io::Write;
use std::panic;
use std::path::{Path, PathBuf};

use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};

use crate::{CapturedPanic, HostFault, capture_panic, current_fault_domain, is_supervised};

/// The host lifecycle phase recorded when a panic occurs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CrashPhase {
  /// The boot stage of the operation.
  Boot = 0,
  /// The init stage of the operation.
  Init = 1,
  /// The runtime stage of the operation.
  Runtime = 2,
  /// The shutdown stage of the operation.
  Shutdown = 3,
  /// The operation is stopped.
  Stopped = 4,
}

static CRASH_PHASE: AtomicU8 = AtomicU8::new(CrashPhase::Boot as u8);
static CRASH_RECORDED: AtomicBool = AtomicBool::new(false);

/// Record the lifecycle phase used by subsequent panic reports.
pub fn set_crash_phase(phase: CrashPhase) {
  CRASH_PHASE.store(phase as u8, Ordering::SeqCst);
}

/// Return the most recently recorded crash phase.
pub fn current_crash_phase() -> CrashPhase {
  match CRASH_PHASE.load(Ordering::SeqCst) {
    1 => CrashPhase::Init,
    2 => CrashPhase::Runtime,
    3 => CrashPhase::Shutdown,
    4 => CrashPhase::Stopped,
    _ => CrashPhase::Boot,
  }
}

/// Install panic reporting that captures supervised faults and restores the terminal for other
/// panics.
pub fn install_panic_hook(restore_terminal: fn(), crash_log_path: PathBuf) {
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

    let logged = final_restore_and_log(
      restore_terminal,
      &crash_log_path,
      &format!(
        "phase={phase:?}\nkind=Panic\nlocation={}\ndetail={}\nbacktrace={}\n",
        panic_info
          .location()
          .map(ToString::to_string)
          .unwrap_or_else(|| "unknown".to_string()),
        panic_info
          .payload_as_str()
          .unwrap_or("panic with non-string payload"),
        Backtrace::force_capture(),
      ),
    );

    if !logged {
      eprintln!("[Crash] {:?} phase: {}", phase, panic_info);
      previous_hook(panic_info);
    }
  }))
}

/// Append a supervised fault record and report whether the record was written.
///
/// # Arguments
///
/// * `run_id` - The identifier of the current host run.
/// * `fault` - The fault.
/// * `crash_log_path` - The destination path for crash records.
pub fn finalize_host_fault(run_id: &str, fault: &HostFault, crash_log_path: &Path) -> bool {
  append_crash_record(
    crash_log_path,
    &format!(
      "run_id={run_id}\nphase={:?}\ndomain={:?}\nkind={:?}\nlocation={}\ndetail={}\nbacktrace={}\n",
      fault.phase,
      fault.domain,
      fault.kind,
      fault.location.as_deref().unwrap_or("unknown"),
      fault.detail,
      fault.backtrace,
    ),
  )
}

fn final_restore_and_log(restore_terminal: fn(), crash_log_path: &Path, record: &str) -> bool {
  restore_terminal();
  append_crash_record(crash_log_path, record)
}

fn append_crash_record(crash_log_path: &Path, record: &str) -> bool {
  if CRASH_RECORDED.swap(true, Ordering::SeqCst) {
    return true;
  }
  let Some(log_dir) = crash_log_path.parent() else {
    return false;
  };
  std::fs::create_dir_all(log_dir)
    .and_then(|_| {
      std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(crash_log_path)
    })
    .and_then(|mut file| writeln!(file, "[HostFault]\n{record}---"))
    .is_ok()
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::process::Command;

  const CHILD_CRASH_PATH: &str = "TUI_GAME_FAULT_TEST_CRASH_PATH";
  const CHILD_MODE: &str = "TUI_GAME_FAULT_TEST_CHILD";

  fn leave_terminal_unchanged() {}

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

  #[test]
  fn panic_hook_writes_to_the_injected_path_from_an_external_working_directory() {
    let nonce = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_nanos();
    let base = std::env::temp_dir().join(format!("fault hook cwd {nonce}"));
    let deployment = base.join("deployment 根");
    let caller = base.join("caller");
    std::fs::create_dir_all(deployment.join("data/log")).unwrap();
    std::fs::create_dir_all(caller.join("assets")).unwrap();
    std::fs::write(caller.join("Cargo.toml"), "fake cwd marker").unwrap();
    let crash_log = deployment.join("data/log/tui_crash.log");

    let output = Command::new(std::env::current_exe().unwrap())
      .arg("--exact")
      .arg("crash::tests::panic_hook_child_entry")
      .arg("--nocapture")
      .current_dir(&caller)
      .env(CHILD_MODE, "1")
      .env(CHILD_CRASH_PATH, &crash_log)
      .output()
      .unwrap();

    assert!(!output.status.success(), "controlled child must panic");
    let record = std::fs::read_to_string(&crash_log).unwrap();
    assert!(record.contains("controlled child panic"));
    assert!(
      !caller.join("data/log/tui_crash.log").exists(),
      "the working directory must not receive a crash log"
    );

    std::fs::remove_dir_all(base).unwrap();
  }

  #[test]
  fn panic_hook_child_entry() {
    if std::env::var_os(CHILD_MODE).is_none() {
      return;
    }
    let crash_log_path = PathBuf::from(std::env::var_os(CHILD_CRASH_PATH).unwrap());
    install_panic_hook(leave_terminal_unchanged, crash_log_path);
    panic!("controlled child panic");
  }
}
