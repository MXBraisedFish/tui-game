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

/// 唯一屏保 Session 的宿主生命周期。全终端显示层仍由 Runtime Overlay 管理。
pub struct ScreensaverService {
  session: Option<LuaSession>,
  package: Option<PackageId>,
  accumulator: Duration,
  generation: u64,
  log_session: Option<LogSessionId>,
}

impl ScreensaverService {
  pub fn new() -> Self {
    Self {
      session: None,
      package: None,
      accumulator: Duration::ZERO,
      generation: 0,
      log_session: None,
    }
  }

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

  pub fn stop(&mut self) -> Option<LogSessionId> {
    if let Some(mut session) = self.session.take() {
      session.stop();
    }
    self.package = None;
    self.accumulator = Duration::ZERO;
    self.log_session.take()
  }

  pub fn log_session(&self) -> Option<LogSessionId> {
    self.log_session
  }

  pub fn is_active(&self) -> bool {
    self.session.is_some()
  }

  pub fn is_faulted(&self) -> bool {
    self
      .session
      .as_ref()
      .is_some_and(|session| session.state() == LuaSessionState::Faulted)
  }

  pub fn package_id(&self) -> Option<&str> {
    self.package.as_ref().map(|package| package.mod_id.as_str())
  }

  pub fn package(&self) -> Option<&PackageId> {
    self.package.as_ref()
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

  pub fn session_token(&self) -> Option<LuaSessionToken> {
    self.session.as_ref().map(|_| LuaSessionToken {
      kind: LuaSessionKind::Screensaver,
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

  pub fn dispatch_event(&mut self, delivery: &LuaEventDelivery) -> Result<(), LuaSessionError> {
    let Some(session) = self.session.as_mut() else {
      return Ok(());
    };
    session.dispatch_event(delivery)
  }

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
  use tg_core_package_id::{PackageSource, PackageType};
  use crate::{LuaPolicy, LuaSessionSpec};

  #[test]
  fn checked_in_screensaver_runs_and_releases_its_object_pool() {
    let entry_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..")
      .join("test_package/screensaver/layer_waves/scripts/main.lua");
    let session = LuaSession::load(
      LuaSessionSpec {
        package_id: "test.layer_waves".to_string(),
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
        "test.layer_waves",
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
