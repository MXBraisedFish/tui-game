//! Game-session lifecycle, frame callbacks, host commands, and optional save support.

use std::time::Duration;

use serde_json::Value as JsonValue;

use tg_core_package_id::{PackageId, PackageSource};
use tg_service_layout::Size;
use tg_service_log::LogSessionId;

use super::{
  LuaDrawCommand, LuaEventDelivery, LuaExecutionStats, LuaHostCommand, LuaObjectPool, LuaSession,
  LuaSessionError, LuaSessionKind, LuaSessionState, LuaSessionToken,
};

const MAX_REAL_DELTA: Duration = Duration::from_millis(250);
const MAX_FIXED_UPDATES_PER_FRAME: usize = 8;

/// The Lua session diagnostics representation used by this module.
///
/// # Fields
///
/// * `entry_path` - The filesystem path for entry.
/// * `stats` - The stats.
/// * `memory_bytes` - The memory measured in bytes.
#[derive(Clone, Debug)]
pub struct LuaSessionDiagnostics {
  /// The filesystem path for entry.
  pub entry_path: std::path::PathBuf,
  /// The stats.
  pub stats: LuaExecutionStats,
  /// The memory measured in bytes.
  pub memory_bytes: usize,
}

/// The loading, active, stopped, or faulted state of the game session.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameSessionState {
  /// The operation is inactive.
  Inactive,
  /// The operation is running.
  Running,
  /// The operation is faulted.
  Faulted,
}

/// Initial save data, dimensions, capabilities, and bindings supplied when starting a game.
///
/// # Fields
///
/// * `target_fps` - The requested frame rate, or no explicit limit.
/// * `min_size` - The min size.
/// * `save_game_enabled` - The save game enabled.
/// * `save_best_enabled` - The save best enabled.
#[derive(Clone, Copy, Debug)]
pub struct GameStartOptions {
  /// The requested frame rate, or no explicit limit.
  pub target_fps: Option<u32>,
  /// The min size.
  pub min_size: Size,
  /// The save game enabled.
  pub save_game_enabled: bool,
  /// The save best enabled.
  pub save_best_enabled: bool,
}

/// The lifecycle owner of one active game session and its save results.
pub struct GameService {
  session: Option<LuaSession>,
  package: Option<PackageId>,
  target_fps: Option<u32>,
  min_size: Size,
  save_game_enabled: bool,
  save_best_enabled: bool,
  accumulator: Duration,
  generation: u64,
  log_session: Option<LogSessionId>,
}

impl GameService {
  /// Create a game service with its initial state.
  pub fn new() -> Self {
    Self {
      session: None,
      package: None,
      target_fps: None,
      min_size: Size::default(),
      save_game_enabled: false,
      save_best_enabled: false,
      accumulator: Duration::ZERO,
      generation: 0,
      log_session: None,
    }
  }

  /// Start the game state addressed by this operation.
  ///
  /// # Arguments
  ///
  /// * `session` - The Lua session receiving the operation.
  /// * `package` - The validated package snapshot.
  /// * `options` - The validated options for the operation.
  /// * `log_session` - The log session.
  pub fn start(
    &mut self,
    session: LuaSession,
    package: PackageId,
    options: GameStartOptions,
    log_session: Option<LogSessionId>,
  ) -> Option<LogSessionId> {
    let previous_log = self.stop();
    self.generation = self.generation.wrapping_add(1).max(1);
    self.package = Some(package);
    self.target_fps = options.target_fps;
    self.min_size = options.min_size;
    self.save_game_enabled = options.save_game_enabled;
    self.save_best_enabled = options.save_best_enabled;
    self.accumulator = Duration::ZERO;
    self.session = Some(session);
    self.log_session = log_session;
    previous_log
  }

  /// Stop the game state addressed by this operation.
  pub fn stop(&mut self) -> Option<LogSessionId> {
    self.package = None;
    let log_session = self.log_session.take();
    if let Some(mut session) = self.session.take() {
      session.stop();
    }
    self.target_fps = None;
    self.min_size = Size::default();
    self.save_game_enabled = false;
    self.save_best_enabled = false;
    self.accumulator = Duration::ZERO;
    log_session
  }

  /// Return the current log session.
  pub fn log_session(&self) -> Option<LogSessionId> {
    self.log_session
  }

  /// Return the state for the addressed object.
  pub fn state(&self) -> GameSessionState {
    match self.session.as_ref().map(LuaSession::state) {
      Some(LuaSessionState::Running) => GameSessionState::Running,
      Some(LuaSessionState::Faulted) => GameSessionState::Faulted,
      _ => GameSessionState::Inactive,
    }
  }

  /// Report whether this game service is active.
  pub fn is_active(&self) -> bool {
    self.session.is_some()
  }

  /// Return the current package id.
  pub fn package_id(&self) -> Option<&str> {
    self.package.as_ref().map(|package| package.mod_id.as_str())
  }

  /// Return the current package.
  pub fn package(&self) -> Option<&PackageId> {
    self.package.as_ref()
  }

