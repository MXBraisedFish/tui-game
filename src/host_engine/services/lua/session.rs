use std::cell::Cell;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::{Duration, Instant};

use mlua::chunk::ChunkMode;
use mlua::{
  Function, HookTriggers, IntoLuaMulti, Lua, LuaOptions, MultiValue, RegistryKey, StdLib, Table,
  Value, VmState,
};
use serde_json::{Map as JsonMap, Number as JsonNumber, Value as JsonValue};

use crate::host_engine::services::{HOST_API_VERSION, Size};

use super::api::{
  self, LuaApiConfig, LuaApiContext, LuaCallPhase, LuaDrawCommand, LuaHostCommand, SharedApiState,
};
use super::events::{LuaEventCallbackId, LuaEventDelivery, LuaEventRoute, LuaRuntimeEvent};
use super::object_pool::{SharedLuaObjectPool, shared_lua_object_pool};
use super::policy::{LuaBudgetKind, LuaExecutionBudget, LuaPolicy};

const REQUIRED_CALLBACKS: &[&str] = &["Init", "HandleEvent", "Update", "UpdateFrame", "Render"];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LuaSessionKind {
  Game,
  Screensaver,
}

impl LuaSessionKind {
  pub fn as_str(self) -> &'static str {
    match self {
      Self::Game => "game",
      Self::Screensaver => "screensaver",
    }
  }
}

impl fmt::Display for LuaSessionKind {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(self.as_str())
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LuaSessionState {
  Loading,
  Running,
  Faulted,
  Stopped,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LuaErrorStage {
  ValidatePolicy,
  ReadSource,
  CreateVm,
  BuildSandbox,
  ExecuteEntry,
  DiscoverCallbacks,
  Callback,
  ExecutionLimit,
  MemoryLimit,
  ContinueDataValidation,
  BestDataValidation,
  SaveValidation,
  EventCallback,
  EventQueue,
}

#[derive(Clone, Debug)]
pub struct LuaSessionSpec {
  pub package_id: String,
  pub session_kind: LuaSessionKind,
  pub entry_path: PathBuf,
  pub fixed_delta: Duration,
  pub base_size: Size,
  pub continue_data: Option<JsonValue>,
  pub best_data: Option<JsonValue>,
  pub save_game_enabled: bool,
  pub save_best_enabled: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LuaSessionError {
  pub package_id: String,
  pub session_kind: LuaSessionKind,
  pub stage: LuaErrorStage,
  pub callback: Option<&'static str>,
  pub message: String,
}

impl fmt::Display for LuaSessionError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "Lua {} session '{}' failed during {:?}",
      self.session_kind, self.package_id, self.stage
    )?;
    if let Some(callback) = self.callback {
      write!(f, " ({callback})")?;
    }
    write!(f, ": {}", self.message)
  }
}

impl std::error::Error for LuaSessionError {}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LuaExecutionStats {
  pub instructions: u64,
  pub elapsed: Duration,
  pub memory_bytes: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LuaExecutionLimitKind {
  Time,
  Instructions,
}

impl LuaExecutionLimitKind {
  fn as_str(self) -> &'static str {
    match self {
      Self::Time => "time",
      Self::Instructions => "instructions",
    }
  }
}

#[derive(Clone, Copy, Debug)]
struct SlowCallbackWarning {
  last_logged: Instant,
  suppressed: u64,
}

struct LuaCallbacks {
  init: RegistryKey,
  handle_event: RegistryKey,
  update: RegistryKey,
  update_frame: RegistryKey,
  render: RegistryKey,
  save_game: Option<RegistryKey>,
  save_best: Option<RegistryKey>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LuaCallbackLifetime {
  Once,
  UntilTerminal,
}

struct LuaRegisteredCallback {
  key: RegistryKey,
  lifetime: LuaCallbackLifetime,
}

pub struct LuaSession {
  callbacks: LuaCallbacks,
  environment: RegistryKey,
  lua: Lua,
  policy: LuaPolicy,
  package_id: String,
  session_kind: LuaSessionKind,
  entry_path: PathBuf,
  fixed_delta: Duration,
  base_size: Size,
  state: LuaSessionState,
  last_stats: LuaExecutionStats,
  objects: SharedLuaObjectPool,
  event_callbacks: HashMap<LuaEventCallbackId, LuaRegisteredCallback>,
  next_event_callback_id: u64,
  api_state: SharedApiState,
  slow_callback_warnings: HashMap<&'static str, SlowCallbackWarning>,
}

impl LuaSession {
  pub(super) fn load(spec: LuaSessionSpec, policy: LuaPolicy) -> Result<Self, LuaSessionError> {
    Self::load_with_api(spec, policy, LuaApiConfig::default())
  }

  pub(super) fn load_with_api(
    spec: LuaSessionSpec,
    policy: LuaPolicy,
    api_config: LuaApiConfig,
  ) -> Result<Self, LuaSessionError> {
    policy
      .validate()
      .map_err(|error| session_error(&spec, LuaErrorStage::ValidatePolicy, None, error))?;
    validate_continue_data(&spec, &policy)?;
    validate_best_data(&spec, &policy)?;
    let source = read_source(&spec, &policy)?;
    let lua = Lua::new_with(StdLib::NONE, LuaOptions::default())
      .map_err(|error| session_error(&spec, LuaErrorStage::CreateVm, None, error))?;
    lua
      .set_memory_limit(policy.memory_limit_bytes)
      .map_err(|error| session_error(&spec, LuaErrorStage::MemoryLimit, None, error))?;

    let scripts_root = spec
      .entry_path
      .ancestors()
      .find(|path| {
        path
          .file_name()
          .is_some_and(|name| name.eq_ignore_ascii_case("scripts"))
      })
      .unwrap_or_else(|| spec.entry_path.parent().unwrap_or(Path::new(".")))
      .to_path_buf();
    let assets_root = scripts_root
      .parent()
      .unwrap_or_else(|| Path::new("."))
      .join("assets");
    let objects = shared_lua_object_pool();
    let (environment, api_state) = api::build_environment(
      &lua,
      LuaApiContext {
        package_id: spec.package_id.clone(),
        session_kind: spec.session_kind,
        scripts_root,
        assets_root,
        debug_enabled: api_config.debug_enabled,
        safe_mode_enabled: spec.session_kind == LuaSessionKind::Screensaver
          || api_config.safe_mode_enabled,
        base_size: spec.base_size,
        key_actions: api_config.key_actions,
        key_default_actions: api_config.key_default_actions,
        language_code: api_config.language_code,
        missing_i18n_template: api_config.missing_i18n_template,
      },
      Rc::downgrade(&objects),
    )
    .map_err(|error| session_error(&spec, LuaErrorStage::BuildSandbox, None, error))?;
    let entry_function = lua
      .load(source)
      .set_name(spec.entry_path.to_string_lossy())
      .set_mode(ChunkMode::Text)
      .set_environment(environment.clone())
      .into_function()
      .map_err(|error| session_error(&spec, LuaErrorStage::ExecuteEntry, None, error))?;
    let (_, load_stats) = run_with_budget(
      &lua,
      entry_function,
      (),
      policy.budget(LuaBudgetKind::Load),
      policy.hook_interval,
      &api_state,
    )
    .map_err(|failure| execution_error(&spec, LuaErrorStage::ExecuteEntry, None, failure))?;

    let callbacks = discover_callbacks(&lua, &environment, &spec)?;
    let environment_key = lua
      .create_registry_value(environment)
      .map_err(|error| session_error(&spec, LuaErrorStage::BuildSandbox, None, error))?;

    let mut session = Self {
      callbacks,
      environment: environment_key,
      lua,
      policy,
      package_id: spec.package_id,
      session_kind: spec.session_kind,
      entry_path: spec.entry_path,
      fixed_delta: spec.fixed_delta,
      base_size: spec.base_size,
      state: LuaSessionState::Loading,
      last_stats: LuaExecutionStats::default(),
      objects,
      event_callbacks: HashMap::new(),
      next_event_callback_id: 1,
      api_state,
      slow_callback_warnings: HashMap::new(),
    };
    let load_budget = session.policy.budget(LuaBudgetKind::Load);
    session.record_slow_callback("Load", load_budget, load_stats);
    let context = session.context_table(spec.continue_data.as_ref(), spec.best_data.as_ref())?;
    session.invoke_required(
      Callback::Init,
      context,
      LuaBudgetKind::Init,
      LuaErrorStage::Callback,
    )?;
    session.state = LuaSessionState::Running;
    Ok(session)
  }

  pub fn package_id(&self) -> &str {
    &self.package_id
  }

  pub fn session_kind(&self) -> LuaSessionKind {
    self.session_kind
  }

  pub fn state(&self) -> LuaSessionState {
    self.state
  }

  pub fn entry_path(&self) -> &std::path::Path {
    &self.entry_path
  }

  pub fn base_size(&self) -> Size {
    self.base_size
  }

  pub fn set_base_size(&mut self, size: Size) {
    self.base_size = size;
    self.api_state.borrow_mut().context.base_size = size;
  }

  pub fn configure_api(
    &mut self,
    debug_enabled: bool,
    safe_mode_enabled: bool,
    key_actions: HashMap<String, Vec<Vec<String>>>,
    key_default_actions: HashMap<String, Vec<Vec<String>>>,
  ) {
    let mut state = self.api_state.borrow_mut();
    state.context.debug_enabled = debug_enabled;
    state.context.safe_mode_enabled =
      self.session_kind == LuaSessionKind::Screensaver || safe_mode_enabled;
    state.context.key_actions = key_actions;
    state.context.key_default_actions = key_default_actions;
  }

  pub fn last_stats(&self) -> LuaExecutionStats {
    self.last_stats
  }

  pub fn memory_used(&self) -> usize {
    self.lua.used_memory()
  }

  pub fn has_objects(&self) -> bool {
    self.objects.borrow().is_some()
  }

  pub fn with_objects<R>(&self, operation: impl FnOnce(&super::LuaObjectPool) -> R) -> Option<R> {
    let objects = self.objects.borrow();
    Some(operation(objects.as_ref()?))
  }

  pub fn with_objects_mut<R>(
    &self,
    operation: impl FnOnce(&mut super::LuaObjectPool) -> R,
  ) -> Option<R> {
    let mut objects = self.objects.borrow_mut();
    Some(operation(objects.as_mut()?))
  }

  pub fn handle_event(&mut self, event: &LuaRuntimeEvent) -> Result<(), LuaSessionError> {
    let event = match self.event_table(event, LuaErrorStage::Callback, "HandleEvent") {
      Ok(event) => event,
      Err(error) => {
        self.mark_faulted();
        return Err(error);
      }
    };
    self.invoke_hot(Callback::HandleEvent, event, LuaBudgetKind::HandleEvent)
  }

  pub fn dispatch_event(&mut self, delivery: &LuaEventDelivery) -> Result<(), LuaSessionError> {
    if let super::LuaEventData::I18n(event) = &delivery.event.data {
      api::apply_i18n_event(&self.api_state, event);
    }
    match delivery.route {
      LuaEventRoute::HandleEvent => self.handle_event(&delivery.event),
      LuaEventRoute::Callback(callback) => self.invoke_event_callback(callback, &delivery.event),
    }
  }

  pub(crate) fn register_event_callback(
    &mut self,
    function: Function,
    lifetime: LuaCallbackLifetime,
  ) -> Result<LuaEventCallbackId, LuaSessionError> {
    let id = LuaEventCallbackId(self.next_event_callback_id);
    self.next_event_callback_id = self.next_event_callback_id.saturating_add(1);
    let key = self
      .lua
      .create_registry_value(function)
      .map_err(|error| self.error(LuaErrorStage::EventCallback, Some("EventCallback"), error))?;
    self
      .event_callbacks
      .insert(id, LuaRegisteredCallback { key, lifetime });
    Ok(id)
  }

  pub(crate) fn unregister_event_callback(&mut self, id: LuaEventCallbackId) -> bool {
    let Some(callback) = self.event_callbacks.remove(&id) else {
      return false;
    };
    self.lua.remove_registry_value(callback.key).is_ok()
  }

  pub fn update(&mut self) -> Result<(), LuaSessionError> {
    self.invoke_hot(
      Callback::Update,
      self.fixed_delta.as_secs_f64(),
      LuaBudgetKind::Update,
    )
  }

  pub fn update_frame(&mut self, real_delta: Duration, alpha: f64) -> Result<(), LuaSessionError> {
    self.invoke_hot(
      Callback::UpdateFrame,
      (real_delta.as_secs_f64(), alpha.clamp(0.0, 1.0)),
      LuaBudgetKind::UpdateFrame,
    )
  }

  pub fn render(&mut self) -> Result<(), LuaSessionError> {
    self.invoke_hot(Callback::Render, (), LuaBudgetKind::Render)
  }

  pub fn take_host_commands(&mut self) -> Vec<LuaHostCommand> {
    let mut state = self.api_state.borrow_mut();
    let mut host = Vec::new();
    state.commands.retain(|command| {
      if matches!(command, LuaHostCommand::Draw(_)) {
        true
      } else {
        host.push(command.clone());
        false
      }
    });
    host
  }

  pub fn take_draw_commands(&mut self) -> Vec<LuaDrawCommand> {
    let mut state = self.api_state.borrow_mut();
    let mut draw = Vec::new();
    state.commands.retain(|command| {
      if let LuaHostCommand::Draw(command) = command {
        draw.push(command.clone());
        false
      } else {
        true
      }
    });
    state.draw_command_count = 0;
    state.draw_text_bytes = 0;
    draw
  }

  pub fn save_game(&mut self) -> Result<Option<JsonValue>, LuaSessionError> {
    self.invoke_save(Callback::SaveGame)
  }

  pub fn save_best(&mut self) -> Result<Option<JsonValue>, LuaSessionError> {
    self.invoke_save(Callback::SaveBest)
  }

  pub fn stop(&mut self) {
    if self.state == LuaSessionState::Stopped {
      return;
    }
    self.state = LuaSessionState::Stopped;
    for (_, callback) in self.event_callbacks.drain() {
      let _ = self.lua.remove_registry_value(callback.key);
    }
    self.objects.borrow_mut().take();
    let _ = self.lua.gc_collect();
  }

  fn context_table(
    &self,
    continue_data: Option<&JsonValue>,
    best_data: Option<&JsonValue>,
  ) -> Result<Table, LuaSessionError> {
    let context = self
      .lua
      .create_table()
      .map_err(|error| self.error(LuaErrorStage::Callback, Some("Init"), error))?;
    let base = self
      .lua
      .create_table()
      .map_err(|error| self.error(LuaErrorStage::Callback, Some("Init"), error))?;
    base
      .set("width", self.base_size.width)
      .and_then(|_| base.set("height", self.base_size.height))
      .map_err(|error| self.error(LuaErrorStage::Callback, Some("Init"), error))?;

    context
      .set("package_id", self.package_id.as_str())
      .and_then(|_| context.set("package_type", self.session_kind.as_str()))
      .and_then(|_| context.set("base", base))
      .and_then(|_| {
        context.set(
          "start_mode",
          if continue_data.is_some() {
            "continue"
          } else {
            "new"
          },
        )
      })
      .and_then(|_| context.set("api_version", HOST_API_VERSION))
      .map_err(|error| self.error(LuaErrorStage::Callback, Some("Init"), error))?;
    if let Some(value) = best_data {
      let value = json_to_lua(&self.lua, value)
        .map_err(|error| self.error(LuaErrorStage::Callback, Some("Init"), error))?;
      context
        .set("best_data", value)
        .map_err(|error| self.error(LuaErrorStage::Callback, Some("Init"), error))?;
    }
    if let Some(value) = continue_data {
      let value = json_to_lua(&self.lua, value)
        .map_err(|error| self.error(LuaErrorStage::Callback, Some("Init"), error))?;
      context
        .set("continue_data", value)
        .map_err(|error| self.error(LuaErrorStage::Callback, Some("Init"), error))?;
    }
    Ok(context)
  }

  fn event_table(
    &self,
    event: &LuaRuntimeEvent,
    stage: LuaErrorStage,
    callback: &'static str,
  ) -> Result<Table, LuaSessionError> {
    let table = self
      .lua
      .create_table()
      .map_err(|error| self.error(stage, Some(callback), error))?;
    let data = event
      .data
      .to_lua_table(&self.lua)
      .map_err(|error| self.error(stage, Some(callback), error))?;

    table
      .set("type", event.data.event_type())
      .and_then(|_| table.set("sequence", event.sequence))
      .and_then(|_| table.set("frame", event.frame))
      .and_then(|_| table.set("data", data))
      .map_err(|error| self.error(stage, Some(callback), error))?;
    Ok(table)
  }

  fn invoke_event_callback(
    &mut self,
    callback_id: LuaEventCallbackId,
    event: &LuaRuntimeEvent,
  ) -> Result<(), LuaSessionError> {
    if self.state != LuaSessionState::Running {
      return Err(self.error(
        LuaErrorStage::EventCallback,
        Some("EventCallback"),
        format!("session is {:?}", self.state),
      ));
    }
    let Some(registered) = self.event_callbacks.get(&callback_id) else {
      return Ok(());
    };
    let remove_after =
      registered.lifetime == LuaCallbackLifetime::Once || event.data.callback_is_terminal();
    let function = self
      .lua
      .registry_value::<Function>(&registered.key)
      .map_err(|error| self.error(LuaErrorStage::EventCallback, Some("EventCallback"), error))?;
    let event_table = match self.event_table(event, LuaErrorStage::EventCallback, "EventCallback") {
      Ok(event) => event,
      Err(error) => {
        self.mark_faulted();
        return Err(error);
      }
    };
    let budget = self.policy.budget(LuaBudgetKind::HandleEvent);
    let outcome = run_with_budget(
      &self.lua,
      function,
      event_table,
      budget,
      self.policy.hook_interval,
      &self.api_state,
    );
    if remove_after {
      self.unregister_event_callback(callback_id);
    }
    match outcome {
      Ok((_, stats)) => {
        self.last_stats = LuaExecutionStats {
          memory_bytes: self.lua.used_memory(),
          ..stats
        };
        self.record_slow_callback("EventCallback", budget, stats);
        if let Err(error) = self.lua.gc_step() {
          self.mark_faulted();
          return Err(self.error(LuaErrorStage::EventCallback, Some("EventCallback"), error));
        }
        Ok(())
      }
      Err(failure) => {
        self.mark_faulted();
        Err(self.execution_error(LuaErrorStage::EventCallback, "EventCallback", failure))
      }
    }
  }

