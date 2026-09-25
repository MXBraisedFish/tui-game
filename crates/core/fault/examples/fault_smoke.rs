//! Minimal entry: installs the crash hook, turns one supervised panic into a host fault and prints it.

use std::sync::atomic::{AtomicBool, Ordering};

use tg_core_fault::{
  CrashPhase, HostFaultDomain, HostFaultPhase, catch_host_fault, install_panic_hook,
  set_crash_phase,
};

static TERMINAL_RESTORED: AtomicBool = AtomicBool::new(false);

fn restore_terminal() {
  TERMINAL_RESTORED.store(true, Ordering::SeqCst);
}

fn main() {
  install_panic_hook(restore_terminal);
  set_crash_phase(CrashPhase::Runtime);
  let fault = catch_host_fault(HostFaultPhase::Runtime, HostFaultDomain::Other, || -> () {
    panic!("smoke panic")
  })
  .expect_err("panic must be converted into a fault");
  assert!(fault.to_string().contains("smoke panic"));
  assert!(fault.location.is_some(), "supervised panic is captured by the hook");
  assert!(!TERMINAL_RESTORED.load(Ordering::SeqCst), "supervised panic keeps the terminal");
  let value = catch_host_fault(HostFaultPhase::Runtime, HostFaultDomain::Other, || 7).unwrap();
  assert_eq!(value, 7);
  println!("fault ok: {fault}");
}
