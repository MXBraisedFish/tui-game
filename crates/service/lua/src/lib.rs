//! Isolated Lua sessions, strict host API bindings, callback budgets, and owned event routing.
//!
//! # Examples
//!
//! ```rust
//! use tg_service_lua::{LuaBudgetKind, LuaPolicy};
//!
//! let policy = LuaPolicy::balanced();
//! assert!(policy.memory_limit_bytes > 0);
//! let render_budget = policy.budget(LuaBudgetKind::Render);
//! assert!(render_budget.warn_duration < render_budget.hard_duration);
//! ```

mod api;
mod events;
mod game;
mod input;
mod object_pool;
pub(crate) use tg_core_sandbox_path as path;
mod policy;
mod screensaver;
mod session;

pub use api::{LuaApiConfig, LuaApiContext, LuaDrawCommand, LuaDrawTarget, LuaHostCommand};
pub use events::{
  LuaActionState, LuaAnimationEvent, LuaAnimationEventKind, LuaAudioEvent, LuaAudioEventKind,
  LuaEnqueueError, LuaEventBroker, LuaEventCallbackId, LuaEventData, LuaEventDelivery,
  LuaEventError, LuaEventErrorCode, LuaEventRoute, LuaFileEntry, LuaFileEvent, LuaFileOperation,
  LuaFileOutcome, LuaHitAreaEvent, LuaHyperlinkEvent, LuaI18nEvent, LuaI18nEventKind,
  LuaImageEvent, LuaImageOutcome, LuaMarkdownEvent, LuaNetworkBody, LuaNetworkEvent,
  LuaNetworkOutcome, LuaRoutableEvent, LuaRuntimeEvent, LuaScrollBoxEvent, LuaSessionToken,
  LuaTaskOperation, LuaTextInputEvent, LuaTimerEvent, LuaTimerEventKind, LuaTimerKind,
  MAX_LUA_EVENTS_PER_FRAME, MAX_LUA_FILE_TASKS_PER_SESSION, MAX_LUA_IMAGE_TASKS_PER_SESSION,
  MAX_LUA_NETWORK_TASKS_PER_SESSION, MAX_LUA_PENDING_EVENTS,
};
pub use game::{GameService, GameStartOptions, LuaSessionDiagnostics};
pub use object_pool::LuaObjectPool;
pub use policy::{LuaBudgetKind, LuaExecutionBudget, LuaPolicy};
pub use screensaver::ScreensaverService;
pub use session::{
  LuaCallbackLifetime, LuaErrorStage, LuaExecutionLimitKind, LuaExecutionStats, LuaSession,
  LuaSessionError, LuaSessionKind, LuaSessionSpec, LuaSessionState,
};

/// A stateless factory creating independent Lua VMs under an execution policy.
pub struct LuaService {
  policy: LuaPolicy,
}

impl LuaService {
  /// Create a Lua service with its initial state.
  pub fn new() -> Self {
    Self {
      policy: LuaPolicy::default(),
    }
  }

  /// Create a stateless Lua session factory using the supplied execution policy.
  pub fn with_policy(policy: LuaPolicy) -> Self {
    Self { policy }
  }

  /// Return the current policy.
  pub fn policy(&self) -> &LuaPolicy {
    &self.policy
  }

  /// Load a fresh isolated Lua VM using the factory's execution policy.
  ///
  /// # Errors
  ///
  /// Propagate package loading, policy validation, Lua initialization, or initial callback
  /// errors.
  pub fn create_session(&self, spec: LuaSessionSpec) -> Result<LuaSession, LuaSessionError> {
    LuaSession::load(spec, self.policy.clone())
  }

  /// Load a fresh isolated Lua VM with the supplied host API context and execution policy.
  ///
  /// # Errors
  ///
  /// Propagate package loading, policy validation, host API construction, or initial callback
  /// errors.
  pub fn create_session_with_api(
    &self,
    spec: LuaSessionSpec,
    api: LuaApiConfig,
  ) -> Result<LuaSession, LuaSessionError> {
    LuaSession::load_with_api(spec, self.policy.clone(), api)
  }
}

impl Default for LuaService {
  fn default() -> Self {
    Self::new()
  }
}