  fn invoke_hot<A>(
    &mut self,
    callback: Callback,
    args: A,
    budget_kind: LuaBudgetKind,
  ) -> Result<(), LuaSessionError>
  where
    A: IntoLuaMulti,
  {
    if self.state != LuaSessionState::Running {
      return Err(self.error(
        LuaErrorStage::Callback,
        Some(callback.name()),
        format!("session is {:?}", self.state),
      ));
    }
    let result = self.invoke_required(callback, args, budget_kind, LuaErrorStage::Callback);
    if result.is_err() {
      self.mark_faulted();
    }
    result
  }

  fn mark_faulted(&mut self) {
    self.state = LuaSessionState::Faulted;
    self.objects.take();
  }

  fn invoke_required<A>(
    &mut self,
    callback: Callback,
    args: A,
    budget_kind: LuaBudgetKind,
    stage: LuaErrorStage,
  ) -> Result<(), LuaSessionError>
  where
    A: IntoLuaMulti,
  {
    let function = self
      .lua
      .registry_value::<Function>(self.callback_key(callback).expect("required callback"))
      .map_err(|error| self.error(stage, Some(callback.name()), error))?;
    {
      let mut api = self.api_state.borrow_mut();
      api.phase = callback.phase();
    }
    let budget = self.policy.budget(budget_kind);
    let outcome = run_with_budget(
      &self.lua,
      function,
      args,
      budget,
      self.policy.hook_interval,
      &self.api_state,
    );
    self.api_state.borrow_mut().phase = LuaCallPhase::Idle;
    match outcome {
      Ok((_, stats)) => {
        self.last_stats = LuaExecutionStats {
          memory_bytes: self.lua.used_memory(),
          ..stats
        };
        self.record_slow_callback(callback.name(), budget, stats);
        self
          .lua
          .gc_step()
          .map_err(|error| self.error(stage, Some(callback.name()), error))?;
        Ok(())
      }
      Err(failure) => Err(self.execution_error(stage, callback.name(), failure)),
    }
  }

  fn invoke_save(&mut self, callback: Callback) -> Result<Option<JsonValue>, LuaSessionError> {
    if self.session_kind != LuaSessionKind::Game {
      return Ok(None);
    }
    if self.state != LuaSessionState::Running {
      return Err(self.error(
        LuaErrorStage::Callback,
        Some(callback.name()),
        format!("session is {:?}", self.state),
      ));
    }
    let result = self.invoke_save_inner(callback);
    if result.is_err() {
      self.mark_faulted();
    }
    result
  }

  fn invoke_save_inner(
    &mut self,
    callback: Callback,
  ) -> Result<Option<JsonValue>, LuaSessionError> {
    let Some(key) = self.callback_key(callback) else {
      return Ok(None);
    };
    let function = self
      .lua
      .registry_value::<Function>(key)
      .map_err(|error| self.error(LuaErrorStage::Callback, Some(callback.name()), error))?;
    self.api_state.borrow_mut().phase = callback.phase();
    let budget = self.policy.budget(LuaBudgetKind::Save);
    let outcome = run_with_budget(
      &self.lua,
      function,
      (),
      budget,
      self.policy.hook_interval,
      &self.api_state,
    );
    self.api_state.borrow_mut().phase = LuaCallPhase::Idle;
    let (values, stats) = outcome
      .map_err(|failure| self.execution_error(LuaErrorStage::Callback, callback.name(), failure))?;
    self.last_stats = LuaExecutionStats {
      memory_bytes: self.lua.used_memory(),
      ..stats
    };
    self.record_slow_callback(callback.name(), budget, stats);
    self
      .lua
      .gc_step()
      .map_err(|error| self.error(LuaErrorStage::Callback, Some(callback.name()), error))?;

    let value = values.into_iter().next().unwrap_or(Value::Nil);
    if matches!(value, Value::Nil) {
      return Err(self.error(
        LuaErrorStage::SaveValidation,
        Some(callback.name()),
        "callback must return a serializable value",
      ));
    }
    if callback == Callback::SaveBest && !matches!(value, Value::Table(_)) {
      return Err(self.error(
        LuaErrorStage::SaveValidation,
        Some(callback.name()),
        "callback must return a serializable table containing string field 'best_string'",
      ));
    }
    let mut seen = HashSet::new();
    let json = lua_to_json(value, 0, self.policy.save_max_depth, &mut seen).map_err(|message| {
      self.error(
        LuaErrorStage::SaveValidation,
        Some(callback.name()),
        message,
      )
    })?;
    let encoded = serde_json::to_vec(&json)
      .map_err(|error| self.error(LuaErrorStage::SaveValidation, Some(callback.name()), error))?;
    if encoded.len() > self.policy.save_limit_bytes {
      return Err(self.error(
        LuaErrorStage::SaveValidation,
        Some(callback.name()),
        format!(
          "serialized save value is {} bytes; limit is {} bytes",
          encoded.len(),
          self.policy.save_limit_bytes
        ),
      ));
    }
    if callback == Callback::SaveBest {
      let best_string = json
        .as_object()
        .and_then(|value| value.get("best_string"))
        .and_then(JsonValue::as_str);
      if best_string.is_none() {
        return Err(self.error(
          LuaErrorStage::SaveValidation,
          Some(callback.name()),
          "callback must return a table containing string field 'best_string'",
        ));
      }
    }
    Ok(Some(json))
  }

  fn callback_key(&self, callback: Callback) -> Option<&RegistryKey> {
    match callback {
      Callback::Init => Some(&self.callbacks.init),
      Callback::HandleEvent => Some(&self.callbacks.handle_event),
      Callback::Update => Some(&self.callbacks.update),
      Callback::UpdateFrame => Some(&self.callbacks.update_frame),
      Callback::Render => Some(&self.callbacks.render),
      Callback::SaveGame => self.callbacks.save_game.as_ref(),
      Callback::SaveBest => self.callbacks.save_best.as_ref(),
    }
  }

  fn execution_error(
    &self,
    stage: LuaErrorStage,
    callback: &'static str,
    failure: LuaExecutionFailure,
  ) -> LuaSessionError {
    let (actual_stage, message) = match failure {
      LuaExecutionFailure::Lua(error) => {
        let error_stage = if is_memory_error(&error) {
          LuaErrorStage::MemoryLimit
        } else {
          stage
        };
        (error_stage, error.to_string())
      }
      LuaExecutionFailure::Limit {
        kind,
        instructions,
        instruction_limit,
        elapsed,
        duration_limit,
      } => (
        LuaErrorStage::ExecutionLimit,
        format_execution_limit(
          kind,
          instructions,
          instruction_limit,
          elapsed,
          duration_limit,
        ),
      ),
    };
    self.error(actual_stage, Some(callback), message)
  }

  fn record_slow_callback(
    &mut self,
    callback: &'static str,
    budget: LuaExecutionBudget,
    stats: LuaExecutionStats,
  ) {
    self.record_slow_callback_at(callback, budget, stats, Instant::now());
  }

  fn record_slow_callback_at(
    &mut self,
    callback: &'static str,
    budget: LuaExecutionBudget,
    stats: LuaExecutionStats,
    now: Instant,
  ) {
    const LOG_INTERVAL: Duration = Duration::from_secs(5);

    if stats.elapsed <= budget.warn_duration || !self.api_state.borrow().context.debug_enabled {
      return;
    }
    let warning = self
      .slow_callback_warnings
      .entry(callback)
      .or_insert(SlowCallbackWarning {
        last_logged: now.checked_sub(LOG_INTERVAL).unwrap_or(now),
        suppressed: 0,
      });
    if now.duration_since(warning.last_logged) < LOG_INTERVAL {
      warning.suppressed = warning.suppressed.saturating_add(1);
      return;
    }
    let suppressed = warning.suppressed;
    warning.last_logged = now;
    warning.suppressed = 0;
    let message = format!(
      "slow Lua callback: callback={callback}; elapsed_ms={:.3}; warn_ms={:.3}; hard_ms={:.3}; instructions={}; instruction_limit={}; suppressed={suppressed}",
      duration_millis(stats.elapsed),
      duration_millis(budget.warn_duration),
      duration_millis(budget.hard_duration),
      stats.instructions,
      budget.max_instructions,
    );
    self
      .api_state
      .borrow_mut()
      .commands
      .push(LuaHostCommand::Log {
        level: "warn".to_string(),
        message,
      });
  }

  fn error(
    &self,
    stage: LuaErrorStage,
    callback: Option<&'static str>,
    message: impl ToString,
  ) -> LuaSessionError {
    LuaSessionError {
      package_id: self.package_id.clone(),
      session_kind: self.session_kind,
      stage,
      callback,
      message: message.to_string(),
    }
  }

  #[cfg(test)]
  fn environment_value(&self, name: &str) -> Value {
    let environment: Table = self.lua.registry_value(&self.environment).unwrap();
    environment.get(name).unwrap()
  }

  #[cfg(test)]
  fn register_environment_event_callback(
    &mut self,
    name: &str,
    lifetime: LuaCallbackLifetime,
  ) -> LuaEventCallbackId {
    let environment: Table = self.lua.registry_value(&self.environment).unwrap();
    let function: Function = environment.get(name).unwrap();
    self.register_event_callback(function, lifetime).unwrap()
  }
}

impl Drop for LuaSession {
  fn drop(&mut self) {
    self.stop();
  }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Callback {
  Init,
  HandleEvent,
  Update,
  UpdateFrame,
  Render,
  SaveGame,
  SaveBest,
}

impl Callback {
  fn name(self) -> &'static str {
    match self {
      Self::Init => "Init",
      Self::HandleEvent => "HandleEvent",
      Self::Update => "Update",
      Self::UpdateFrame => "UpdateFrame",
      Self::Render => "Render",
      Self::SaveGame => "SaveGame",
      Self::SaveBest => "SaveBest",
    }
  }

  fn phase(self) -> LuaCallPhase {
    match self {
      Self::Init => LuaCallPhase::Init,
      Self::HandleEvent => LuaCallPhase::Event,
      Self::Update => LuaCallPhase::Update,
      Self::UpdateFrame => LuaCallPhase::UpdateFrame,
      Self::Render => LuaCallPhase::Render,
      Self::SaveGame => LuaCallPhase::SaveGame,
      Self::SaveBest => LuaCallPhase::SaveBest,
    }
  }
}

fn read_source(spec: &LuaSessionSpec, policy: &LuaPolicy) -> Result<String, LuaSessionError> {
  if !has_lua_extension(&spec.entry_path) {
    return Err(session_error(
      spec,
      LuaErrorStage::ReadSource,
      None,
      "entry source must use the .lua extension",
    ));
  }

  let file = File::open(&spec.entry_path)
    .map_err(|error| session_error(spec, LuaErrorStage::ReadSource, None, error))?;
  let metadata = file
    .metadata()
    .map_err(|error| session_error(spec, LuaErrorStage::ReadSource, None, error))?;
  if !metadata.is_file() {
    return Err(session_error(
      spec,
      LuaErrorStage::ReadSource,
      None,
      "entry source is not a regular file",
    ));
  }
  if metadata.len() > policy.source_limit_bytes as u64 {
    return Err(session_error(
      spec,
      LuaErrorStage::ReadSource,
      None,
      format!(
        "entry source is {} bytes; limit is {} bytes",
        metadata.len(),
        policy.source_limit_bytes
      ),
    ));
  }

  let read_limit = policy.source_limit_bytes.saturating_add(1) as u64;
  let mut bytes = Vec::with_capacity(metadata.len() as usize);
  file
    .take(read_limit)
    .read_to_end(&mut bytes)
    .map_err(|error| session_error(spec, LuaErrorStage::ReadSource, None, error))?;
  if bytes.len() > policy.source_limit_bytes {
    return Err(session_error(
      spec,
      LuaErrorStage::ReadSource,
      None,
      format!(
        "entry source exceeds the {} byte limit while being read",
        policy.source_limit_bytes
      ),
    ));
  }
  String::from_utf8(bytes)
    .map_err(|error| session_error(spec, LuaErrorStage::ReadSource, None, error))
}

fn has_lua_extension(path: &Path) -> bool {
  path
    .extension()
    .and_then(|extension| extension.to_str())
    .is_some_and(|extension| extension.eq_ignore_ascii_case("lua"))
}

fn validate_continue_data(
  spec: &LuaSessionSpec,
  policy: &LuaPolicy,
) -> Result<(), LuaSessionError> {
  let Some(value) = spec.continue_data.as_ref() else {
    return Ok(());
  };
  validate_json_depth(value, 0, policy.save_max_depth, "continue data").map_err(|message| {
    session_error(
      spec,
      LuaErrorStage::ContinueDataValidation,
      Some("Init"),
      message,
    )
  })?;
  let encoded = serde_json::to_vec(value).map_err(|error| {
    session_error(
      spec,
      LuaErrorStage::ContinueDataValidation,
      Some("Init"),
      error,
    )
  })?;
  if encoded.len() > policy.save_limit_bytes {
    return Err(session_error(
      spec,
      LuaErrorStage::ContinueDataValidation,
      Some("Init"),
      format!(
        "continue data is {} bytes; limit is {} bytes",
        encoded.len(),
        policy.save_limit_bytes
      ),
    ));
  }
  Ok(())
}

fn validate_best_data(spec: &LuaSessionSpec, policy: &LuaPolicy) -> Result<(), LuaSessionError> {
  let Some(value) = spec.best_data.as_ref() else {
    return Ok(());
  };
  validate_json_depth(value, 0, policy.save_max_depth, "best data").map_err(|message| {
    session_error(
      spec,
      LuaErrorStage::BestDataValidation,
      Some("Init"),
      message,
    )
  })?;
  let encoded = serde_json::to_vec(value)
    .map_err(|error| session_error(spec, LuaErrorStage::BestDataValidation, Some("Init"), error))?;
  if encoded.len() > policy.save_limit_bytes {
    return Err(session_error(
      spec,
      LuaErrorStage::BestDataValidation,
      Some("Init"),
      format!(
        "best data is {} bytes; limit is {} bytes",
        encoded.len(),
        policy.save_limit_bytes
      ),
    ));
  }
  Ok(())
}

fn validate_json_depth(
  value: &JsonValue,
  depth: usize,
  max_depth: usize,
  label: &str,
) -> Result<(), String> {
  if depth > max_depth {
    return Err(format!("{label} exceeds maximum depth {max_depth}"));
  }
  match value {
    JsonValue::Array(values) => {
      for value in values {
        validate_json_depth(value, depth + 1, max_depth, label)?;
      }
    }
    JsonValue::Object(values) => {
      for value in values.values() {
        validate_json_depth(value, depth + 1, max_depth, label)?;
      }
    }
    _ => {}
  }
  Ok(())
}

fn discover_callbacks(
  lua: &Lua,
  environment: &Table,
  spec: &LuaSessionSpec,
) -> Result<LuaCallbacks, LuaSessionError> {
  let required = |name: &'static str| -> Result<RegistryKey, LuaSessionError> {
    let value = environment
      .get::<Value>(name)
      .map_err(|error| session_error(spec, LuaErrorStage::DiscoverCallbacks, Some(name), error))?;
    let Value::Function(function) = value else {
      return Err(session_error(
        spec,
        LuaErrorStage::DiscoverCallbacks,
        Some(name),
        format!("required callback '{name}' is missing or is not a function"),
      ));
    };
    lua
      .create_registry_value(function)
      .map_err(|error| session_error(spec, LuaErrorStage::DiscoverCallbacks, Some(name), error))
  };
  let optional = |name: &'static str| -> Result<Option<RegistryKey>, LuaSessionError> {
    match environment
      .get::<Value>(name)
      .map_err(|error| session_error(spec, LuaErrorStage::DiscoverCallbacks, Some(name), error))?
    {
      Value::Nil => Ok(None),
      Value::Function(function) => lua
        .create_registry_value(function)
        .map(Some)
        .map_err(|error| session_error(spec, LuaErrorStage::DiscoverCallbacks, Some(name), error)),
      _ => Err(session_error(
        spec,
        LuaErrorStage::DiscoverCallbacks,
        Some(name),
        format!("optional callback '{name}' exists but is not a function"),
      )),
    }
  };

  let mut keys = Vec::with_capacity(REQUIRED_CALLBACKS.len());
  for name in REQUIRED_CALLBACKS {
    keys.push(required(name)?);
  }
  let mut keys = keys.into_iter();
  Ok(LuaCallbacks {
    init: keys.next().unwrap(),
    handle_event: keys.next().unwrap(),
    update: keys.next().unwrap(),
    update_frame: keys.next().unwrap(),
    render: keys.next().unwrap(),
    save_game: if spec.session_kind != LuaSessionKind::Game {
      None
    } else if spec.save_game_enabled {
      Some(required("SaveGame")?)
    } else {
      optional("SaveGame")?
    },
    save_best: if spec.session_kind != LuaSessionKind::Game {
      None
    } else if spec.save_best_enabled {
      Some(required("SaveBest")?)
    } else {
      optional("SaveBest")?
    },
  })
}

enum LuaExecutionFailure {
  Lua(mlua::Error),
  Limit {
    kind: LuaExecutionLimitKind,
    instructions: u64,
    instruction_limit: u64,
    elapsed: Duration,
    duration_limit: Duration,
  },
}

