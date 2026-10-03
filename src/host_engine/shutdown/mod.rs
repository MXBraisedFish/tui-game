//! Shutdown-phase ordering of pending writes, script stops, service release, and terminal restoration.

use crate::host_engine::app::{EngineServices, RuntimeWorld};
use crate::host_engine::core::{ExitState, set_crash_phase};

use super::services::{HostLogMessage, LogSource};

/// Stop script sessions, wait for pending writes, release services, and restore terminal state.
///
/// # Arguments
///
/// * `services` - The application services supplied by the lifecycle phase.
/// * `world` - The application-owned runtime state.
/// * `_exit_state` - The exit state.
pub fn close(services: &mut EngineServices, mut world: RuntimeWorld, _exit_state: ExitState) {
  let write_barrier = services.async_runtime.write_barrier();
  write_barrier.stop_new_writes();
  services.async_runtime.stop_all_managed_threads();
  services.audio.shutdown();
  write_barrier.wait();
  services.async_runtime.shutdown();
  for (path, error) in write_barrier.snapshot().failed {
    services.log.error_message(
      LogSource::Shutdown,
      HostLogMessage::new(
        "log_info.shutdown.write_failed",
        "A pending write could not be completed for {path}: {error}",
      )
      .param("path", path.display().to_string())
      .param("error", error),
    );
  }

  if let Some(id) = services.screensaver.stop() {
    services.log.close_session(id);
  }
  if let Some(id) = services.game.stop() {
    services.log.close_session(id);
  }
  let _ = services.input_method.release_input_method();

  services.terminal.exit();
  services.log.info_message(
    LogSource::Shutdown,
    HostLogMessage::new(
      "log_info.shutdown.completed",
      "Shutdown completed and terminal ownership was released.",
    ),
  );
  world.state.enter_stopped();
  set_crash_phase(world.state.crash_phase());
}
