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

use tg_core_version::HOST_API_VERSION;
use tg_service_layout::Size;

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

#[derive(Clone, Debug)]
pub struct LuaSessionError {
  pub package_id: String,
  pub session_kind: LuaSessionKind,
  pub stage: LuaErrorStage,
  pub callback: Option<&'static str>,
  pub message: String,
  /// 会话完成注册前发生故障时，已成功产生且允许提交的诊断命令。
  pub diagnostic_commands: Vec<LuaHostCommand>,
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
  #[cfg(test)]
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
  #[cfg(test)]
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
    let lua = Lua::new_with(StdLib::TABLE, LuaOptions::default())
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
    #[cfg(test)]
    let environment_key = lua
      .create_registry_value(environment)
      .map_err(|error| session_error(&spec, LuaErrorStage::BuildSandbox, None, error))?;

    let mut session = Self {
      callbacks,
      #[cfg(test)]
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
      #[cfg(test)]
      next_event_callback_id: 1,
      api_state,
      slow_callback_warnings: HashMap::new(),
    };
    let load_budget = session.policy.budget(LuaBudgetKind::Load);
    session.record_slow_callback("Load", load_budget, load_stats);
    let context = session.context_table(spec.continue_data.as_ref(), spec.best_data.as_ref())?;
    if let Err(mut error) = session.invoke_required(
      Callback::Init,
      context,
      LuaBudgetKind::Init,
      LuaErrorStage::Callback,
    ) {
      error.diagnostic_commands = session.take_pending_diagnostic_commands();
      return Err(error);
    }
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
    key_actions: HashMap<String, Vec<Vec<String>>>,
    key_default_actions: HashMap<String, Vec<Vec<String>>>,
  ) {
    let mut state = self.api_state.borrow_mut();
    state.context.debug_enabled = debug_enabled;
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
    if let super::LuaEventData::Image(event) = &delivery.event.data {
      api::apply_image_event(&self.api_state, event);
    }
    match delivery.route {
      LuaEventRoute::HandleEvent => self.handle_event(&delivery.event),
      LuaEventRoute::Callback(callback) => self.invoke_event_callback(callback, &delivery.event),
    }
  }

  #[cfg(test)]
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

  fn take_pending_diagnostic_commands(&mut self) -> Vec<LuaHostCommand> {
    let mut state = self.api_state.borrow_mut();
    let commands = std::mem::take(&mut state.commands);
    commands
      .into_iter()
      .filter(|command| {
        matches!(
          command,
          LuaHostCommand::Log { .. }
            | LuaHostCommand::Print { .. }
            | LuaHostCommand::Ignored { .. }
        )
      })
      .collect()
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
      diagnostic_commands: Vec::new(),
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
  if result.is_err() {
    // lua_resume 不会自行展开一个无恢复点的失败协程。Lua 5.4 的 reset
    // 会关闭待关闭变量，并把原始错误作为 __close 的第二个参数传入。
    // Hook 此时仍然安装，关闭元方法继续受当前回调预算约束。
    if let Ok(noop) = lua.create_function(|_, _: MultiValue| Ok(MultiValue::new())) {
      // Lua 5.4 在清理一个失败线程时会再次返回该线程原本的错误；这里
      // 只需要它完成栈展开，面向调用者仍保留 resume 取得的原始错误。
      let _ = thread.reset(noop);
    }
  }
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
    diagnostic_commands: Vec::new(),
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

  use crate::LuaFileOperation;
  use crate::{LuaEventBroker, LuaSessionToken, LuaTaskOperation};
  use tg_service_async::{AsyncRuntime, TaskStatusEvent};
  use tg_service_image::{ImageEvent, ImageService};
  use tg_service_widget::SliceId;

  use super::super::LuaEventData;
  use super::*;

  static TEST_ID: AtomicU64 = AtomicU64::new(1);

  enum ImageRuntimeEvent {
    Image(ImageEvent),
    TaskStatus(TaskStatusEvent),
  }

  impl From<ImageEvent> for ImageRuntimeEvent {
    fn from(event: ImageEvent) -> Self {
      Self::Image(event)
    }
  }

  impl From<TaskStatusEvent> for ImageRuntimeEvent {
    fn from(event: TaskStatusEvent) -> Self {
      Self::TaskStatus(event)
    }
  }

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
      package_id: "test_package".to_string(),
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
  fn lua_compatibility_baseline_runs_the_same_sample_on_lua_54() {
    let sample = valid_script(
      r##"
        local sequence = { 7, 11 }
        local ipairs_iterator, ipairs_state, ipairs_control = ipairs(sequence)
        local ipairs_first, ipairs_second = ipairs_iterator(ipairs_state, ipairs_control)
        local pairs_iterator, pairs_state, pairs_control = pairs(sequence)
        local pairs_first, pairs_second = pairs_iterator(pairs_state, pairs_control)

        compatibility = {
          ipairs_state_is_input = ipairs_state == sequence,
          ipairs_first = ipairs_first,
          ipairs_second = ipairs_second,
          ipairs_second_is_nil = ipairs_second == nil,
          pairs_state_is_input = pairs_state == sequence,
          pairs_second_is_nil = pairs_second == nil,
        }
      "##,
    );

    let baseline = Lua::new_with(StdLib::ALL_SAFE, LuaOptions::default()).unwrap();
    baseline.load(&sample).exec().unwrap();
    let baseline_version = baseline.globals().get::<String>("_VERSION").unwrap();
    let baseline_result = baseline.globals().get::<Value>("compatibility").unwrap();

    let session =
      LuaSession::load(spec(&sample, LuaSessionKind::Game), LuaPolicy::default()).unwrap();
    let host_version = session.lua.globals().get::<String>("_VERSION").unwrap();
    let host_result = session.environment_value("compatibility");

    assert_eq!(baseline_version, host_version);
    assert!(
      host_version.contains("5.4"),
      "unexpected Lua version: {host_version}"
    );
    assert_eq!(
      lua_to_json(baseline_result, 0, 32, &mut HashSet::new()).unwrap(),
      serde_json::json!({
        "ipairs_state_is_input": true,
        "ipairs_first": 1,
        "ipairs_second": 7,
        "ipairs_second_is_nil": false,
        "pairs_state_is_input": true,
        "pairs_second_is_nil": false,
      })
    );
    assert_eq!(
      lua_to_json(host_result, 0, 32, &mut HashSet::new()).unwrap(),
      serde_json::json!({
        "ipairs_state_is_input": true,
        "ipairs_first": 1,
        "ipairs_second": 7,
        "ipairs_second_is_nil": false,
        "pairs_state_is_input": true,
        "pairs_second_is_nil": false,
      })
    );
  }

  #[test]
  fn standard_base_functions_follow_lua54_table_and_vararg_semantics() {
    let source = valid_script(
      r##"
        function Init(ctx)
          local sequence = { "first", "second", table = "ordinary field", value = 9 }
          local ipairs_iterator, ipairs_state, ipairs_control = ipairs(sequence)
          local first_key, first_value = ipairs_iterator(ipairs_state, ipairs_control)
          local second_key, second_value = ipairs_iterator(ipairs_state, first_key)
          local end_key = ipairs_iterator(ipairs_state, second_key)

          local copied = {}
          for key, value in pairs(sequence) do copied[key] = value end

          local named = { table = "ordinary field" }
          local named_key, named_value = next(named)
          local named_pairs = {}
          for key, value in pairs(named) do named_pairs[key] = value end

          local count = select("#", "first", nil, "third")
          local second, third = select(2, "first", nil, "third")
          local negative = select(-1, "first", nil, "third")
          local cyclic = { "cycle is valid" }
          cyclic.self = cyclic
          local cycle_iterator, cycle_state, cycle_control = ipairs(cyclic)
          local cycle_key, cycle_value = cycle_iterator(cycle_state, cycle_control)

          local custom = setmetatable({ left = 1, right = 2 }, {
            __pairs = function(subject)
              local position = 0
              return function(state, previous)
                position = position + 1
                if position == 1 then return "right", state.right end
                if position == 2 then return "left", state.left end
              end, subject, "start"
            end,
          })
          local custom_iterator, custom_state, custom_control = pairs(custom)
          local custom_key, custom_value = custom_iterator(custom_state, custom_control)

          local record = { index = 1, value = "record" }
          local text = tostring({ value = "ordinary field" })
          local table_text = string.find(text, "^table: 0x") ~= nil
          debug.assert(ipairs_state == sequence and ipairs_control == 0
              and first_key == 1 and first_value == "first"
              and second_key == 2 and second_value == "second" and end_key == nil
              and copied["table"] == "ordinary field" and copied.value == 9
              and named_key == "table" and named_value == "ordinary field"
              and named_pairs.table == "ordinary field"
              and count == 3 and second == nil and third == "third" and negative == "third"
              and cycle_state == cyclic and cycle_key == 1 and cycle_value == "cycle is valid"
              and custom_state.left == 1 and custom_control == "start"
              and custom_key == "right" and custom_value == 2
              and type(record) == "table" and rawlen({ value = 1 }) == 0
              and rawequal(record, record) and table_text)
        end
      "##,
    );
    LuaSession::load(spec(&source, LuaSessionKind::Game), LuaPolicy::default()).unwrap();
  }

  #[test]
  fn base_library_matches_native_lua54_on_plain_table_operations() {
    let source = valid_script(
      r##"
        local sequence = { "first", "second" }
        local ipairs_iterator, ipairs_state, ipairs_control = ipairs(sequence)
        local first_index, first_value = ipairs_iterator(ipairs_state, ipairs_control)
        local second_index, second_value = ipairs_iterator(ipairs_state, first_index)
        local pair_table = { table = "ordinary field" }
        local raw_target = setmetatable({}, { __index = { fallback = "inherited" } })
        local raw_write_result = rawset(raw_target, "value", 8)
        local next_key, next_value = next(pair_table)
        local pairs_iterator, pairs_state, pairs_control = pairs(pair_table)
        local pairs_key, pairs_value = pairs_iterator(pairs_state, pairs_control)
        local count = select("#", "first", nil, pair_table, nil)
        local selected, selected_tail = select(3, "first", nil, pair_table, nil)
        local negative = select(-1, "first", nil, "last")
        local meta = { __index = { fallback = 7 }, __metatable = "locked" }
        local protected = setmetatable({}, meta)
        local custom = setmetatable({ value = 4 }, {
          __pairs = function(subject)
            return next, subject, nil
          end,
        })
        local custom_iterator, custom_state, custom_control = pairs(custom)
        local custom_key, custom_value = custom_iterator(custom_state, custom_control)

        compatibility_base = {
          ipairs_state_is_input = ipairs_state == sequence,
          ipairs_control = ipairs_control,
          ipairs_first_index = first_index,
          ipairs_first_value = first_value,
          ipairs_second_index = second_index,
          ipairs_second_value = second_value,
          next_key = next_key,
          next_value = next_value,
          pairs_state_is_input = pairs_state == pair_table,
          pairs_key = pairs_key,
          pairs_value = pairs_value,
          select_count = count,
          select_preserved_table = selected == pair_table,
          select_preserved_trailing_nil = selected_tail == nil,
          select_negative = negative,
          rawget_skips_index = rawget(raw_target, "fallback") == nil
            and raw_target.fallback == "inherited",
          rawset_returns_table = raw_write_result == raw_target,
          rawset_uses_raw_slot = rawget(raw_target, "value") == 8,
          rawset_handles_named_table_field = rawset(pair_table, "table", "updated") == pair_table
            and rawget(pair_table, "table") == "updated",
          rawset_nil_removes_slot = rawset(pair_table, "table", nil) == pair_table
            and rawget(pair_table, "table") == nil,
          type_of_named_table = type({ index = 1, value = 2 }),
          type_of_float = type(1.5),
          tonumber_decimal = tonumber("12.5"),
          tonumber_base = tonumber("ff", 16),
          tonumber_hex_float = tonumber("0x1p2"),
          tonumber_invalid_hex_exponent = tonumber("0x1pfoo") == nil,
          tostring_integer = tostring(12),
          tostring_string = tostring("text"),
          rawlen_named_table = rawlen({ value = 1 }),
          rawequal_numeric = rawequal(1, 1.0),
          protected_marker = getmetatable(protected),
          protected_index = protected.fallback,
          custom_state_is_input = custom_state == custom,
          custom_key = custom_key,
          custom_value = custom_value,
          setmetatable_nil_removes_metatable = (function()
            local removable = setmetatable({}, { __index = { fallback = 7 } })
            local result = setmetatable(removable, nil)
            return result == removable and getmetatable(removable) == nil
              and removable.fallback == nil
          end)(),
          number_metatable = getmetatable(1),
        }
      "##,
    );
    let baseline = Lua::new_with(StdLib::ALL_SAFE, LuaOptions::default()).unwrap();
    baseline.load(&source).exec().unwrap();
    let baseline_result = baseline
      .globals()
      .get::<Value>("compatibility_base")
      .unwrap();

    let session =
      LuaSession::load(spec(&source, LuaSessionKind::Game), LuaPolicy::default()).unwrap();
    let host_result = session.environment_value("compatibility_base");

    assert_eq!(
      lua_to_json(baseline_result, 0, 32, &mut HashSet::new()).unwrap(),
      lua_to_json(host_result, 0, 32, &mut HashSet::new()).unwrap()
    );
  }

  #[test]
  fn table_standard_library_matches_native_lua54_behavior() {
    let source = valid_script(
      r##"
        local sequence = { "a", "c" }
        table.insert(sequence, 2, "b")
        table.insert(sequence, "d")
        local removed = table.remove(sequence, 2)

        local moved = { 1, 2, 3, 4 }
        local move_result = table.move(moved, 1, 3, 2)
        local target = {}
        local target_result = table.move(moved, 2, 3, 1, target)

        local packed = table.pack("first", nil, "last", nil)
        local first, second, third, fourth = table.unpack(packed, 1, packed.n)
        local virtual = setmetatable({ [1] = "left" }, {
          __index = function(_, index)
            if index == 2 then return "right" end
          end,
          __len = function() return 2 end,
        })
        local values = {
          { name = "beta" },
          { name = "alpha" },
        }
        setmetatable(values[1], { __lt = function(left, right) return left.name < right.name end })
        setmetatable(values[2], { __lt = function(left, right) return left.name < right.name end })
        table.sort(values)

        compatibility_table = {
          concat = table.concat({ 1, "二", 3 }, "|", 1, 3),
          concat_uses_index_and_length = table.concat(virtual, ","),
          insert_remove = table.concat(sequence, "") .. ":" .. removed,
          move_returns_source = move_result == moved,
          move_overlap = table.concat(moved, ","),
          move_returns_target = target_result == target,
          move_target = table.concat(target, ","),
          packed_count = packed.n,
          packed_nil_positions = first == "first" and second == nil
            and third == "last" and fourth == nil,
          sort_uses_less_metamethod = values[1].name == "alpha"
            and values[2].name == "beta",
        }
      "##,
    );
    let baseline = Lua::new_with(StdLib::ALL_SAFE, LuaOptions::default()).unwrap();
    baseline.load(&source).exec().unwrap();
    let baseline_result = baseline
      .globals()
      .get::<Value>("compatibility_table")
      .unwrap();

    let session =
      LuaSession::load(spec(&source, LuaSessionKind::Game), LuaPolicy::default()).unwrap();
    let host_result = session.environment_value("compatibility_table");

    assert_eq!(
      lua_to_json(baseline_result, 0, 32, &mut HashSet::new()).unwrap(),
      lua_to_json(host_result, 0, 32, &mut HashSet::new()).unwrap()
    );
  }

  #[test]
  fn lua54_string_baseline_records_byte_indices_and_native_returns() {
    let baseline = Lua::new_with(StdLib::ALL_SAFE, LuaOptions::default()).unwrap();
    let result = baseline
      .load(
        r#"
          local find_start, find_finish, find_capture = string.find("A你B", "(你)")
          local match_first, match_second = string.match("id=7", "(%a+)=(%d+)")
          local iterator = string.gmatch("a1 b2", "(%a)(%d)")
          local first_letter, first_digit = iterator()
          local second_letter, second_digit = iterator()
          local replaced, replacement_count = string.gsub("a1 b2", "(%a)(%d)", "%2%1")
          local plain_start, plain_finish = string.find("x.y", ".", 1, true)
          local reversed = string.reverse("你ab")
          local byte_one, byte_two, byte_three, byte_four, byte_five = string.byte(reversed, 1, 5)

          native_string = {
            sub = string.sub("A你B", 2, 4),
            find_start = find_start,
            find_finish = find_finish,
            find_capture = find_capture,
            match_first = match_first,
            match_second = match_second,
            first_letter = first_letter,
            first_digit = first_digit,
            second_letter = second_letter,
            second_digit = second_digit,
            replaced = replaced,
            replacement_count = replacement_count,
            plain_start = plain_start,
            plain_finish = plain_finish,
            lowered_unicode = string.lower("ÄBC"),
            reversed_bytes = { byte_one, byte_two, byte_three, byte_four, byte_five },
          }

          native_format = {
            formatted = string.format("%s:%04d", "v", 7),
            general_scientific = string.format("%.3g", 12345),
            general_fixed = string.format("%.3g", 12.5),
            general_small = string.format("%.3g", 0.0000125),
            general_rounds_to_scientific = string.format("%.3g", 999.9),
            general_rounds_to_fixed = string.format("%.3g", 0.00009999),
            general_alternate = string.format("%#.3g", 12.0),
            scientific = string.format("%.2e", 12.5),
            scientific_upper = string.format("%.2E", 12.5),
            fixed_infinity = string.format("%f", 1 / 0),
            fixed_nan = string.format("%f", 0 / 0),
            fixed_negative_nan = string.format("%f", -(0 / 0)),
            fixed_negative_infinity = string.format("%+f", -1 / 0),
            quoted_nul = string.format("%q", "a\0b"),
            quoted_control = string.format("%q", "\1"),
            quoted_control_digit = string.format("%q", "\1" .. "2"),
            quoted_newline = string.format("%q", "a\nb"),
            quoted_tab = string.format("%q", "\t"),
            quoted_carriage_return = string.format("%q", "\r"),
            quoted_backspace = string.format("%q", "\b"),
            quoted_zero_digit = string.format("%q", "\0" .. "2"),
            quoted_quote = string.format("%q", "\""),
            quoted_backslash = string.format("%q", "\\"),
            unicode_character = string.format("%c", 0x4f60),
          }
        "#,
      )
      .exec()
      .and_then(|()| baseline.globals().get::<Value>("native_string"))
      .unwrap();
    let format_result = baseline.globals().get::<Value>("native_format").unwrap();

    assert_eq!(
      lua_to_json(result, 0, 32, &mut HashSet::new()).unwrap(),
      serde_json::json!({
        "sub": "你",
        "find_start": 2,
        "find_finish": 4,
        "find_capture": "你",
        "match_first": "id",
        "match_second": "7",
        "first_letter": "a",
        "first_digit": "1",
        "second_letter": "b",
        "second_digit": "2",
        "replaced": "1a 2b",
        "replacement_count": 2,
        "plain_start": 2,
        "plain_finish": 2,
        "lowered_unicode": "Äbc",
        "reversed_bytes": [98, 97, 160, 189, 228],
      })
    );

    assert_eq!(
      lua_to_json(format_result, 0, 32, &mut HashSet::new()).unwrap(),
      serde_json::json!({
        "formatted": "v:0007",
        "general_scientific": "1.23e+04",
        "general_fixed": "12.5",
        "general_small": "1.25e-05",
        "general_rounds_to_scientific": "1e+03",
        "general_rounds_to_fixed": "0.0001",
        "general_alternate": "12.0",
        "scientific": "1.25e+01",
        "scientific_upper": "1.25E+01",
        "fixed_infinity": "inf",
        "fixed_nan": "nan",
        "fixed_negative_nan": "nan",
        "fixed_negative_infinity": "-inf",
        "quoted_nul": "\"a\\0b\"",
        "quoted_control": "\"\\1\"",
        "quoted_control_digit": "\"\\0012\"",
        "quoted_newline": "\"a\\\nb\"",
        "quoted_tab": "\"\\9\"",
        "quoted_carriage_return": "\"\\13\"",
        "quoted_backspace": "\"\\8\"",
        "quoted_zero_digit": "\"\\0002\"",
        "quoted_quote": "\"\\\"\"",
        "quoted_backslash": "\"\\\\\"",
        "unicode_character": "`",
      })
    );
  }

  #[test]
  fn lua54_math_utf8_baseline_records_native_returns_and_byte_offsets() {
    let baseline = Lua::new_with(StdLib::ALL_SAFE, LuaOptions::default()).unwrap();
    let result = baseline
      .load(
        r#"
          local integer_part, fractional_part = math.modf(-3.25)
          local mantissa, exponent = math.frexp(12.8)
          local codepoint_one, codepoint_two, codepoint_three = utf8.codepoint("A你B", 1, 5)
          local iterator, state, control = utf8.codes("A你B")
          local first_position, first_codepoint = iterator(state, control)
          control = first_position
          local second_position, second_codepoint = iterator(state, control)
          control = second_position
          local third_position, third_codepoint = iterator(state, control)

          native_math_utf8 = {
            floor_negative = math.floor(-3.1),
            ceil_negative = math.ceil(-3.1),
            fmod_negative = math.fmod(-7, 3),
            min_multiple = math.min(4, -2, 7),
            max_multiple = math.max(4, -2, 7),
            modf_integer = integer_part,
            modf_fraction = fractional_part,
            frexp_mantissa = mantissa,
            frexp_exponent = exponent,
            tointeger_exact = math.tointeger(3.0),
            tointeger_fractional_is_nil = math.tointeger(3.25) == nil,
            type_integer = math.type(3),
            type_float = math.type(3.0),
            unsigned_wrap_comparison = not math.ult(-1, 1),
            utf8_char = utf8.char(65, 20320),
            utf8_len = utf8.len("A你B"),
            utf8_range_len = utf8.len("A你B", 2, 4),
            utf8_codepoint_one = codepoint_one,
            utf8_codepoint_two = codepoint_two,
            utf8_codepoint_three = codepoint_three,
            utf8_first_position = first_position,
            utf8_first_codepoint = first_codepoint,
            utf8_second_position = second_position,
            utf8_second_codepoint = second_codepoint,
            utf8_third_position = third_position,
            utf8_third_codepoint = third_codepoint,
            utf8_offset = utf8.offset("A你B", 2, 1),
          }
        "#,
      )
      .exec()
      .and_then(|()| baseline.globals().get::<Value>("native_math_utf8"))
      .unwrap();

    assert_eq!(
      lua_to_json(result, 0, 32, &mut HashSet::new()).unwrap(),
      serde_json::json!({
        "floor_negative": -4,
        "ceil_negative": -3,
        "fmod_negative": -1,
        "min_multiple": -2,
        "max_multiple": 7,
        "modf_integer": -3,
        "modf_fraction": -0.25,
        "frexp_mantissa": 0.8,
        "frexp_exponent": 4,
        "tointeger_exact": 3,
        "tointeger_fractional_is_nil": true,
        "type_integer": "integer",
        "type_float": "float",
        "unsigned_wrap_comparison": true,
        "utf8_char": "A你",
        "utf8_len": 3,
        "utf8_range_len": 1,
        "utf8_codepoint_one": 65,
        "utf8_codepoint_two": 20320,
        "utf8_codepoint_three": 66,
        "utf8_first_position": 1,
        "utf8_first_codepoint": 65,
        "utf8_second_position": 2,
        "utf8_second_codepoint": 20320,
        "utf8_third_position": 5,
        "utf8_third_codepoint": 66,
        "utf8_offset": 2,
      })
    );
  }

  #[test]
  fn string_primitive_metatable_stays_isolated_from_native_string_methods() {
    let baseline = Lua::new_with(StdLib::ALL_SAFE, LuaOptions::default()).unwrap();
    let native_string_methods = baseline
      .load(
        r#"
          local metatable = getmetatable("text")
          return metatable ~= nil and type(metatable.__index.sub) == "function"
        "#,
      )
      .eval::<bool>()
      .unwrap();
    assert!(native_string_methods);

    let source = valid_script(
      r#"
        string_metatable_compatibility = getmetatable("text") == nil
      "#,
    );
    let session =
      LuaSession::load(spec(&source, LuaSessionKind::Game), LuaPolicy::default()).unwrap();
    assert_eq!(
      session.environment_value("string_metatable_compatibility"),
      Value::Boolean(true)
    );
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

    let time = tg_service_time::TimeService::new();
    let timer = first
      .with_objects_mut(|objects| time.create_count_up(&mut objects.runtime_mut().time))
      .unwrap();
    assert_eq!(
      first
        .with_objects(|objects| time.state(&objects.runtime().time, timer))
        .flatten(),
      Some(tg_service_time::TimerState::Idle)
    );
    assert_eq!(
      second
        .with_objects(|objects| time.state(&objects.runtime().time, timer))
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
          create_request_id = i18n.create()
          duplicate_create_result = i18n.create({ language_code = "ignored" })
          local extra_language_argument = debug.pcall(function() i18n.get_language_code("extra") end)
          debug.assert(not extra_language_argument)
        end
        function HandleEvent(event)
          if event.type == "i18n" then
            if event.data.kind == "created" then
              created_event_request_id = event.data.request_id
              if event.data.ok then
                translated = i18n.get_value("menu", "title")
                missing = i18n.get_value("menu", "missing")
                reload_request_id = i18n.reload()
              end
            elseif event.data.kind == "reloaded" then
              reload_completed_request_id = event.data.request_id
            end
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
    assert_eq!(
      session.environment_value("create_request_id"),
      Value::Integer(1)
    );
    assert_eq!(
      session.environment_value("duplicate_create_result"),
      Value::Nil
    );
    let commands = session.take_host_commands();
    assert_eq!(
      commands
        .iter()
        .filter(|command| matches!(command, LuaHostCommand::I18nRequest { .. }))
        .count(),
      1
    );
    let request_id = commands
      .iter()
      .find_map(|command| match command {
        LuaHostCommand::I18nRequest { request_id, .. } => Some(*request_id),
        _ => None,
      })
      .expect("i18n create should enqueue a request");
    assert_eq!(request_id, 1);

    let delivery = LuaEventDelivery {
      event: LuaRuntimeEvent {
        sequence: 1,
        frame: 1,
        data: LuaEventData::I18n(super::super::LuaI18nEvent {
          request_id,
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
    assert_eq!(
      session.environment_value("created_event_request_id"),
      Value::Integer(request_id as i64)
    );
    assert_eq!(
      session.environment_value("reload_request_id"),
      Value::Integer(2)
    );

    let reload_commands = session.take_host_commands();
    assert!(reload_commands.iter().any(|command| matches!(
      command,
      LuaHostCommand::I18nRequest {
        request_id: 2,
        kind: super::super::LuaI18nEventKind::Reloaded,
        ..
      }
    )));
    session
      .dispatch_event(&LuaEventDelivery {
        event: LuaRuntimeEvent {
          sequence: 2,
          frame: 2,
          data: LuaEventData::I18n(super::super::LuaI18nEvent {
            request_id: 2,
            kind: super::super::LuaI18nEventKind::Reloaded,
            ok: true,
            message: "i18n instance reloaded".to_string(),
            language_code: "zh_cn".to_string(),
            callback_language_code: "en_us".to_string(),
            namespaces: None,
          }),
        },
        route: LuaEventRoute::HandleEvent,
      })
      .unwrap();
    assert_eq!(
      session.environment_value("reload_completed_request_id"),
      Value::Integer(2)
    );
  }

  #[test]
  fn image_load_returns_request_id_routes_rich_text_and_rejects_unsafe_arguments() {
    let source = valid_script(
      r#"
        function Init(ctx)
          image_request_id = image.load("./sample", {
            block_width = 2,
            block_height = 1,
            crop_x = 1,
            crop_y = 2,
            crop_width = 3,
            crop_height = 4,
            scale = 0.75,
            cache = false,
            mode = "mix_block",
            background = color.rgb(12, 34, 56),
          })
          debug.assert(color.rgb(12, 34, 56) == "rgb(12,34,56)")
          debug.assert(color.hex(12, 34, 56) == '#0c2238')
          local named_color_args = debug.pcall(function() color.rgb{ r = 12, g = 34, b = 56 } end)
          local extra_color_arg = debug.pcall(function() color.rgb(12, 34, 56, {}) end)
          local bad_color_channel = debug.pcall(function() color.rgb(12, 34, 256) end)
          local traversal = debug.pcall(function() image.load("../outside.png") end)
          local unsupported = debug.pcall(function() image.load("sample.gif") end)
          local negative_crop = debug.pcall(function() image.load("sample.png", { crop_x = -1 }) end)
          local named_color = debug.pcall(function() image.load("sample.png", { background = "red" }) end)
          local invalid_mode = debug.pcall(function() image.load("sample.png", { mode = "native" }) end)
          for _ = 1, 3 do image.load("sample") end
          local over_limit = debug.pcall(function() image.load("sample") end)
          debug.assert(image_request_id == 1)
          debug.assert(not named_color_args)
          debug.assert(not extra_color_arg)
          debug.assert(not bad_color_channel)
          debug.assert(not traversal)
          debug.assert(not unsupported)
          debug.assert(not negative_crop)
          debug.assert(not named_color)
          debug.assert(not invalid_mode)
          debug.assert(not over_limit)
        end
        function HandleEvent(event)
          if event.type == "image" and event.data.request_id == image_request_id and event.data.ok then
            draw.text(2, 3, event.data.output)
            image_after_completion = image.load("sample")
          end
        end
      "#,
    );
    let mut spec = spec(&source, LuaSessionKind::Game);
    let package_dir = spec.entry_path.parent().unwrap().to_path_buf();
    let scripts_dir = package_dir.join("scripts");
    fs::create_dir_all(&scripts_dir).unwrap();
    spec.entry_path = scripts_dir.join("main.lua");
    fs::write(&spec.entry_path, &source).unwrap();
    let assets_dir = package_dir.join("assets");
    fs::create_dir_all(&assets_dir).unwrap();
    image::RgbImage::from_pixel(8, 8, image::Rgb([220, 100, 30]))
      .save(assets_dir.join("sample.png"))
      .unwrap();

    let mut session = LuaSession::load(spec, LuaPolicy::default()).unwrap();
    assert_eq!(
      session.environment_value("image_request_id"),
      Value::Integer(1)
    );
    let commands = session.take_host_commands();
    assert_eq!(
      commands
        .iter()
        .filter(|command| matches!(command, LuaHostCommand::ImageRequest { .. }))
        .count(),
      crate::MAX_LUA_IMAGE_TASKS_PER_SESSION
    );
    let image_request = commands.iter().find_map(|command| match command {
      LuaHostCommand::ImageRequest { request_id, params } => Some((*request_id, params)),
      _ => None,
    });
    let (request_id, params) = image_request.expect("Lua request should reach the host");
    assert_eq!(request_id, 1);
    assert_eq!(
      PathBuf::from(&params.image_path),
      assets_dir.join("sample.png").canonicalize().unwrap()
    );
    assert_eq!(params.output_width, Some(2));
    assert_eq!(params.output_height, Some(1));
    assert_eq!((params.crop_x, params.crop_y), (1, 2));
    assert_eq!((params.crop_width, params.crop_height), (Some(3), Some(4)));
    assert_eq!(params.scale, 0.75);
    assert!(!params.cache);
    assert_eq!(params.mode, tg_service_image::ImageConvertMode::MixBlock);
    assert_eq!(params.background, [12, 34, 56]);

    let runtime = AsyncRuntime::<ImageRuntimeEvent>::with_worker_count(1);
    let image_service = ImageService::new(None);
    let task_id = image_service.convert_async(&runtime, params.clone());
    let token = LuaSessionToken {
      kind: LuaSessionKind::Game,
      generation: 1,
    };
    let mut broker = LuaEventBroker::new();
    broker.synchronize_sessions(Some(token), None);
    broker
      .register_task(
        task_id,
        token,
        LuaTaskOperation::ImageConvert { request_id },
        LuaEventRoute::HandleEvent,
      )
      .unwrap();

    let started = std::time::Instant::now();
    let mut task_finished = false;
    while started.elapsed() < Duration::from_secs(5) {
      for event in runtime.poll_events() {
        match event {
          ImageRuntimeEvent::Image(event) => {
            broker
              .route_service_event(1, crate::LuaRoutableEvent::Image(&event))
              .unwrap();
          }
          ImageRuntimeEvent::TaskStatus(TaskStatusEvent::Finished { id }) if id == task_id => {
            task_finished = true;
          }
          ImageRuntimeEvent::TaskStatus(_) => {}
        }
      }
      if task_finished {
        break;
      }
      std::thread::sleep(Duration::from_millis(1));
    }
    assert!(task_finished, "async image conversion should finish");
    let mut deliveries = broker.drain_frame(LuaSessionKind::Game);
    assert_eq!(deliveries.len(), 1);
    let delivery = deliveries.pop().unwrap();
    let crate::LuaEventData::Image(crate::LuaImageEvent {
      request_id: event_request_id,
      outcome: crate::LuaImageOutcome::Converted(output),
    }) = &delivery.event.data
    else {
      panic!("expected converted image output");
    };
    assert_eq!(*event_request_id, request_id);
    assert!(output.starts_with("f%"));
    let output = output.clone();
    session.dispatch_event(&delivery).unwrap();
    let draw_commands = session.take_draw_commands();
    assert!(draw_commands.iter().any(|command| matches!(
      command,
      LuaDrawCommand::Text { x: 2, y: 3, params, .. } if params.text == output
    )));
    assert_eq!(
      session.environment_value("image_after_completion"),
      Value::Integer(5)
    );
    assert!(
      session
        .take_host_commands()
        .iter()
        .any(|command| matches!(command, LuaHostCommand::ImageRequest { request_id: 5, .. }))
    );

    session.stop();
    fs::remove_dir_all(package_dir).unwrap();
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
      "setmetatable",
      "getmetatable",
      "rawget",
      "rawset",
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
          local ok = debug.pcall(function() math.PI = 0 end)
          debug.assert(not ok, { message = "math must be read-only" })
          local iterator, state, control = pairs(math)
          local first_key, first_value = iterator(state, control)
          debug.assert(type(first_key) == "string" and first_value ~= nil
              and state ~= math and next(state) == nil)
          local count = 0
          for key, value in pairs(math) do
            debug.assert(type(key) == "string" and value ~= nil)
            count = count + 1
          end
          debug.assert(count > 0)
          local next_key, next_value = next(math)
          debug.assert(type(next_key) == "string" and next_value ~= nil)
          local raw_write = debug.pcall(function() rawset(math, "PI", 0) end)
          local raw_base_write = debug.pcall(function() base.rawset(base, "ipairs", nil) end)
          debug.assert(rawlen(math) == 0 and rawget(math, "PI") == nil and math.PI > 3
              and #char.ASCII_LETTER > 0 and table.concat(char.ASCII_LETTER) ~= ""
              and not select(1, debug.pcall(function() table.insert(char.ASCII_LETTER, "!") end))
              and not raw_write and not raw_base_write, { message = "raw access exposed or modified a read-only API backing table" })
        end
      "#,
    );
    LuaSession::load(spec(&source, LuaSessionKind::Game), LuaPolicy::default()).unwrap();
  }

  #[test]
  fn base_metatable_api_uses_lua54_positional_parameters_and_preserves_protection() {
    let source = valid_script(
      r#"
        function Init(ctx)
          local function fails(func)
            return not select(1, debug.pcall(func))
          end

          local target = {}
          local metatable = { __index = { fallback = 7 } }
          local result = setmetatable(target, metatable)
          debug.assert(result == target and target.fallback == 7
              and getmetatable(target) == metatable
              and base.getmetatable(target) == metatable)

          local removed = setmetatable(target, nil)
          debug.assert(removed == target and getmetatable(target) == nil
              and target.fallback == nil)

          local protected = {}
          setmetatable(protected, { __metatable = "locked" })
          debug.assert(getmetatable(protected) == "locked"
              and fails(function()
                setmetatable(protected, {})
              end))

          debug.assert(getmetatable(base) == false
              and fails(function() setmetatable(base, {}) end)
              and fails(function() setmetatable(target, false) end)
              and getmetatable(1) == nil)
        end
      "#,
    );
    LuaSession::load(spec(&source, LuaSessionKind::Game), LuaPolicy::default()).unwrap();
  }

  #[test]
  fn lua_metamethods_work_through_vm_and_custom_base_functions() {
    let source = valid_script(
      r#"
        function Init(ctx)
          local function fails(func)
            return not select(1, debug.pcall(func))
          end

          local writes = {}
          local metatable = {
            __index = function(_, key)
              if key == "missing" then return 7 end
            end,
            __newindex = function(_, key, value) writes[key] = value end,
            __len = function() return 12 end,
            __call = function(_, value) return value * 2 end,
            __add = function(left, right) return left.raw + right.raw end,
            __concat = function(left, right) return left.raw .. right.raw end,
            __eq = function(left, right) return left.raw == right.raw end,
            __lt = function(left, right) return left.raw < right.raw end,
            __tostring = function(value) return "value:" .. value.raw end,
          }
          local left = setmetatable({ raw = 2 }, metatable)
          local same = setmetatable({ raw = 2 }, metatable)
          local greater = setmetatable({ raw = 3 }, metatable)
          left.created = 9
          debug.assert(left.missing == 7 and writes.created == 9 and #left == 12
              and left(4) == 8 and left + greater == 5 and left .. greater == "23"
              and left == same and left < greater and tostring(left) == "value:2")

          local custom = { left = 1, right = 2 }
          setmetatable(custom, {
              __pairs = function(subject)
                local keys = { "right", "left" }
                local position = 0
                return function(state, previous)
                  position = position + 1
                  local key = keys[position]
                  if key ~= nil then return key, state[key] end
                end, subject, nil
              end,
            })
          local iterator, state, control = pairs(custom)
          local first_key, first_value = iterator(state, control)
          local second_key, second_value = iterator(state, first_key)
          debug.assert(state == custom and first_key == "right" and first_value == 2
              and second_key == "left" and second_value == 1
              and iterator(state, second_key) == nil)

          local virtual = setmetatable({ [1] = "a" }, {
              __index = function(_, index)
                if index == 2 then return "b" end
              end,
            })
          local array_iterator, array_state, array_index = ipairs(virtual)
          local array_first, array_first_value = array_iterator(array_state, array_index)
          local array_second, array_second_value = array_iterator(array_state, array_first)
          debug.assert(array_state == virtual and array_index == 0
              and array_first == 1 and array_first_value == "a"
              and array_second == 2 and array_second_value == "b"
              and array_iterator(array_state, array_second) == nil)

          local invalid = setmetatable({}, { __tostring = true })
          debug.assert(fails(function() tostring(invalid) end))

          local named = setmetatable({}, { __name = "Type" })
          debug.assert(string.find(tostring(named), "^Type: 0x") ~= nil)
        end
      "#,
    );
    LuaSession::load(spec(&source, LuaSessionKind::Game), LuaPolicy::default()).unwrap();
  }

  #[test]
  fn callback_error_closes_to_be_closed_variables() {
    let source = valid_script(
      r#"
        function UpdateFrame(dt, alpha)
          local metatable = {
            __close = function(_, error_value)
              debug.print(error_value == nil and "closed normally" or "closed with error")
            end,
          }
          do
            local normal <close> = setmetatable({}, metatable)
          end
          local failed <close> = setmetatable({}, metatable)
          debug.assert(false)
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

    session
      .update_frame(Duration::from_millis(16), 0.5)
      .expect_err("UpdateFrame must fail");
    let messages = session
      .take_host_commands()
      .into_iter()
      .filter_map(|command| match command {
        LuaHostCommand::Print { message, .. } => Some(message),
        _ => None,
      })
      .collect::<Vec<_>>();
    assert_eq!(messages, ["closed normally", "closed with error"]);
  }

  #[test]
  fn lua_gc_preserves_weak_tables_and_runs_finalizers() {
    let source = valid_script("");
    let session =
      LuaSession::load(spec(&source, LuaSessionKind::Game), LuaPolicy::default()).unwrap();
    let environment: Table = session.lua.registry_value(&session.environment).unwrap();
    environment.set("finalized", 0).unwrap();
    session
      .lua
      .load(
        r#"
          weak_values = setmetatable({}, { __mode = "v" })
          do
            local value = {}
            weak_values[1] = value
          end
          do
            local value = setmetatable({}, {
              __gc = function() finalized = finalized + 1 end,
            })
          end
        "#,
      )
      .set_environment(environment.clone())
      .exec()
      .unwrap();

    session.lua.gc_collect().unwrap();
    session.lua.gc_collect().unwrap();

    let weak_values: Table = environment.get("weak_values").unwrap();
    assert!(matches!(
      weak_values.raw_get::<Value>(1).unwrap(),
      Value::Nil
    ));
    assert_eq!(environment.get::<i64>("finalized").unwrap(), 1);
  }

  #[test]
  fn stop_releases_registered_callbacks_before_collecting() {
    let source = valid_script("");
    let mut session =
      LuaSession::load(spec(&source, LuaSessionKind::Game), LuaPolicy::default()).unwrap();
    let environment: Table = session.lua.registry_value(&session.environment).unwrap();
    environment.set("finalized", 0).unwrap();
    let callback = session
      .lua
      .load(
        r#"
          return (function()
            local marker = setmetatable({}, {
              __gc = function() finalized = finalized + 1 end,
            })
            return function() return marker end
          end)()
        "#,
      )
      .set_environment(environment.clone())
      .eval::<Function>()
      .unwrap();
    session
      .register_event_callback(callback, LuaCallbackLifetime::UntilTerminal)
      .unwrap();

    session.stop();

    assert_eq!(session.state(), LuaSessionState::Stopped);
    assert_eq!(environment.get::<i64>("finalized").unwrap(), 1);
  }

  #[test]
  fn instruction_limit_still_closes_pending_variables() {
    let source = valid_script(
      r#"
        function Update(dt)
          local closing <close> = setmetatable({}, {
            __close = function() close_called = true end,
          })
          while true do end
        end
      "#,
    );
    let mut session =
      LuaSession::load(spec(&source, LuaSessionKind::Game), LuaPolicy::default()).unwrap();

    let error = session.update().unwrap_err();

    assert_eq!(error.stage, LuaErrorStage::ExecutionLimit);
    assert!(
      session
        .environment_value("close_called")
        .as_boolean()
        .unwrap_or(false)
    );
  }

  #[test]
  fn align_resolve_rect_returns_two_position_values() {
    let source = valid_script(
      r#"
        function Init(ctx)
          local function fails(func)
            return not select(1, debug.pcall(func))
          end
          local top_x, top_y = align.resolve_rect(10, 4, align.LEFT, align.TOP)
          local center_x, center_y = align.resolve_rect(10, 4, align.CENTER, align.CENTER)
          local right_x, bottom_y = align.resolve_rect(10, 4, align.RIGHT, align.BOTTOM)
          debug.assert(top_x == 0 and top_y == 0)
          debug.assert(center_x == 55 and center_y == 17)
          debug.assert(right_x == 110 and bottom_y == 34)
          debug.assert(select('#', align.resolve_rect(10, 4, align.LEFT, align.TOP)) == 2)
          debug.assert(align.resolve_x(10, align.RIGHT, { offset_x = -2 }) == 108)
          debug.assert(align.resolve_y(4, align.BOTTOM, { relative_y = 20 }) == 16)
          debug.assert(fails(function() align.resolve_rect(10, 4, align.LEFT, align.TOP, { typo = 1 }) end))
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
          local plain = measurement.get_text_width(text, { text_mode = string.PLAIN_TEXT })
          local rich = measurement.get_text_width(text, { text_mode = string.RICH_TEXT })
          local auto = measurement.get_text_width(text, { text_mode = string.AUTO })
          debug.assert(plain == 16, { message = "plain mode must preserve all syntax" })
          debug.assert(rich == 6, { message = "rich mode must preserve the f% prefix" })
          debug.assert(auto == 4, { message = "auto mode must consume the f% prefix" })
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
          local width, height = measurement.get_text_size("Hello\n世界")
          debug.assert(width == 5 and height == 2)
          debug.assert(select('#', measurement.get_text_size("Hello\n世界")) == 2)
          debug.assert(measurement.get_text_width("abcdef", { max_width = 3, auto_wrap = true, word_wrap = false }) == 3)
          debug.assert(measurement.get_text_height("abcdef", { max_width = 3, max_height = 1, overflow_marker = "..", auto_wrap = true, word_wrap = false }) == 1)
          local rich_width, rich_height = measurement.get_text_size("f%{value:name}", { rich_params = { name = "终端" }, text_mode = string.AUTO, horizontal_align = align.RIGHT })
          debug.assert(rich_width == 4 and rich_height == 1)
          local empty_width, empty_height = measurement.get_text_size("")
          debug.assert(empty_width == 0 and empty_height == 0)
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
            return not select(1, debug.pcall(func))
          end

          debug.assert(fails(function()
            measurement.get_text_width("text", { x = 0 })
          end))
          debug.assert(fails(function()
            measurement.get_text_width("text", { bold = true })
          end))
          debug.assert(fails(function()
            measurement.get_text_width("text", { fg = color.RED })
          end))
          debug.assert(fails(function()
            measurement.get_text_width("text", { slice_layer = "base" })
          end))
          debug.assert(fails(function()
            measurement.get_text_width("text", { max_width = 0 })
          end))
          debug.assert(fails(function()
            measurement.get_text_width("text", { max_height = 65536 })
          end))
          debug.assert(fails(function()
            measurement.get_text_width("text", { max_width = 1.5 })
          end))
          debug.assert(fails(function()
            measurement.get_text_width("text", { auto_wrap = "true" })
          end))
          debug.assert(fails(function()
            measurement.get_text_width("text", { horizontal_align = "bottom" })
          end))
          debug.assert(fails(function()
            measurement.get_text_width("text", { text_mode = "unknown" })
          end))
          debug.assert(fails(function()
            measurement.get_text_width(nil)
          end))
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
          debug.assert(measurement.get_text_width("f%{key:jump}") == 3)
          debug.assert(measurement.get_text_width("f%{key_default:jump}") == 4)
        end
        function Render()
          draw.text(1, 1, "ordinary")
          draw.text(1, 2, "f%{key:jump}")
          draw.text(1, 3, "f%{key_default:jump}")
          draw.text(1, 4, "f%{value:name}", { rich_params = { name = "TUI" } })
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
          local first = random.create({ type = random.INT, min = -10, max = 10, seed = 2468 })
          local second = random.create({ type = random.INT, min = -10, max = 10, seed = 2468 })
          debug.assert(random.generate(first) == random.generate(second))
          debug.assert(random.generate(first) == random.generate(second))

          slice_id = slice.create(20, 20, { layer = 3 })
          slice.draw(slice_id, -4, 2)
          local info = slice.get_info(slice_id)
          debug.assert(info.width == 20 and info.height == 20 and info.layer == 1)

          local json = serialization.json_encode({
              title = "TUI GAME",
              enabled = true,
              values = { 1, 2, 3 },
            })
          local decoded = serialization.json_decode(json)
          debug.assert(decoded.title == "TUI GAME" and decoded.enabled
              and decoded.values[3] == 3)

          local packed = serialization.binary_pack("<i2c3", 513, "abc")
          local unpacked_values, unpacked_next = serialization.binary_unpack("<i2c3", packed)
          debug.assert(unpacked_values[1] == 513
              and unpacked_values[2] == "abc"
              and unpacked_next == 6)

          local xml = serialization.xml_encode({
              root = {
                _attr = { version = "1.0" },
                child = { "Hello", _attr = { id = 1 } },
              },
            })
          local xml_data = serialization.xml_decode(xml)
          debug.assert(xml_data.root._attr.version == "1.0"
              and xml_data.root.child._attr.id == "1"
              and xml_data.root.child._text == "Hello")

          local encoded = encoding.base64_encode("TUI GAME")
          debug.assert(encoding.base64_decode(encoded) == "TUI GAME")
          local url = encoding.url_encode("a b/中")
          debug.assert(encoding.url_decode(url) == "a b/中")
          local hex = encoding.hex_encode("abc")
          debug.assert(encoding.hex_decode(hex) == "abc")

          local bytes = "\\0\\255binary"
          debug.assert(encoding.base64_decode(encoding.base64_encode(bytes)) == bytes)
          debug.assert(encoding.hex_decode(encoding.hex_encode(bytes)) == bytes)
        end

        function Render()
          draw.fill_rect(-2, 1, 5, 2, { char = "#", slice_layer = slice_id })
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
          local function fails(func)
            return not select(1, debug.pcall(func))
          end

          local direct_int = random.randint()
          local direct_float = random.randfloat()
          debug.assert(direct_int >= -2147483648 and direct_int <= 2147483647)
          debug.assert(direct_float >= 0 and direct_float <= 1)

          local generator = random.create()
          local initial = random.get_info(generator)
          debug.assert(initial.type == random.INT
              and initial.min == -2147483648
              and initial.max == 2147483647
              and initial.step == 0)
          debug.assert(random.count() == 1 and random.list().n == 1)
          debug.assert(fails(function() random.generate() end)
              and fails(function() random.count(true) end)
              and fails(function() slice.exists() end)
              and fails(function() slice.count(true) end))
          debug.assert(random.set(generator, { type = random.FLOAT, min = -2.5, max = 3.5, seed = 42, step = 5 }))
          local range_min, range_max = random.get_range(generator)
          debug.assert(random.get_type(generator) == random.FLOAT
              and range_min == -2.5
              and range_max == 3.5
              and select('#', random.get_range(generator)) == 2
              and random.get_range("rng_999") == nil
              and select('#', random.get_range("rng_999")) == 1
              and random.get_seed(generator) == 42
              and random.get_step(generator) == 5)
          local value = random.generate(generator)
          debug.assert(value >= -2.5 and value <= 3.5
              and random.get_step(generator) == 6)
          debug.assert(random.set_type(generator, random.FLOAT))
          debug.assert(random.set_seed(generator, 42))
          debug.assert(fails(function() random.set_type(generator) end))
          debug.assert(fails(function() random.set_seed(generator) end))
          local missing_step = debug.pcall(function() random.set_step(generator, nil) end)
          debug.assert(not missing_step)

          debug.assert(slice.exists("base"))
          local base = slice.get_info("base")
          debug.assert(base.width == ctx.base.width
              and base.height == ctx.base.height
              and base.layer == 0
              and base.bg == color.TRANSPARENT)
          local first = slice.create(10, 4, { bg = color.BLUE })
          local inserted = slice.create(3, 2, { layer = 1 })
          local first_width, first_height = slice.get_size(first)
          debug.assert(first_width == 10 and first_height == 4
              and select('#', slice.get_size(first)) == 2
              and slice.get_size("slice_999") == nil
              and select('#', slice.get_size("slice_999")) == 1)
          debug.assert(fails(function() random.create({ typo = true }) end)
              and fails(function() slice.create(2, 2, { typo = true }) end))
          local slices = slice.list()
          debug.assert(slices.n == 2
              and slices[1].id == inserted
              and slices[1].layer == 1
              and slices[2].id == first
              and slices[2].layer == 2
              and slices[2].bg == color.BLUE)
          debug.assert(slice.set(first))
          debug.assert(slice.set(first, { bg = color.RED }))
          debug.assert(slice.get_background(first) == color.RED)
          debug.assert(slice.set_background(first, color.TRANSPARENT))
          debug.assert(slice.get_background(first) == color.TRANSPARENT)
          debug.assert(slice.set_background(first, color.NONE))
          debug.assert(slice.get_background(first) == color.NONE)
          debug.assert(fails(function() slice.set_background(first) end))
          debug.assert(slice.get_background(first) == color.NONE)
          debug.assert(not slice.set_background("base", color.BLUE))
          debug.assert(not slice.set_background("slice_999", color.BLUE))
          local invalid_background = debug.pcall(function()
              slice.set_background(first, "not-a-color")
            end)
          debug.assert(not invalid_background)
          debug.assert(slice.set_size(first, 12, 4)
              and slice.get_width(first) == 12
              and slice.get_height(first) == 4)
          debug.assert(fails(function() slice.set_size(first, 12) end))
          debug.assert(slice.set_layer(first, 999))
          debug.assert(slice.get_layer(first) == 2)
          debug.assert(slice.delete(inserted))
          debug.assert(slice.get_layer(first) == 1)
          debug.assert(slice["50P"] == nil
              and slice.list_by_layer == nil
              and random.set_params == nil)
          debug.assert(slice.clear()
              and slice.count() == 0
              and slice.list().n == 0)
          debug.assert(random.clear() and random.count() == 0)
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
          generator = random.create({ seed = 7 })
          layer = slice.create(2, 2)
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
            tg_service_random::RandomService::new()
              .configured_ids(&objects.runtime().random_generators)
              .len(),
            1
          );
          assert_eq!(
            tg_service_widget::SliceService::new()
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
          tg_service_random::RandomService::new()
            .configured_ids(&objects.runtime().random_generators)
            .len(),
          1
        );
        assert_eq!(
          tg_service_widget::SliceService::new()
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
          local cyclic_ok = debug.pcall(function() serialization.json_encode(cyclic) end)
          local sparse_ok = debug.pcall(function()
              serialization.json_encode({ [1] = "a", [3] = "c" })
            end)
          local entity_ok = debug.pcall(function()
              serialization.xml_decode("<!DOCTYPE root [<!ENTITY secret 'hidden'>]><root>&secret;</root>")
            end)
          local mismatched_xml_ok = debug.pcall(function()
              serialization.xml_decode("<root><child></root>")
            end)
          local mixed_xml_ok = debug.pcall(function()
              serialization.xml_decode("<root><child/>text</root>")
            end)
          local positional_encoding_ok = debug.pcall(function() return encoding.hex_decode("00") end)
          local missing_encoding_arg_ok = debug.pcall(function() encoding.hex_decode() end)
          local wrong_encoding_type_ok = debug.pcall(function() encoding.hex_decode(123) end)
          local invalid_hex_ok = debug.pcall(function() encoding.hex_decode("0xz1") end)
          local positional_serialization_ok = debug.pcall(function() return serialization.json_encode({}) end)
          local missing_serialization_arg_ok = debug.pcall(function() serialization.json_decode() end)
          local positional_packsize_ok = debug.pcall(function() serialization.binary_packsize("<I4") end)
          debug.assert(not cyclic_ok
              and not sparse_ok
              and not entity_ok
              and not mismatched_xml_ok
              and not mixed_xml_ok
              and positional_encoding_ok
              and not missing_encoding_arg_ok
              and not wrong_encoding_type_ok
              and not invalid_hex_ok
              and positional_serialization_ok
              and not missing_serialization_arg_ok
              and positional_packsize_ok)
        end
      "#,
    );
    LuaSession::load(spec(&source, LuaSessionKind::Game), LuaPolicy::default()).unwrap();
  }

  #[test]
  fn serialization_json_top_level_null_empty_and_numeric_edges_are_stable() {
    let source = valid_script(
      r#"
        function Init(ctx)
          local encoded_nil = serialization.json_encode(nil)
          local decoded_null = serialization.json_decode("null")
          local empty_table = serialization.json_encode({})
          local max_integer = serialization.json_decode("9223372036854775807")
          local above_i64 = serialization.json_decode("9223372036854775808")
          local nested = serialization.json_decode('{"object":{"nil":null},"array":[1,null,3]}')
          local nested_json = serialization.json_encode(nested)
          local nested_round_trip = serialization.json_decode(nested_json)
          local yaml = serialization.yaml_encode({ object = { null = serialization.NULL } })
          local yaml_value = serialization.yaml_decode(yaml)
          local mutation_ok = debug.pcall(function() serialization.NULL.field = true end)
          local infinity_ok = debug.pcall(function() serialization.json_encode(1 / 0) end)
          local nan_ok = debug.pcall(function() serialization.json_encode(0 / 0) end)

          debug.assert(encoded_nil == "null"
              and decoded_null == serialization.NULL
              and empty_table == "{}", { message = "top-level JSON null and empty table contract failed" })
          debug.assert(max_integer == math.MAX_INTEGER
              and math.type(max_integer) == "integer"
              and math.type(above_i64) == "float", { message = "JSON integer boundary contract failed" })
          debug.assert(nested.object["nil"] == serialization.NULL
              and #nested.array == 3
              and nested.array[1] == 1
              and nested.array[2] == serialization.NULL
              and nested.array[3] == 3
              and nested_round_trip.object["nil"] == serialization.NULL
              and #nested_round_trip.array == 3
              and nested_round_trip.array[1] == 1
              and nested_round_trip.array[2] == serialization.NULL
              and nested_round_trip.array[3] == 3, { message = "nested JSON null round-trip contract failed" })
          debug.assert(yaml_value.object.null == serialization.NULL, { message = "YAML null round-trip contract failed" })
          debug.assert(not mutation_ok and not infinity_ok and not nan_ok, { message = "NULL immutability or non-finite number rejection failed" })
        end
      "#,
    );
    LuaSession::load(spec(&source, LuaSessionKind::Game), LuaPolicy::default()).unwrap();
  }

  #[test]
  fn serialization_null_is_preserved_or_rejected_by_format_contract() {
    let source = valid_script(
      r#"
        function Init(ctx)
          local csv_ok = debug.pcall(function()
              serialization.csv_encode({ { "value", serialization.NULL } })
            end)
          local ini_ok = debug.pcall(function()
              serialization.ini_encode({ entry = serialization.NULL })
            end)
          local toml_ok = debug.pcall(function()
              serialization.toml_encode({ entry = serialization.NULL })
            end)
          local xml_ok = debug.pcall(function()
              serialization.xml_encode({ root = { child = serialization.NULL } })
            end)
          debug.assert(not csv_ok and not ini_ok and not toml_ok and not xml_ok)
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
          local json = serialization.json_encode({ name = "TUI", enabled = true, values = { 1, 2 } })
          local json_value = serialization.json_decode(json)
          debug.assert(json_value.name == "TUI" and json_value.values[2] == 2)

          local csv = serialization.csv_encode({ { "name", "score" }, { "player", 9 } })
          local csv_value = serialization.csv_decode(csv)
          debug.assert(csv_value[2][1] == "player" and csv_value[2][2] == "9")

          local yaml = serialization.yaml_encode({ name = "TUI", enabled = true })
          local yaml_value = serialization.yaml_decode(yaml)
          debug.assert(yaml_value.name == "TUI" and yaml_value.enabled)

          local toml = serialization.toml_encode({ name = "TUI", version = 1 })
          local toml_value = serialization.toml_decode(toml)
          debug.assert(toml_value.name == "TUI" and toml_value.version == 1)

          local ini = serialization.ini_encode({ server = { host = "127.0.0.1", port = 8080 } })
          local ini_value = serialization.ini_decode(ini)
          debug.assert(ini_value.server.host == "127.0.0.1"
              and ini_value.server.port == "8080")

          local first = serialization.binary_pack("<I2z", 513, "ok")
          local all = first .. first
          local unpacked_first, next_pos = serialization.binary_unpack("<I2z", all)
          local unpacked_second = serialization.binary_unpack("<I2z", all, { pos = next_pos })
          debug.assert(unpacked_first[1] == 513
              and unpacked_first[2] == "ok"
              and unpacked_second[1] == 513
              and unpacked_second[2] == "ok")
          debug.assert(serialization.binary_packsize("<I2c2x") == 5)
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
          draw.fill_rect(2, 1, 10, 4, { bg = color.BLUE })
          draw.render()
          local no_arguments = debug.pcall(function() draw.render() end)
          local extra_argument = debug.pcall(function() draw.render(true) end)
          local missing_text = debug.pcall(function() draw.text(1, 1) end)
          local unknown_draw_option = debug.pcall(function() draw.text(1, 1, "x", { boid = true }) end)
          local unknown_rect_option = debug.pcall(function() draw.fill_rect(2, 1, 10, 4, { color = color.BLUE }) end)
          debug.assert(no_arguments and not extra_argument
              and not missing_text and not unknown_draw_option and not unknown_rect_option)
        end
        function Update(dt)
          draw.erase_rect(3, 2, 8, 2)
        end
        function Render()
          local ok = debug.pcall(function() draw.render() end)
          debug.assert(not ok)
          draw.text(1, 1, "render")
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
          for _, item in ipairs(char.ASCII_LETTER) do
            x = x + 2
            if x % 20 == 0 then
              x = 2
              y = y + 1
            end
            draw.text(x, y, item)
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
          debug.print("plain")
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
  fn debug_assert_accepts_nil_and_reports_an_assertion_failure() {
    let source = valid_script(
      r#"
        function Init(ctx)
          debug.assert(nil, { message = "nil assertion" })
        end
      "#,
    );
    let error = match LuaSession::load(spec(&source, LuaSessionKind::Game), LuaPolicy::default()) {
      Ok(_) => panic!("nil unexpectedly passed the assertion"),
      Err(error) => error,
    };

    assert_eq!(error.stage, LuaErrorStage::Callback);
    assert!(
      error.message.contains("nil assertion"),
      "unexpected error: {}",
      error.message
    );
    assert!(!error.message.contains("expected non-nil value"));

    let default_source = valid_script(
      r#"
        function Init(ctx)
          debug.assert(nil)
        end
      "#,
    );
    let default_error = match LuaSession::load(
      spec(&default_source, LuaSessionKind::Game),
      LuaPolicy::default(),
    ) {
      Ok(_) => panic!("an omitted value unexpectedly passed the assertion"),
      Err(error) => error,
    };
    assert!(default_error.message.contains("assertion failed"));
  }

  #[test]
  fn debug_print_before_init_fault_is_returned_with_the_load_error() {
    let source = valid_script(
      r#"
        function Init(ctx)
          debug.print("before init fault")
          debug.assert(false)
        end
      "#,
    );
    let error = match LuaSession::load_with_api(
      spec(&source, LuaSessionKind::Game),
      LuaPolicy::default(),
      LuaApiConfig {
        debug_enabled: true,
        ..LuaApiConfig::default()
      },
    ) {
      Ok(_) => panic!("Init unexpectedly succeeded"),
      Err(error) => error,
    };

    assert!(error.diagnostic_commands.iter().any(|command| matches!(
      command,
      LuaHostCommand::Print { message, .. } if message == "before init fault"
    )));
  }

  #[test]
  fn successful_debug_print_before_callback_fault_remains_pending() {
    let source = valid_script(
      r#"
        function Update(dt)
          debug.print(1)
          debug.print()
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
          debug.assert(measurement.get_text_width(12345) == 5)
          debug.print(100, { title = false })
          debug.info(true)
        end

        function Render()
          draw.text(1, 1, 200, { overflow_marker = 9 })
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
          debug.assert(debug.VERSION == "Lua 5.4 / TUI GAME API 1"
              and debug.TRACE == "trace"
              and debug.DEBUG == "debug"
              and debug.INFO == "info"
              and debug.WARN == "warn"
              and debug.ERROR == "error"
              and debug.FATAL == "fatal")
          debug.assert(debug.assert("value") == "value")
          debug.print("custom", { title = "Title", level = debug.WARN, time = true, type_head = true })
          debug.info("positional")
          local extra_skip_argument = debug.pcall(function() event.skip_action("extra") end)
          debug.assert(not extra_skip_argument)
          debug.info("info")
          debug.warn("warn")
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
    for (expected_message, expected_level) in [
      ("positional", "info"),
      ("info", "info"),
      ("warn", "warn"),
      ("error", "error"),
    ] {
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
  fn protected_calls_follow_lua_multiple_return_semantics() {
    let source = valid_script(
      r##"
        function Init(ctx)
          local function pack(...)
            return { n = select("#", ...), ... }
          end
          local returned = pack(debug.pcall(function()
            return "first", nil, "third", nil
          end))
          debug.assert(returned[1] and returned.n == 5
              and returned[2] == "first" and returned[3] == nil
              and returned[4] == "third" and returned[5] == nil)

          local received_count, received_second, received_third
          local called = debug.pcall(function(...)
            received_count = select("#", ...)
            received_second = select(2, ...)
            received_third = select(3, ...)
          end, "first", nil, "third")
          debug.assert(called and received_count == 3
              and received_second == nil and received_third == "third")

          local failed, failure = debug.pcall(function()
            debug.assert(false, { message = "failed" })
          end)
          debug.assert(not failed and type(failure) == "string")

          local table_ok, table_result = debug.pcall(function(value)
            return value
          end, { message = "vararg table" })
          debug.assert(table_ok and table_result.message == "vararg table")

          local missing_function, missing_function_error = debug.pcall(function()
            debug.pcall()
          end)
          debug.assert(not missing_function and type(missing_function_error) == "string")

          local invalid_handler, invalid_handler_error = debug.pcall(function()
            debug.xpcall(function() return true end, true)
          end)
          debug.assert(not invalid_handler and type(invalid_handler_error) == "string")

          local unknown_assertion_option = debug.pcall(function()
            debug.assert(true, { typo = true })
          end)
          debug.assert(not unknown_assertion_option)

          local handled, handled_error = debug.xpcall(function()
            debug.assert(false, { message = "failed" })
          end, function(message)
            return "handled"
          end)
          debug.assert(not handled and handled_error == "handled")
        end
      "##,
    );
    LuaSession::load(spec(&source, LuaSessionKind::Game), LuaPolicy::default()).unwrap();
  }

  #[test]
  fn api_rejects_invalid_math_domains_and_excessively_deep_parameters() {
    let source = valid_script(
      r#"
        function Init(ctx)
          local sqrt_ok = debug.pcall(function() math.sqrt(-1) end)
          local pow_ok = debug.pcall(function() math.pow(-1, 0.5) end)
          local value = {}
          local cursor = value
          for index = 1, 33 do
            cursor.child = {}
            cursor = cursor.child
          end
          local depth_ok, depth_ok_value1 = debug.pcall(function() return type(value) end)
          local cyclic = {}
          cyclic.self = cyclic
          local cycle_ok, cycle_ok_value1 = debug.pcall(function() return type(cyclic) end)
          local unknown_ok = debug.pcall(function()
              measurement.get_text_width("value", { unknown = true })
            end)
          local packed = table.pack("first", nil, nil)
          debug.assert(not sqrt_ok and not pow_ok and depth_ok
              and depth_ok_value1 == "table" and cycle_ok
              and cycle_ok_value1 == "table" and not unknown_ok
              and packed.n == 3 and packed[1] == "first" and packed[3] == nil)
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
            return not select(1, debug.pcall(func))
          end

          debug.assert(string.lower("ÄBC") == "äbc")
          debug.assert(string.upper("äbc") == "ÄBC")
          debug.assert(string.reverse("你ab") == "ba你")

          local parts = string.split("a<>你<>", "<>")
          debug.assert(#parts == 3 and parts[1] == "a" and parts[2] == "你" and parts[3] == "")
          debug.assert(fails(function() string.split("abc", "") end))
          debug.assert(fails(function() string.split("abc", "b", { typo = true }) end))

          debug.assert(string.sub("甲乙丙", 2) == "乙丙")
          debug.assert(string.sub("甲乙丙", -2, { finish = -1 }) == "乙丙")
          debug.assert(string.sub("甲乙丙", -99, { finish = 99 }) == "甲乙丙")
          debug.assert(string.rep("A", 3, { sep = ":" }) == "A:A:A")
          debug.assert(string.rep("", 9223372036854775807) == "")

          local first, last, found = string.find("你ab你", "(a)(b)")
          debug.assert(first == 2 and last == 3
              and found.n == 2 and found[1] == "a" and found[2] == "b")
          local plain_start, plain_finish, plain = string.find("a.b", ".", { plain = true })
          debug.assert(plain_start == 2 and plain_finish == 2
              and plain.n == 1 and plain[1] == ".")
          local init_start = string.find("abc", "a", { init = 0 })
          debug.assert(init_start == 1)
          debug.assert(string.find("abc", "a", { init = 99 }) == nil)
          debug.assert(fails(function() string.find("abc") end)
              and fails(function() string.match("abc", "a", { init = "bad" }) end)
              and fails(function() string.find("abc", "a", { init = 99, plain = "yes" }) end)
              and fails(function() string.find("abc", "a", { typo = true }) end))
          debug.assert(select('#', string.find("abc", "z")) == 1)
          local _, _, no_capture = string.find("abc", "b")
          debug.assert(no_capture.n == 1 and no_capture[1] == "b")

          local matched = string.match("x=42", "(%a+)=(%d+)")
          debug.assert(matched.n == 2 and matched[1] == "x" and matched[2] == "42")
          local whole = string.match("x=42", "%d+")
          debug.assert(whole.n == 1 and whole[1] == "42")
          local position = string.match("abc", "()b")
          debug.assert(position.n == 1 and position[1] == 2)

          local iterator = string.gmatch("a1 b2", "(%a)(%d)")
          local first = iterator()
          local second = iterator()
          local unicode_iterator = string.gmatch("甲乙丙", ".")
          local unicode_first = unicode_iterator()
          local unicode_second = unicode_iterator()
          local unicode_third = unicode_iterator()
          debug.assert(first.n == 2 and first[1] == "a" and first[2] == "1"
              and second.n == 2 and second[1] == "b" and second[2] == "2"
              and iterator() == nil and unicode_first[1] == "甲"
              and unicode_second[1] == "乙" and unicode_third[1] == "丙"
              and unicode_iterator() == nil)
          local iterated = ""
          for item in string.gmatch("a1 b2", "(%a)(%d)") do
            iterated = iterated .. item[1] .. item[2]
          end
          debug.assert(iterated == "a1b2")
          local many = string.rep("x", 10001)
          local lazy_pattern = string.gmatch(many, ".")
          debug.assert(lazy_pattern()[1] == "x")

          local replaced, replace_count = string.gsub("ab", "(%a)", "%1%1")
          debug.assert(replaced == "aabb" and replace_count == 2)
          local unchanged, unchanged_count = string.gsub("ab", ".", "x", { limit = 0 })
          debug.assert(unchanged == "ab" and unchanged_count == 0)
          debug.assert(select('#', string.gsub("ab", "z", "x")) == 2)
          local table_replaced, table_count = string.gsub("ab", ".", { a = "A" })
          debug.assert(table_replaced == "Ab" and table_count == 2)
          local function_replaced, function_count = string.gsub("a1", ".", function(value)
              if value == "a" then return 9 end
              return false
            end)
          debug.assert(function_replaced == "91" and function_count == 2)
          debug.assert(fails(function() string.gsub("ab", ".", "x", { limit = -2 }) end))
          debug.assert(fails(function() string.gsub("a", ".", function() return true end) end))

          local regex_start, regex_finish, regex_found = string.regex_find("你ab你", "(a)(b)")
          debug.assert(regex_start == 2 and regex_finish == 3
              and regex_found.n == 2 and regex_found[1] == "a" and regex_found[2] == "b")
          local regex_match = string.regex_match("x=42", "([a-z]+)=([0-9]+)")
          debug.assert(regex_match[1] == "x" and regex_match[2] == "42")
          local regex_iterated = ""
          for item in string.regex_gmatch("a1 b2", "([a-z])([0-9])") do
            regex_iterated = regex_iterated .. item[1] .. item[2]
          end
          debug.assert(regex_iterated == "a1b2")
          local lazy_regex = string.regex_gmatch(many, ".")
          debug.assert(lazy_regex()[1] == "x")
          local regex_replaced, regex_count = string.regex_gsub(
            "a1 b2", "([a-z])([0-9])", "$2$1", { limit = -1 })
          debug.assert(regex_replaced == "1a 2b" and regex_count == 2)
          debug.assert(string.regex_test("abc", "^a"))
          debug.assert(string.regex_find("ba", "^a", { init = 2 }) == nil
              and string.regex_match("ba", "^a", { init = 2 }) == nil)
          local many_regex_captures = string.rep("()", 33)
          debug.assert(fails(function()
              string.regex_match("a", many_regex_captures .. "a")
            end))
          debug.assert(string.regex_escape("[a-z]") == "\\[a\\-z\\]")
          local regex_parts = string.regex_split("a, b;c", "[,;]\\s*")
          debug.assert(#regex_parts == 3 and regex_parts[2] == "b")

          debug.assert(string.rich_text_to_plain_text("f%<fg:red>A</fg>{value:tail}",
              { rich_params = { tail = "B" } }) == "AB")
          debug.assert(string.rich_text_to_plain_text("f%{key:jump}") == "[Space]")
          debug.assert(string.rich_text_to_plain_text("f%{key:jump}", { key_params = false }) == "{key:jump}")
          debug.assert(string.format("%s:%04d", "v", 7) == "v:0007")
          local format_payload = { value = "data" }
          debug.assert(string.format("%s", format_payload) == tostring(format_payload))
          debug.assert(string.format("%-4s|%+d|%#x|%.2f|%%", "a", 2, 255, 1.25)
            == "a   |+2|0xff|1.25|%")
          debug.assert(string.format("%.3g", 12345) == "1.23e+04"
              and string.format("%.3g", 12.5) == "12.5"
              and string.format("%.3g", 0.0000125) == "1.25e-05"
              and string.format("%.3g", 999.9) == "1e+03"
              and string.format("%.3g", 0.00009999) == "0.0001"
              and string.format("%#.3g", 12.0) == "12.0"
              and string.format("%.2e", 12.5) == "1.25e+01"
              and string.format("%.2E", 12.5) == "1.25E+01")
          debug.assert(string.format("%.100s", "甲乙丙") == "甲乙丙"
              and string.format("%.2s", "甲乙丙") == "甲乙"
              and string.format("%5s", "甲") == "    甲")
          debug.assert(string.format("%q", "a\0b") == "\"a\\0b\""
              and string.format("%q", "\0" .. "2")
                == "\"\\0002\""
              and string.format("%q", "\1" .. "2")
                == "\"\\0012\""
              and string.format("%q", "a\nb")
                == "\"a\\\nb\""
              and string.format("%q", "\t\b\r")
                == "\"\\9\\8\\13\""
              and string.format("%q", "\"\\")
                == "\"\\\"\\\\\""
              and string.format("%c", 0x4f60) == "你")
          local fixed_infinity = string.format("%f", 1 / 0)
          debug.assert(fixed_infinity == "inf", { message = "fixed infinity: " .. fixed_infinity })
          local fixed_nan = string.format("%f", 0 / 0)
          debug.assert(fixed_nan == "nan", { message = "fixed NaN: " .. fixed_nan })
          local fixed_negative_nan = string.format("%f", -(0 / 0))
          debug.assert(fixed_negative_nan == "nan", { message = "fixed negative NaN: " .. fixed_negative_nan })
          local fixed_negative_infinity = string.format("%+f", -1 / 0)
          debug.assert(fixed_negative_infinity == "-inf", { message = "fixed negative infinity: " .. fixed_negative_infinity })
          debug.assert(string.format("plain") == "plain")
          debug.assert(not select(1, debug.pcall(function() string.format() end)))
          debug.assert(fails(function()
              string.format("%1048576sX", "x")
            end))
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
            return not select(1, debug.pcall(func))
          end

          local joined = table.concat({ 1, "二", 3 }, "|", 1, 3)
          debug.assert(joined == "1|二|3")
          debug.assert(fails(function()
              table.concat({ 1, true }, ",")
            end))

          local inserted = { "a", "c" }
          table.insert(inserted, 2, "b")
          table.insert(inserted, "d")
          debug.assert(inserted[1] == "a" and inserted[2] == "b"
              and inserted[3] == "c" and inserted[4] == "d")
          local removed = table.remove(inserted, 2)
          debug.assert(removed == "b" and #inserted == 3 and inserted[2] == "c")

          local moved = { 1, 2, 3, 4 }
          local moved_result = table.move(moved, 1, 3, 2)
          debug.assert(moved_result == moved and moved[1] == 1 and moved[2] == 1
              and moved[3] == 2 and moved[4] == 3)
          local target = {}
          debug.assert(table.move(moved, 2, 3, 1, target) == target
              and target[1] == 1 and target[2] == 2)
          debug.assert(fails(function()
              table.move({}, 0, 1, 9223372036854775807)
            end))
          debug.assert(fails(function() table.move({}, 1, 16385, 1) end)
              and fails(function() table.concat({}, ",", 1, 16385) end)
              and fails(function() table.unpack({}, 1, 16385) end))

          local packed = table.pack("a", nil, "c")
          debug.assert(packed.n == 3 and packed[1] == "a"
              and packed[2] == nil and packed[3] == "c")
          local directly_packed = table.pack("x", nil, "z")
          debug.assert(directly_packed.n == 3 and directly_packed[1] == "x"
              and directly_packed[2] == nil and directly_packed[3] == "z")
          local first, second, third = table.unpack(packed, 1, packed.n)
          debug.assert(first == "a" and second == nil and third == "c")

          local sparse = {
            [1] = "a", [3] = "c", [0] = "zero", [-1] = "negative", name = "value",
          }
          local count, contiguous = table.count(sparse)
          local array_count, array_contiguous, array_indexes = table.count_array(sparse)
          debug.assert(count == 5 and not contiguous
              and array_count == 2 and not array_contiguous
              and array_indexes[1] == 1 and array_indexes[2] == 3
              and table.count_hash(sparse) == 3)
          local compacted = table.compact(sparse)
          debug.assert(compacted == sparse and sparse[1] == "a" and sparse[2] == "c"
              and sparse[3] == nil and sparse[0] == "zero"
              and sparse[-1] == "negative" and sparse.name == "value")
          local compacted_count, compacted_contiguous = table.count(sparse)
          local compacted_array_count, compacted_array_contiguous, compacted_indexes = table.count_array(sparse)
          debug.assert(compacted_count == 5 and compacted_contiguous
              and compacted_array_count == 2 and compacted_array_contiguous
              and compacted_indexes[1] == 1 and compacted_indexes[2] == 2)
          local empty_count, empty_contiguous, empty_indexes = table.count_array({})
          debug.assert(empty_count == 0 and empty_contiguous and #empty_indexes == 0)
          local keyed_payload = { table = "data" }
          local keyed_count, keyed_contiguous = table.count(keyed_payload)
          debug.assert(keyed_count == 1 and keyed_contiguous)
          debug.assert(fails(function() table.compact(char.ASCII_LETTER) end))
          debug.assert(table.pretty({}) == "{}"
              and fails(function() table.pretty("not a table") end))

          local sortable = { 3, 1, 2 }
          table.sort(sortable)
          debug.assert(sortable[1] == 1 and sortable[2] == 2 and sortable[3] == 3)
          table.sort(sortable, function(left, right) return left > right end)
          debug.assert(sortable[1] == 3 and sortable[2] == 2 and sortable[3] == 1)
          local oversized = {}
          for index = 1, 4097 do oversized[index] = index end
          debug.assert(fails(function() table.sort(oversized) end))

          local child = { value = 7 }
          local original = { child = child, alias = child, callback = function() end }
          local copied = table.deepcopy(original)
          debug.assert(copied ~= original and copied.child ~= child
              and copied.child == copied.alias and copied.child.value == 7
              and copied.callback == original.callback)
          copied.child.value = 9
          debug.assert(original.child.value == 7)
          local cyclic = {}
          cyclic.self = cyclic
          local cyclic_copy = table.deepcopy(cyclic)
          debug.assert(cyclic_copy ~= cyclic and cyclic_copy.self == cyclic_copy)

          local visual = table.pretty({ 1, 3, "a", n = 3 })
          debug.assert(visual == "{[1] = 1, [2] = 3, [3] = \"a\", n = 3}")
          local nested = table.pretty({ text = "a\nb", enabled = true, callback = function() end })
          debug.assert(string.find(nested, "callback", { plain = true }) ~= nil
              and string.find(nested, "<function:", { plain = true }) ~= nil
              and string.find(nested, "\\n", { plain = true }) ~= nil)
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
            return not select(1, debug.pcall(func))
          end

          debug.assert(utf8.len("A你B") == 3)
          debug.assert(utf8.byte_len("A你B") == 5)
          debug.assert(utf8.is_ascii("ABC") and not utf8.is_ascii("A你"))

          debug.assert(utf8.codepoint_to_char{ 65, 20320 } == "A你")
          debug.assert(utf8.codepoint_to_char({ 65, 66 }) == "AB")
          debug.assert(utf8.ascii_to_char{ 65, 66, 67 } == "ABC")
          debug.assert(fails(function() utf8.ascii_to_char{ 128 } end)
              and fails(function() utf8.codepoint_to_char{ 55296 } end))

          local codepoints = utf8.char_to_codepoint("A你B")
          debug.assert(codepoints.n == 3 and codepoints[1] == 65
              and codepoints[2] == 20320 and codepoints[3] == 66)
          local ascii = utf8.char_to_ascii("A你B", { start = 1, finish = 3 })
          debug.assert(ascii.n == 3 and ascii[1] == 65
              and ascii[2] == nil and ascii[3] == 66)
          local one_ascii = utf8.char_to_ascii("ABC", { start = 2 })
          debug.assert(one_ascii.n == 2 and one_ascii[1] == 66 and one_ascii[2] == 67)
          debug.assert(utf8.char_to_ascii("").n == 0
              and utf8.char_to_codepoint("").n == 0)
          local reversed_range = utf8.char_to_codepoint("ABC", { start = 3, finish = 1 })
          debug.assert(reversed_range.n == 0)

          debug.assert(utf8.char_position("A你B", 2) == 2
              and utf8.char_position("A你B", 2, { start = 2 }) == 5
              and utf8.char_position("A你B", 9) == nil)
          debug.assert(fails(function()
              utf8.char_position("ABC", 0)
            end))

          local iterator = utf8.codepoints("A你B")
          local first_position, first_codepoint = iterator()
          local second_position, second_codepoint = iterator()
          local third_position, third_codepoint = iterator()
          debug.assert(first_position == 1 and first_codepoint == 65
              and second_position == 2 and second_codepoint == 20320
              and third_position == 5 and third_codepoint == 66
              and select('#', iterator()) == 1 and iterator() == nil)
          local total = 0
          for byte_position, codepoint in utf8.codepoints(string.rep("x", 20000)) do
            total = total + 1
            if total == 1 then debug.assert(byte_position == 1 and codepoint == 120) end
            if total == 2 then break end
          end
          debug.assert(total == 2)

          local from_start_position, from_start_codepoint = utf8.next("A你B")
          local after_first_position, after_first_codepoint = utf8.next("A你B", { pos = 1 })
          local after_second_position, after_second_codepoint = utf8.next("A你B", { pos = 2 })
          debug.assert(from_start_position == 1 and from_start_codepoint == 65
              and after_first_position == 2 and after_first_codepoint == 20320
              and after_second_position == 5 and after_second_codepoint == 66
              and utf8.next("A你B", { pos = 3 }) == 5
              and utf8.next("A你B", { pos = 5 }) == nil
              and select('#', utf8.next("A你B")) == 2)
          debug.assert(fails(function() utf8.next("ABC", { pos = 0 }) end))
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
            return not select(1, debug.pcall(func))
          end

          debug.assert(math.type(math.PI) == "float")
          debug.assert(math.type(math.E) == "float")
          debug.assert(math.POSITIVE_INFINITE == math.INFINITE)
          debug.assert(math.POSITIVE_INFINITE > math.MAX_INTEGER)
          debug.assert(math.NEGATIVE_INFINITE < math.MIN_INTEGER)
          debug.assert(math.type(math.MAX_INTEGER) == "integer")
          debug.assert(math.type(math.MIN_INTEGER) == "integer")
          debug.assert(math.type(1.5) == "float")
          debug.assert(fails(function() math.type() end))
          debug.assert(near(math.DEG * math.PI, 180.0, 0.000000000001))
          debug.assert(near(math.RAD * 180.0, math.PI, 0.000000000001))

          debug.assert(type(math.lg) == "function")
          debug.assert(type(math.ln) == "function")
          debug.assert(type(math.type) == "function")
          debug.assert(math.log10 == nil and math.number_type == nil)
          debug.assert(type(1) == "number" and type(1.5) == "number")

          debug.assert(math.abs(-5) == 5 and math.type(math.abs(-5)) == "float")
          debug.assert(math.ceil(3.1) == 4 and math.type(math.ceil(3.1)) == "integer")
          debug.assert(math.floor(-3.1) == -4 and math.type(math.floor(-3.1)) == "integer")
          debug.assert(math.round(3.5) == 4 and math.round(-3.5) == -4)
          debug.assert(math.round_to(3.14159, 2) == 3.14)
          debug.assert(math.round_to(12345, -2) == 12300)
          debug.assert(math.round_to(1e308, 1) == 1e308)

          debug.assert(math.fmod(7, 3) == 1)
          debug.assert(math.fmod(-7, 3) == -1)
          debug.assert(math.fmod(math.MIN_INTEGER, -1) == 0)
          debug.assert(math.type(math.fmod(7, 3)) == "integer")
          debug.assert(math.pow(2, 3) == 8)
          debug.assert(near(math.exp(1), math.E, 0.000000000001))
          debug.assert(math.log(8, 2) == 3)
          debug.assert(math.lg(100) == 2)
          debug.assert(near(math.ln(math.E), 1, 0.000000000001))
          debug.assert(math.sqrt(9) == 3)
          debug.assert(math.ldexp(3, 2) == 12)
          debug.assert(math.ldexp(0, math.MAX_INTEGER) == 0)
          debug.assert(math.ldexp(1, math.MIN_INTEGER) == 0)

          local mantissa, exponent = math.frexp(12.8)
          debug.assert(near(mantissa, 0.8, 0.000000000001)
              and exponent == 4
              and math.type(exponent) == "integer"
              and select('#', math.frexp(12.8)) == 2)
          local largest_mantissa, largest_exponent = math.frexp(1.7976931348623157e308)
          debug.assert(largest_mantissa >= 0.5 and largest_mantissa < 1.0
              and largest_exponent == 1024)

          debug.assert(near(math.sin(math.PI / 2), 1, 0.000000000001))
          debug.assert(near(math.cos(0), 1, 0.000000000001))
          debug.assert(near(math.tan(0), 0, 0.000000000001))
          debug.assert(near(math.asin(1), math.PI / 2, 0.000000000001))
          debug.assert(near(math.acos(1), 0, 0.000000000001))
          debug.assert(near(math.atan(1), math.PI / 4, 0.000000000001))
          debug.assert(near(math.atan2(1, 1), math.PI / 4, 0.000000000001))
          debug.assert(near(math.deg(math.PI), 180, 0.000000000001))
          debug.assert(near(math.rad(180), math.PI, 0.000000000001))
          debug.assert(math.normalize_angle(450) == 90)
          debug.assert(math.normalize_angle(-90) == 270)
          debug.assert(math.type(math.normalize_angle(0)) == "float")

          debug.assert(math.max({ 1, 5, 3 }) == 5)
          debug.assert(math.min({ 1, -2, 3 }) == -2)
          debug.assert(math.max(table.pack(4, -2, 7)) == 7)
          local integer_part, fractional_part = math.modf(3.14)
          debug.assert(integer_part == 3
              and math.type(integer_part) == "integer"
              and near(fractional_part, 0.14, 0.000000000001)
              and select('#', math.modf(3.14)) == 2)
          debug.assert(math.tointeger(3.0) == 3)
          debug.assert(math.tointeger(3.14) == nil)
          debug.assert(math.tointeger(9223372036854775808.0) == nil)
          debug.assert(math.type("3") == nil)
          debug.assert(math.ult(-1, 0) == false)
          debug.assert(math.approx_equal(0.1 + 0.2, 0.3))
          debug.assert(math.approx_equal(1.0, 1.005, { epsilon = 0.01 }))
          debug.assert(not math.approx_equal(1.0, 1.01, { epsilon = 0.001 }))
          debug.assert(math.approx_equal(-0.0, 0.0, { epsilon = 0.0 }))
          debug.assert(near(math.percent(5, 16), 0.3125, 0.000000000001))
          debug.assert(math.percent(5, 16, {}) == 0.3125)
          debug.assert(near(math.percent(5, 16, { as_percent = true }), 31.25, 0.000000000001))

          debug.assert(math.factorial(0) == 1)
          debug.assert(math.type(math.factorial(5)) == "float")
          debug.assert(math.factorial(170) > 7e306)
          debug.assert(math.combination(5, 2) == 10)
          debug.assert(math.type(math.combination(5, 2)) == "integer")

          debug.assert(fails(function() math.abs(math.INFINITE) end))
          debug.assert(fails(function() math.ceil(1e20) end))
          debug.assert(fails(function() math.round_to(1, 309) end))
          debug.assert(fails(function() math.fmod(1, 0) end))
          debug.assert(fails(function() math.fmod(9223372036854775808.0, 1) end))
          debug.assert(fails(function() math.fmod{ x = 1, y = 2 } end))
          debug.assert(fails(function() math.pow(-1, 0.5) end))
          debug.assert(fails(function() math.ldexp(1, math.MAX_INTEGER) end))
          debug.assert(fails(function() math.log(2) end))
          debug.assert(fails(function() math.sqrt(-1) end))
          debug.assert(fails(function() math.asin(2) end))
          debug.assert(fails(function() math.max{} end))
          debug.assert(fails(function() math.max({ [1] = 1, [3] = 3, n = 3 }) end))
          debug.assert(fails(function() math.max({ 1, 2, n = 1 }) end))
          debug.assert(fails(function() math.max({ 1, n = "2" }) end))
          debug.assert(fails(function() math.modf(1e20) end))
          debug.assert(fails(function() math.approx_equal(math.INFINITE, 1) end))
          debug.assert(fails(function() math.approx_equal(1, 1, { epsilon = -1 }) end))
          debug.assert(fails(function() math.approx_equal(1, 1, { extra = true }) end))
          debug.assert(fails(function() math.approx_equal(1, 1, true) end))
          debug.assert(fails(function() math.factorial(171) end))
          debug.assert(fails(function() math.combination(67, 33) end))
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
          local result = debug.pcall(function() game.exit_game() end)
          debug.assert(not result)
        end
        function SaveGame()
          local save = debug.pcall(function() game.save_game() end)
          local exit = debug.pcall(function() game.exit_game() end)
          debug.assert(not save and not exit)
          game.save_best()
          return { saved = true }
        end
        function SaveBest()
          local save = debug.pcall(function() game.save_best() end)
          local exit = debug.pcall(function() game.exit_game() end)
          debug.assert(not save and not exit)
          game.save_game()
          return { best_string = "best" }
        end
        function Update(dt)
          local extra_argument = debug.pcall(function() game.exit_game("extra") end)
          debug.assert(not extra_argument)
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
          file.write()
          file.list_dir()
          file.create_dir()
          file.remove()
          debug.info("ignored")
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
  fn event_action_controls_require_a_game_session() {
    let source = valid_script(
      r#"
        function Init(ctx)
          event.skip_action()
          event.clear_action()
        end
      "#,
    );
    let load = |session_kind| {
      let mut session = LuaSession::load_with_api(
        spec(&source, session_kind),
        LuaPolicy::default(),
        LuaApiConfig::default(),
      )
      .unwrap();
      session.take_host_commands()
    };

    let permitted = load(LuaSessionKind::Game);
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

    for commands in [load(LuaSessionKind::Screensaver)] {
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
            debug.assert(file.read_byte == nil)
            debug.assert(file.write_byte == nil)
            debug.assert(file.exists("."))
            debug.assert(file.exists("./input.txt"))
            debug.assert(not file.exists("./missing.txt"))
            list_request_id = file.list_dir(".", { recursive = true })
            read_text_request_id = file.read("./input.txt")
            read_bytes_request_id = file.read("./input.bin", {
              byte = true, event_tip = "read-binary",
            })
            write_text_request_id = file.write("./output.txt", "output")
            write_bytes_request_id = file.write("./output.bin", "a\0\255b", {
              byte = true,
              event_tip = "write-binary",
            })
            create_dir_request_id = file.create_dir("./created/nested/leaf", { event_tip = "created" })
            remove_request_id = file.remove("./input.txt", { event_tip = "removed" })
            debug.assert(type(list_request_id) == "number"
                and type(read_text_request_id) == "number"
                and type(read_bytes_request_id) == "number"
                and type(write_text_request_id) == "number"
                and type(write_bytes_request_id) == "number"
                and type(create_dir_request_id) == "number"
                and type(remove_request_id) == "number")
            local empty = debug.pcall(function() file.list_dir("") end)
            local traversal = debug.pcall(function() file.list_dir("./folder/../") end)
            debug.assert(not empty and not traversal)
          end

          function HandleEvent(event)
            if event.type == "file" and event.data.ok then
              completed_request_id = event.data.request_id
            end
          end
        "#,
      ),
    )
    .unwrap();
    let mut session = LuaSession::load_with_api(
      LuaSessionSpec {
        package_id: "test_assets_root".to_string(),
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
      LuaApiConfig::default(),
    )
    .unwrap();

    let expected_root = assets_root.canonicalize().unwrap();
    let requests = session.take_host_commands();
    let lua_request_id = |name| {
      let Value::Integer(value) = session.environment_value(name) else {
        panic!("{name} should return a Lua integer request id");
      };
      value as u64
    };
    let read_text_request_id = lua_request_id("read_text_request_id");
    assert!(requests.iter().any(|command| matches!(
      command,
      LuaHostCommand::FileRequest {
        task: tg_service_file::FileTask::LuaListDir { path, recursive: true, .. },
        virtual_path,
        ..
      } if path == &expected_root && virtual_path == "."
    )));
    assert!(requests.iter().any(|command| matches!(
      command,
      LuaHostCommand::FileRequest {
        task: tg_service_file::FileTask::LuaReadText { path, .. },
        virtual_path,
        request_id,
        operation: LuaFileOperation::ReadText,
        ..
      } if path == &expected_root.join("input.txt")
        && virtual_path == "input.txt"
        && *request_id == read_text_request_id
    )));
    assert!(requests.iter().any(|command| matches!(
      command,
      LuaHostCommand::FileRequest {
        task: tg_service_file::FileTask::LuaReadBytes { path },
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
        task: tg_service_file::FileTask::LuaWriteText { path, .. },
        virtual_path,
        ..
      } if path == &expected_root.join("output.txt") && virtual_path == "output.txt"
    )));
    assert!(requests.iter().any(|command| matches!(
      command,
      LuaHostCommand::FileRequest {
        task: tg_service_file::FileTask::LuaWriteBytes { path, bytes },
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
        task: tg_service_file::FileTask::LuaCreateDir { path, .. },
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
        task: tg_service_file::FileTask::LuaRemove { path, recursive: false, .. },
        operation: LuaFileOperation::Remove,
        virtual_path,
        event_tip: Some(event_tip),
        ..
      } if path == &expected_root.join("input.txt")
        && virtual_path == "input.txt"
        && event_tip == "removed"
    )));

    session
      .dispatch_event(&LuaEventDelivery {
        event: LuaRuntimeEvent {
          sequence: 1,
          frame: 1,
          data: LuaEventData::File(super::super::LuaFileEvent {
            request_id: read_text_request_id,
            kind: LuaFileOperation::ReadText,
            path: "input.txt".to_string(),
            tip: None,
            outcome: super::super::LuaFileOutcome::Text("input".to_string()),
          }),
        },
        route: LuaEventRoute::HandleEvent,
      })
      .unwrap();
    assert_eq!(
      session.environment_value("completed_request_id"),
      Value::Integer(read_text_request_id as i64)
    );

    fs::remove_dir_all(package_root).unwrap();
  }

  #[test]
  fn game_file_access_and_debug_logging_are_independently_gated() {
    let package_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
      .join("../../..")
      .join("test_package/game/permissions_lab");
    let entry_path = package_root.join("scripts/main.lua");
    let make_spec = || LuaSessionSpec {
      package_id: "test_permissions_lab".to_string(),
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

    let mut game_without_debug =
      LuaSession::load_with_api(make_spec(), LuaPolicy::default(), LuaApiConfig::default())
        .unwrap();
    game_without_debug.handle_event(&action).unwrap();
    let game_commands = game_without_debug.take_host_commands();
    assert!(game_commands.iter().any(|command| matches!(
      command,
      LuaHostCommand::FileRequest {
        operation: LuaFileOperation::WriteText,
        virtual_path,
        ..
      } if virtual_path == "state/probe.log"
    )));
    assert!(
      !game_commands
        .iter()
        .any(|command| matches!(command, LuaHostCommand::Print { .. }))
    );

    let mut permitted = LuaSession::load_with_api(
      make_spec(),
      LuaPolicy::default(),
      LuaApiConfig {
        debug_enabled: true,
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
          debug.assert(_ENV ~= nil and _G == nil and rawget(_ENV, "_G") == nil
              and rawget(_ENV, "base") == base and rawget(_ENV, "rawget") == rawget
              and load == nil and rawget(_ENV, "package") == nil
              and debug.getregistry == nil)
          local extra_require_argument = debug.pcall(function() loader.require("cached", "extra") end)
          debug.assert(not extra_require_argument)
          local first, first_gap, first_tail = loader.require("cached")
          local second, second_gap, second_tail = loader.require("./cached.lua")
          debug.assert(first == second and first.count == 1 and first.leaked == "main-only"
              and first.sandboxed and first.global_hidden)
          debug.assert(first_gap == nil and second_gap == nil and first_tail == 3 and second_tail == 3)

          local fresh_first = loader.dofile("fresh")
          local fresh_second = loader.dofile("./fresh.lua")
          debug.assert(fresh_first ~= fresh_second
              and fresh_first.count == 1 and fresh_second.count == 2)

          local compiled_first = loader.loadfile("compiled")
          local compiled_second = loader.loadfile("compiled.lua")
          debug.assert(type(compiled_first) == "function"
              and type(compiled_second) == "function"
              and compiled_first ~= compiled_second
              and compiled_count == nil)
          local compiled_result_first = compiled_first()
          local compiled_result_second = compiled_second()
          debug.assert(compiled_result_first ~= compiled_result_second
              and compiled_result_first.count == 1 and compiled_result_second.count == 2)

          local traversal_ok = debug.pcall(function() loader.require("../outside") end)
          local extension_ok = debug.pcall(function() loader.dofile("module.txt") end)
          local bytecode_ok = debug.pcall(function() loader.loadfile("bytecode.lua") end)
          local cycle_ok = debug.pcall(function() loader.require("cycle") end)
          debug.assert(not traversal_ok and not extension_ok and not bytecode_ok
              and not cycle_ok)
        end
      "#,
    );
    let session_spec = spec(&source, LuaSessionKind::Game);
    let scripts_root = session_spec.entry_path.parent().unwrap();
    fs::write(
      scripts_root.join("cached.lua"),
      "required_count = (required_count or 0) + 1\nreturn { count = required_count, leaked = private_state, sandboxed = _ENV ~= nil and _G == nil and debug.getregistry == nil, global_hidden = load == nil and package == nil }, nil, 3",
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
    let source =
      valid_script("function Update(dt) debug.pcall(function() while true do end end) end");
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
          for _, item in ipairs(char.ASCII) do
            x = x + 1
            if x > 20 then
              x = 1
              y = y + 1
            end
            draw.text(x, y, item)
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
    assert_eq!(save["package_id"], "test_package");
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
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
    for (directory, session_kind, expected_package_count) in [
      ("test_package/game", LuaSessionKind::Game, 4),
      ("test_package/screensaver", LuaSessionKind::Screensaver, 3),
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
        package_count, expected_package_count,
        "unexpected test package count in {directory}"
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
        package_id: "invalid_utf8".to_string(),
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
        package_id: "invalid_extension".to_string(),
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
      r#"
        function Update(dt)
          debug.pcall(function()
            local values = {}
            while true do values[#values + 1] = {} end
          end)
        end
      "#,
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

    let mut healthy_session = LuaSession::load(
      spec(&valid_script(""), LuaSessionKind::Game),
      LuaPolicy::default(),
    )
    .unwrap();
    healthy_session.update().unwrap();
  }

  #[test]
  fn migrated_documentation_lua_examples_parse() {
    fn collect_markdown_files(directory: &Path, files: &mut Vec<PathBuf>) {
      for entry in fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
          collect_markdown_files(&path, files);
        } else if path.extension().is_some_and(|extension| extension == "md") {
          files.push(path);
        }
      }
    }

    let docs_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../dev_docs/zh_cn");
    let mut markdown_files = Vec::new();
    collect_markdown_files(&docs_root.join("api"), &mut markdown_files);
    let mut format_files = Vec::new();
    collect_markdown_files(&docs_root.join("format"), &mut format_files);
    format_files.retain(|path| !path.file_name().is_some_and(|name| name == "EVENT.md"));
    markdown_files.extend(format_files);
    markdown_files.extend(
      [
        "API.md",
        "CALLBACK.md",
        "EVENT.md",
        "LUA_API_MIGRATION.md",
        "LUA_COMPATIBILITY.md",
      ]
      .into_iter()
      .map(|name| docs_root.join(name)),
    );
    markdown_files.push(docs_root.join("education/I18N.md"));

    let lua = Lua::new();
    let mut example_count = 0;
    for path in markdown_files {
      let markdown = fs::read_to_string(&path).unwrap();
      let mut in_lua_fence = false;
      let mut compile_example = true;
      let mut in_call_section = false;
      let mut in_output_section = false;
      let mut example = String::new();
      let mut fence_line = 0;

      for (line_index, line) in markdown.lines().enumerate() {
        if line.trim_start().starts_with('#') {
          in_call_section = line.trim() == "### 调用";
          in_output_section = false;
        }
        if line.trim() == "输出：" || line.trim().starts_with("**输出") {
          in_output_section = true;
        }
        if line.trim() == "```lua" && !in_lua_fence {
          in_lua_fence = true;
          compile_example = !in_call_section && !in_output_section;
          fence_line = line_index + 1;
          example.clear();
        } else if line.trim() == "```" && in_lua_fence {
          if compile_example {
            example_count += 1;
            let chunk = if example.trim_start().starts_with('{') {
              format!("return {example}")
            } else {
              example.clone()
            };
            lua.load(&chunk).into_function().unwrap_or_else(|error| {
              panic!(
                "invalid Lua example in {} starting at line {}: {error}",
                path.display(),
                fence_line
              )
            });
          }
          in_lua_fence = false;
        } else if in_lua_fence {
          example.push_str(line);
          example.push('\n');
        }
      }

      assert!(
        !in_lua_fence,
        "unclosed Lua example in {} starting at line {}",
        path.display(),
        fence_line
      );
    }

    assert!(example_count > 0, "no API documentation Lua examples found");
  }
}