fn run_with_budget<A>(
  lua: &Lua,
  function: Function,
  args: A,
  budget: LuaExecutionBudget,
  hook_interval: u32,
  api_state: &SharedApiState,
) -> Result<(MultiValue, LuaExecutionStats), LuaExecutionFailure>
where
  A: IntoLuaMulti,
{
  {
    let mut api = api_state.borrow_mut();
    api.fatal_budget_exceeded = false;
    api.fatal_api_error = false;
  }
  let thread = lua
    .create_thread(function)
    .map_err(LuaExecutionFailure::Lua)?;
  let started = Instant::now();
  let instructions = Rc::new(Cell::new(0_u64));
  let exceeded = Rc::new(Cell::new(None));
  let hook_instructions = Rc::clone(&instructions);
  let hook_exceeded = Rc::clone(&exceeded);
  let hook_api_state = api_state.clone();
  thread
    .set_hook(
      HookTriggers::new().every_nth_instruction(hook_interval),
      move |_, _| {
        let current = hook_instructions
          .get()
          .saturating_add(u64::from(hook_interval));
        hook_instructions.set(current);
        let limit = if current > budget.max_instructions {
          Some(LuaExecutionLimitKind::Instructions)
        } else if started.elapsed() > budget.hard_duration {
          Some(LuaExecutionLimitKind::Time)
        } else {
          None
        };
        if let Some(limit) = limit {
          hook_exceeded.set(Some(limit));
          hook_api_state.borrow_mut().fatal_budget_exceeded = true;
          Err(mlua::Error::RuntimeError(format!(
            "Lua {} execution limit exceeded",
            limit.as_str()
          )))
        } else {
          Ok(VmState::Continue)
        }
      },
    )
    .map_err(LuaExecutionFailure::Lua)?;

  let result = thread.resume::<MultiValue>(args);
  thread.remove_hook();
  let elapsed = started.elapsed();
  let values = match result {
    Err(error) if is_memory_error(&error) => return Err(LuaExecutionFailure::Lua(error)),
    Err(_) if exceeded.get().is_some() || api_state.borrow().fatal_budget_exceeded => {
      return Err(LuaExecutionFailure::Limit {
        kind: exceeded
          .get()
          .unwrap_or(LuaExecutionLimitKind::Instructions),
        instructions: instructions.get(),
        instruction_limit: budget.max_instructions,
        elapsed,
        duration_limit: budget.hard_duration,
      });
    }
    Err(_) if elapsed > budget.hard_duration => {
      return Err(LuaExecutionFailure::Limit {
        kind: LuaExecutionLimitKind::Time,
        instructions: instructions.get(),
        instruction_limit: budget.max_instructions,
        elapsed,
        duration_limit: budget.hard_duration,
      });
    }
    Err(error) => return Err(LuaExecutionFailure::Lua(error)),
    Ok(values) => values,
  };
  if let Some(kind) = exceeded.get() {
    return Err(LuaExecutionFailure::Limit {
      kind,
      instructions: instructions.get(),
      instruction_limit: budget.max_instructions,
      elapsed,
      duration_limit: budget.hard_duration,
    });
  }
  if instructions.get() > budget.max_instructions {
    return Err(LuaExecutionFailure::Limit {
      kind: LuaExecutionLimitKind::Instructions,
      instructions: instructions.get(),
      instruction_limit: budget.max_instructions,
      elapsed,
      duration_limit: budget.hard_duration,
    });
  }
  if elapsed > budget.hard_duration {
    return Err(LuaExecutionFailure::Limit {
      kind: LuaExecutionLimitKind::Time,
      instructions: instructions.get(),
      instruction_limit: budget.max_instructions,
      elapsed,
      duration_limit: budget.hard_duration,
    });
  }
  if !thread.is_finished() {
    return Err(LuaExecutionFailure::Lua(mlua::Error::RuntimeError(
      "Lua callback yielded before completion".to_string(),
    )));
  }
  if api_state.borrow().fatal_api_error {
    return Err(LuaExecutionFailure::Lua(mlua::Error::RuntimeError(
      "fatal Lua API resource limit exceeded".to_string(),
    )));
  }
  Ok((
    values,
    LuaExecutionStats {
      instructions: instructions.get(),
      elapsed,
      memory_bytes: lua.used_memory(),
    },
  ))
}

fn is_memory_error(error: &mlua::Error) -> bool {
  match error {
    mlua::Error::MemoryError(_) => true,
    mlua::Error::BadArgument { cause, .. } | mlua::Error::CallbackError { cause, .. } => {
      is_memory_error(cause)
    }
    _ => false,
  }
}

fn session_error(
  spec: &LuaSessionSpec,
  stage: LuaErrorStage,
  callback: Option<&'static str>,
  message: impl ToString,
) -> LuaSessionError {
  LuaSessionError {
    package_id: spec.package_id.clone(),
    session_kind: spec.session_kind,
    stage,
    callback,
    message: message.to_string(),
  }
}

fn execution_error(
  spec: &LuaSessionSpec,
  stage: LuaErrorStage,
  callback: Option<&'static str>,
  failure: LuaExecutionFailure,
) -> LuaSessionError {
  match failure {
    LuaExecutionFailure::Lua(error) => {
      let stage = if is_memory_error(&error) {
        LuaErrorStage::MemoryLimit
      } else {
        stage
      };
      session_error(spec, stage, callback, error)
    }
    LuaExecutionFailure::Limit {
      kind,
      instructions,
      instruction_limit,
      elapsed,
      duration_limit,
    } => session_error(
      spec,
      LuaErrorStage::ExecutionLimit,
      callback,
      format_execution_limit(
        kind,
        instructions,
        instruction_limit,
        elapsed,
        duration_limit,
      ),
    ),
  }
}

fn duration_millis(duration: Duration) -> f64 {
  duration.as_secs_f64() * 1_000.0
}

fn format_execution_limit(
  kind: LuaExecutionLimitKind,
  instructions: u64,
  instruction_limit: u64,
  elapsed: Duration,
  duration_limit: Duration,
) -> String {
  format!(
    "{} execution limit exceeded: elapsed_ms={:.3}; time_limit_ms={:.3}; instructions={instructions}; instruction_limit={instruction_limit}",
    kind.as_str(),
    duration_millis(elapsed),
    duration_millis(duration_limit),
  )
}

fn json_to_lua(lua: &Lua, value: &JsonValue) -> mlua::Result<Value> {
  match value {
    JsonValue::Null => Ok(Value::Nil),
    JsonValue::Bool(value) => Ok(Value::Boolean(*value)),
    JsonValue::Number(value) => {
      if let Some(integer) = value.as_i64() {
        Ok(Value::Integer(integer))
      } else {
        Ok(Value::Number(value.as_f64().unwrap_or_default()))
      }
    }
    JsonValue::String(value) => lua.create_string(value).map(Value::String),
    JsonValue::Array(values) => {
      let table = lua.create_table_with_capacity(values.len(), 0)?;
      for (index, value) in values.iter().enumerate() {
        table.raw_set(index + 1, json_to_lua(lua, value)?)?;
      }
      Ok(Value::Table(table))
    }
    JsonValue::Object(values) => {
      let table = lua.create_table_with_capacity(0, values.len())?;
      for (key, value) in values {
        table.raw_set(key.as_str(), json_to_lua(lua, value)?)?;
      }
      Ok(Value::Table(table))
    }
  }
}

fn lua_to_json(
  value: Value,
  depth: usize,
  max_depth: usize,
  seen: &mut HashSet<usize>,
) -> Result<JsonValue, String> {
  if depth > max_depth {
    return Err(format!("save value exceeds maximum depth {max_depth}"));
  }
  match value {
    Value::Nil => Ok(JsonValue::Null),
    Value::Boolean(value) => Ok(JsonValue::Bool(value)),
    Value::Integer(value) => Ok(JsonValue::Number(JsonNumber::from(value))),
    Value::Number(value) => {
      if !value.is_finite() {
        return Err("save value contains a non-finite number".to_string());
      }
      JsonNumber::from_f64(value)
        .map(JsonValue::Number)
        .ok_or_else(|| "save value contains an invalid number".to_string())
    }
    Value::String(value) => value
      .to_str()
      .map(|value| JsonValue::String(value.to_string()))
      .map_err(|_| "save value contains a non-UTF-8 string".to_string()),
    Value::Table(table) => {
      let pointer = table.to_pointer() as usize;
      if !seen.insert(pointer) {
        return Err("save value contains a table cycle".to_string());
      }
      let result = table_to_json(table, depth, max_depth, seen);
      seen.remove(&pointer);
      result
    }
    Value::Function(_) | Value::Thread(_) | Value::UserData(_) | Value::LightUserData(_) => {
      Err(format!(
        "save value contains unsupported Lua type '{}'",
        value.type_name()
      ))
    }
    Value::Error(error) => Err(format!("save value contains Lua error: {error}")),
    Value::Other(_) => Err("save value contains an unsupported Lua value".to_string()),
  }
}

fn table_to_json(
  table: Table,
  depth: usize,
  max_depth: usize,
  seen: &mut HashSet<usize>,
) -> Result<JsonValue, String> {
  let mut integer_values = BTreeMap::<i64, Value>::new();
  let mut string_values = BTreeMap::<String, Value>::new();

  for pair in table.pairs::<Value, Value>() {
    let (key, value) = pair.map_err(|error| error.to_string())?;
    match key {
      Value::Integer(index) if index > 0 => {
        integer_values.insert(index, value);
      }
      Value::String(key) => {
        let key = key
          .to_str()
          .map_err(|_| "save object contains a non-UTF-8 key".to_string())?
          .to_string();
        string_values.insert(key, value);
      }
      _ => return Err("save object keys must be strings or sequential integers".to_string()),
    }
  }

  if !integer_values.is_empty() && !string_values.is_empty() {
    return Err("save table cannot mix array and object keys".to_string());
  }
  if !integer_values.is_empty() {
    let expected_len = integer_values.len() as i64;
    if integer_values.keys().copied().ne(1..=expected_len) {
      return Err("save array keys must be contiguous and start at 1".to_string());
    }
    let values = integer_values
      .into_values()
      .map(|value| lua_to_json(value, depth + 1, max_depth, seen))
      .collect::<Result<Vec<_>, _>>()?;
    return Ok(JsonValue::Array(values));
  }

  let mut object = JsonMap::new();
  for (key, value) in string_values {
    object.insert(key, lua_to_json(value, depth + 1, max_depth, seen)?);
  }
  Ok(JsonValue::Object(object))
}

#[cfg(test)]
mod tests {
  use std::fs;
  use std::sync::atomic::{AtomicU64, Ordering};

  use crate::host_engine::services::{LuaFileOperation, SliceId};

  use super::super::LuaEventData;
  use super::*;

  static TEST_ID: AtomicU64 = AtomicU64::new(1);

  fn script_path(source: &str) -> PathBuf {
    let id = TEST_ID.fetch_add(1, Ordering::Relaxed);
    let directory = std::env::temp_dir().join(format!(
      "tui_game_lua_session_{}_{}",
      std::process::id(),
      id
    ));
    fs::create_dir_all(&directory).unwrap();
    let path = directory.join("main.lua");
    fs::write(&path, source).unwrap();
    path
  }

  fn spec(source: &str, kind: LuaSessionKind) -> LuaSessionSpec {
    LuaSessionSpec {
      package_id: "test.package".to_string(),
      session_kind: kind,
      entry_path: script_path(source),
      fixed_delta: Duration::from_secs_f64(1.0 / 60.0),
      base_size: Size {
        width: 120,
        height: 40,
      },
      continue_data: None,
      best_data: None,
      save_game_enabled: false,
      save_best_enabled: false,
    }
  }

  fn valid_script(extra: &str) -> String {
    format!(
      r##"
        function Init(ctx) init_ctx = ctx end
        function HandleEvent(event) last_event = event end
        function Update(dt) last_update = dt end
        function UpdateFrame(dt, alpha) last_frame = {{ dt, alpha }} end
        function Render() render_called = true end
        {extra}
      "##
    )
  }

  #[test]
  fn creates_isolated_game_and_screensaver_vms() {
    let mut first = LuaSession::load(
      spec(
        &valid_script("private_value = 'game'"),
        LuaSessionKind::Game,
      ),
      LuaPolicy::default(),
    )
    .unwrap();
    let second = LuaSession::load(
      spec(
        &valid_script("private_value = 'screensaver'"),
        LuaSessionKind::Screensaver,
      ),
      LuaPolicy::default(),
    )
    .unwrap();

    assert_eq!(
      first.environment_value("private_value"),
      Value::String(first.lua.create_string("game").unwrap())
    );
    assert_eq!(
      second.environment_value("private_value"),
      Value::String(second.lua.create_string("screensaver").unwrap())
    );
    assert_eq!(first.environment_value("_G"), Value::Nil);
    assert_eq!(second.environment_value("_G"), Value::Nil);

    let first_pool_id = first.with_objects(|objects| objects.ui().id()).unwrap();
    let second_pool_id = second.with_objects(|objects| objects.ui().id()).unwrap();
    assert_ne!(first_pool_id, second_pool_id);

    let time = crate::host_engine::services::TimeService::new();
    let timer = first
      .with_objects_mut(|objects| time.create_count_up(objects.runtime_mut()))
      .unwrap();
    assert_eq!(
      first
        .with_objects(|objects| time.state(objects.runtime(), timer))
        .flatten(),
      Some(crate::host_engine::services::TimerState::Idle)
    );
    assert_eq!(
      second
        .with_objects(|objects| time.state(objects.runtime(), timer))
        .flatten(),
      None
    );

    first.stop();
    assert!(!first.has_objects());
  }

  #[test]
  fn i18n_api_loads_asynchronously_and_applies_data_before_handle_event() {
    let source = valid_script(
      r#"
        function Init(ctx)
          system_language = i18n.get_language_code()
          i18n.create{}
          i18n.create{ language_code = "ignored" }
        end
        function HandleEvent(event)
          if event.type == "i18n" and event.data.ok then
            translated = i18n.get_value{ namespace = "menu", key = "title" }
            missing = i18n.get_value{ namespace = "menu", key = "missing" }
          end
        end
      "#,
    );
    let mut session = LuaSession::load_with_api(
      spec(&source, LuaSessionKind::Game),
      LuaPolicy::default(),
      LuaApiConfig {
        language_code: "zh_cn".to_string(),
        missing_i18n_template: "[缺少：{value:missing_key}]".to_string(),
        ..LuaApiConfig::default()
      },
    )
    .unwrap();
    assert_eq!(
      session.environment_value("system_language"),
      Value::String(session.lua.create_string("zh_cn").unwrap())
    );
    let commands = session.take_host_commands();
    assert_eq!(
      commands
        .iter()
        .filter(|command| matches!(command, LuaHostCommand::I18nRequest { .. }))
        .count(),
      1
    );

    let delivery = LuaEventDelivery {
      event: LuaRuntimeEvent {
        sequence: 1,
        frame: 1,
        data: LuaEventData::I18n(super::super::LuaI18nEvent {
          kind: super::super::LuaI18nEventKind::Created,
          ok: true,
          message: "i18n instance created".to_string(),
          language_code: "zh_cn".to_string(),
          callback_language_code: "en_us".to_string(),
          namespaces: Some(HashMap::from([(
            "menu".to_string(),
            HashMap::from([("title".to_string(), "标题".to_string())]),
          )])),
        }),
      },
      route: LuaEventRoute::HandleEvent,
    };
    session.dispatch_event(&delivery).unwrap();
    assert_eq!(
      session.environment_value("translated"),
      Value::String(session.lua.create_string("标题").unwrap())
    );
    assert_eq!(
      session.environment_value("missing"),
      Value::String(session.lua.create_string("[缺少：menu.missing]").unwrap())
    );
  }

  #[test]
  fn sandbox_exposes_only_host_libraries_and_hides_native_globals() {
    let session = LuaSession::load(
      spec(&valid_script(""), LuaSessionKind::Game),
      LuaPolicy::default(),
    )
    .unwrap();
    for name in [
      "base",
      "math",
      "string",
      "utf8",
      "table",
      "align",
      "char",
      "color",
      "measurement",
      "draw",
      "debug",
      "game",
      "event",
      "loader",
      "file",
      "random",
      "slice",
      "serialization",
      "encoding",
      "i18n",
      "ipairs",
      "pairs",
      "next",
      "select",
      "rawequal",
      "rawlen",
      "tonumber",
      "tostring",
      "type",
    ] {
      assert_ne!(session.environment_value(name), Value::Nil, "{name}");
    }
    for name in [
      "_G",
      "_VERSION",
      "assert",
      "error",
      "pcall",
      "xpcall",
      "load",
      "loadfile",
      "loadstring",
      "dofile",
      "require",
      "rawget",
      "rawset",
      "os",
      "io",
      "package",
      "coroutine",
    ] {
      assert_eq!(session.environment_value(name), Value::Nil, "{name}");
    }
  }

  #[test]
  fn api_tables_are_read_only_and_iterators_do_not_expose_backing_tables() {
    let source = valid_script(
      r#"
        function Init(ctx)
          local ok = debug.pcall{ func = function() math.PI = 0 end }
          debug.assert{ value = not ok.ok, message = "math must be read-only" }
          local iterator = pairs(math)
          local item = iterator()
          debug.assert{ value = type{ value = item } == "table" }
          debug.assert{ value = item.index ~= nil and item.value ~= nil }
          local count = 0
          for pair in pairs(math) do
            debug.assert{ value = pair.index ~= nil and pair.value ~= nil }
            count = count + 1
          end
          debug.assert{ value = count > 0 }
          local first = next{ table = math, index = nil }
          debug.assert{ value = first.index ~= nil and first.value ~= nil }
          debug.assert{ value = math.PI > 3, message = "iterator leaked backing table" }
        end
      "#,
    );
    LuaSession::load(spec(&source, LuaSessionKind::Game), LuaPolicy::default()).unwrap();
  }

  #[test]
  fn align_resolve_rect_returns_a_named_coordinate_table() {
    let source = valid_script(
      r#"
        function Init(ctx)
          local top_left, extra = align.resolve_rect{
            width = 10,
            height = 4,
            horizontal_align = align.LEFT,
            vertical_align = align.TOP,
          }
          local center = align.resolve_rect{
            width = 10,
            height = 4,
            horizontal_align = align.CENTER,
            vertical_align = align.CENTER,
          }
          local bottom_right = align.resolve_rect{
            width = 10,
            height = 4,
            horizontal_align = align.RIGHT,
            vertical_align = align.BOTTOM,
          }
          debug.assert{ value = type{ value = top_left } == "table" }
          debug.assert{ value = top_left.x == 0 and top_left.y == 0 }
          debug.assert{ value = center.x == 55 and center.y == 17 }
          debug.assert{ value = bottom_right.x == 110 and bottom_right.y == 34 }
          debug.assert{ value = extra == nil }
        end
      "#,
    );
    let mut session_spec = spec(&source, LuaSessionKind::Game);
    session_spec.base_size = Size {
      width: 120,
      height: 38,
    };
    LuaSession::load(session_spec, LuaPolicy::default()).unwrap();
  }