  /// Return the current session token.
  pub fn session_token(&self) -> Option<LuaSessionToken> {
    self.session.as_ref().map(|_| LuaSessionToken {
      kind: LuaSessionKind::Game,
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

  /// Return the current package source.
  pub fn package_source(&self) -> Option<&PackageSource> {
    self.package.as_ref().map(|package| &package.source)
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

  /// Return the package's requested frame-rate limit, if one was declared.
  pub fn target_fps(&self) -> Option<u32> {
    self.target_fps
  }

  /// Return the terminal dimensions required by the package.
  pub fn min_size(&self) -> Size {
    self.min_size
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

  /// Return the active keyboard subscription generation, or `None` when input is rejected.
  pub fn input_generation(&self, data: &crate::LuaEventData) -> Option<u64> {
    self
      .session
      .as_ref()
      .and_then(|session| session.input_generation(data))
  }

  /// Close live game input and collect releases before an ownership boundary.
  pub fn close_input(&mut self, actions: bool, keys: bool) -> Vec<crate::LuaEventData> {
    self
      .session
      .as_mut()
      .map_or_else(Vec::new, |session| session.close_input(actions, keys))
  }

  /// Drain closing releases created by a script subscription change.
  pub fn take_input_releases(&mut self) -> Vec<crate::LuaEventData> {
    self
      .session
      .as_mut()
      .map_or_else(Vec::new, LuaSession::take_input_releases)
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

  /// Invoke the game-save callback and return its validated serializable result.
  ///
  /// # Errors
  ///
  /// Return a session error for a failing save callback or a result that violates the supported
  /// save-data contract.
  pub fn save_game(&mut self) -> Result<Option<JsonValue>, LuaSessionError> {
    if !self.save_game_enabled {
      return Ok(None);
    }
    self
      .session
      .as_mut()
      .map_or(Ok(None), LuaSession::save_game)
  }

  /// Invoke the best-score callback and return its validated score result.
  ///
  /// # Errors
  ///
  /// Return a session error for a failing score callback or a score that violates the supported
  /// result contract.
  pub fn save_best(&mut self) -> Result<Option<JsonValue>, LuaSessionError> {
    if !self.save_best_enabled {
      return Ok(None);
    }
    self
      .session
      .as_mut()
      .map_or(Ok(None), LuaSession::save_best)
  }
}

impl Default for GameService {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(test)]
mod tests {
  use std::fs;
  use std::sync::atomic::{AtomicU64, Ordering};

  use super::*;
  use tg_core_package_id::{PackageSource, PackageType};

  fn test_package_id() -> PackageId {
    PackageId::new(PackageSource::Mod, PackageType::Game, "test_game").unwrap()
  }
  use crate::{LuaPolicy, LuaSessionKind, LuaSessionSpec};

  static TEST_ID: AtomicU64 = AtomicU64::new(1);

  fn test_session() -> LuaSession {
    let directory = std::env::temp_dir().join(format!(
      "tui_game_service_{}_{}",
      std::process::id(),
      TEST_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&directory).unwrap();
    let entry_path = directory.join("main.lua");
    fs::write(
      &entry_path,
      r#"
        local updates = 0
        local frames = 0
        function Init(ctx) end
        function HandleEvent(event) end
        function Update(dt) updates = updates + 1 end
        function UpdateFrame(dt, alpha) frames = frames + 1 end
        function Render() end
        function SaveGame() return { updates = updates, frames = frames } end
      "#,
    )
    .unwrap();
    LuaSession::load(
      LuaSessionSpec {
        package_id: "test_game_service".to_string(),
        session_kind: LuaSessionKind::Game,
        entry_path,
        fixed_delta: Duration::from_secs_f64(1.0 / 60.0),
        base_size: Size {
          width: 80,
          height: 24,
        },
        continue_data: None,
        best_data: None,
        save_game_enabled: true,
        save_best_enabled: false,
      },
      LuaPolicy::default(),
    )
    .unwrap()
  }

  #[test]
  fn inactive_service_ignores_updates() {
    let mut service = GameService::new();
    assert_eq!(service.advance(Duration::from_secs(1)).unwrap(), 0);
    assert_eq!(service.state(), GameSessionState::Inactive);
  }

  #[test]
  fn package_target_fps_remains_optional_for_the_host_scheduler() {
    let mut service = GameService::new();
    service.start(
      test_session(),
      test_package_id(),
      GameStartOptions {
        target_fps: None,
        min_size: Size {
          width: 40,
          height: 12,
        },
        save_game_enabled: false,
        save_best_enabled: false,
      },
      None,
    );

    assert_eq!(service.target_fps(), None);
    service.stop();
    assert_eq!(service.target_fps(), None);
  }

  #[test]
  fn fixed_update_clamps_delta_and_catches_up_at_most_eight_times() {
    let mut service = GameService::new();
    service.start(
      test_session(),
      test_package_id(),
      GameStartOptions {
        target_fps: Some(120),
        min_size: Size {
          width: 40,
          height: 12,
        },
        save_game_enabled: true,
        save_best_enabled: false,
      },
      None,
    );

    assert_eq!(service.advance(Duration::from_secs(2)).unwrap(), 8);
    assert_eq!(
      service
        .advance(Duration::from_secs_f64(1.0 / 60.0))
        .unwrap(),
      1
    );
    let save = service.save_game().unwrap().unwrap();
    assert_eq!(save["updates"], 9);
    assert_eq!(save["frames"], 2);
    service.stop();
  }

  #[test]
  fn each_started_session_receives_a_new_generation() {
    let mut service = GameService::new();
    assert!(!service.has_objects());
    service.start(
      test_session(),
      test_package_id(),
      GameStartOptions {
        target_fps: Some(60),
        min_size: Size {
          width: 40,
          height: 12,
        },
        save_game_enabled: true,
        save_best_enabled: false,
      },
      None,
    );
    let first = service.session_token().unwrap();
    assert!(service.has_objects());
    service.stop();
    assert!(service.session_token().is_none());
    assert!(!service.has_objects());

    service.start(
      test_session(),
      test_package_id(),
      GameStartOptions {
        target_fps: Some(60),
        min_size: Size {
          width: 40,
          height: 12,
        },
        save_game_enabled: true,
        save_best_enabled: false,
      },
      None,
    );
    let second = service.session_token().unwrap();
    assert_eq!(first.kind, LuaSessionKind::Game);
    assert_eq!(second.kind, LuaSessionKind::Game);
    assert_ne!(first.generation, second.generation);
  }
}
