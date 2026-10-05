//! Api support for the lua service.

mod args;
mod libraries;
pub(crate) mod readonly;

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Instant;

use mlua::{Lua, Table};

use super::LuaSessionKind;
use super::object_pool::WeakLuaObjectPool;
use super::{LuaI18nEvent, LuaI18nEventKind, LuaImageEvent};
use crate::LuaFileOperation;
use tg_core_style::TextColor;
use tg_service_file::FileTask;
use tg_service_image::ImageConvertParams;
use tg_service_layout::Size;
use tg_service_random::RandomGeneratorId;
use tg_service_render::BorderStyle;
use tg_service_text_layout::DrawTextParams;
use tg_service_widget::SliceId;

/// The script callback phase currently using the host API.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LuaCallPhase {
  /// The loading stage of the operation.
  Loading,
  /// The init stage of the operation.
  Init,
  /// The event stage of the operation.
  Event,
  /// The update stage of the operation.
  Update,
  /// The update frame stage of the operation.
  UpdateFrame,
  /// The render stage of the operation.
  Render,
  /// The save game stage of the operation.
  SaveGame,
  /// The save best stage of the operation.
  SaveBest,
  /// The idle stage of the operation.
  Idle,
}

/// Package-scoped paths, capabilities, bindings, and initial data for API construction.
///
/// # Fields
///
/// * `debug_enabled` - The debug enabled.
/// * `key_actions` - The key actions indexed by their declared keys.
/// * `key_default_actions` - The key default actions indexed by their declared keys.
/// * `language_code` - The registered language code.
/// * `missing_i18n_template` - The missing i18n template.
#[derive(Clone, Debug)]
pub struct LuaApiConfig {
  /// The debug enabled.
  pub debug_enabled: bool,
  /// The key actions indexed by their declared keys.
  pub key_actions: HashMap<String, Vec<Vec<String>>>,
  /// The key default actions indexed by their declared keys.
  pub key_default_actions: HashMap<String, Vec<Vec<String>>>,
  /// The registered language code.
  pub language_code: String,
  /// The missing i18n template.
  pub missing_i18n_template: String,
}

impl Default for LuaApiConfig {
  fn default() -> Self {
    Self {
      debug_enabled: false,
      key_actions: HashMap::new(),
      key_default_actions: HashMap::new(),
      language_code: "en_us".to_string(),
      missing_i18n_template: "[Missing i18n Key: {value:missing_key}]".to_string(),
    }
  }
}

/// The service access and configuration supplied while constructing a Lua host API.
///
/// # Fields
///
/// * `package_id` - The stable source, type, and name of the package.
/// * `session_kind` - The session kind.
/// * `scripts_root` - The scripts root.
/// * `assets_root` - The assets root.
/// * `debug_enabled` - The debug enabled.
/// * `base_size` - The base size.
/// * `key_actions` - The key actions indexed by their declared keys.
/// * `key_default_actions` - The key default actions indexed by their declared keys.
/// * `language_code` - The registered language code.
/// * `missing_i18n_template` - The missing i18n template.
#[derive(Clone, Debug)]
pub struct LuaApiContext {
  /// The stable source, type, and name of the package.
  pub package_id: String,
  /// The session kind.
  pub session_kind: LuaSessionKind,
  /// The scripts root.
  pub scripts_root: PathBuf,
  /// The assets root.
  pub assets_root: PathBuf,
  /// The debug enabled.
  pub debug_enabled: bool,
  /// The base size.
  pub base_size: Size,
  /// The key actions indexed by their declared keys.
  pub key_actions: HashMap<String, Vec<Vec<String>>>,
  /// The key default actions indexed by their declared keys.
  pub key_default_actions: HashMap<String, Vec<Vec<String>>>,
  /// The registered language code.
  pub language_code: String,
  /// The missing i18n template.
  pub missing_i18n_template: String,
}

/// A session-owned base, slice, or scroll-box drawing destination.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LuaDrawTarget {
  /// The base setting for Lua draw target.
  Base,
  /// The slice setting for Lua draw target.
  Slice(SliceId),
}