  #[test]
  fn lua_text_modes_share_the_expected_measurement_semantics() {
    let source = valid_script(
      r#"
        function Init(ctx)
          local text = "f%<fg:green>Test"
          local plain = measurement.get_text_width{
            text = text,
            text_mode = string.PLAIN_TEXT,
          }
          local rich = measurement.get_text_width{
            text = text,
            text_mode = string.RICH_TEXT,
          }
          local auto = measurement.get_text_width{
            text = text,
            text_mode = string.AUTO,
          }
          debug.assert{ value = plain == 16, message = "plain mode must preserve all syntax" }
          debug.assert{ value = rich == 6, message = "rich mode must preserve the f% prefix" }
          debug.assert{ value = auto == 4, message = "auto mode must consume the f% prefix" }
        end
      "#,
    );
    LuaSession::load(spec(&source, LuaSessionKind::Game), LuaPolicy::default()).unwrap();
  }

  #[test]
  fn measurement_api_returns_documented_types_and_uses_draw_text_layout() {
    let source = valid_script(
      r#"
        function Init(ctx)
          local size, extra = measurement.get_text_size{
            text = "Hello\n世界",
          }
          debug.assert{ value = type{ value = size } == "table" }
          debug.assert{ value = size.width == 5 and size.height == 2 }
          debug.assert{ value = extra == nil }
          debug.assert{
            value = measurement.get_text_width{
              text = "abcdef",
              max_width = 3,
              auto_wrap = true,
              word_wrap = false,
            } == 3,
          }
          debug.assert{
            value = measurement.get_text_height{
              text = "abcdef",
              max_width = 3,
              max_height = 1,
              overflow_marker = "..",
              auto_wrap = true,
              word_wrap = false,
            } == 1,
          }
          local rich = measurement.get_text_size{
            text = "f%{value:name}",
            rich_params = { name = "终端" },
            text_mode = string.AUTO,
            horizontal_align = align.RIGHT,
          }
          debug.assert{ value = rich.width == 4 and rich.height == 1 }
          local empty = measurement.get_text_size{ text = "" }
          debug.assert{ value = empty.width == 0 and empty.height == 0 }
        end
      "#,
    );
    LuaSession::load(spec(&source, LuaSessionKind::Game), LuaPolicy::default()).unwrap();
  }

  #[test]
  fn measurement_api_rejects_position_style_target_and_invalid_layout_parameters() {
    let source = valid_script(
      r#"
        function Init(ctx)
          local function fails(func)
            return not debug.pcall{ func = func }.ok
          end

          debug.assert{ value = fails(function()
            measurement.get_text_width{ text = "text", x = 0 }
          end) }
          debug.assert{ value = fails(function()
            measurement.get_text_width{ text = "text", bold = true }
          end) }
          debug.assert{ value = fails(function()
            measurement.get_text_width{ text = "text", fg = color.RED }
          end) }
          debug.assert{ value = fails(function()
            measurement.get_text_width{ text = "text", slice_layer = "base" }
          end) }
          debug.assert{ value = fails(function()
            measurement.get_text_width{ text = "text", max_width = 0 }
          end) }
          debug.assert{ value = fails(function()
            measurement.get_text_width{ text = "text", max_height = 65536 }
          end) }
          debug.assert{ value = fails(function()
            measurement.get_text_width{ text = "text", max_width = 1.5 }
          end) }
          debug.assert{ value = fails(function()
            measurement.get_text_width{ text = "text", auto_wrap = "true" }
          end) }
          debug.assert{ value = fails(function()
            measurement.get_text_width{ text = "text", horizontal_align = "bottom" }
          end) }
          debug.assert{ value = fails(function()
            measurement.get_text_width{ text = "text", text_mode = "unknown" }
          end) }
          debug.assert{ value = fails(function()
            measurement.get_text_width{}
          end) }
        end
      "#,
    );
    LuaSession::load(spec(&source, LuaSessionKind::Game), LuaPolicy::default()).unwrap();
  }

  #[test]
  fn text_parsing_only_attaches_the_key_map_requested_by_rich_text() {
    let source = valid_script(
      r#"
        function Init(ctx)
          debug.assert{
            value = measurement.get_text_width{ text = "f%{key:jump}" } == 3,
          }
          debug.assert{
            value = measurement.get_text_width{ text = "f%{key_default:jump}" } == 4,
          }
        end
        function Render()
          draw.text{ x = 1, y = 1, text = "ordinary" }
          draw.text{ x = 1, y = 2, text = "f%{key:jump}" }
          draw.text{ x = 1, y = 3, text = "f%{key_default:jump}" }
          draw.text{
            x = 1,
            y = 4,
            text = "f%{value:name}",
            rich_params = { name = "TUI" },
          }
        end
      "#,
    );
    let mut session = LuaSession::load_with_api(
      spec(&source, LuaSessionKind::Game),
      LuaPolicy::default(),
      LuaApiConfig {
        key_actions: HashMap::from([("jump".to_string(), vec![vec!["1".to_string()]])]),
        key_default_actions: HashMap::from([("jump".to_string(), vec![vec!["f1".to_string()]])]),
        ..LuaApiConfig::default()
      },
    )
    .unwrap();
    session.render().unwrap();
    let commands = session.take_draw_commands();
    let params = commands
      .iter()
      .filter_map(|command| match command {
        LuaDrawCommand::Text { params, .. } => Some(params),
        _ => None,
      })
      .collect::<Vec<_>>();
    assert_eq!(params.len(), 4);
    assert!(params[0].params.is_none());
    let user = params[1].params.as_ref().unwrap();
    assert!(user.key_actions.contains_key("jump"));
    assert!(user.key_default_actions.is_empty());
    let default = params[2].params.as_ref().unwrap();
    assert!(default.key_actions.is_empty());
    assert!(default.key_default_actions.contains_key("jump"));
    let values = params[3].params.as_ref().unwrap();
    assert_eq!(values.values.get("name").map(String::as_str), Some("TUI"));
    assert!(values.key_actions.is_empty());
    assert!(values.key_default_actions.is_empty());
  }

  #[test]
  fn random_slice_serialization_and_encoding_libraries_work_together() {
    let source = valid_script(
      r##"
        local slice_id

        function Init(ctx)
          local first = random.create{
            type = random.INT,
            min = -10,
            max = 10,
            seed = 2468,
          }
          local second = random.create{
            type = random.INT,
            min = -10,
            max = 10,
            seed = 2468,
          }
          debug.assert{ value = random.generate(first) == random.generate(second) }
          debug.assert{ value = random.generate(first) == random.generate(second) }

          slice_id = slice.create{ width = 20, height = 20, layer = 3 }
          slice.draw{ id = slice_id, x = -4, y = 2 }
          local info = slice.get_info(slice_id)
          debug.assert{
            value = info.width == 20 and info.height == 20 and info.layer == 1,
          }

          local json = serialization.json_encode{
            title = "TUI GAME",
            enabled = true,
            values = { 1, 2, 3 },
          }
          local decoded = serialization.json_decode(json)
          debug.assert{
            value = decoded.title == "TUI GAME" and decoded.enabled
              and decoded.values[3] == 3,
          }

          local packed = serialization.binary_pack{
            fmt = "<i2c3",
            values = { 513, "abc" },
          }
          local unpacked = serialization.binary_unpack{
            fmt = "<i2c3",
            data = packed,
          }
          debug.assert{
            value = unpacked.values[1] == 513
              and unpacked.values[2] == "abc"
              and unpacked.next_pos == 6,
          }

          local xml = serialization.xml_encode{
            root = {
              _attr = { version = "1.0" },
              child = { "Hello", _attr = { id = 1 } },
            },
          }
          local xml_data = serialization.xml_decode(xml)
          debug.assert{
            value = xml_data.root._attr.version == "1.0"
              and xml_data.root.child._attr.id == "1"
              and xml_data.root.child._text == "Hello",
          }

          local encoded = encoding.base64_encode("TUI GAME")
          debug.assert{ value = encoding.base64_decode(encoded) == "TUI GAME" }
          debug.assert{ value = encoding.url_decode(encoding.url_encode("a b/中")) == "a b/中" }
          debug.assert{ value = encoding.hex_decode(encoding.hex_encode("abc")) == "abc" }
        end

        function Render()
          draw.fill_rect{
            x = -2,
            y = 1,
            width = 5,
            height = 2,
            char = "#",
            slice_layer = slice_id,
          }
        end
      "##,
    );
    let mut session =
      LuaSession::load(spec(&source, LuaSessionKind::Game), LuaPolicy::default()).unwrap();
    session.render().unwrap();

    let commands = session.take_draw_commands();
    assert!(commands.iter().any(|command| matches!(
      command,
      LuaDrawCommand::FillRect {
        target: super::super::LuaDrawTarget::Slice(SliceId(1)),
        x: -2,
        y: 1,
        width: 5,
        height: 2,
        ..
      }
    )));
  }

  #[test]
  fn random_and_slice_follow_the_documented_object_protocol() {
    let source = valid_script(
      r#"
        function Init(ctx)
          local direct_int = random.randint{}
          local direct_float = random.randfloat{}
          debug.assert{
            value = direct_int >= -2147483648 and direct_int <= 2147483647,
          }
          debug.assert{ value = direct_float >= 0 and direct_float <= 1 }

          local generator = random.create{}
          local initial = random.get_info(generator)
          debug.assert{
            value = initial.type == random.INT
              and initial.min == -2147483648
              and initial.max == 2147483647
              and initial.step == 0,
          }
          debug.assert{ value = random.count() == 1 and random.list().n == 1 }
          debug.assert{
            value = random.set{
              id = generator,
              type = random.FLOAT,
              min = -2.5,
              max = 3.5,
              seed = 42,
              step = 5,
            },
          }
          local range = random.get_range(generator)
          debug.assert{
            value = random.get_type(generator) == random.FLOAT
              and range.min == -2.5
              and range.max == 3.5
              and random.get_seed(generator) == 42
              and random.get_step(generator) == 5,
          }
          local value = random.generate(generator)
          debug.assert{
            value = value >= -2.5 and value <= 3.5 and random.get_step(generator) == 6,
          }
          debug.assert{ value = random.set_type{ id = generator } }
          debug.assert{ value = random.set_seed{ id = generator } }
          local missing_step = debug.pcall{
            func = function() random.set_step{ id = generator } end,
          }
          debug.assert{ value = not missing_step.ok }

          debug.assert{ value = slice.exists("base") }
          local base = slice.get_info("base")
          debug.assert{
            value = base.width == ctx.base.width
              and base.height == ctx.base.height
              and base.layer == 0
              and base.bg == color.TRANSPARENT,
          }
          local first = slice.create{ width = 10, height = 4, bg = color.BLUE }
          local inserted = slice.create{ width = 3, height = 2, layer = 1 }
          local slices = slice.list()
          debug.assert{
            value = slices.n == 2
              and slices[1].id == inserted
              and slices[1].layer == 1
              and slices[2].id == first
              and slices[2].layer == 2
              and slices[2].bg == color.BLUE,
          }
          debug.assert{ value = slice.set{ id = first } }
          debug.assert{ value = slice.set{ id = first, bg = color.RED } }
          debug.assert{ value = slice.get_background(first) == color.RED }
          debug.assert{
            value = slice.set_background{ id = first, bg = color.TRANSPARENT },
          }
          debug.assert{
            value = slice.get_background(first) == color.TRANSPARENT,
          }
          debug.assert{ value = slice.set_background{ id = first, bg = color.NONE } }
          debug.assert{ value = slice.get_background(first) == color.NONE }
          debug.assert{ value = slice.set_background{ id = first } }
          debug.assert{ value = slice.get_background(first) == color.NONE }
          debug.assert{
            value = not slice.set_background{ id = "base", bg = color.BLUE },
          }
          debug.assert{
            value = not slice.set_background{ id = "slice_999", bg = color.BLUE },
          }
          local invalid_background = debug.pcall{
            func = function()
              slice.set_background{ id = first, bg = "not-a-color" }
            end,
          }
          debug.assert{ value = not invalid_background.ok }
          debug.assert{
            value = slice.set_size{ id = first, width = 12 }
              and slice.get_width(first) == 12
              and slice.get_height(first) == 4,
          }
          debug.assert{ value = slice.set_layer{ id = first, layer = 999 } }
          debug.assert{ value = slice.get_layer(first) == 2 }
          debug.assert{ value = slice.delete(inserted) }
          debug.assert{ value = slice.get_layer(first) == 1 }
          debug.assert{
            value = slice["50P"] == nil
              and slice.list_by_layer == nil
              and random.set_params == nil,
          }
          debug.assert{
            value = slice.clear()
              and slice.count() == 0
              and slice.list().n == 0,
          }
          debug.assert{ value = random.clear() and random.count() == 0 }
        end
      "#,
    );
    LuaSession::load(spec(&source, LuaSessionKind::Game), LuaPolicy::default()).unwrap();
  }

  #[test]
  fn random_and_slice_objects_are_isolated_and_released_with_the_session() {
    let source = valid_script(
      r#"
        function Init(ctx)
          generator = random.create{ seed = 7 }
          layer = slice.create{ width = 2, height = 2 }
        end
      "#,
    );
    let mut first =
      LuaSession::load(spec(&source, LuaSessionKind::Game), LuaPolicy::default()).unwrap();
    let second = LuaSession::load(
      spec(&source, LuaSessionKind::Screensaver),
      LuaPolicy::default(),
    )
    .unwrap();

    for session in [&first, &second] {
      session
        .with_objects(|objects| {
          assert_eq!(
            crate::host_engine::services::RandomService::new()
              .configured_ids(objects.runtime())
              .len(),
            1
          );
          assert_eq!(
            crate::host_engine::services::SliceService::new()
              .ids(objects.ui())
              .len(),
            1
          );
        })
        .unwrap();
    }
    let first_pool = first.with_objects(|objects| objects.ui().id()).unwrap();
    let second_pool = second.with_objects(|objects| objects.ui().id()).unwrap();
    assert_ne!(first_pool, second_pool);

    first.stop();
    assert!(!first.has_objects());
    assert!(second.has_objects());
    second
      .with_objects(|objects| {
        assert_eq!(
          crate::host_engine::services::RandomService::new()
            .configured_ids(objects.runtime())
            .len(),
          1
        );
        assert_eq!(
          crate::host_engine::services::SliceService::new()
            .ids(objects.ui())
            .len(),
          1
        );
      })
      .unwrap();
  }

  #[test]
  fn serialization_rejects_cycles_sparse_tables_entities_and_malformed_encoding() {
    let source = valid_script(
      r#"
        function Init(ctx)
          local cyclic = {}
          cyclic.self = cyclic
          local cyclic_ok = debug.pcall{
            func = function() serialization.json_encode(cyclic) end,
          }
          local sparse_ok = debug.pcall{
            func = function() serialization.json_encode{ [1] = "a", [3] = "c" } end,
          }
          local entity_ok = debug.pcall{
            func = function()
              serialization.xml_decode(
                "<!DOCTYPE root [<!ENTITY secret 'hidden'>]><root>&secret;</root>"
              )
            end,
          }
          local mismatched_xml_ok = debug.pcall{
            func = function()
              serialization.xml_decode("<root><child></root>")
            end,
          }
          local mixed_xml_ok = debug.pcall{
            func = function()
              serialization.xml_decode("<root><child/>text</root>")
            end,
          }
          local hex_ok = debug.pcall{
            func = function() encoding.hex_decode("0xz1") end,
          }
          debug.assert{
            value = not cyclic_ok.ok
              and not sparse_ok.ok
              and not entity_ok.ok
              and not mismatched_xml_ok.ok
              and not mixed_xml_ok.ok
              and not hex_ok.ok,
          }
        end
      "#,
    );
    LuaSession::load(spec(&source, LuaSessionKind::Game), LuaPolicy::default()).unwrap();
  }

  #[test]
  fn serialization_formats_follow_the_public_parameter_and_result_protocol() {
    let source = valid_script(
      r#"
        function Init(ctx)
          local json = serialization.json_encode{
            value = { name = "TUI", enabled = true, values = { 1, 2 } },
          }
          local json_value = serialization.json_decode{ s = json }
          debug.assert{
            value = json_value.name == "TUI" and json_value.values[2] == 2,
          }

          local csv = serialization.csv_encode{
            rows = { { "name", "score" }, { "player", 9 } },
          }
          local csv_value = serialization.csv_decode(csv)
          debug.assert{
            value = csv_value[2][1] == "player" and csv_value[2][2] == "9",
          }

          local yaml = serialization.yaml_encode{
            value = { name = "TUI", enabled = true },
          }
          local yaml_value = serialization.yaml_decode(yaml)
          debug.assert{ value = yaml_value.name == "TUI" and yaml_value.enabled }

          local toml = serialization.toml_encode{
            value = { name = "TUI", version = 1 },
          }
          local toml_value = serialization.toml_decode(toml)
          debug.assert{
            value = toml_value.name == "TUI" and toml_value.version == 1,
          }

          local ini = serialization.ini_encode{
            server = { host = "127.0.0.1", port = 8080 },
          }
          local ini_value = serialization.ini_decode(ini)
          debug.assert{
            value = ini_value.server.host == "127.0.0.1"
              and ini_value.server.port == "8080",
          }

          local first = serialization.binary_pack{
            fmt = "<I2z",
            values = { 513, "ok" },
          }
          local all = first .. first
          local unpacked_first = serialization.binary_unpack{
            fmt = "<I2z",
            data = all,
          }
          local unpacked_second = serialization.binary_unpack{
            fmt = "<I2z",
            data = all,
            pos = unpacked_first.next_pos,
          }
          debug.assert{
            value = unpacked_first.values[1] == 513
              and unpacked_first.values[2] == "ok"
              and unpacked_second.values[1] == 513
              and unpacked_second.values[2] == "ok",
          }
          debug.assert{
            value = serialization.binary_packsize("<I2c2x") == 5,
          }
        end
      "#,
    );
    LuaSession::load(spec(&source, LuaSessionKind::Game), LuaPolicy::default()).unwrap();
  }

