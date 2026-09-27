//! Application-owned state, event routing and service composition.

mod async_runtime;
mod boot_output;
mod event_queue;
mod runtime;
mod services;
mod state_machine;
mod world;

pub use async_runtime::{AsyncRuntime, EngineEvent};
pub use boot_output::BootOutput;
pub use event_queue::EngineEventQueue;
pub(crate) use runtime::{run, run_exception};
pub use services::EngineServices;
pub(crate) use services::current_deployment_root;
pub use state_machine::{
  GameState, HostMachineState, HostState, MainHostState, OverlayKind, OverlayStackTransition,
  RuntimeClosingState, UiNodeKind, UiNodeState,
};
pub use world::RuntimeWorld;