/// A structured drawing request collected during a script callback.
#[derive(Clone, Debug)]
pub enum LuaDrawCommand {
  /// The text setting for Lua draw command.
  Text {
    /// The object or resource affected by the operation.
    target: LuaDrawTarget,
    /// The horizontal coordinate in terminal cells.
    x: i32,
    /// The vertical coordinate in terminal cells.
    y: i32,
    /// The formatting or rendering parameters.
    params: DrawTextParams,
  },
  /// The fill rect setting for Lua draw command.
  FillRect {
    /// The object or resource affected by the operation.
    target: LuaDrawTarget,
    /// The horizontal coordinate in terminal cells.
    x: i32,
    /// The vertical coordinate in terminal cells.
    y: i32,
    /// The width in terminal columns.
    width: u16,
    /// The height in terminal rows.
    height: u16,
    /// The fill char.
    fill_char: Option<String>,
    /// The foreground color override, or `None` to inherit the default.
    fg: Option<TextColor>,
    /// The background color override, or `None` to inherit the default.
    bg: Option<TextColor>,
  },
  /// The stroke rect setting for Lua draw command.
  StrokeRect {
    /// The object or resource affected by the operation.
    target: LuaDrawTarget,
    /// The horizontal coordinate in terminal cells.
    x: i32,
    /// The vertical coordinate in terminal cells.
    y: i32,
    /// The width in terminal columns.
    width: u16,
    /// The height in terminal rows.
    height: u16,
    /// The border.
    border: BorderStyle,
    /// The foreground color override, or `None` to inherit the default.
    fg: Option<TextColor>,
    /// The background color override, or `None` to inherit the default.
    bg: Option<TextColor>,
  },
  /// The erase rect setting for Lua draw command.
  EraseRect {
    /// The object or resource affected by the operation.
    target: LuaDrawTarget,
    /// The horizontal coordinate in terminal cells.
    x: i32,
    /// The vertical coordinate in terminal cells.
    y: i32,
    /// The width in terminal columns.
    width: u16,
    /// The height in terminal rows.
    height: u16,
  },
}

/// A script request to the host for actions outside immediate drawing.
#[derive(Clone, Debug)]
pub enum LuaHostCommand {
  /// The log setting for Lua host command.
  Log {
    /// The level.
    level: String,
    /// The diagnostic or display message.
    message: String,
  },
  /// The print setting for Lua host command.
  Print {
    /// The diagnostic or display message.
    message: String,
    /// The title.
    title: Option<String>,
    /// Whether the printed log header includes time.
    time: bool,
    /// The level.
    level: Option<String>,
    /// Whether the printed log header includes its source category.
    type_head: bool,
  },
  /// The ignored setting for Lua host command.
  Ignored {
    /// The method.
    method: &'static str,
    /// The reason.
    reason: &'static str,
  },
  /// The exit game setting for Lua host command.
  ExitGame,
  /// A request to save game.
  SaveGame,
  /// A request to save best.
  SaveBest,
  /// The skip actions setting for Lua host command.
  SkipActions,
  /// Close action or key input after the current callback without reentering Lua.
  InputRejected { actions: bool, keys: bool },
  /// A request to clear actions.
  ClearActions,
  /// The file request setting for Lua host command.
  FileRequest {
    /// The identifier of the request.
    request_id: u64,
    /// The task.
    task: FileTask,
    /// The operation to execute within the boundary.
    operation: LuaFileOperation,
    /// The filesystem path for virtual.
    virtual_path: String,
    /// The event tip.
    event_tip: Option<String>,
  },
  /// The i18n request setting for Lua host command.
  I18nRequest {
    /// The identifier of the request.
    request_id: u64,
    /// The task.
    task: FileTask,
    /// The Lua i18n event kind carried by this Lua host command.
    kind: LuaI18nEventKind,
    /// The registered language code.
    language_code: String,
    /// The callback language code.
    callback_language_code: String,
  },
  /// The image request setting for Lua host command.
  ImageRequest {
    /// The identifier of the request.
    request_id: u64,
    /// The formatting or rendering parameters.
    params: ImageConvertParams,
  },
  /// The draw setting for Lua host command.
  Draw(LuaDrawCommand),
}

/// Session-local API queues, callback registrations, and owned object access.
///
/// # Fields
///
/// * `context` - The state and services needed for the operation.
/// * `objects` - The session or page object pool.
/// * `phase` - The Lua call phase carried by this Lua api state.
/// * `commands` - The ordered commands retained by this owner.
/// * `draw_command_count` - The draw command count.
/// * `draw_text_bytes` - The draw text measured in bytes.
/// * `loader_stack` - The ordered loader stack retained by this owner.
/// * `loader_source_bytes` - The loader source measured in bytes.
/// * `next_file_request_id` - The identifier of the next file request.
/// * `next_i18n_request_id` - The identifier of the next i18n request.
/// * `next_image_request_id` - The identifier of the next image request.
/// * `pending_file_request_ids` - File requests awaiting their result events.
/// * `pending_image_request_ids` - The pending image request ids.
/// * `i18n` - The service resolving localized text.
/// * `direct_random_id` - The identifier of the direct random.
/// * `ignored_methods` - The ignored methods.
/// * `fatal_budget_exceeded` - The fatal budget exceeded.
/// * `fatal_api_error` - The fatal api error.
/// * `debug_log_window_started` - The debug log window started.
/// * `debug_log_count` - The debug log count.
/// * `debug_log_dropped` - The debug log dropped.
#[derive(Debug)]
pub(crate) struct LuaApiState {
  /// The state and services needed for the operation.
  pub context: LuaApiContext,
  /// The session or page object pool.
  pub objects: WeakLuaObjectPool,
  /// The Lua call phase carried by this Lua api state.
  pub phase: LuaCallPhase,
  /// The ordered commands retained by this owner.
  pub commands: Vec<LuaHostCommand>,
  /// The draw command count.
  pub draw_command_count: usize,
  /// The draw text measured in bytes.
  pub draw_text_bytes: usize,
  /// The ordered loader stack retained by this owner.
  pub loader_stack: Vec<PathBuf>,
  /// The loader source measured in bytes.
  pub loader_source_bytes: usize,
  /// The identifier of the next file request.
  pub next_file_request_id: u64,
  /// The identifier of the next i18n request.
  pub next_i18n_request_id: u64,
  /// The identifier of the next image request.
  pub next_image_request_id: u64,
  /// File requests awaiting their result events.
  pub pending_file_request_ids: HashSet<u64>,
  /// The pending image request ids.
  pub pending_image_request_ids: HashSet<u64>,
  /// The service resolving localized text.
  pub i18n: LuaI18nState,
  /// Session-local subscriptions, delivery records, and closing releases.
  pub input: super::input::LuaInputState,
  /// The identifier of the direct random.
  pub direct_random_id: Option<RandomGeneratorId>,
  /// The ignored methods.
  pub ignored_methods: HashSet<&'static str>,
  /// The fatal budget exceeded.
  pub fatal_budget_exceeded: bool,
  /// The fatal api error.
  pub fatal_api_error: bool,
  /// The debug log window started.
  pub debug_log_window_started: Instant,
  /// The debug log count.
  pub debug_log_count: u32,
  /// The debug log dropped.
  pub debug_log_dropped: u32,
}