  #[test]
  fn drawing_is_allowed_in_all_callbacks_but_render_requests_are_not_reentrant() {
    let source = valid_script(
      r#"
        function Init(ctx)
          draw.fill_rect{ x = 2, y = 1, width = 10, height = 4, bg = color.BLUE }
          draw.render()
        end
        function Update(dt)
          draw.erase_rect{ x = 3, y = 2, width = 8, height = 2 }
        end
        function Render()
          local ok = debug.pcall{ func = function() draw.render() end }
          debug.assert{ value = not ok.ok }
          draw.text{ x = 1, y = 1, text = "render" }
        end
      "#,
    );
    let mut session = LuaSession::load(spec(&source, LuaSessionKind::Game), LuaPolicy::default())
      .expect("drawing during Init must be accepted");
    session
      .update()
      .expect("drawing during Update must be accepted");
    session
      .render()
      .expect("ordinary drawing during Render must remain valid");

    let commands = session.take_draw_commands();
    assert!(matches!(
      commands.first(),
      Some(LuaDrawCommand::FillRect { .. })
    ));
    assert!(matches!(
      commands.get(1),
      Some(LuaDrawCommand::EraseRect { .. })
    ));
    assert!(matches!(commands.get(2), Some(LuaDrawCommand::Text { .. })));
  }

  #[test]
  fn draw_limit_is_reset_when_the_host_finishes_each_frame() {
    let source = valid_script(
      r#"
        function Update(dt)
          local x = 0
          local y = 0
          for item in ipairs(char.ASCII_LETTER) do
            x = x + 2
            if x % 20 == 0 then
              x = 2
              y = y + 1
            end
            draw.text{ x = x, y = y, text = item.value }
          end
        end
      "#,
    );
    let mut session = LuaSession::load(spec(&source, LuaSessionKind::Game), LuaPolicy::default())
      .expect("the drawing fixture must load");

    for _ in 0..100 {
      session
        .update()
        .expect("one frame must stay below the limit");
      assert_eq!(session.take_draw_commands().len(), 52);
    }
  }

  #[test]
  fn debug_print_uses_header_free_defaults() {
    let source = valid_script(
      r#"
        function Init(ctx)
          debug.print{ message = "plain" }
        end
      "#,
    );
    let mut session = LuaSession::load_with_api(
      spec(&source, LuaSessionKind::Game),
      LuaPolicy::default(),
      LuaApiConfig {
        debug_enabled: true,
        ..LuaApiConfig::default()
      },
    )
    .unwrap();
    assert!(session.take_host_commands().iter().any(|command| matches!(
      command,
      LuaHostCommand::Print {
        message,
        title: None,
        time: false,
        level: None,
        type_head: false,
      } if message == "plain"
    )));
  }

  #[test]
  fn successful_debug_print_before_callback_fault_remains_pending() {
    let source = valid_script(
      r#"
        function Update(dt)
          debug.print{ message = 1 }
          debug.print{ missing_message }
        end
      "#,
    );
    let mut session = LuaSession::load_with_api(
      spec(&source, LuaSessionKind::Game),
      LuaPolicy::default(),
      LuaApiConfig {
        debug_enabled: true,
        ..LuaApiConfig::default()
      },
    )
    .unwrap();

    let error = session.update().expect_err("the second print must fail");
    assert_eq!(error.stage, LuaErrorStage::Callback);
    assert_eq!(session.state(), LuaSessionState::Faulted);
    assert!(session.take_host_commands().iter().any(|command| matches!(
      command,
      LuaHostCommand::Print { message, .. } if message == "1"
    )));
  }

  #[test]
  fn user_facing_text_parameters_convert_lua_values_to_strings() {
    let source = valid_script(
      r#"
        function Init(ctx)
          debug.assert{
            value = measurement.get_text_width{ text = 12345 } == 5,
          }
          debug.print{ message = 100, title = false }
          debug.info(true)
        end

        function Render()
          draw.text{
            x = 1,
            y = 1,
            text = 200,
            overflow_marker = 9,
          }
        end
      "#,
    );
    let mut session = LuaSession::load_with_api(
      spec(&source, LuaSessionKind::Game),
      LuaPolicy::default(),
      LuaApiConfig {
        debug_enabled: true,
        ..LuaApiConfig::default()
      },
    )
    .unwrap();

    let commands = session.take_host_commands();
    assert!(commands.iter().any(|command| matches!(
      command,
      LuaHostCommand::Print {
        message,
        title: Some(title),
        time: false,
        level: None,
        type_head: false,
      } if message == "100" && title == "false"
    )));
    assert!(commands.iter().any(|command| matches!(
      command,
      LuaHostCommand::Print {
        message,
        title: None,
        time: true,
        level: Some(level),
        type_head: true,
      } if message == "true" && level == "info"
    )));

    session.render().unwrap();
    let draw_commands = session.take_draw_commands();
    assert!(draw_commands.iter().any(|command| matches!(
      command,
      LuaDrawCommand::Text { params, .. }
        if params.text == "200" && params.overflow_marker.as_deref() == Some("9")
    )));
  }

  #[test]
  fn debug_print_constants_and_convenience_methods_use_the_standard_options() {
    let source = valid_script(
      r#"
        function Init(ctx)
          debug.assert{
            value = debug.VERSION == "Lua 5.4 / TUI GAME API 1"
              and debug.TRACE == "trace"
              and debug.DEBUG == "debug"
              and debug.INFO == "info"
              and debug.WARN == "warn"
              and debug.ERROR == "error"
              and debug.FATAL == "fatal",
          }
          debug.print{
            message = "custom",
            title = "Title",
            level = debug.WARN,
            time = true,
            type_head = true,
          }
          debug.info("info")
          debug.warn{ message = "warn" }
          debug.error("error")
        end
      "#,
    );
    let mut session = LuaSession::load_with_api(
      spec(&source, LuaSessionKind::Game),
      LuaPolicy::default(),
      LuaApiConfig {
        debug_enabled: true,
        ..LuaApiConfig::default()
      },
    )
    .unwrap();
    let commands = session.take_host_commands();
    assert!(commands.iter().any(|command| matches!(
      command,
      LuaHostCommand::Print {
        message,
        title: Some(title),
        time: true,
        level: Some(level),
        type_head: true,
      } if message == "custom" && title == "Title" && level == "warn"
    )));
    for (expected_message, expected_level) in
      [("info", "info"), ("warn", "warn"), ("error", "error")]
    {
      assert!(commands.iter().any(|command| matches!(
        command,
        LuaHostCommand::Print {
          message,
          title: None,
          time: true,
          level: Some(level),
          type_head: true,
        } if message == expected_message && level == expected_level
      )));
    }
  }

  #[test]
  fn protected_calls_return_named_result_tables() {
    let source = valid_script(
      r#"
        function Init(ctx)
          local success = debug.pcall{
            func = function(left, right)
              return left + right, nil, "tail"
            end,
            values = { 2, 3 },
          }
          debug.assert{
            value = success.ok
              and type(success.values) == "table"
              and success.values.n == 3
              and success.values[1] == 5
              and success.values[2] == nil
              and success.values[3] == "tail"
              and success.error == nil,
          }

          local failure = debug.pcall{
            func = function()
              debug.assert{ value = false, message = "failed" }
            end,
          }
          debug.assert{
            value = not failure.ok
              and type(failure.error) == "string"
              and failure.values == nil,
          }

          local obsolete_message = debug.pcall{
            func = function()
              debug.pcall{ func = function() end, message = "removed" }
            end,
          }
          debug.assert{
            value = not obsolete_message.ok and type(obsolete_message.error) == "string",
          }

          local handled = debug.xpcall{
            func = function()
              debug.assert{ value = false, message = "failed" }
            end,
            error_callback = function(message)
              return "handled"
            end,
          }
          debug.assert{
            value = not handled.ok and handled.error == "handled",
          }
        end
      "#,
    );
    LuaSession::load(spec(&source, LuaSessionKind::Game), LuaPolicy::default()).unwrap();
  }

  #[test]
  fn api_rejects_invalid_math_domains_and_excessively_deep_parameters() {
    let source = valid_script(
      r#"
        function Init(ctx)
          local sqrt_ok = debug.pcall{ func = function() math.sqrt(-1) end }
          local pow_ok = debug.pcall{ func = function() math.pow{ x = -1, y = 0.5 } end }
          local value = {}
          local cursor = value
          for index = 1, 33 do
            cursor.child = {}
            cursor = cursor.child
          end
          local depth_ok = debug.pcall{ func = function() base.type(value) end }
          local cyclic = {}
          cyclic.self = cyclic
          local cycle_ok = debug.pcall{ func = function() base.type(cyclic) end }
          local unknown_ok = debug.pcall{
            func = function()
              measurement.get_text_width{ text = "value", unknown = true }
            end,
          }
          local packed = table.pack{ values = { [1] = "first", n = 3 } }
          debug.assert{
            value = not sqrt_ok.ok and not pow_ok.ok and not depth_ok.ok and not cycle_ok.ok
              and not unknown_ok.ok
              and packed.n == 3 and packed[1] == "first" and packed[3] == nil,
          }
        end
      "#,
    );
    LuaSession::load(spec(&source, LuaSessionKind::Game), LuaPolicy::default()).unwrap();
  }

