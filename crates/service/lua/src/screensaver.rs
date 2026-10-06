//! Screensaver-session lifecycle and frame callbacks isolated from the game session.

use std::time::Duration;

use tg_core_package_id::PackageId;
use tg_service_layout::Size;
use tg_service_log::LogSessionId;

use super::{
  LuaDrawCommand, LuaEventDelivery, LuaHostCommand, LuaObjectPool, LuaSession,
  LuaSessionDiagnostics, LuaSessionError, LuaSessionKind, LuaSessionState, LuaSessionToken,
};

const MAX_REAL_DELTA: Duration = Duration::from_millis(250);
const MAX_FIXED_UPDATES_PER_FRAME: usize = 8;

/// The lifecycle owner of one screensaver session independent of game-session state.
pub struct ScreensaverService {
  session: Option<LuaSession>,
  package: Option<PackageId>,
  accumulator: Duration,
  generation: u64,
  log_session: Option<LogSessionId>,
}

impl ScreensaverService {
  /// Create a screensaver service with its initial state.
  pub fn new() -> Self {
    Self {
      session: None,
      package: None,
      accumulator: Duration::ZERO,
      generation: 0,
      log_session: None,
    }
  }

  /// Start the screensaver state addressed by this operation.
  ///
  /// # Arguments
  ///
  /// * `session` - The Lua session receiving the operation.
  /// * `package` - The validated package snapshot.
  /// * `log_session` - The log session.
  pub fn start(
    &mut self,
    session: LuaSession,
    package: PackageId,
    log_session: Option<LogSessionId>,
  ) -> Option<LogSessionId> {
    let previous_log = self.stop();
    self.generation = self.generation.wrapping_add(1).max(1);
    self.package = Some(package);
    self.accumulator = Duration::ZERO;
    self.session = Some(session);
    self.log_session = log_session;
    previous_log
  }

  /// Stop the screensaver state addressed by this operation.
  pub fn stop(&mut self) -> Option<LogSessionId> {
    if let Some(mut session) = self.session.take() {
      session.stop();
    }
    self.package = None;
    self.accumulator = Duration::ZERO;
    self.log_session.take()
  }

  /// Return the current log session.
  pub fn log_session(&self) -> Option<LogSessionId> {
    self.log_session
  }

  /// Report whether this screensaver service is active.
  pub fn is_active(&self) -> bool {
    self.session.is_some()
  }

  /// Report whether this screensaver service is faulted.
  pub fn is_faulted(&self) -> bool {
    self
      .session
      .as_ref()
      .is_some_and(|session| session.state() == LuaSessionState::Faulted)
  }

  /// Return the current package id.
  pub fn package_id(&self) -> Option<&str> {
    self.package.as_ref().map(|package| package.mod_id.as_str())
  }

  /// Return the current package.
  pub fn package(&self) -> Option<&PackageId> {
    self.package.as_ref()
  }

  /// Report whether the session owns a host object pool.
  pub fn has_objects(&self) -> bool {
    self.session.as_ref().is_some_and(LuaSession::has_objects)
  }

  /// Read the session-owned object pool through the supplied callback when it exists.
  pub fn with_objects<R>(&self, operation: impl FnOnce(&LuaObjectPool) -> R) -> Option<R> {
    self.session.as_ref()?.with_objects(operation)
  }

  /// Mutate the session-owned object pool through the supplied callback when it exists.
  pub fn with_objects_mut<R>(&self, operation: impl FnOnce(&mut LuaObjectPool) -> R) -> Option<R> {
    self.session.as_ref()?.with_objects_mut(operation)
  }

  /// Return the current session token.
  pub fn session_token(&self) -> Option<LuaSessionToken> {
    self.session.as_ref().map(|_| LuaSessionToken {
      kind: LuaSessionKind::Screensaver,
      generation: self.generation,
    })
  }

  /// Return the current diagnostics.
  pub fn diagnostics(&self) -> Option<LuaSessionDiagnostics> {
    self.session.as_ref().map(|session| LuaSessionDiagnostics {
      entry_path: session.entry_path().to_path_buf(),
      stats: session.last_stats(),
      memory_bytes: session.memory_used(),
    })
  }

  /// Update the base dimensions exposed to subsequent script drawing callbacks.
  pub fn set_base_size(&mut self, size: Size) {
    if let Some(session) = self.session.as_mut() {
      session.set_base_size(size);
    }
  }

