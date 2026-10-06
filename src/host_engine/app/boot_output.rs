//! Boot results carried into the initialized application state.

use crate::host_engine::app::{EngineServices, RuntimeWorld};
use crate::host_engine::core::HostFault;

/// Initialized services, retained application state, and an optional supervised boot fault.
///
/// # Fields
///
/// * `services` - The application services supplied by the lifecycle phase.
/// * `world` - The application-owned runtime state.
/// * `fault` - The fault.
pub struct BootOutput {
  /// The application services supplied by the lifecycle phase.
  pub services: EngineServices,
  /// The application-owned runtime state.
  pub world: RuntimeWorld,
  /// The fault.
  pub fault: Option<HostFault>,
}

impl BootOutput {
  /// Retain initialized services and application state, extracting any returned or caught boot
  /// fault.
  ///
  /// # Arguments
  ///
  /// * `services` - The application services supplied by the lifecycle phase.
  /// * `world` - The application-owned runtime state.
  /// * `result` - The result.
  pub(crate) fn from_boot_result(
    services: EngineServices,
    world: RuntimeWorld,
    result: Result<Result<(), HostFault>, HostFault>,
  ) -> Self {
    let fault = match result {
      Ok(Ok(())) => None,
      Ok(Err(fault)) | Err(fault) => Some(fault),
    };
    Self {
      services,
      world,
      fault,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::host_engine::core::{HostFaultDomain, HostFaultPhase};
  use std::path::PathBuf;

  #[test]
  fn boot_fault_keeps_the_partially_initialized_services_available() {
    let root = PathBuf::from(r"E:\Code\tg-test\boot-fault").join(std::process::id().to_string());
    let services = EngineServices::for_test(root.clone());
    let fault = HostFault::error(
      HostFaultPhase::Boot,
      HostFaultDomain::I18n,
      "test boot failure",
    );

    let output = BootOutput::from_boot_result(services, RuntimeWorld::new(), Err(fault));

    assert_eq!(output.services.storage.root_dir(), root);
    assert_eq!(output.fault.unwrap().phase, HostFaultPhase::Boot);
  }
}