  #[test]
  fn string_api_matches_documented_tables_unicode_and_limits() {
    let source = valid_script(
      r#"
        function Init(ctx)
          local function fails(func)
            return not debug.pcall{ func = func }.ok
          end

          debug.assert{ value = string.lower("ÄBC") == "äbc" }
          debug.assert{ value = string.upper{ text = "äbc" } == "ÄBC" }
          debug.assert{ value = string.reverse("你ab") == "ba你" }

          local parts = string.split{ text = "a<>你<>", sep = "<>" }
          debug.assert{
            value = #parts == 3 and parts[1] == "a" and parts[2] == "你" and parts[3] == "",
          }
          debug.assert{ value = fails(function() string.split{ text = "abc", sep = "" } end) }
          debug.assert{ value = fails(function() string.split{ text = "abc", char = "b" } end) }

          debug.assert{ value = string.sub{ text = "甲乙丙", start = 2 } == "乙丙" }
          debug.assert{ value = string.sub{ text = "甲乙丙", start = -2, finish = -1 } == "乙丙" }
          debug.assert{ value = string.sub{ text = "甲乙丙", start = -99, finish = 99 } == "甲乙丙" }
          debug.assert{ value = string.rep{ text = "A", times = 3, sep = ":" } == "A:A:A" }

          local found = string.find{ text = "你ab你", pattern = "(a)(b)" }
          debug.assert{
            value = found.start == 2 and found.finish == 3
              and found.captures.n == 2
              and found.captures[1] == "a" and found.captures[2] == "b",
          }
          local plain = string.find{ text = "a.b", pattern = ".", plain = true }
          debug.assert{
            value = plain.start == 2 and plain.finish == 2
              and plain.captures.n == 1 and plain.captures[1] == ".",
          }
          debug.assert{ value = string.find{ text = "abc", pattern = "a", init = 0 }.start == 1 }
          debug.assert{ value = string.find{ text = "abc", pattern = "a", init = 99 } == nil }
          local no_capture = string.find{ text = "abc", pattern = "b" }
          debug.assert{
            value = no_capture.captures.n == 1 and no_capture.captures[1] == "b",
          }

          local matched = string.match{ text = "x=42", pattern = "(%a+)=(%d+)" }
          debug.assert{
            value = matched.n == 2 and matched[1] == "x" and matched[2] == "42",
          }
          local whole = string.match{ text = "x=42", pattern = "%d+" }
          debug.assert{ value = whole.n == 1 and whole[1] == "42" }
          local position = string.match{ text = "abc", pattern = "()b" }
          debug.assert{ value = position.n == 1 and position[1] == 2 }

          local iterator = string.gmatch{ text = "a1 b2", pattern = "(%a)(%d)" }
          local first = iterator()
          local second = iterator()
          debug.assert{
            value = first.n == 2 and first[1] == "a" and first[2] == "1"
              and second.n == 2 and second[1] == "b" and second[2] == "2"
              and iterator() == nil,
          }
          local iterated = ""
          for item in string.gmatch{ text = "a1 b2", pattern = "(%a)(%d)" } do
            iterated = iterated .. item[1] .. item[2]
          end
          debug.assert{ value = iterated == "a1b2" }
          local many = string.rep{ text = "x", times = 10001 }
          local lazy_pattern = string.gmatch{ text = many, pattern = "." }
          debug.assert{ value = lazy_pattern()[1] == "x" }

          local replaced = string.gsub{ text = "ab", pattern = "(%a)", repl = "%1%1" }
          debug.assert{ value = replaced.result == "aabb" and replaced.count == 2 }
          local unchanged = string.gsub{ text = "ab", pattern = ".", repl = "x", limit = 0 }
          debug.assert{ value = unchanged.result == "ab" and unchanged.count == 0 }
          local table_replaced = string.gsub{
            text = "ab", pattern = ".", repl = { a = "A" },
          }
          debug.assert{ value = table_replaced.result == "Ab" and table_replaced.count == 2 }
          local function_replaced = string.gsub{
            text = "a1", pattern = ".", repl = function(value)
              if value == "a" then return 9 end
              return false
            end,
          }
          debug.assert{ value = function_replaced.result == "91" and function_replaced.count == 2 }
          debug.assert{
            value = fails(function()
              string.gsub{ text = "ab", pattern = ".", repl = "x", limit = -2 }
            end),
          }
          debug.assert{
            value = fails(function()
              string.gsub{ text = "a", pattern = ".", repl = function() return true end }
            end),
          }

          local regex_found = string.regex_find{ text = "你ab你", pattern = "(a)(b)" }
          debug.assert{
            value = regex_found.start == 2 and regex_found.finish == 3
              and regex_found.captures.n == 2
              and regex_found.captures[1] == "a" and regex_found.captures[2] == "b",
          }
          local regex_match = string.regex_match{ text = "x=42", pattern = "([a-z]+)=([0-9]+)" }
          debug.assert{ value = regex_match[1] == "x" and regex_match[2] == "42" }
          local regex_iterated = ""
          for item in string.regex_gmatch{ text = "a1 b2", pattern = "([a-z])([0-9])" } do
            regex_iterated = regex_iterated .. item[1] .. item[2]
          end
          debug.assert{ value = regex_iterated == "a1b2" }
          local lazy_regex = string.regex_gmatch{ text = many, pattern = "." }
          debug.assert{ value = lazy_regex()[1] == "x" }
          local regex_replaced = string.regex_gsub{
            text = "a1 b2", pattern = "([a-z])([0-9])", repl = "$2$1", limit = -1,
          }
          debug.assert{ value = regex_replaced.result == "1a 2b" and regex_replaced.count == 2 }
          debug.assert{ value = string.regex_test{ text = "abc", pattern = "^a" } }
          debug.assert{ value = string.regex_escape("[a-z]") == "\\[a\\-z\\]" }
          local regex_parts = string.regex_split{ text = "a, b;c", pattern = "[,;]\\s*" }
          debug.assert{ value = #regex_parts == 3 and regex_parts[2] == "b" }

          debug.assert{
            value = string.rich_text_to_plain_text{
              text = "f%<fg:red>A</fg>{value:tail}", rich_params = { tail = "B" },
            } == "AB",
          }
          debug.assert{
            value = string.rich_text_to_plain_text{ text = "f%{key:jump}" } == "[Space]",
          }
          debug.assert{
            value = string.rich_text_to_plain_text{
              text = "f%{key:jump}", key_params = false,
            } == "{key:jump}",
          }
          debug.assert{
            value = string.format{ format_string = "%s:%04d", values = { "v", 7 } } == "v:0007",
          }
          debug.assert{
            value = string.format{
              format_string = "%-4s|%+d|%#x|%.2f|%%",
              values = { "a", 2, 255, 1.25 },
            } == "a   |+2|0xff|1.25|%",
          }
        end
      "#,
    );
    LuaSession::load_with_api(
      spec(&source, LuaSessionKind::Game),
      LuaPolicy::default(),
      LuaApiConfig {
        key_actions: HashMap::from([("jump".to_string(), vec![vec!["space".to_string()]])]),
        ..LuaApiConfig::default()
      },
    )
    .unwrap();
  }

  #[test]
  fn table_api_matches_documented_operations_and_pretty_output() {
    let source = valid_script(
      r#"
        function Init(ctx)
          local function fails(func)
            return not debug.pcall{ func = func }.ok
          end

          local joined = table.concat{
            table = { 1, "二", 3 }, sep = "|", start = 1, finish = 3,
          }
          debug.assert{ value = joined == "1|二|3" }
          debug.assert{
            value = fails(function()
              table.concat{ table = { 1, 2 }, separator = "," }
            end),
          }

          local inserted = { "a", "c" }
          table.insert{ table = inserted, position = 2, value = "b" }
          table.insert{ table = inserted, value = "d" }
          debug.assert{
            value = inserted[1] == "a" and inserted[2] == "b"
              and inserted[3] == "c" and inserted[4] == "d",
          }
          local removed = table.remove{ table = inserted, position = 2 }
          debug.assert{
            value = removed == "b" and #inserted == 3 and inserted[2] == "c",
          }

          local moved = { 1, 2, 3, 4 }
          local moved_result = table.move{
            source = moved, start = 1, finish = 3, target_index = 2,
          }
          debug.assert{
            value = moved_result == moved and moved[1] == 1 and moved[2] == 1
              and moved[3] == 2 and moved[4] == 3,
          }
          local target = {}
          debug.assert{
            value = table.move{
              source = moved, start = 2, finish = 3, target_index = 1, target = target,
            } == target and target[1] == 1 and target[2] == 2,
          }
          debug.assert{
            value = fails(function()
              table.move{
                source = {}, start = 0, finish = 1, target_index = 9223372036854775807,
              }
            end),
          }

          local packed = table.pack{ values = { [1] = "a", [3] = "c", n = 3 } }
          debug.assert{
            value = packed.n == 3 and packed[1] == "a"
              and packed[2] == nil and packed[3] == "c",
          }
          local directly_packed = table.pack{ "x", nil, "z", n = 3 }
          debug.assert{
            value = directly_packed.n == 3 and directly_packed[1] == "x"
              and directly_packed[2] == nil and directly_packed[3] == "z",
          }
          local first, second, third = table.unpack{
            table = packed, start = 1, finish = packed.n,
          }
          debug.assert{ value = first == "a" and second == nil and third == "c" }

          local sortable = { 3, 1, 2 }
          table.sort{ table = sortable }
          debug.assert{ value = sortable[1] == 1 and sortable[2] == 2 and sortable[3] == 3 }
          table.sort{
            table = sortable,
            comparator = function(left, right) return left > right end,
          }
          debug.assert{ value = sortable[1] == 3 and sortable[2] == 2 and sortable[3] == 1 }

          local child = { value = 7 }
          local original = { child = child, alias = child, callback = function() end }
          local copied = table.deepcopy(original)
          debug.assert{
            value = copied ~= original and copied.child ~= child
              and copied.child == copied.alias and copied.child.value == 7
              and copied.callback == original.callback,
          }
          copied.child.value = 9
          debug.assert{ value = original.child.value == 7 }

          local visual = table.pretty({ 1, 3, "a", n = 3 })
          debug.assert{ value = visual == "{[1] = 1, [2] = 3, [3] = \"a\", n = 3}" }
          local nested = table.pretty{
            table = { text = "a\nb", enabled = true, callback = function() end },
          }
          debug.assert{
            value = string.find{ text = nested, pattern = "callback", plain = true } ~= nil
              and string.find{ text = nested, pattern = "<function:", plain = true } ~= nil
              and string.find{ text = nested, pattern = "\\n", plain = true } ~= nil,
          }
        end
      "#,
    );
    LuaSession::load(spec(&source, LuaSessionKind::Game), LuaPolicy::default()).unwrap();
  }

  #[test]
  fn utf8_api_uses_scalar_indices_lazy_iterators_and_table_results() {
    let source = valid_script(
      r#"
        function Init(ctx)
          local function fails(func)
            return not debug.pcall{ func = func }.ok
          end

          debug.assert{ value = utf8.len("A你B") == 3 }
          debug.assert{ value = utf8.byte_len{ text = "A你B" } == 5 }
          debug.assert{ value = utf8.is_ascii("ABC") and not utf8.is_ascii("A你") }

          debug.assert{ value = utf8.codepoint_to_char{ 65, 20320 } == "A你" }
          debug.assert{
            value = utf8.codepoint_to_char{ values = { 65, 66 } } == "AB",
          }
          debug.assert{ value = utf8.ascii_to_char{ 65, 66, 67 } == "ABC" }
          debug.assert{
            value = fails(function() utf8.ascii_to_char{ 128 } end)
              and fails(function() utf8.codepoint_to_char{ 55296 } end),
          }

          local codepoints = utf8.char_to_codepoint{ text = "A你B" }
          debug.assert{
            value = codepoints.n == 3 and codepoints[1] == 65
              and codepoints[2] == 20320 and codepoints[3] == 66,
          }
          local ascii = utf8.char_to_ascii{ text = "A你B", start = 1, finish = 3 }
          debug.assert{
            value = ascii.n == 3 and ascii[1] == 65
              and ascii[2] == nil and ascii[3] == 66,
          }
          local one_ascii = utf8.char_to_ascii{ text = "ABC", start = 2 }
          debug.assert{
            value = one_ascii.n == 2 and one_ascii[1] == 66 and one_ascii[2] == 67,
          }
          debug.assert{
            value = utf8.char_to_ascii{ text = "" }.n == 0
              and utf8.char_to_codepoint{ text = "" }.n == 0,
          }
          local reversed_range = utf8.char_to_codepoint{
            text = "ABC", start = 3, finish = 1,
          }
          debug.assert{ value = reversed_range.n == 0 }

          debug.assert{
            value = utf8.char_position{ text = "A你B", index = 2 } == 2
              and utf8.char_position{ text = "A你B", start = 2, index = 2 } == 5
              and utf8.char_position{ text = "A你B", index = 9 } == nil,
          }
          debug.assert{
            value = fails(function()
              utf8.char_position{ text = "ABC", index = 0 }
            end),
          }

          local iterator = utf8.codepoints("A你B")
          local first = iterator()
          local second = iterator()
          local third = iterator()
          debug.assert{
            value = first.byte_position == 1 and first.codepoint == 65
              and second.byte_position == 2 and second.codepoint == 20320
              and third.byte_position == 5 and third.codepoint == 66
              and iterator() == nil,
          }
          local total = 0
          for item in utf8.codepoints(string.rep{ text = "x", times = 20000 }) do
            total = total + 1
            if total == 2 then break end
          end
          debug.assert{ value = total == 2 }

          local from_start = utf8.next{ text = "A你B" }
          local after_first = utf8.next{ text = "A你B", pos = 1 }
          local after_second = utf8.next{ text = "A你B", pos = 2 }
          debug.assert{
            value = from_start.position == 1 and from_start.codepoint == 65
              and after_first.position == 2 and after_first.codepoint == 20320
              and after_second.position == 5 and after_second.codepoint == 66
              and utf8.next{ text = "A你B", pos = 5 } == nil,
          }
          debug.assert{
            value = fails(function() utf8.next{ text = "ABC", pos = 0 } end),
          }
        end
      "#,
    );
    LuaSession::load(spec(&source, LuaSessionKind::Game), LuaPolicy::default()).unwrap();
  }

  #[test]
  fn math_api_matches_documented_names_types_and_boundaries() {
    let source = valid_script(
      r#"
        function Init(ctx)
          local function near(left, right, tolerance)
            return math.abs(left - right) < tolerance
          end
          local function fails(func)
            return not debug.pcall{ func = func }.ok
          end

          debug.assert{ value = math.number_type(math.PI) == "float" }
          debug.assert{ value = math.number_type(math.E) == "float" }
          debug.assert{ value = math.POSITIVE_INFINITE == math.INFINITE }
          debug.assert{ value = math.POSITIVE_INFINITE > math.MAX_INTEGER }
          debug.assert{ value = math.NEGATIVE_INFINITE < math.MIN_INTEGER }
          debug.assert{ value = math.number_type(math.MAX_INTEGER) == "integer" }
          debug.assert{ value = math.number_type(math.MIN_INTEGER) == "integer" }
          debug.assert{ value = near(math.DEG * math.PI, 180.0, 0.000000000001) }
          debug.assert{ value = near(math.RAD * 180.0, math.PI, 0.000000000001) }

          debug.assert{ value = type(math.lg) == "function" }
          debug.assert{ value = type(math.ln) == "function" }
          debug.assert{ value = type(math.number_type) == "function" }
          debug.assert{ value = math.log10 == nil and math.type == nil }

          debug.assert{ value = math.abs(-5) == 5 and math.number_type(math.abs(-5)) == "float" }
          debug.assert{ value = math.ceil(3.1) == 4 and math.number_type(math.ceil(3.1)) == "integer" }
          debug.assert{ value = math.floor(-3.1) == -4 and math.number_type(math.floor(-3.1)) == "integer" }
          debug.assert{ value = math.round(3.5) == 4 and math.round(-3.5) == -4 }
          debug.assert{ value = math.round_to{ value = 3.14159, digits = 2 } == 3.14 }
          debug.assert{ value = math.round_to{ value = 12345, digits = -2 } == 12300 }
          debug.assert{ value = math.round_to{ value = 1e308, digits = 1 } == 1e308 }

          debug.assert{ value = math.fmod{ x = 7, y = 3 } == 1 }
          debug.assert{ value = math.fmod{ x = -7, y = 3 } == -1 }
          debug.assert{ value = math.fmod{ x = math.MIN_INTEGER, y = -1 } == 0 }
          debug.assert{ value = math.number_type(math.fmod{ x = 7, y = 3 }) == "integer" }
          debug.assert{ value = math.pow{ x = 2, y = 3 } == 8 }
          debug.assert{ value = near(math.exp(1), math.E, 0.000000000001) }
          debug.assert{ value = math.log{ value = 8, base = 2 } == 3 }
          debug.assert{ value = math.lg(100) == 2 }
          debug.assert{ value = near(math.ln(math.E), 1, 0.000000000001) }
          debug.assert{ value = math.sqrt(9) == 3 }
          debug.assert{ value = math.ldexp{ x = 3, exp = 2 } == 12 }
          debug.assert{ value = math.ldexp{ x = 0, exp = math.MAX_INTEGER } == 0 }
          debug.assert{ value = math.ldexp{ x = 1, exp = math.MIN_INTEGER } == 0 }

          local split = math.frexp(12.8)
          debug.assert{
            value = near(split.mantissa, 0.8, 0.000000000001)
              and split.exponent == 4
              and math.number_type(split.exponent) == "integer",
          }
          local largest = math.frexp(1.7976931348623157e308)
          debug.assert{
            value = largest.mantissa >= 0.5 and largest.mantissa < 1.0
              and largest.exponent == 1024,
          }

          debug.assert{ value = near(math.sin(math.PI / 2), 1, 0.000000000001) }
          debug.assert{ value = near(math.cos(0), 1, 0.000000000001) }
          debug.assert{ value = near(math.tan(0), 0, 0.000000000001) }
          debug.assert{ value = near(math.asin(1), math.PI / 2, 0.000000000001) }
          debug.assert{ value = near(math.acos(1), 0, 0.000000000001) }
          debug.assert{ value = near(math.atan(1), math.PI / 4, 0.000000000001) }
          debug.assert{ value = near(math.atan2{ y = 1, x = 1 }, math.PI / 4, 0.000000000001) }
          debug.assert{ value = near(math.deg(math.PI), 180, 0.000000000001) }
          debug.assert{ value = near(math.rad(180), math.PI, 0.000000000001) }
          debug.assert{ value = math.normalize_angle(450) == 90 }
          debug.assert{ value = math.normalize_angle(-90) == 270 }
          debug.assert{ value = math.number_type(math.normalize_angle(0)) == "float" }

          debug.assert{ value = math.max{ 1, 5, 3 } == 5 }
          debug.assert{ value = math.min{ values = { 1, -2, 3 } } == -2 }
          local parts = math.modf(3.14)
          debug.assert{
            value = parts.integer_part == 3
              and math.number_type(parts.integer_part) == "integer"
              and near(parts.fractional_part, 0.14, 0.000000000001),
          }
          debug.assert{ value = math.tointeger(3.0) == 3 }
          debug.assert{ value = math.tointeger(3.14) == nil }
          debug.assert{ value = math.tointeger(9223372036854775808.0) == nil }
          debug.assert{ value = math.number_type("3") == nil }
          debug.assert{ value = math.ult{ left = -1, right = 0 } == false }
          debug.assert{ value = math.approx_equal{ left = 0.1 + 0.2, right = 0.3 } }
          debug.assert{ value = math.approx_equal{ left = 1.0, right = 1.005, epsilon = 0.01 } }
          debug.assert{ value = not math.approx_equal{ left = 1.0, right = 1.01, epsilon = 0.001 } }
          debug.assert{ value = math.approx_equal{ left = -0.0, right = 0.0, epsilon = 0.0 } }
          debug.assert{ value = near(math.percent{ value = 5, total = 16 }, 0.3125, 0.000000000001) }
          debug.assert{ value = near(math.percent{ value = 5, total = 16, as_percent = true }, 31.25, 0.000000000001) }

          debug.assert{ value = math.factorial(0) == 1 }
          debug.assert{ value = math.number_type(math.factorial{ n = 5 }) == "float" }
          debug.assert{ value = math.factorial(170) > 7e306 }
          debug.assert{ value = math.combination{ n = 5, k = 2 } == 10 }
          debug.assert{ value = math.number_type(math.combination{ n = 5, k = 2 }) == "integer" }

          debug.assert{ value = fails(function() math.abs(math.INFINITE) end) }
          debug.assert{ value = fails(function() math.ceil(1e20) end) }
          debug.assert{ value = fails(function() math.round_to{ value = 1, digits = 309 } end) }
          debug.assert{ value = fails(function() math.fmod{ x = 1, y = 0 } end) }
          debug.assert{ value = fails(function() math.fmod{ x = 9223372036854775808.0, y = 1 } end) }
          debug.assert{ value = fails(function() math.pow{ left = 2, right = 3 } end) }
          debug.assert{ value = fails(function() math.ldexp{ x = 1, exp = math.MAX_INTEGER } end) }
          debug.assert{ value = fails(function() math.log{ value = 2 } end) }
          debug.assert{ value = fails(function() math.sqrt(-1) end) }
          debug.assert{ value = fails(function() math.asin(2) end) }
          debug.assert{ value = fails(function() math.max{} end) }
          debug.assert{ value = fails(function() math.max{ values = { [1] = 1, [3] = 3, n = 3 } } end) }
          debug.assert{ value = fails(function() math.modf(1e20) end) }
          debug.assert{ value = fails(function() math.approx_equal{ left = math.INFINITE, right = 1 } end) }
          debug.assert{ value = fails(function() math.approx_equal{ left = 1, right = 1, epsilon = -1 } end) }
          debug.assert{ value = fails(function() math.approx_equal{ left = 1, right = 1, extra = true } end) }
          debug.assert{ value = fails(function() math.factorial(171) end) }
          debug.assert{ value = fails(function() math.combination{ n = 67, k = 33 } end) }
        end
      "#,
    );
    LuaSession::load(spec(&source, LuaSessionKind::Game), LuaPolicy::default()).unwrap();
  }

  #[test]
  fn game_commands_enforce_callback_reentrancy_boundaries() {
    let source = valid_script(
      r#"
        function Init(ctx)
          local result = debug.pcall{ func = function() game.exit_game() end }
          debug.assert{ value = not result.ok }
        end
        function SaveGame()
          local save = debug.pcall{ func = function() game.save_game() end }
          local exit = debug.pcall{ func = function() game.exit_game() end }
          debug.assert{ value = not save.ok and not exit.ok }
          game.save_best()
          return { saved = true }
        end
        function SaveBest()
          local save = debug.pcall{ func = function() game.save_best() end }
          local exit = debug.pcall{ func = function() game.exit_game() end }
          debug.assert{ value = not save.ok and not exit.ok }
          game.save_game()
          return { best_string = "best" }
        end
        function Update(dt)
          game.exit_game()
        end
      "#,
    );
    let mut session =
      LuaSession::load(spec(&source, LuaSessionKind::Game), LuaPolicy::default()).unwrap();
    assert_eq!(session.save_game().unwrap().unwrap()["saved"], true);
    let save_game_commands = session.take_host_commands();
    assert!(
      !save_game_commands
        .iter()
        .any(|command| matches!(command, LuaHostCommand::SaveGame))
    );
    assert!(
      !save_game_commands
        .iter()
        .any(|command| matches!(command, LuaHostCommand::ExitGame))
    );
    assert!(
      save_game_commands
        .iter()
        .any(|command| matches!(command, LuaHostCommand::SaveBest))
    );

    assert_eq!(session.save_best().unwrap().unwrap()["best_string"], "best");
    let save_best_commands = session.take_host_commands();
    assert!(
      !save_best_commands
        .iter()
        .any(|command| matches!(command, LuaHostCommand::SaveBest))
    );
    assert!(
      !save_best_commands
        .iter()
        .any(|command| matches!(command, LuaHostCommand::ExitGame))
    );
    assert!(
      save_best_commands
        .iter()
        .any(|command| matches!(command, LuaHostCommand::SaveGame))
    );

    session.update().unwrap();
    assert!(
      session
        .take_host_commands()
        .iter()
        .any(|command| matches!(command, LuaHostCommand::ExitGame))
    );
  }

  #[test]
  fn restricted_calls_are_ignored_before_parameter_validation() {
    let source = valid_script(
      r#"
        function Init(ctx)
          game.exit_game("ignored")
          event.clear_action("ignored")
          file.write("ignored")
          file.list_dir("ignored")
          file.create_dir("ignored")
          file.remove("ignored")
          debug.info({ invalid = true })
        end
      "#,
    );
    let mut session = LuaSession::load(
      spec(&source, LuaSessionKind::Screensaver),
      LuaPolicy::default(),
    )
    .unwrap();
    let commands = session.take_host_commands();
    for method in [
      "game.exit_game",
      "event.clear_action",
      "file.write",
      "file.list_dir",
      "file.create_dir",
      "file.remove",
      "debug.info",
    ] {
      assert!(
        commands.iter().any(|command| matches!(
          command,
          LuaHostCommand::Ignored { method: found, .. } if *found == method
        )),
        "missing ignored command for {method}"
      );
    }
  }

  #[test]
  fn api_configuration_is_active_during_entry_and_init() {
    let source = valid_script(
      r#"
        debug.info("entry")
        function Init(ctx)
          debug.info("init")
          event.clear_action()
        end
      "#,
    );
    let mut session = LuaSession::load_with_api(
      spec(&source, LuaSessionKind::Game),
      LuaPolicy::default(),
      LuaApiConfig {
        debug_enabled: true,
        safe_mode_enabled: false,
        key_actions: HashMap::new(),
        key_default_actions: HashMap::new(),
        ..LuaApiConfig::default()
      },
    )
    .unwrap();
    let commands = session.take_host_commands();
    assert_eq!(
      commands
        .iter()
        .filter(|command| matches!(command, LuaHostCommand::Print { .. }))
        .count(),
      2
    );
    assert!(
      commands
        .iter()
        .any(|command| matches!(command, LuaHostCommand::ClearActions))
    );
  }

  #[test]
  fn event_action_controls_require_an_unrestricted_game_session() {
    let source = valid_script(
      r#"
        function Init(ctx)
          event.skip_action()
          event.clear_action()
        end
      "#,
    );
    let load = |session_kind, safe_mode_enabled| {
      let mut session = LuaSession::load_with_api(
        spec(&source, session_kind),
        LuaPolicy::default(),
        LuaApiConfig {
          safe_mode_enabled,
          ..LuaApiConfig::default()
        },
      )
      .unwrap();
      session.take_host_commands()
    };

    let permitted = load(LuaSessionKind::Game, false);
    assert!(
      permitted
        .iter()
        .any(|command| matches!(command, LuaHostCommand::SkipActions))
    );
    assert!(
      permitted
        .iter()
        .any(|command| matches!(command, LuaHostCommand::ClearActions))
    );

    for commands in [
      load(LuaSessionKind::Game, true),
      load(LuaSessionKind::Screensaver, false),
      load(LuaSessionKind::Screensaver, true),
    ] {
      assert!(!commands.iter().any(|command| matches!(
        command,
        LuaHostCommand::SkipActions | LuaHostCommand::ClearActions
      )));
      for method in ["event.skip_action", "event.clear_action"] {
        assert!(commands.iter().any(|command| matches!(
          command,
          LuaHostCommand::Ignored { method: found, .. } if *found == method
        )));
      }
    }
  }

  #[test]
  fn file_apis_share_current_directory_and_parent_traversal_rules() {
    let id = TEST_ID.fetch_add(1, Ordering::Relaxed);
    let package_root = std::env::temp_dir().join(format!(
      "tui_game_lua_assets_root_{}_{}",
      std::process::id(),
      id
    ));
    let scripts_root = package_root.join("scripts");
    let assets_root = package_root.join("assets");
    fs::create_dir_all(&scripts_root).unwrap();
    fs::create_dir_all(&assets_root).unwrap();
    fs::write(assets_root.join("input.txt"), "input").unwrap();
    fs::write(assets_root.join("input.bin"), [0x00, 0xff, 0x7f]).unwrap();
    let entry_path = scripts_root.join("main.lua");
    fs::write(
      &entry_path,
      valid_script(
        r#"
          function Init(ctx)
            debug.assert{ value = file.read_byte == nil }
            debug.assert{ value = file.write_byte == nil }
            debug.assert{ value = file.exists(".") }
            debug.assert{ value = file.exists{ path = "./input.txt" } }
            debug.assert{ value = not file.exists("./missing.txt") }
            file.list_dir{ path = ".", recursive = true }
            file.read{ path = "./input.txt" }
            file.read{ path = "./input.bin", byte = true, event_tip = "read-binary" }
            file.write{ path = "./output.txt", text = "output" }
            file.write{
              path = "./output.bin",
              text = "a\0\255b",
              byte = true,
              event_tip = "write-binary",
            }
            file.create_dir{ path = "./created/nested/leaf", event_tip = "created" }
            file.remove{ path = "./input.txt", event_tip = "removed" }
            local empty = debug.pcall{
              func = function() file.list_dir{ path = "" } end,
            }
            local traversal = debug.pcall{
              func = function() file.list_dir{ path = "./folder/../" } end,
            }
            debug.assert{ value = not empty.ok and not traversal.ok }
          end
        "#,
      ),
    )
    .unwrap();
    let mut session = LuaSession::load_with_api(
      LuaSessionSpec {
        package_id: "test.assets_root".to_string(),
        session_kind: LuaSessionKind::Game,
        entry_path,
        fixed_delta: Duration::from_secs_f64(1.0 / 60.0),
        base_size: Size {
          width: 120,
          height: 40,
        },
        continue_data: None,
        best_data: None,
        save_game_enabled: false,
        save_best_enabled: false,
      },
      LuaPolicy::default(),
      LuaApiConfig {
        safe_mode_enabled: false,
        ..LuaApiConfig::default()
      },
    )
    .unwrap();

    let expected_root = assets_root.canonicalize().unwrap();
    let requests = session.take_host_commands();
    assert!(requests.iter().any(|command| matches!(
      command,
      LuaHostCommand::FileRequest {
        task: crate::host_engine::services::FileTask::LuaListDir { path, recursive: true, .. },
        virtual_path,
        ..
      } if path == &expected_root && virtual_path == "."
    )));
    assert!(requests.iter().any(|command| matches!(
      command,
      LuaHostCommand::FileRequest {
        task: crate::host_engine::services::FileTask::LuaReadText { path, .. },
        virtual_path,
        ..
      } if path == &expected_root.join("input.txt") && virtual_path == "input.txt"
    )));
    assert!(requests.iter().any(|command| matches!(
      command,
      LuaHostCommand::FileRequest {
        task: crate::host_engine::services::FileTask::LuaReadBytes { path },
        operation: LuaFileOperation::ReadBytes,
        virtual_path,
        event_tip: Some(event_tip),
        ..
      } if path == &expected_root.join("input.bin")
        && virtual_path == "input.bin"
        && event_tip == "read-binary"
    )));
    assert!(requests.iter().any(|command| matches!(
      command,
      LuaHostCommand::FileRequest {
        task: crate::host_engine::services::FileTask::LuaWriteText { path, .. },
        virtual_path,
        ..
      } if path == &expected_root.join("output.txt") && virtual_path == "output.txt"
    )));
    assert!(requests.iter().any(|command| matches!(
      command,
      LuaHostCommand::FileRequest {
        task: crate::host_engine::services::FileTask::LuaWriteBytes { path, bytes },
        operation: LuaFileOperation::WriteBytes,
        virtual_path,
        event_tip: Some(event_tip),
        ..
      } if path == &expected_root.join("output.bin")
        && bytes == b"a\0\xffb"
        && virtual_path == "output.bin"
        && event_tip == "write-binary"
    )));
    assert!(requests.iter().any(|command| matches!(
      command,
      LuaHostCommand::FileRequest {
        task: crate::host_engine::services::FileTask::LuaCreateDir { path, .. },
        operation: LuaFileOperation::CreateDir,
        virtual_path,
        event_tip: Some(event_tip),
        ..
      } if path == &expected_root.join("created/nested/leaf")
        && virtual_path == "created/nested/leaf"
        && event_tip == "created"
    )));
    assert!(requests.iter().any(|command| matches!(
      command,
      LuaHostCommand::FileRequest {
        task: crate::host_engine::services::FileTask::LuaRemove { path, recursive: false, .. },
        operation: LuaFileOperation::Remove,
        virtual_path,
        event_tip: Some(event_tip),
        ..
      } if path == &expected_root.join("input.txt")
        && virtual_path == "input.txt"
        && event_tip == "removed"
    )));

    fs::remove_dir_all(package_root).unwrap();
  }

  #[test]
  fn safe_mode_lab_exercises_debug_logging_and_file_write_permissions() {
    let package_root =
      PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_package/game/safe_mode_lab");
    let entry_path = package_root.join("scripts/main.lua");
    let make_spec = || LuaSessionSpec {
      package_id: "test.safe_mode_lab".to_string(),
      session_kind: LuaSessionKind::Game,
      entry_path: entry_path.clone(),
      fixed_delta: Duration::from_secs_f64(1.0 / 60.0),
      base_size: Size {
        width: 120,
        height: 40,
      },
      continue_data: None,
      best_data: None,
      save_game_enabled: true,
      save_best_enabled: true,
    };
    let action = LuaRuntimeEvent {
      sequence: 1,
      frame: 1,
      data: LuaEventData::Action {
        action: "write_probe".to_string(),
        state: super::super::LuaActionState::Pressed,
      },
    };

    let mut restricted =
      LuaSession::load_with_api(make_spec(), LuaPolicy::default(), LuaApiConfig::default())
        .unwrap();
    restricted.handle_event(&action).unwrap();
    let restricted_commands = restricted.take_host_commands();
    assert!(restricted_commands.iter().any(|command| matches!(
      command,
      LuaHostCommand::Ignored {
        method: "file.write",
        ..
      }
    )));
    assert!(
      !restricted_commands
        .iter()
        .any(|command| matches!(command, LuaHostCommand::FileRequest { .. }))
    );

    let mut permitted = LuaSession::load_with_api(
      make_spec(),
      LuaPolicy::default(),
      LuaApiConfig {
        debug_enabled: true,
        safe_mode_enabled: false,
        ..LuaApiConfig::default()
      },
    )
    .unwrap();
    permitted.handle_event(&action).unwrap();
    let permitted_commands = permitted.take_host_commands();
    assert!(
      permitted_commands
        .iter()
        .any(|command| matches!(command, LuaHostCommand::Print { .. }))
    );
    assert!(permitted_commands.iter().any(|command| matches!(
      command,
      LuaHostCommand::FileRequest {
        operation: LuaFileOperation::WriteText,
        virtual_path,
        ..
      } if virtual_path == "state/probe.log"
    )));
  }

  #[test]
  fn loader_matches_module_semantics_and_rejects_unsafe_sources() {
    let source = valid_script(
      r#"
        private_state = "main-only"
        function Init(ctx)
          local first, first_gap, first_tail = loader.require("cached")
          local second, second_gap, second_tail = loader.require{ path = "./cached.lua" }
          debug.assert{
            value = first == second and first.count == 1 and first.leaked == "main-only",
          }
          debug.assert{
            value = first_gap == nil and second_gap == nil and first_tail == 3 and second_tail == 3,
          }

          local fresh_first = loader.dofile("fresh")
          local fresh_second = loader.dofile{ path = "./fresh.lua" }
          debug.assert{
            value = fresh_first ~= fresh_second
              and fresh_first.count == 1 and fresh_second.count == 2,
          }

          local compiled_first = loader.loadfile("compiled")
          local compiled_second = loader.loadfile{ path = "compiled.lua" }
          debug.assert{
            value = type(compiled_first) == "function"
              and type(compiled_second) == "function"
              and compiled_first ~= compiled_second
              and compiled_count == nil,
          }
          local compiled_result_first = compiled_first()
          local compiled_result_second = compiled_second()
          debug.assert{
            value = compiled_result_first ~= compiled_result_second
              and compiled_result_first.count == 1 and compiled_result_second.count == 2,
          }

          local traversal_ok = debug.pcall{
            func = function() loader.require("../outside") end,
          }
          local extension_ok = debug.pcall{
            func = function() loader.dofile("module.txt") end,
          }
          local bytecode_ok = debug.pcall{
            func = function() loader.loadfile("bytecode.lua") end,
          }
          local cycle_ok = debug.pcall{
            func = function() loader.require("cycle") end,
          }
          debug.assert{
            value = not traversal_ok.ok and not extension_ok.ok and not bytecode_ok.ok
              and not cycle_ok.ok,
          }
        end
      "#,
    );
    let session_spec = spec(&source, LuaSessionKind::Game);
    let scripts_root = session_spec.entry_path.parent().unwrap();
    fs::write(
      scripts_root.join("cached.lua"),
      "required_count = (required_count or 0) + 1\nreturn { count = required_count, leaked = private_state }, nil, 3",
    )
    .unwrap();
    fs::write(
      scripts_root.join("fresh.lua"),
      "dofile_count = (dofile_count or 0) + 1\nreturn { count = dofile_count }",
    )
    .unwrap();
    fs::write(
      scripts_root.join("compiled.lua"),
      "compiled_count = (compiled_count or 0) + 1\nreturn { count = compiled_count }",
    )
    .unwrap();
    fs::write(scripts_root.join("module.txt"), "return {}").unwrap();
    fs::write(scripts_root.join("bytecode.lua"), [0x1b, b'L', b'u', b'a']).unwrap();
    fs::write(
      scripts_root.join("cycle.lua"),
      "return loader.require('cycle')",
    )
    .unwrap();

    LuaSession::load(session_spec, LuaPolicy::default()).unwrap();
  }

  #[test]
  fn rejects_missing_required_callback() {
    let error = match LuaSession::load(
      spec("function Init(ctx) end", LuaSessionKind::Game),
      LuaPolicy::default(),
    ) {
      Ok(_) => panic!("session unexpectedly loaded"),
      Err(error) => error,
    };
    assert_eq!(error.stage, LuaErrorStage::DiscoverCallbacks);
    assert_eq!(error.callback, Some("HandleEvent"));
  }

  #[test]
  fn save_validation_accepts_json_and_rejects_cycles() {
    let source = valid_script(
      r#"
        function SaveGame()
          return { name = "save", values = { 1, 2, 3 }, enabled = true }
        end
        function SaveBest()
          local value = {}
          value.self = value
          return value
        end
      "#,
    );
    let mut session =
      LuaSession::load(spec(&source, LuaSessionKind::Game), LuaPolicy::default()).unwrap();
    let save = session.save_game().unwrap().unwrap();
    assert_eq!(save["name"], "save");
    assert_eq!(save["values"][2], 3);
    assert_eq!(
      session.save_best().unwrap_err().stage,
      LuaErrorStage::SaveValidation
    );
  }

  #[test]
  fn save_game_accepts_serializable_scalar_and_table_values() {
    for (expression, expected) in [
      ("true", JsonValue::Bool(true)),
      ("42", JsonValue::from(42)),
      ("3.5", JsonValue::from(3.5)),
      (r#""continue""#, JsonValue::from("continue")),
      (
        "{ level = 3, values = { 1, 2 } }",
        serde_json::json!({ "level": 3, "values": [1, 2] }),
      ),
    ] {
      let source = valid_script(&format!("function SaveGame() return {expression} end"));
      let mut session =
        LuaSession::load(spec(&source, LuaSessionKind::Game), LuaPolicy::default()).unwrap();
      assert_eq!(session.save_game().unwrap(), Some(expected));
    }
  }

  #[test]
  fn save_callbacks_reject_non_serializable_values_and_non_table_best() {
    let invalid_game_source = valid_script(
      r#"
        function SaveGame()
          return function() end
        end
      "#,
    );
    let mut game_session = LuaSession::load(
      spec(&invalid_game_source, LuaSessionKind::Game),
      LuaPolicy::default(),
    )
    .unwrap();

    let save_game_error = game_session.save_game().unwrap_err();
    assert_eq!(save_game_error.stage, LuaErrorStage::SaveValidation);
    assert!(
      save_game_error
        .message
        .contains("unsupported Lua type 'function'")
    );
    assert_eq!(game_session.state(), LuaSessionState::Faulted);

    let invalid_best_source = valid_script(
      r#"
        function SaveBest()
          return "best"
        end
      "#,
    );
    let mut best_session = LuaSession::load(
      spec(&invalid_best_source, LuaSessionKind::Game),
      LuaPolicy::default(),
    )
    .unwrap();
    let save_best_error = best_session.save_best().unwrap_err();
    assert_eq!(save_best_error.stage, LuaErrorStage::SaveValidation);
    assert!(
      save_best_error
        .message
        .contains("must return a serializable table")
    );
    assert_eq!(best_session.state(), LuaSessionState::Faulted);
  }

  #[test]
  fn enabled_save_callbacks_are_required_and_best_needs_best_string() {
    let source = valid_script("");
    let mut game_save_spec = spec(&source, LuaSessionKind::Game);
    game_save_spec.save_game_enabled = true;
    let error = LuaSession::load(game_save_spec, LuaPolicy::default())
      .err()
      .expect("SaveGame should be required");
    assert_eq!(error.stage, LuaErrorStage::DiscoverCallbacks);
    assert_eq!(error.callback, Some("SaveGame"));

    let source = valid_script("function SaveGame() return {} end");
    let mut best_spec = spec(&source, LuaSessionKind::Game);
    best_spec.save_game_enabled = true;
    best_spec.save_best_enabled = true;
    let error = LuaSession::load(best_spec, LuaPolicy::default())
      .err()
      .expect("SaveBest should be required");
    assert_eq!(error.stage, LuaErrorStage::DiscoverCallbacks);
    assert_eq!(error.callback, Some("SaveBest"));

    let source = valid_script(
      "function SaveGame() return {} end\nfunction SaveBest() return { score = 1 } end",
    );
    let mut invalid_best_spec = spec(&source, LuaSessionKind::Game);
    invalid_best_spec.save_game_enabled = true;
    invalid_best_spec.save_best_enabled = true;
    let mut session = LuaSession::load(invalid_best_spec, LuaPolicy::default()).unwrap();
    let error = session.save_best().unwrap_err();
    assert_eq!(error.stage, LuaErrorStage::SaveValidation);
    assert!(error.message.contains("best_string"));
  }

  #[test]
  fn instruction_budget_faults_an_infinite_update() {
    let source = valid_script("function Update(dt) while true do end end");
    let mut session =
      LuaSession::load(spec(&source, LuaSessionKind::Game), LuaPolicy::default()).unwrap();
    let error = session.update().unwrap_err();
    assert_eq!(error.stage, LuaErrorStage::ExecutionLimit);
    assert!(
      error
        .message
        .contains("instructions execution limit exceeded")
    );
    assert!(error.message.contains("instruction_limit=200000"));
    assert!(error.message.contains("time_limit_ms=75.000"));
    assert_eq!(session.state(), LuaSessionState::Faulted);
    assert!(!session.has_objects());
  }

  #[test]
  fn instruction_budget_cannot_be_hidden_by_pcall() {
    let source = valid_script(
      "function Update(dt) debug.pcall{ func = function() while true do end end } end",
    );
    let mut session =
      LuaSession::load(spec(&source, LuaSessionKind::Game), LuaPolicy::default()).unwrap();
    let error = session.update().unwrap_err();
    assert_eq!(error.stage, LuaErrorStage::ExecutionLimit);
    assert_eq!(session.state(), LuaSessionState::Faulted);
  }

  fn install_test_sleep(session: &LuaSession, duration: Duration) {
    let sleep = session
      .lua
      .create_function(move |_, ()| {
        std::thread::sleep(duration);
        Ok(())
      })
      .unwrap();
    let environment: Table = session.lua.registry_value(&session.environment).unwrap();
    environment.set("sleep_for_test", sleep).unwrap();
  }

  #[test]
  fn rust_api_time_is_included_in_the_hard_callback_budget() {
    let source = valid_script("function Update(dt) sleep_for_test() end");
    let mut session =
      LuaSession::load(spec(&source, LuaSessionKind::Game), LuaPolicy::default()).unwrap();
    install_test_sleep(&session, Duration::from_millis(90));

    let error = session.update().unwrap_err();
    assert_eq!(error.stage, LuaErrorStage::ExecutionLimit);
    assert!(error.message.contains("time execution limit exceeded"));
    assert!(error.message.contains("time_limit_ms=75.000"));
    assert!(error.message.contains("instruction_limit=200000"));
  }

  #[test]
  fn slow_callback_warnings_require_debug_mode() {
    let source = valid_script("function Update(dt) sleep_for_test() end");
    let mut debug_session = LuaSession::load_with_api(
      spec(&source, LuaSessionKind::Game),
      LuaPolicy::default(),
      LuaApiConfig {
        debug_enabled: true,
        ..LuaApiConfig::default()
      },
    )
    .unwrap();
    install_test_sleep(&debug_session, Duration::from_millis(25));
    debug_session.update().unwrap();
    assert!(
      debug_session
        .take_host_commands()
        .iter()
        .any(|command| matches!(
          command,
          LuaHostCommand::Log { level, message }
            if level == "warn" && message.contains("callback=Update")
              && message.contains("warn_ms=20.000") && message.contains("hard_ms=75.000")
        ))
    );

    let mut release_session =
      LuaSession::load(spec(&source, LuaSessionKind::Game), LuaPolicy::default()).unwrap();
    install_test_sleep(&release_session, Duration::from_millis(25));
    release_session.update().unwrap();
    assert!(
      !release_session
        .take_host_commands()
        .iter()
        .any(|command| matches!(
          command,
          LuaHostCommand::Log { message, .. } if message.contains("slow Lua callback")
        ))
    );
  }

  #[test]
  fn slow_callback_warnings_are_rate_limited_per_callback() {
    let source = valid_script("");
    let mut session = LuaSession::load_with_api(
      spec(&source, LuaSessionKind::Game),
      LuaPolicy::default(),
      LuaApiConfig {
        debug_enabled: true,
        ..LuaApiConfig::default()
      },
    )
    .unwrap();
    let _ = session.take_host_commands();
    let budget = session.policy.budget(LuaBudgetKind::Render);
    let stats = LuaExecutionStats {
      instructions: 4_000,
      elapsed: Duration::from_millis(21),
      memory_bytes: session.memory_used(),
    };
    let now = Instant::now();
    session.record_slow_callback_at(
      "Update",
      budget,
      LuaExecutionStats {
        elapsed: Duration::from_millis(19),
        ..stats
      },
      now,
    );
    session.record_slow_callback_at("Render", budget, stats, now);
    session.record_slow_callback_at("Render", budget, stats, now + Duration::from_secs(1));
    session.record_slow_callback_at("Render", budget, stats, now + Duration::from_secs(5));

    let warnings = session
      .take_host_commands()
      .into_iter()
      .filter_map(|command| match command {
        LuaHostCommand::Log { message, .. } if message.contains("slow Lua callback") => {
          Some(message)
        }
        _ => None,
      })
      .collect::<Vec<_>>();
    assert_eq!(warnings.len(), 2);
    assert!(warnings[1].contains("suppressed=1"));
  }

  #[test]
  fn ascii_text_rendering_stays_within_the_callback_budget() {
    let source = valid_script(
      r#"
        function Render()
          local x = 0
          local y = 0
          for item in ipairs(char.ASCII) do
            x = x + 1
            if x > 20 then
              x = 1
              y = y + 1
            end
            draw.text{ x = x, y = y, text = item.value }
          end
        end
      "#,
    );
    let mut session =
      LuaSession::load(spec(&source, LuaSessionKind::Game), LuaPolicy::default()).unwrap();
    session.render().unwrap();
    assert!(session.take_draw_commands().len() >= 90);
  }

  #[test]
  fn lifecycle_context_events_and_draw_are_stable() {
    let source = r#"
      local calls = {}
      local context = nil
      local event_value = nil
      local render_argument = false

      function Init(ctx)
        context = ctx
        calls[#calls + 1] = "Init"
      end
      function HandleEvent(event)
        event_value = event
        calls[#calls + 1] = "HandleEvent"
      end
      function Update(dt)
        calls[#calls + 1] = "Update"
      end
      function UpdateFrame(dt, alpha)
        calls[#calls + 1] = "UpdateFrame"
      end
      function Render(value)
        render_argument = value ~= nil
        calls[#calls + 1] = "Render"
      end
      function SaveGame()
        return {
          calls = calls,
          package_id = context.package_id,
          package_type = context.package_type,
          base_width = context.base.width,
          base_height = context.base.height,
          start_mode = context.start_mode,
          has_fixed_delta = context.fixed_delta ~= nil,
          has_terminal = context.terminal ~= nil,
          api_version = context.api_version,
          continue_level = context.continue_data.level,
          best_score = context.best_data.score,
          best_string = context.best_data.best_string,
          event_type = event_value.type,
          event_sequence = event_value.sequence,
          event_frame = event_value.frame,
          event_action = event_value.data.action,
          event_state = event_value.data.state,
          render_argument = render_argument
        }
      end
    "#;
    let mut session_spec = spec(source, LuaSessionKind::Game);
    session_spec.continue_data = Some(serde_json::json!({"level": 4}));
    session_spec.best_data = Some(serde_json::json!({
      "best_string": "Best: 12",
      "score": 12
    }));
    let mut session = LuaSession::load(session_spec, LuaPolicy::default()).unwrap();
    session
      .handle_event(&LuaRuntimeEvent {
        sequence: 9,
        frame: 15,
        data: LuaEventData::Action {
          action: "jump".to_string(),
          state: super::super::LuaActionState::Pressed,
        },
      })
      .unwrap();
    session.update().unwrap();
    session
      .update_frame(Duration::from_millis(16), 0.5)
      .unwrap();
    session.render().unwrap();

    let save = session.save_game().unwrap().unwrap();
    assert_eq!(
      save["calls"],
      serde_json::json!(["Init", "HandleEvent", "Update", "UpdateFrame", "Render"])
    );
    assert_eq!(save["package_id"], "test.package");
    assert_eq!(save["package_type"], "game");
    assert_eq!(save["base_width"], 120);
    assert_eq!(save["base_height"], 40);
    assert_eq!(save["start_mode"], "continue");
    assert_eq!(save["has_fixed_delta"], false);
    assert_eq!(save["has_terminal"], false);
    assert_eq!(save["api_version"], 1);
    assert_eq!(save["continue_level"], 4);
    assert_eq!(save["best_score"], 12);
    assert_eq!(save["best_string"], "Best: 12");
    assert_eq!(save["event_type"], "action");
    assert_eq!(save["event_sequence"], 9);
    assert_eq!(save["event_frame"], 15);
    assert_eq!(save["event_action"], "jump");
    assert_eq!(save["event_state"], "pressed");
    assert_eq!(save["render_argument"], false);
  }

  #[test]
  fn init_context_uses_new_mode_and_omits_missing_optional_data() {
    let source = valid_script(
      r#"
        function SaveGame()
          return {
            start_mode = init_ctx.start_mode,
            base_width = init_ctx.base.width,
            base_height = init_ctx.base.height,
            has_best_data = init_ctx.best_data ~= nil,
            has_continue_data = init_ctx.continue_data ~= nil,
            has_terminal = init_ctx.terminal ~= nil,
            has_fixed_delta = init_ctx.fixed_delta ~= nil,
          }
        end
      "#,
    );
    let mut session =
      LuaSession::load(spec(&source, LuaSessionKind::Game), LuaPolicy::default()).unwrap();
    let save = session.save_game().unwrap().unwrap();
    assert_eq!(save["start_mode"], "new");
    assert_eq!(save["base_width"], 120);
    assert_eq!(save["base_height"], 40);
    assert_eq!(save["has_best_data"], false);
    assert_eq!(save["has_continue_data"], false);
    assert_eq!(save["has_terminal"], false);
    assert_eq!(save["has_fixed_delta"], false);
  }

  #[test]
  fn registered_callback_receives_the_envelope_without_calling_handle_event() {
    let source = valid_script(
      r#"
        handle_count = 0
        callback_count = 0
        function HandleEvent(event)
          handle_count = handle_count + 1
        end
        function ServiceCallback(event)
          callback_count = callback_count + 1
          callback_type = event.type
          callback_sequence = event.sequence
          callback_frame = event.frame
          callback_gained = event.data.gained
        end
      "#,
    );
    let mut session =
      LuaSession::load(spec(&source, LuaSessionKind::Game), LuaPolicy::default()).unwrap();
    let callback =
      session.register_environment_event_callback("ServiceCallback", LuaCallbackLifetime::Once);
    let delivery = LuaEventDelivery {
      event: LuaRuntimeEvent {
        sequence: 17,
        frame: 29,
        data: LuaEventData::Focus { gained: false },
      },
      route: LuaEventRoute::Callback(callback),
    };

    session.dispatch_event(&delivery).unwrap();
    assert!(session.event_callbacks.is_empty());
    // 一次性回调已回收；重复完成事件不能转投 HandleEvent。
    session.dispatch_event(&delivery).unwrap();

    assert_eq!(session.environment_value("handle_count"), Value::Integer(0));
    assert_eq!(
      session.environment_value("callback_count"),
      Value::Integer(1)
    );
    assert_eq!(
      session.environment_value("callback_type"),
      Value::String(session.lua.create_string("focus").unwrap())
    );
    assert_eq!(
      session.environment_value("callback_sequence"),
      Value::Integer(17)
    );
    assert_eq!(
      session.environment_value("callback_frame"),
      Value::Integer(29)
    );
    assert_eq!(
      session.environment_value("callback_gained"),
      Value::Boolean(false)
    );
  }

  #[test]
  fn persistent_callback_is_removed_after_its_terminal_event() {
    let source = valid_script(
      r#"
        callback_count = 0
        function AnimationCallback(event)
          callback_count = callback_count + 1
        end
      "#,
    );
    let mut session =
      LuaSession::load(spec(&source, LuaSessionKind::Game), LuaPolicy::default()).unwrap();
    let callback = session
      .register_environment_event_callback("AnimationCallback", LuaCallbackLifetime::UntilTerminal);
    let delivery = |kind| LuaEventDelivery {
      event: LuaRuntimeEvent {
        sequence: 1,
        frame: 1,
        data: LuaEventData::Animation(super::super::LuaAnimationEvent { id: 4, kind }),
      },
      route: LuaEventRoute::Callback(callback),
    };

    session
      .dispatch_event(&delivery(super::super::LuaAnimationEventKind::Marker {
        name: "half".to_string(),
      }))
      .unwrap();
    session
      .dispatch_event(&delivery(super::super::LuaAnimationEventKind::Finished))
      .unwrap();
    assert!(session.event_callbacks.is_empty());
    session
      .dispatch_event(&delivery(super::super::LuaAnimationEventKind::Finished))
      .unwrap();

    assert_eq!(
      session.environment_value("callback_count"),
      Value::Integer(2)
    );
  }

  #[test]
  fn event_callback_uses_the_handle_event_execution_budget() {
    let source = valid_script(
      r#"
        function BusyCallback(event)
          while true do end
        end
      "#,
    );
    let mut session =
      LuaSession::load(spec(&source, LuaSessionKind::Game), LuaPolicy::default()).unwrap();
    let callback =
      session.register_environment_event_callback("BusyCallback", LuaCallbackLifetime::Once);
    let error = session
      .dispatch_event(&LuaEventDelivery {
        event: LuaRuntimeEvent {
          sequence: 1,
          frame: 1,
          data: LuaEventData::Focus { gained: true },
        },
        route: LuaEventRoute::Callback(callback),
      })
      .unwrap_err();

    assert_eq!(error.stage, LuaErrorStage::ExecutionLimit);
    assert_eq!(error.callback, Some("EventCallback"));
    assert_eq!(session.state(), LuaSessionState::Faulted);
  }

  #[test]
  fn test_package_entries_execute_the_basic_lifecycle() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for (directory, session_kind) in [
      ("test_package/game", LuaSessionKind::Game),
      ("test_package/screensaver", LuaSessionKind::Screensaver),
    ] {
      let package_root = manifest_dir.join(directory);
      let mut package_count = 0;

      for entry in fs::read_dir(&package_root).unwrap() {
        let entry = entry.unwrap();
        if !entry.file_type().unwrap().is_dir() {
          continue;
        }
        package_count += 1;

        let package_id = entry.file_name().to_string_lossy().into_owned();
        let entry_path = entry.path().join("scripts").join("main.lua");
        let mut session = LuaSession::load(
          LuaSessionSpec {
            package_id: package_id.clone(),
            session_kind,
            entry_path,
            fixed_delta: Duration::from_secs_f64(1.0 / 60.0),
            base_size: Size {
              width: 120,
              height: 40,
            },
            continue_data: None,
            best_data: None,
            save_game_enabled: session_kind == LuaSessionKind::Game,
            save_best_enabled: session_kind == LuaSessionKind::Game,
          },
          LuaPolicy::default(),
        )
        .unwrap_or_else(|error| panic!("{package_id} failed to load: {error}"));

        session
          .handle_event(&LuaRuntimeEvent {
            sequence: 1,
            frame: 1,
            data: LuaEventData::Resize {
              width: 100,
              height: 30,
            },
          })
          .unwrap_or_else(|error| panic!("{package_id} HandleEvent failed: {error}"));
        session
          .update()
          .unwrap_or_else(|error| panic!("{package_id} Update failed: {error}"));
        session
          .update_frame(Duration::from_millis(16), 0.5)
          .unwrap_or_else(|error| panic!("{package_id} UpdateFrame failed: {error}"));
        session
          .render()
          .unwrap_or_else(|error| panic!("{package_id} Render failed: {error}"));

        if session_kind == LuaSessionKind::Game {
          assert!(
            session.save_game().unwrap().is_some(),
            "{package_id} SaveGame returned no value"
          );
          assert!(
            session.save_best().unwrap().is_some(),
            "{package_id} SaveBest returned no value"
          );
        }
      }

      assert_eq!(
        package_count, 3,
        "expected three test packages in {directory}"
      );
    }
  }

  #[test]
  fn source_limits_and_utf8_are_checked_before_vm_creation() {
    let mut policy = LuaPolicy::default();
    policy.source_limit_bytes = 16;
    let too_large = match LuaSession::load(spec(&valid_script(""), LuaSessionKind::Game), policy) {
      Ok(_) => panic!("oversized source unexpectedly loaded"),
      Err(error) => error,
    };
    assert_eq!(too_large.stage, LuaErrorStage::ReadSource);

    let path = script_path("");
    fs::write(&path, [0xff, 0xfe, 0xfd]).unwrap();
    let invalid_utf8 = match LuaSession::load(
      LuaSessionSpec {
        package_id: "invalid.utf8".to_string(),
        session_kind: LuaSessionKind::Game,
        entry_path: path,
        fixed_delta: Duration::from_secs_f64(1.0 / 60.0),
        base_size: Size {
          width: 80,
          height: 24,
        },
        continue_data: None,
        best_data: None,
        save_game_enabled: false,
        save_best_enabled: false,
      },
      LuaPolicy::default(),
    ) {
      Ok(_) => panic!("non-UTF-8 source unexpectedly loaded"),
      Err(error) => error,
    };
    assert_eq!(invalid_utf8.stage, LuaErrorStage::ReadSource);
  }

  #[test]
  fn invalid_policy_and_non_lua_entries_are_rejected_before_vm_creation() {
    let mut policy = LuaPolicy::default();
    policy.hook_interval = 0;
    let invalid_policy =
      match LuaSession::load(spec(&valid_script(""), LuaSessionKind::Game), policy) {
        Ok(_) => panic!("invalid policy unexpectedly loaded"),
        Err(error) => error,
      };
    assert_eq!(invalid_policy.stage, LuaErrorStage::ValidatePolicy);

    let path = script_path(&valid_script(""));
    let text_path = path.with_extension("txt");
    fs::rename(path, &text_path).unwrap();
    let non_lua = match LuaSession::load(
      LuaSessionSpec {
        package_id: "invalid.extension".to_string(),
        session_kind: LuaSessionKind::Game,
        entry_path: text_path,
        fixed_delta: Duration::from_secs_f64(1.0 / 60.0),
        base_size: Size {
          width: 80,
          height: 24,
        },
        continue_data: None,
        best_data: None,
        save_game_enabled: false,
        save_best_enabled: false,
      },
      LuaPolicy::default(),
    ) {
      Ok(_) => panic!("non-Lua entry unexpectedly loaded"),
      Err(error) => error,
    };
    assert_eq!(non_lua.stage, LuaErrorStage::ReadSource);
  }

  #[test]
  fn continue_data_obeys_save_size_and_depth_limits() {
    let mut oversized_spec = spec(&valid_script(""), LuaSessionKind::Game);
    oversized_spec.continue_data = Some(serde_json::json!({ "value": "too large" }));
    let mut policy = LuaPolicy::default();
    policy.save_limit_bytes = 4;
    let oversized = match LuaSession::load(oversized_spec, policy) {
      Ok(_) => panic!("oversized continue data unexpectedly loaded"),
      Err(error) => error,
    };
    assert_eq!(oversized.stage, LuaErrorStage::ContinueDataValidation);

    let mut deep_spec = spec(&valid_script(""), LuaSessionKind::Game);
    deep_spec.continue_data = Some(serde_json::json!({ "a": { "b": { "c": true } } }));
    let mut policy = LuaPolicy::default();
    policy.save_max_depth = 2;
    let too_deep = match LuaSession::load(deep_spec, policy) {
      Ok(_) => panic!("deep continue data unexpectedly loaded"),
      Err(error) => error,
    };
    assert_eq!(too_deep.stage, LuaErrorStage::ContinueDataValidation);
  }

  #[test]
  fn memory_limit_faults_only_the_current_session() {
    let source = valid_script(
      "function Update(dt) local value = string.rep{ text = 'x', times = 1024 * 1024 } end",
    );
    let mut session =
      LuaSession::load(spec(&source, LuaSessionKind::Game), LuaPolicy::default()).unwrap();
    session
      .lua
      .set_memory_limit(session.lua.used_memory() + 128 * 1024)
      .unwrap();
    let error = session.update().unwrap_err();
    assert_eq!(error.stage, LuaErrorStage::MemoryLimit);
    assert_eq!(session.state(), LuaSessionState::Faulted);
  }
}
