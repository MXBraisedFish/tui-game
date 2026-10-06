//! Lifecycle entry points delegating runtime work to the application layer.

use crate::host_engine::app::{EngineServices, RuntimeWorld};
use crate::host_engine::core::ExitState;

/// Delegate normal runtime execution to the application layer.
pub fn run(services: &mut EngineServices, world: &mut RuntimeWorld) -> ExitState {
  crate::host_engine::app::run(services, world)
}

/// Delegate exceptional runtime execution to the application layer.
pub fn run_exception(services: &mut EngineServices, world: &mut RuntimeWorld) -> ExitState {
  crate::host_engine::app::run_exception(services, world)
}
