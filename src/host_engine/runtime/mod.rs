//! Runtime lifecycle phase entry points.

use crate::host_engine::app::{EngineServices, RuntimeWorld};
use crate::host_engine::core::ExitState;

/// Runs the application runtime phase.
pub fn run(services: &mut EngineServices, world: &mut RuntimeWorld) -> ExitState {
  crate::host_engine::app::run(services, world)
}

/// Runs the application exception screen after a supervised runtime fault.
pub fn run_exception(services: &mut EngineServices, world: &mut RuntimeWorld) -> ExitState {
  crate::host_engine::app::run_exception(services, world)
}
