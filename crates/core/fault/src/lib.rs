use std::backtrace::Backtrace;
use std::cell::{Cell, RefCell};
use std::fmt;
use std::panic::{self, AssertUnwindSafe};

mod crash;

pub use crash::{CrashPhase, finalize_host_fault, install_panic_hook, set_crash_phase};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostFaultPhase {
  Boot,
  Runtime,
  Shutdown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostFaultDomain {
  Storage,
  Terminal,
  I18n,
  Package,
  Input,
  Audio,
  Network,
  Lua,
  Ui,
  Render,
  Async,
  Other,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostFaultKind {
  Error,
  Panic,
  Invariant,
}

#[derive(Clone, Debug)]
pub struct HostFault {
  pub phase: HostFaultPhase,
  pub domain: HostFaultDomain,
  pub kind: HostFaultKind,
  pub detail: String,
  pub location: Option<String>,
  pub backtrace: String,
}

impl HostFault {
  pub fn error(phase: HostFaultPhase, domain: HostFaultDomain, detail: impl Into<String>) -> Self {
    Self {
      phase,
      domain,
      kind: HostFaultKind::Error,
      detail: detail.into(),
      location: None,
      backtrace: Backtrace::force_capture().to_string(),
    }
  }
}

impl fmt::Display for HostFault {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      formatter,
      "{:?} fault in {:?}: {}",
      self.kind, self.domain, self.detail
    )
  }
}

impl std::error::Error for HostFault {}

#[derive(Clone, Debug)]
pub struct CapturedPanic {
  pub domain: HostFaultDomain,
  pub detail: String,
  pub location: Option<String>,
  pub backtrace: String,
}

thread_local! {
  static SUPERVISED: Cell<bool> = const { Cell::new(false) };
  static CURRENT_DOMAIN: Cell<HostFaultDomain> = const { Cell::new(HostFaultDomain::Other) };
  static CAPTURED_PANIC: RefCell<Option<CapturedPanic>> = const { RefCell::new(None) };
}

pub fn is_supervised() -> bool {
  SUPERVISED.get()
}

pub fn current_fault_domain() -> HostFaultDomain {
  CURRENT_DOMAIN.get()
}

pub fn with_fault_domain<T>(domain: HostFaultDomain, operation: impl FnOnce() -> T) -> T {
  let previous = CURRENT_DOMAIN.replace(domain);
  let result = operation();
  CURRENT_DOMAIN.set(previous);
  result
}

pub fn capture_panic(report: CapturedPanic) {
  CAPTURED_PANIC.with_borrow_mut(|slot| *slot = Some(report));
}

/// Runs one coarse host region under panic supervision. External/data errors
/// should still be represented as normal Results inside the region; only a
/// host panic crosses this boundary.
pub fn catch_host_fault<T>(
  phase: HostFaultPhase,
  domain: HostFaultDomain,
  operation: impl FnOnce() -> T,
) -> Result<T, HostFault> {
  let was_supervised = SUPERVISED.replace(true);
  let previous_domain = CURRENT_DOMAIN.replace(domain);
  CAPTURED_PANIC.with_borrow_mut(|slot| *slot = None);
  let result = panic::catch_unwind(AssertUnwindSafe(operation));
  SUPERVISED.set(was_supervised);
  CURRENT_DOMAIN.set(previous_domain);

  match result {
    Ok(value) => Ok(value),
    Err(payload) => {
      let captured = CAPTURED_PANIC.with_borrow_mut(Option::take);
      let detail = captured
        .as_ref()
        .map(|report| report.detail.clone())
        .unwrap_or_else(|| panic_payload(&payload));
      Err(HostFault {
        phase,
        domain: captured.as_ref().map_or(domain, |report| report.domain),
        kind: HostFaultKind::Panic,
        detail,
        location: captured.as_ref().and_then(|report| report.location.clone()),
        backtrace: captured
          .map(|report| report.backtrace)
          .unwrap_or_else(|| Backtrace::force_capture().to_string()),
      })
    }
  }
}

pub(crate) fn panic_payload(payload: &Box<dyn std::any::Any + Send>) -> String {
  if let Some(message) = payload.downcast_ref::<&str>() {
    (*message).to_string()
  } else if let Some(message) = payload.downcast_ref::<String>() {
    message.clone()
  } else {
    "panic with non-string payload".to_string()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn supervised_panic_becomes_host_fault() {
    let result = catch_host_fault(HostFaultPhase::Runtime, HostFaultDomain::Ui, || {
      panic!("broken ui invariant")
    });
    let fault = result.unwrap_err();
    assert_eq!(fault.phase, HostFaultPhase::Runtime);
    assert_eq!(fault.domain, HostFaultDomain::Ui);
    assert_eq!(fault.kind, HostFaultKind::Panic);
    assert!(fault.detail.contains("broken ui invariant"));
  }
}
