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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LuaCallPhase {
  Loading,
  Init,
  Event,
  Update,
  UpdateFrame,
  Render,
  SaveGame,
  SaveBest,
  Idle,
}

#[derive(Clone, Debug)]
pub struct LuaApiConfig {
  pub debug_enabled: bool,
  pub key_actions: HashMap<String, Vec<Vec<String>>>,
  pub key_default_actions: HashMap<String, Vec<Vec<String>>>,
  pub language_code: String,
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

#[derive(Clone, Debug)]
pub struct LuaApiContext {
  pub package_id: String,
  pub session_kind: LuaSessionKind,
  pub scripts_root: PathBuf,
  pub assets_root: PathBuf,
  pub debug_enabled: bool,
  pub base_size: Size,
  pub key_actions: HashMap<String, Vec<Vec<String>>>,
  pub key_default_actions: HashMap<String, Vec<Vec<String>>>,
  pub language_code: String,
  pub missing_i18n_template: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LuaDrawTarget {
  Base,
  Slice(SliceId),
}

#[derive(Clone, Debug)]
pub enum LuaDrawCommand {
  Text {
    target: LuaDrawTarget,
    x: i32,
    y: i32,
    params: DrawTextParams,
  },
  FillRect {
    target: LuaDrawTarget,
    x: i32,
    y: i32,
    width: u16,
    height: u16,
    fill_char: Option<String>,
    fg: Option<TextColor>,
    bg: Option<TextColor>,
  },
  StrokeRect {
    target: LuaDrawTarget,
    x: i32,
    y: i32,
    width: u16,
    height: u16,
    border: BorderStyle,
    fg: Option<TextColor>,
    bg: Option<TextColor>,
  },
  EraseRect {
    target: LuaDrawTarget,
    x: i32,
    y: i32,
    width: u16,
    height: u16,
  },
}

#[derive(Clone, Debug)]
pub enum LuaHostCommand {
  Log {
    level: String,
    message: String,
  },
  Print {
    message: String,
    title: Option<String>,
    time: bool,
    level: Option<String>,
    type_head: bool,
  },
  Ignored {
    method: &'static str,
    reason: &'static str,
  },
  ExitGame,
  SaveGame,
  SaveBest,
  SkipActions,
  ClearActions,
  RequestRender,
  FileRequest {
    request_id: u64,
    task: FileTask,
    operation: LuaFileOperation,
    virtual_path: String,
    event_tip: Option<String>,
  },
  I18nRequest {
    task: FileTask,
    kind: LuaI18nEventKind,
    language_code: String,
    callback_language_code: String,
  },
  ImageRequest {
    request_id: u64,
    params: ImageConvertParams,
  },
  Draw(LuaDrawCommand),
}

#[derive(Debug)]
pub(crate) struct LuaApiState {
  pub context: LuaApiContext,
  pub objects: WeakLuaObjectPool,
  pub phase: LuaCallPhase,
  pub commands: Vec<LuaHostCommand>,
  pub draw_command_count: usize,
  pub draw_text_bytes: usize,
  pub loader_stack: Vec<PathBuf>,
  pub loader_source_bytes: usize,
  pub next_file_request_id: u64,
  pub next_image_request_id: u64,
  pub pending_image_request_ids: HashSet<u64>,
  pub i18n: LuaI18nState,
  pub direct_random_id: Option<RandomGeneratorId>,
  pub ignored_methods: HashSet<&'static str>,
  pub fatal_budget_exceeded: bool,
  pub fatal_api_error: bool,
  pub debug_log_window_started: Instant,
  pub debug_log_count: u32,
  pub debug_log_dropped: u32,
}

#[derive(Debug, Default)]
pub(crate) struct LuaI18nState {
  pub created: bool,
  pub loading: bool,
  pub language_code: Option<String>,
  pub callback_language_code: Option<String>,
  pub namespaces: HashMap<String, HashMap<String, String>>,
}

pub(crate) type SharedApiState = Rc<RefCell<LuaApiState>>;

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
    next_image_request_id: 1,
    pending_image_request_ids: HashSet::new(),
    i18n: LuaI18nState::default(),
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

pub(crate) fn apply_image_event(state: &SharedApiState, event: &LuaImageEvent) {
  state
    .borrow_mut()
    .pending_image_request_ids
    .remove(&event.request_id);
}
