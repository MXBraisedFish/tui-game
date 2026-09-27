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

#[derive(Clone, Debug)]
pub struct LuaSessionDiagnostics {
  pub entry_path: std::path::PathBuf,
  pub stats: LuaExecutionStats,
  pub memory_bytes: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameSessionState {
  Inactive,
  Running,
  Faulted,
}

#[derive(Clone, Copy, Debug)]
pub struct GameStartOptions {
  pub target_fps: u32,
  pub min_size: Size,
  pub save_game_enabled: bool,
  pub save_best_enabled: bool,
}

/// 唯一游戏 Session 的宿主生命周期。
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
    self.target_fps = Some(options.target_fps);
    self.min_size = options.min_size;
    self.save_game_enabled = options.save_game_enabled;
    self.save_best_enabled = options.save_best_enabled;
    self.accumulator = Duration::ZERO;
    self.session = Some(session);
    self.log_session = log_session;
    previous_log
  }

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

  pub fn log_session(&self) -> Option<LogSessionId> {
    self.log_session
  }

  pub fn state(&self) -> GameSessionState {
    match self.session.as_ref().map(LuaSession::state) {
      Some(LuaSessionState::Running) => GameSessionState::Running,
      Some(LuaSessionState::Faulted) => GameSessionState::Faulted,
      _ => GameSessionState::Inactive,
    }
  }

  pub fn is_active(&self) -> bool {
    self.session.is_some()
  }

  pub fn package_id(&self) -> Option<&str> {
    self.package.as_ref().map(|package| package.mod_id.as_str())
  }

  pub fn package(&self) -> Option<&PackageId> {
    self.package.as_ref()
  }

  pub fn session_token(&self) -> Option<LuaSessionToken> {
    self.session.as_ref().map(|_| LuaSessionToken {
      kind: LuaSessionKind::Game,
      generation: self.generation,
    })
  }

  pub fn diagnostics(&self) -> Option<LuaSessionDiagnostics> {
    self.session.as_ref().map(|session| LuaSessionDiagnostics {
      entry_path: session.entry_path().to_path_buf(),
      stats: session.last_stats(),
      memory_bytes: session.memory_used(),
    })
  }

  pub fn set_base_size(&mut self, size: Size) {
    if let Some(session) = self.session.as_mut() {
      session.set_base_size(size);
    }
  }

  pub fn package_source(&self) -> Option<&PackageSource> {
    self.package.as_ref().map(|package| &package.source)
  }

  pub fn has_objects(&self) -> bool {
    self.session.as_ref().is_some_and(LuaSession::has_objects)
  }

  pub fn with_objects<R>(&self, operation: impl FnOnce(&LuaObjectPool) -> R) -> Option<R> {
    self.session.as_ref()?.with_objects(operation)
  }

  pub fn with_objects_mut<R>(&self, operation: impl FnOnce(&mut LuaObjectPool) -> R) -> Option<R> {
    self.session.as_ref()?.with_objects_mut(operation)
  }

  pub fn target_fps(&self) -> Option<u32> {
    self.target_fps
  }

  pub fn min_size(&self) -> Size {
    self.min_size
  }

  pub fn dispatch_event(&mut self, delivery: &LuaEventDelivery) -> Result<(), LuaSessionError> {
    let Some(session) = self.session.as_mut() else {
      return Ok(());
    };
    session.dispatch_event(delivery)
  }

  /// 执行固定 60 Hz 更新和一次帧更新，返回本帧固定更新次数。
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

  pub fn render(&mut self) -> Result<(), LuaSessionError> {
    let Some(session) = self.session.as_mut() else {
      return Ok(());
    };
    session.render()
  }

  pub fn take_host_commands(&mut self) -> Vec<LuaHostCommand> {
    self
      .session
      .as_mut()
      .map(LuaSession::take_host_commands)
      .unwrap_or_default()
  }

  pub fn take_draw_commands(&mut self) -> Vec<LuaDrawCommand> {
    self
      .session
      .as_mut()
      .map(LuaSession::take_draw_commands)
      .unwrap_or_default()
  }

  pub fn save_game(&mut self) -> Result<Option<JsonValue>, LuaSessionError> {
    if !self.save_game_enabled {
      return Ok(None);
    }
    self
      .session
      .as_mut()
      .map_or(Ok(None), LuaSession::save_game)
  }

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
    PackageId::new(PackageSource::Mod, PackageType::Game, "test.game").unwrap()
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
        package_id: "test.game.service".to_string(),
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
  fn fixed_update_clamps_delta_and_catches_up_at_most_eight_times() {
    let mut service = GameService::new();
    service.start(
      test_session(),
      test_package_id(),
      GameStartOptions {
        target_fps: 120,
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
        target_fps: 60,
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
        target_fps: 60,
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