  /// Deliver one owned event to its callback or HandleEvent under the session budget.
  ///
  /// # Errors
  ///
  /// Return a session error for invalid session state, callback failures, invalid script data, or
  /// execution/memory budget exhaustion.
  pub fn dispatch_event(&mut self, delivery: &LuaEventDelivery) -> Result<(), LuaSessionError> {
    let Some(session) = self.session.as_mut() else {
      return Ok(());
    };
    session.dispatch_event(delivery)
  }

  /// Execute fixed 60 Hz updates and one frame update, returning the fixed-update count.
  ///
  /// # Errors
  ///
  /// Return a session error for invalid session state, callback failures, invalid script data, or
  /// execution/memory budget exhaustion.
  pub fn advance(&mut self, real_delta: Duration) -> Result<usize, LuaSessionError> {
    let Some(session) = self.session.as_mut() else {
      return Ok(0);
    };
    let real_delta = real_delta.min(MAX_REAL_DELTA);
    let fixed_delta = Duration::from_secs_f64(1.0 / 60.0);
    self.accumulator = self.accumulator.saturating_add(real_delta);

    let mut updates = 0;
    while self.accumulator >= fixed_delta && updates < MAX_FIXED_UPDATES_PER_FRAME {
      session.update()?;
      self.accumulator = self.accumulator.saturating_sub(fixed_delta);
      updates += 1;
    }
    if updates == MAX_FIXED_UPDATES_PER_FRAME && self.accumulator >= fixed_delta {
      self.accumulator =
        Duration::from_secs_f64(self.accumulator.as_secs_f64() % fixed_delta.as_secs_f64());
    }
    let alpha = self.accumulator.as_secs_f64() / fixed_delta.as_secs_f64();
    session.update_frame(real_delta, alpha)?;
    Ok(updates)
  }

  /// Invoke the script Render callback and collect bounded draw commands for the frame.
  ///
  /// # Errors
  ///
  /// Return a session error for invalid session state, callback failures, invalid script data, or
  /// execution/memory budget exhaustion.
  pub fn render(&mut self) -> Result<(), LuaSessionError> {
    let Some(session) = self.session.as_mut() else {
      return Ok(());
    };
    session.render()
  }

  /// Drain host commands produced by the session callbacks.
  pub fn take_host_commands(&mut self) -> Vec<LuaHostCommand> {
    self
      .session
      .as_mut()
      .map(LuaSession::take_host_commands)
      .unwrap_or_default()
  }

  /// Drain the structured drawing commands produced for the current frame.
  pub fn take_draw_commands(&mut self) -> Vec<LuaDrawCommand> {
    self
      .session
      .as_mut()
      .map(LuaSession::take_draw_commands)
      .unwrap_or_default()
  }
}

impl Default for ScreensaverService {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;

  use super::*;
  use crate::{LuaPolicy, LuaSessionSpec};
  use tg_core_package_id::{PackageSource, PackageType};

  #[test]
  fn checked_in_screensaver_runs_and_releases_its_object_pool() {
    let entry_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
      .join("../../..")
      .join("test_package/screensaver/layer_waves/scripts/main.lua");
    let session = LuaSession::load(
      LuaSessionSpec {
        package_id: "test_layer_waves".to_string(),
        session_kind: LuaSessionKind::Screensaver,
        entry_path,
        fixed_delta: Duration::from_secs_f64(1.0 / 60.0),
        base_size: Size {
          width: 100,
          height: 30,
        },
        continue_data: None,
        best_data: None,
        save_game_enabled: false,
        save_best_enabled: false,
      },
      LuaPolicy::default(),
    )
    .expect("test screensaver should load");

    let mut service = ScreensaverService::new();
    service.start(
      session,
      PackageId::new(
        PackageSource::Official,
        PackageType::Screensaver,
        "test_layer_waves",
      )
      .unwrap(),
      None,
    );
    assert!(service.is_active());
    assert!(service.has_objects());
    assert_eq!(service.advance(Duration::from_millis(20)).unwrap(), 1);
    service.render().unwrap();
    assert!(!service.take_draw_commands().is_empty());

    service.stop();
    assert!(!service.is_active());
    assert!(!service.has_objects());
    assert!(service.session_token().is_none());
  }
}