/// Package translations and the active runtime language exposed to one Lua session.
///
/// # Fields
///
/// * `created` - The created.
/// * `loading` - The loading.
/// * `language_code` - The registered language code.
/// * `callback_language_code` - The callback language code.
/// * `namespaces` - The namespaces indexed by their declared keys.
#[derive(Debug, Default)]
pub(crate) struct LuaI18nState {
  /// The created.
  pub created: bool,
  /// The loading.
  pub loading: bool,
  /// The registered language code.
  pub language_code: Option<String>,
  /// The callback language code.
  pub callback_language_code: Option<String>,
  /// The namespaces indexed by their declared keys.
  pub namespaces: HashMap<String, HashMap<String, String>>,
}

/// Shared interior-mutable access to one session API state.
pub(crate) type SharedApiState = Rc<RefCell<LuaApiState>>;

/// Build the session's isolated Lua environment and install its allowed host libraries.
///
/// # Arguments
///
/// * `lua` - The Lua VM in which values and callbacks are created.
/// * `context` - The state and services needed for the operation.
/// * `objects` - The session or page object pool.
///
/// # Errors
///
/// Propagate Lua errors while creating the isolated environment, installing libraries, or
/// configuring its read-only tables.
pub(crate) fn build_environment(
  lua: &Lua,
  context: LuaApiContext,
  objects: WeakLuaObjectPool,
) -> mlua::Result<(Table, SharedApiState)> {
  let state = Rc::new(RefCell::new(LuaApiState {
    context,
    objects,
    phase: LuaCallPhase::Loading,
    commands: Vec::new(),
    draw_command_count: 0,
    draw_text_bytes: 0,
    loader_stack: Vec::new(),
    loader_source_bytes: 0,
    next_file_request_id: 1,
    next_i18n_request_id: 1,
    next_image_request_id: 1,
    pending_file_request_ids: HashSet::new(),
    pending_image_request_ids: HashSet::new(),
    i18n: LuaI18nState::default(),
    input: super::input::LuaInputState::default(),
    direct_random_id: None,
    ignored_methods: HashSet::new(),
    fatal_budget_exceeded: false,
    fatal_api_error: false,
    debug_log_window_started: Instant::now(),
    debug_log_count: 0,
    debug_log_dropped: 0,
  }));
  let environment = lua.create_table()?;
  libraries::install(lua, &environment, state.clone())?;
  Ok((environment, state))
}

/// Apply a completed package translation request to the matching session context.
pub(crate) fn apply_i18n_event(state: &SharedApiState, event: &LuaI18nEvent) {
  let mut state = state.borrow_mut();
  state.i18n.loading = false;
  if event.ok {
    state.i18n.created = true;
    state.i18n.language_code = Some(event.language_code.clone());
    state.i18n.callback_language_code = Some(event.callback_language_code.clone());
    if let Some(namespaces) = &event.namespaces {
      state.i18n.namespaces = namespaces.clone();
    }
  } else if event.kind == LuaI18nEventKind::Created {
    state.i18n.created = false;
  }
}

/// Release the file request's reserved capacity before its result callback runs.
pub(crate) fn apply_file_event(state: &SharedApiState, event: &crate::LuaFileEvent) {
  state
    .borrow_mut()
    .pending_file_request_ids
    .remove(&event.request_id);
}

/// Apply a completed image request to the matching session context.
pub(crate) fn apply_image_event(state: &SharedApiState, event: &LuaImageEvent) {
  state
    .borrow_mut()
    .pending_image_request_ids
    .remove(&event.request_id);
}
