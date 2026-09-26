//! Package service: scans, validates and hot-reloads game/screensaver package manifests.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs::File;
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::sync::{
  Arc,
  atomic::{AtomicBool, Ordering},
};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};
use tg_service_async::AsyncRuntime;

use crossbeam_channel::{Receiver, RecvTimeoutError, Sender, unbounded};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::Deserialize;

pub use tg_core_package_id::{PackageId, PackageSource, PackageType};

use tg_core_input::canonical_key_token;
use tg_core_log::{HostLogMessage, LogSource};
use tg_core_version::{HOST_API_VERSION, PACKAGE_MANIFEST_VERSION};
use tg_service_async::{ManagedThreadId, TaskId};
use tg_service_log::LogService;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

const VALID_TARGET_FPS: &[u32] = &[30, 60, 120];
const MAX_PACKAGE_MANIFEST_BYTES: usize = 1024 * 1024;
const MAX_PACKAGE_TEXT_BYTES: usize = 1024 * 1024;
const MAX_PACKAGE_I18N_BYTES: usize = 1024 * 1024;

/// 包文本字段：普通字符串直接使用，对象形式明确声明纯文本或包内 i18n。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PackageText {
  Literal(String),
  I18n {
    path: String,
    key: String,
    callback: String,
  },
}

/// 包完整信息
#[derive(Clone, Debug)]
pub struct PackageInfo {
  pub id: PackageId,
  pub source: PackageSource,
  pub dir_name: String,
  pub mod_id: String,
  pub package_type: PackageType,
  pub version: String,
  pub version_code: u32,
  pub api_min: u32,
  pub api_max: u32,
  pub entry: String,
  pub display: PackageDisplay,
  pub runtime: PackageRuntime,
  pub game: Option<GameConfig>,
  pub screensaver: Option<ScreensaverConfig>,
  pub path: PathBuf,
  watched_files: Vec<PathBuf>,
}

/// 面向 UI 列表的轻量包条目快照。
#[derive(Clone, Debug)]
pub struct PackageListEntry {
  pub id: PackageId,
  pub mod_id: String,
  pub source: PackageSource,
  pub package_type: PackageType,
  pub key_actions: HashMap<String, Vec<Vec<String>>>,
  pub key_default_actions: HashMap<String, Vec<Vec<String>>>,
  pub title: String,
  pub game_name: String,
  pub screensaver_name: String,
  pub game_detail: String,
  pub description: String,
  pub author: String,
  pub version: String,
  pub icon: PackageAsset,
  pub icon_path: Option<String>,
  pub banner: PackageAsset,
  pub path: PathBuf,
  pub enabled: bool,
  pub debug: bool,
  pub safe_mode: bool,
  pub mouse_required: bool,
  pub truecolor_required: bool,
  pub high_privilege_required: bool,
  pub supported_languages: Vec<String>,
  pub score_enabled: bool,
  pub score_empty_text: String,
  pub best_string: Option<String>,
  pub min_width: u32,
  pub min_height: u32,
  pub screensaver_command: String,
}

/// 包显示信息
#[derive(Clone, Debug)]
pub struct PackageDisplay {
  pub title: String,
  pub description: String,
  pub author: String,
  pub icon: PackageAsset,
  pub banner: PackageAsset,
}

/// 包展示资源。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PackageAsset {
  Image { path: String },
  Text { path: String, lines: Vec<String> },
}

impl PackageAsset {
  pub fn default_icon() -> Self {
    default_icon_asset()
  }

  pub fn default_banner() -> Self {
    default_banner_asset()
  }
}

/// 包运行时要求
#[derive(Clone, Debug)]
pub struct PackageRuntime {
  pub min_width: u32,
  pub min_height: u32,
}

/// 游戏配置
#[derive(Clone, Debug)]
pub struct GameConfig {
  pub name: String,
  pub detail: String,
  pub high_privilege: bool,
  pub mouse: bool,
  pub truecolor: bool,
  pub target_fps: u32,
  pub save: bool,
  pub supported_languages: Vec<String>,
  pub score: Option<ScoreConfig>,
  pub actions: HashMap<String, ActionConfig>,
  pub action_order: Vec<String>,
}

/// 分数配置
#[derive(Clone, Debug)]
pub struct ScoreConfig {
  pub enabled: bool,
  pub empty_text: String,
}

/// 动作绑定配置
#[derive(Clone, Debug)]
pub struct ActionConfig {
  pub description: String,
  pub keys: Vec<Vec<String>>,
  pub lock: bool,
}

/// 屏保配置
#[derive(Clone, Debug)]
pub struct ScreensaverConfig {
  pub name: String,
  pub truecolor: bool,
  pub command: String,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct PackageSnapshot {
  games: Vec<PackageInfo>,
  screensavers: Vec<PackageInfo>,
}

#[derive(Clone, Debug)]
pub(crate) struct ScanRequest {
  root: PathBuf,
  language_code: String,
  missing_template: String,
}

#[derive(Clone, Debug)]
pub enum PackageTask {
  Scan(ScanRequest),
}

#[derive(Clone, Debug)]
pub enum PackageAsyncEvent {
  Event(PackageEvent),
  WatchChanged {
    package_dirs: Vec<PathBuf>,
  },
  SnapshotReady {
    snapshot: PackageSnapshot,
    finished: PackageEvent,
    watched_files: Vec<PathBuf>,
  },
}

#[derive(Clone, Debug)]
pub enum PackageEvent {
  Info(HostLogMessage),
  Warn(HostLogMessage),
  PackageWarn {
    package_id: PackageId,
    message: HostLogMessage,
  },
  Diagnostic {
    package_id: Option<PackageId>,
    diagnostic: PackageDiagnostic,
  },
  Loaded {
    package_id: PackageId,
    relative_package_path: String,
  },
  ScanStarted {
    total: usize,
  },
  ScanProgress {
    scanned: usize,
    total: usize,
  },
  WatchChanged {
    folders: usize,
  },
  ScanFinished {
    total: usize,
    games: usize,
    screensavers: usize,
    errors: u32,
    duplicates: u32,
  },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageDiagnostic {
  pub code: String,
  pub relative_package_path: String,
  pub field_path: String,
  pub line: Option<usize>,
  pub column: Option<usize>,
  pub reason: String,
}

#[derive(Debug)]
struct PackageReadError {
  code: &'static str,
  field_path: String,
  line: Option<usize>,
  column: Option<usize>,
  reason: String,
  related: Vec<PackageReadError>,
}

impl From<String> for PackageReadError {
  fn from(reason: String) -> Self {
    Self::semantic(reason)
  }
}

impl From<&str> for PackageReadError {
  fn from(reason: &str) -> Self {
    Self::semantic(reason.to_string())
  }
}

impl PackageReadError {
  fn semantic(reason: String) -> Self {
    Self::at(infer_field_path(&reason), reason)
  }

  fn at(field_path: impl Into<String>, reason: impl Into<String>) -> Self {
    Self {
      code: "semantic",
      field_path: field_path.into(),
      line: None,
      column: None,
      reason: reason.into(),
      related: Vec::new(),
    }
  }

  fn combine(mut errors: Vec<Self>) -> Self {
    debug_assert!(!errors.is_empty());
    let mut first = errors.remove(0);
    first.related.extend(errors);
    first
  }

  fn into_diagnostics(self, relative_package_path: String) -> Vec<PackageDiagnostic> {
    let mut diagnostics = vec![PackageDiagnostic {
      code: self.code.to_string(),
      relative_package_path: relative_package_path.clone(),
      field_path: self.field_path,
      line: self.line,
      column: self.column,
      reason: self.reason,
    }];
    diagnostics.extend(
      self
        .related
        .into_iter()
        .flat_map(|error| error.into_diagnostics(relative_package_path.clone())),
    );
    diagnostics
  }

  fn push_if(errors: &mut Vec<Self>, condition: bool, field_path: &str, reason: String) {
    if condition {
      errors.push(Self::at(field_path, reason));
    }
  }
}

struct ScanReport {
  snapshot: PackageSnapshot,
  events: Vec<PackageEvent>,
  errors: u32,
  duplicates: u32,
  watched_files: Vec<PathBuf>,
}

/// 包管理服务，负责扫描和加载游戏/屏保包。
pub struct PackageService {
  snapshot: PackageSnapshot,
  user_game_key_actions: BTreeMap<String, BTreeMap<String, Vec<Vec<String>>>>,
  last_scan: Option<ScanRequest>,
  watcher_thread: Option<ManagedThreadId>,
  watcher_tx: Option<Sender<PackageWatcherCommand>>,
}

enum PackageWatcherCommand {
  SetFiles(Vec<PathBuf>),
}

impl PackageService {
  pub fn new() -> Self {
    Self {
      snapshot: PackageSnapshot::default(),
      user_game_key_actions: BTreeMap::new(),
      last_scan: None,
      watcher_thread: None,
      watcher_tx: None,
    }
  }

  pub fn configure_scan(&mut self, root_dir: &Path, language_code: &str, missing_template: &str) {
    self.last_scan = Some(ScanRequest {
      root: root_dir.to_path_buf(),
      language_code: language_code.to_string(),
      missing_template: missing_template.to_string(),
    });
  }

  /// 扫描所有目录下的包（官方和模组的游戏与屏保），启动期同步等待完成。
  pub fn scan_all(
    &mut self,
    root_dir: &Path,
    log: &mut LogService,
    language_code: &str,
    missing_template: &str,
  ) {
    self.configure_scan(root_dir, language_code, missing_template);
    let request = self.last_scan.clone().unwrap();
    let mut scan_events = Vec::new();
    let total_candidates = count_package_candidates(&request);
    scan_events.push(PackageEvent::ScanStarted {
      total: total_candidates,
    });
    let report = scan_all_packages(
      &request,
      &mut |event| scan_events.push(event),
      total_candidates,
    );
    let finished = scan_finished_event(&report);
    self.snapshot = report.snapshot;
    for event in scan_events
      .into_iter()
      .chain(report.events)
      .chain([finished])
    {
      log_package_event(log, event);
    }
  }

  /// 请求后台重新扫描。热加载后续只需调用这个入口。
  pub fn request_rescan<
    E: From<PackageAsyncEvent> + From<tg_service_async::TaskStatusEvent> + Send + 'static,
  >(
    &self,
    async_runtime: &AsyncRuntime<E>,
  ) -> bool {
    let Some(request) = self.last_scan.clone() else {
      return false;
    };
    async_runtime.submit(PackageTask::Scan(request));
    true
  }

  /// 启动 package.json 热更新监听。监听线程只产生事件；快照仍由主线程替换。
  pub fn start_watcher<E>(&mut self, async_runtime: &mut AsyncRuntime<E>) -> bool
  where
    E: From<PackageAsyncEvent> + From<tg_service_async::TaskStatusEvent> + Send + 'static,
  {
    if self.watcher_thread.is_some() {
      return false;
    }
    let Some(request) = self.last_scan.clone() else {
      return false;
    };

    let (watcher_tx, watcher_rx) = unbounded();
    let id = async_runtime.spawn_managed_listener(true, move |event_tx, stop| {
      run_package_watcher(request, watcher_rx, event_tx, stop)
    });
    self.watcher_thread = Some(id);
    self.watcher_tx = Some(watcher_tx.clone());
    let _ = watcher_tx.send(PackageWatcherCommand::SetFiles(snapshot_watched_files(
      &self.snapshot,
    )));
    true
  }

  /// 请求使用指定语言重新扫描。语言切换后调用。
  pub fn request_rescan_for_language<
    E: From<PackageAsyncEvent> + From<tg_service_async::TaskStatusEvent> + Send + 'static,
  >(
    &mut self,
    async_runtime: &AsyncRuntime<E>,
    language_code: &str,
    missing_template: &str,
  ) -> bool {
    let Some(mut request) = self.last_scan.clone() else {
      return false;
    };
    request.language_code = language_code.to_string();
    request.missing_template = missing_template.to_string();
    self.last_scan = Some(request.clone());
    async_runtime.submit(PackageTask::Scan(request));
    true
  }

  pub fn handle_async_event(
    &mut self,
    event: PackageAsyncEvent,
    log: &mut LogService,
  ) -> PackageEvent {
    match event {
      PackageAsyncEvent::Event(event) => {
        log_package_event(log, event.clone());
        event
      }
      PackageAsyncEvent::WatchChanged { package_dirs } => {
        let event = PackageEvent::WatchChanged {
          folders: package_dirs.len(),
        };
        log_package_event(log, event.clone());
        event
      }
      PackageAsyncEvent::SnapshotReady {
        snapshot,
        finished,
        watched_files,
      } => {
        self.snapshot = snapshot;
        if let Some(tx) = &self.watcher_tx {
          let _ = tx.send(PackageWatcherCommand::SetFiles(watched_files));
        }
        log_package_event(log, finished.clone());
        finished
      }
    }
  }

  pub fn games(&self) -> Vec<PackageInfo> {
    self.snapshot.games.clone()
  }

  pub fn screensavers(&self) -> Vec<PackageInfo> {
    self.snapshot.screensavers.clone()
  }

  /// 按来源和包 ID 精确查找游戏，避免官方包与模组同名时选错。
  pub fn find_game(&self, source: &PackageSource, mod_id: &str) -> Option<PackageInfo> {
    find_package(&self.snapshot.games, source, mod_id)
  }

  /// 按来源和包 ID 精确查找屏保。
  pub fn find_screensaver(&self, source: &PackageSource, mod_id: &str) -> Option<PackageInfo> {
    find_package(&self.snapshot.screensavers, source, mod_id)
  }

  pub fn find_by_id(&self, package_id: &PackageId) -> Option<PackageInfo> {
    let packages = match package_id.package_type {
      PackageType::Game => &self.snapshot.games,
      PackageType::Screensaver => &self.snapshot.screensavers,
    };
    packages
      .iter()
      .find(|package| package.id == *package_id)
      .cloned()
  }

  /// 启动前重新验证扫描快照，并重新解析入口以抵御热更新或损坏数据。
  pub fn validate_for_launch(&self, package: &PackageInfo) -> Result<PathBuf, String> {
    validate_loaded_package(package)?;
    resolve_package_entry_path(package)
  }

  pub fn validate_game_action_map(
    &self,
    actions: &HashMap<String, Vec<Vec<String>>>,
  ) -> Result<(), String> {
    for (action, keys) in actions {
      let normalized = normalize_action_keys(action, keys.clone())?;
      if normalized != *keys {
        return Err(format!(
          "game action '{action}' contains non-canonical keys"
        ));
      }
    }
    Ok(())
  }

  /// 解析包的实际 Lua 入口，并确保规范路径仍位于 scripts/ 内。
  pub fn resolve_entry_path(&self, package: &PackageInfo) -> Result<PathBuf, String> {
    resolve_package_entry_path(package)
  }

  /// Resolve a package audio asset without permitting access outside its assets directory.
  pub fn resolve_audio_asset(
    &self,
    package: &PackageInfo,
    relative: &Path,
  ) -> Result<
    tg_core_audio::ResolvedAudioFile,
    tg_core_audio::AudioError,
  > {
    resolve_package_file(&package.path, Path::new("assets"), relative)
      .map(tg_core_audio::ResolvedAudioFile::new)
      .ok_or_else(|| {
        tg_core_audio::AudioError::sanitized(
          tg_core_audio::AudioErrorCode::InvalidPath,
        )
      })
  }

  pub fn mod_games(&self) -> Vec<PackageListEntry> {
    self
      .games()
      .into_iter()
      .filter(|info| info.source == PackageSource::Mod)
      .map(|info| self.package_list_entry(info))
      .collect()
  }

  pub fn game_list(&self) -> Vec<PackageListEntry> {
    self
      .games()
      .into_iter()
      .map(|info| self.package_list_entry(info))
      .collect()
  }

  pub fn mod_screensavers(&self) -> Vec<PackageListEntry> {
    self
      .screensavers()
      .into_iter()
      .filter(|info| info.source == PackageSource::Mod)
      .map(package_list_entry)
      .collect()
  }

  /// 返回全部官方与模组屏保的轻量 UI 快照。
  pub fn screensaver_list(&self) -> Vec<PackageListEntry> {
    self
      .screensavers()
      .into_iter()
      .map(package_list_entry)
      .collect()
  }

  pub fn total_count(&self) -> usize {
    self.snapshot.games.len() + self.snapshot.screensavers.len()
  }

  pub fn set_user_game_key_actions(
    &mut self,
    actions: BTreeMap<String, BTreeMap<String, Vec<Vec<String>>>>,
  ) {
    self.user_game_key_actions = actions;
  }

  fn package_list_entry(&self, info: PackageInfo) -> PackageListEntry {
    let mut entry = package_list_entry(info);
    if let Some(actions) = self.user_game_key_actions.get(&entry.id.storage_key()) {
      entry.key_actions = actions
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    }
    entry
  }
}

fn find_package(
  packages: &[PackageInfo],
  source: &PackageSource,
  mod_id: &str,
) -> Option<PackageInfo> {
  packages
    .iter()
    .find(|package| &package.source == source && package.mod_id == mod_id)
    .cloned()
}

pub(crate) fn run_package_task<E: From<PackageAsyncEvent>>(
  task_id: TaskId,
  task: PackageTask,
  event_tx: &Sender<E>,
) -> Result<(), String> {
  match task {
    PackageTask::Scan(request) => {
      let total_candidates = count_package_candidates(&request);
      send_package_event(
        event_tx,
        PackageEvent::ScanStarted {
          total: total_candidates,
        },
      );
      let mut report = scan_all_packages(
        &request,
        &mut |event| send_package_event(event_tx, event),
        total_candidates,
      );
      for event in report.events.drain(..) {
        send_package_event(event_tx, event);
      }
      let finished = scan_finished_event(&report);
      let _ = event_tx.send(E::from(PackageAsyncEvent::SnapshotReady {
        snapshot: report.snapshot,
        finished,
        watched_files: report.watched_files,
      }));
      let _ = task_id;
      Ok(())
    }
  }
}

fn send_package_event<E: From<PackageAsyncEvent>>(event_tx: &Sender<E>, event: PackageEvent) {
  let _ = event_tx.send(E::from(PackageAsyncEvent::Event(event)));
}

fn scan_finished_event(report: &ScanReport) -> PackageEvent {
  PackageEvent::ScanFinished {
    total: report.snapshot.games.len() + report.snapshot.screensavers.len(),
    games: report.snapshot.games.len(),
    screensavers: report.snapshot.screensavers.len(),
    errors: report.errors,
    duplicates: report.duplicates,
  }
}

fn log_package_event(log: &mut LogService, event: PackageEvent) {
  match event {
    PackageEvent::Info(message) => {
      log.info_message(LogSource::Pack, message.clone());
      log.info_package_scan_message(message);
    }
    PackageEvent::Warn(message) => {
      log.warn_message(LogSource::Pack, message.clone());
      log.warn_package_scan_message(message);
    }
    PackageEvent::PackageWarn {
      package_id: _,
      message,
    } => {
      log.warn_package_scan_message(message);
    }
    PackageEvent::Diagnostic {
      package_id,
      diagnostic,
    } => {
      let message = HostLogMessage::new(
        "log_info.package.diagnostic",
        "Package manifest {path} failed [{code}] at {field} ({line}:{column}): {reason}",
      )
      .param("code", &diagnostic.code)
      .param("path", &diagnostic.relative_package_path)
      .param("field", &diagnostic.field_path)
      .param(
        "line",
        diagnostic.line.map_or("-".to_string(), |v| v.to_string()),
      )
      .param(
        "column",
        diagnostic.column.map_or("-".to_string(), |v| v.to_string()),
      )
      .param("reason", &diagnostic.reason);
      let message = if let Some(package_id) = package_id {
        message.param("package", package_id.storage_key())
      } else {
        message.param("package", "-")
      };
      log.warn_package_scan_message(message);
    }
    PackageEvent::Loaded {
      package_id,
      relative_package_path,
    } => log.info_package_scan_message(
      HostLogMessage::new(
        "log_info.package.loaded",
        "Package {package} loaded successfully from {path}.",
      )
      .param("package", package_id.storage_key())
      .param("path", relative_package_path),
    ),
    PackageEvent::ScanStarted { total } => {
      let message = HostLogMessage::new(
        "log_info.package.scan_started",
        "Package scan started ({total} candidates).",
      )
      .param("total", total.to_string());
      log.info_message(LogSource::Pack, message.clone());
      log.info_package_scan_message(message);
    }
    PackageEvent::ScanProgress { .. } => {}
    PackageEvent::WatchChanged { folders } => log.info_message(
      LogSource::Pack,
      HostLogMessage::new(
        "log_info.hot_reload.changed",
        "Package files changed; a rescan was scheduled.",
      )
      .param("folders", folders.to_string()),
    ),
    PackageEvent::ScanFinished {
      total,
      games,
      screensavers,
      errors,
      duplicates,
    } => {
      let message = HostLogMessage::new(
        "log_info.package.scan_finished",
        "Package scan finished: {success} loaded, {failed} failed, {duplicate} duplicates.",
      )
      .param("success", (games + screensavers).to_string())
      .param("failed", errors.to_string())
      .param("duplicate", duplicates.to_string())
      .param("total", total.to_string());
      log.info_message(LogSource::Pack, message.clone());
      log.info_package_scan_message(message);
    }
  }
}

fn infer_field_path(reason: &str) -> String {
  let candidate = reason
    .split_whitespace()
    .next()
    .unwrap_or("$")
    .trim_matches(|character: char| {
      !character.is_ascii_alphanumeric() && !matches!(character, '.' | '_' | '[' | ']' | '-')
    });
  if candidate.contains('.') || candidate.contains('[') || candidate.contains('_') {
    candidate.to_string()
  } else {
    "$".to_string()
  }
}

fn run_package_watcher<E: From<PackageAsyncEvent> + Send + 'static>(
  request: ScanRequest,
  command_rx: Receiver<PackageWatcherCommand>,
  event_tx: Sender<E>,
  stop: Arc<AtomicBool>,
) -> JoinHandle<()> {
  std::thread::spawn(move || {
    let roots = package_scan_roots(&request);
    let (raw_event_tx, raw_event_rx) = unbounded::<Event>();

    let mut watcher = match RecommendedWatcher::new(
      move |result: notify::Result<Event>| {
        if let Ok(event) = result {
          let _ = raw_event_tx.send(event);
        }
      },
      Config::default(),
    ) {
      Ok(watcher) => watcher,
      Err(error) => {
        send_package_event(
          &event_tx,
          PackageEvent::Warn(
            HostLogMessage::new(
              "log_info.package.watch_failed",
              "Package watcher could not start: {error}",
            )
            .param("error", error.to_string()),
          ),
        );
        return;
      }
    };

    let mut watched_dirs = HashSet::<PathBuf>::new();
    let mut watched_files = HashMap::<PathBuf, PathBuf>::new();

    for root in &roots {
      if root.exists() {
        watch_package_dir(&mut watcher, &mut watched_dirs, root, &event_tx);
      } else {
        send_package_event(
          &event_tx,
          PackageEvent::Warn(
            HostLogMessage::new(
              "log_info.package.root_missing",
              "Package watch root does not exist: {path}",
            )
            .param("path", root.display().to_string()),
          ),
        );
      }
    }
    send_package_event(
      &event_tx,
      PackageEvent::Info(
        HostLogMessage::new(
          "log_info.package.watch_started",
          "Package watcher started on {count} roots.",
        )
        .param("count", roots.len().to_string()),
      ),
    );

    let debounce = Duration::from_millis(500);
    let mut pending = HashSet::<PathBuf>::new();
    let mut last_event_at: Option<Instant> = None;

    while !stop.load(Ordering::SeqCst) {
      for command in command_rx.try_iter() {
        match command {
          PackageWatcherCommand::SetFiles(files) => {
            watched_files = watched_file_package_dirs(&roots, files);
            sync_package_watch_dirs(
              &mut watcher,
              &mut watched_dirs,
              &roots,
              watched_files.keys(),
              &event_tx,
            );
          }
        }
      }

      match raw_event_rx.recv_timeout(Duration::from_millis(100)) {
        Ok(event) => {
          queue_package_watch_event(
            &mut watcher,
            &mut watched_dirs,
            &roots,
            &watched_files,
            event,
            &event_tx,
            &mut pending,
          );
          last_event_at = Some(Instant::now());
        }
        Err(RecvTimeoutError::Timeout) => {}
        Err(RecvTimeoutError::Disconnected) => break,
      }

      for event in raw_event_rx.try_iter() {
        queue_package_watch_event(
          &mut watcher,
          &mut watched_dirs,
          &roots,
          &watched_files,
          event,
          &event_tx,
          &mut pending,
        );
        last_event_at = Some(Instant::now());
      }

      if !pending.is_empty() && last_event_at.is_some_and(|time| time.elapsed() >= debounce) {
        let mut package_dirs = pending.drain().collect::<Vec<_>>();
        package_dirs.sort();
        let _ = event_tx.send(E::from(PackageAsyncEvent::WatchChanged { package_dirs }));
        last_event_at = None;
      }
    }
  })
}

fn snapshot_watched_files(snapshot: &PackageSnapshot) -> Vec<PathBuf> {
  snapshot
    .games
    .iter()
    .chain(snapshot.screensavers.iter())
    .flat_map(|info| info.watched_files.clone())
    .collect()
}

fn watched_file_package_dirs(roots: &[PathBuf], files: Vec<PathBuf>) -> HashMap<PathBuf, PathBuf> {
  files
    .into_iter()
    .filter_map(|file| watched_file_package_dir(roots, &file).map(|dir| (file, dir)))
    .collect()
}

fn sync_package_watch_dirs<'a, E: From<PackageAsyncEvent>>(
  watcher: &mut RecommendedWatcher,
  watched_dirs: &mut HashSet<PathBuf>,
  roots: &[PathBuf],
  files: impl Iterator<Item = &'a PathBuf>,
  event_tx: &Sender<E>,
) {
  let mut next_dirs = roots.iter().cloned().collect::<HashSet<_>>();
  next_dirs.extend(roots.iter().flat_map(first_level_package_dirs));
  next_dirs.extend(files.filter_map(|file| file.parent().map(Path::to_path_buf)));

  for dir in watched_dirs
    .difference(&next_dirs)
    .cloned()
    .collect::<Vec<_>>()
  {
    let _ = watcher.unwatch(&dir);
    watched_dirs.remove(&dir);
  }

  for dir in next_dirs {
    if dir.exists() {
      watch_package_dir(watcher, watched_dirs, &dir, event_tx);
    }
  }
}

fn first_level_package_dirs(root: &PathBuf) -> Vec<PathBuf> {
  std::fs::read_dir(root)
    .map(|entries| {
      entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect()
    })
    .unwrap_or_default()
}

fn watch_package_dir<E: From<PackageAsyncEvent>>(
  watcher: &mut RecommendedWatcher,
  watched_dirs: &mut HashSet<PathBuf>,
  dir: &Path,
  event_tx: &Sender<E>,
) {
  let dir = dir.to_path_buf();
  if !watched_dirs.insert(dir.clone()) {
    return;
  }

  if let Err(error) = watcher.watch(&dir, RecursiveMode::NonRecursive) {
    watched_dirs.remove(&dir);
    send_package_event(
      event_tx,
      PackageEvent::Warn(
        HostLogMessage::new(
          "log_info.package.watch_path_failed",
          "Package path {path} could not be watched: {error}",
        )
        .param("path", dir.display().to_string())
        .param("error", error.to_string()),
      ),
    );
  }
}

fn queue_package_watch_event<E: From<PackageAsyncEvent>>(
  watcher: &mut RecommendedWatcher,
  watched_dirs: &mut HashSet<PathBuf>,
  roots: &[PathBuf],
  watched_files: &HashMap<PathBuf, PathBuf>,
  event: Event,
  event_tx: &Sender<E>,
  pending: &mut HashSet<PathBuf>,
) {
  if !matches!(
    event.kind,
    EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
  ) {
    return;
  }

  for path in event.paths {
    if let Some(package_dir) = watched_package_dir(roots, &path) {
      if package_dir.exists() {
        watch_package_dir(watcher, watched_dirs, &package_dir, event_tx);
      }
      pending.insert(package_dir);
      continue;
    }

    if let Some(package_dir) = watched_files.get(&path) {
      pending.insert(package_dir.clone());
    }
  }
}

fn watched_package_dir(roots: &[PathBuf], path: &Path) -> Option<PathBuf> {
  if path.file_name().and_then(|name| name.to_str()) == Some("package.json") {
    let dir = path.parent()?;
    return roots
      .iter()
      .any(|root| dir.parent().is_some_and(|parent| parent == root))
      .then(|| dir.to_path_buf());
  }

  roots
    .iter()
    .any(|root| path.parent().is_some_and(|parent| parent == root))
    .then(|| path.to_path_buf())
}

fn watched_file_package_dir(roots: &[PathBuf], path: &Path) -> Option<PathBuf> {
  roots.iter().find_map(|root| {
    let relative = path.strip_prefix(root).ok()?;
    let mut components = relative.components();
    let package_name = components.next()?;
    components.next()?;
    match package_name {
      Component::Normal(name) => Some(root.join(name)),
      _ => None,
    }
  })
}

fn package_scan_roots(request: &ScanRequest) -> Vec<PathBuf> {
  [
    "scripts/game",
    "scripts/screensaver",
    "data/mod/game",
    "data/mod/screensaver",
  ]
  .into_iter()
  .map(|relative| request.root.join(relative))
  .collect()
}

fn package_list_entry(info: PackageInfo) -> PackageListEntry {
  let icon_path = match &info.display.icon {
    PackageAsset::Image { path } => resolve_package_image_path(&info.path, Path::new(path))
      .map(|path| path.to_string_lossy().to_string()),
    _ => None,
  };
  let icon = icon_path
    .as_ref()
    .map(|path| PackageAsset::Image { path: path.clone() })
    .unwrap_or_else(|| info.display.icon.clone());
  let banner = match &info.display.banner {
    PackageAsset::Image { path } => resolve_package_image_path(&info.path, Path::new(path))
      .map(|path| PackageAsset::Image {
        path: path.to_string_lossy().to_string(),
      })
      .unwrap_or_else(default_banner_asset),
    PackageAsset::Text { .. } => info.display.banner.clone(),
  };
  let mouse_required = info.game.as_ref().is_some_and(|game| game.mouse);
  let truecolor_required = info.game.as_ref().is_some_and(|game| game.truecolor)
    || info
      .screensaver
      .as_ref()
      .is_some_and(|screensaver| screensaver.truecolor);
  let high_privilege_required = info.game.as_ref().is_some_and(|game| game.high_privilege);
  let supported_languages = info
    .game
    .as_ref()
    .map(|game| game.supported_languages.clone())
    .unwrap_or_default();
  let key_actions: HashMap<String, Vec<Vec<String>>> = info
    .game
    .as_ref()
    .map(|game| {
      game
        .actions
        .iter()
        .map(|(name, action)| (name.clone(), action.keys.clone()))
        .collect()
    })
    .unwrap_or_default();
  let game_name = info
    .game
    .as_ref()
    .map(|game| game.name.clone())
    .unwrap_or_default();
  let screensaver_name = info
    .screensaver
    .as_ref()
    .map(|screensaver| screensaver.name.clone())
    .unwrap_or_default();
  let game_detail = info
    .game
    .as_ref()
    .map(|game| game.detail.clone())
    .unwrap_or_else(|| info.display.description.clone());
  let score_enabled = info
    .game
    .as_ref()
    .and_then(|game| game.score.as_ref())
    .is_some_and(|score| score.enabled);
  let score_empty_text = info
    .game
    .as_ref()
    .and_then(|game| game.score.as_ref())
    .map(|score| score.empty_text.clone())
    .unwrap_or_default();
  let screensaver_command = info
    .screensaver
    .as_ref()
    .map(|screensaver| screensaver.command.clone())
    .unwrap_or_default();

  PackageListEntry {
    id: info.id,
    mod_id: info.mod_id,
    source: info.source,
    package_type: info.package_type,
    key_default_actions: key_actions.clone(),
    key_actions,
    title: info.display.title,
    game_name,
    screensaver_name,
    game_detail,
    description: info.display.description,
    author: info.display.author,
    version: info.version,
    icon,
    icon_path,
    banner,
    path: info.path,
    enabled: true,
    debug: false,
    safe_mode: true,
    mouse_required,
    truecolor_required,
    high_privilege_required,
    supported_languages,
    score_enabled,
    score_empty_text,
    best_string: None,
    min_width: info.runtime.min_width,
    min_height: info.runtime.min_height,
    screensaver_command,
  }
}

fn scan_all_packages(
  request: &ScanRequest,
  emit_event: &mut impl FnMut(PackageEvent),
  total: usize,
) -> ScanReport {
  let mut report = ScanReport {
    snapshot: PackageSnapshot::default(),
    events: Vec::new(),
    errors: 0,
    duplicates: 0,
    watched_files: Vec::new(),
  };
  let mut scanned = 0;

  scan_dir(
    &mut report,
    request,
    emit_event,
    total,
    &mut scanned,
    "scripts/game",
    PackageType::Game,
    PackageSource::Official,
  );
  scan_dir(
    &mut report,
    request,
    emit_event,
    total,
    &mut scanned,
    "scripts/screensaver",
    PackageType::Screensaver,
    PackageSource::Official,
  );
  scan_dir(
    &mut report,
    request,
    emit_event,
    total,
    &mut scanned,
    "data/mod/game",
    PackageType::Game,
    PackageSource::Mod,
  );
  scan_dir(
    &mut report,
    request,
    emit_event,
    total,
    &mut scanned,
    "data/mod/screensaver",
    PackageType::Screensaver,
    PackageSource::Mod,
  );

  report
}

// 递归扫描指定目录下的所有包并加载
fn scan_dir(
  report: &mut ScanReport,
  request: &ScanRequest,
  emit_event: &mut impl FnMut(PackageEvent),
  total: usize,
  scanned: &mut usize,
  relative: &str,
  expected_type: PackageType,
  source: PackageSource,
) {
  let dir = request.root.join(relative);
  let Some((canonical_root, canonical_dir)) = canonical_scan_root(&request.root, &dir) else {
    return;
  };
  if !canonical_dir.starts_with(&canonical_root) {
    return;
  }
  let entries = match std::fs::read_dir(&dir) {
    Ok(e) => e,
    Err(_) => return,
  };

  for entry in entries.flatten() {
    let path = entry.path();
    if !path.is_dir() {
      continue;
    }
    let dir_name = path.file_name().unwrap().to_string_lossy().to_string();
    let package_is_direct_child = path
      .canonicalize()
      .ok()
      .and_then(|canonical| canonical.parent().map(|parent| parent == canonical_dir))
      .unwrap_or(false);
    if !package_is_direct_child {
      report.events.push(PackageEvent::Warn(
        HostLogMessage::new(
          "log_info.package.path_rejected",
          "Package directory {path} escapes its scan root and was rejected.",
        )
        .param("path", format!("{relative}/{dir_name}")),
      ));
      report.errors += 1;
      *scanned += 1;
      emit_event(PackageEvent::ScanProgress {
        scanned: *scanned,
        total,
      });
      continue;
    }

    match read_package(&path, &dir_name, &expected_type, &source, request) {
      Ok(info) => {
        if has_package_id(&report.snapshot, &info.id) {
          report.events.push(PackageEvent::PackageWarn {
            package_id: info.id.clone(),
            message: HostLogMessage::new(
              "log_info.package.duplicate",
              "Duplicate package id {package} was found at {path}; the first package was kept.",
            )
            .param("package", info.id.to_string())
            .param("path", dir_name),
          });
          report.duplicates += 1;
          *scanned += 1;
          emit_event(PackageEvent::ScanProgress {
            scanned: *scanned,
            total,
          });
          continue;
        }
        report.watched_files.extend(info.watched_files.clone());
        report.events.push(PackageEvent::Loaded {
          package_id: info.id.clone(),
          relative_package_path: format!("{relative}/{dir_name}/package.json"),
        });
        insert(&mut report.snapshot, info);
      }
      Err(error) => {
        let package_id = read_manifest_package_id(&path, source);
        let relative_package_path = format!("{relative}/{dir_name}/package.json");
        report.events.extend(
          error
            .into_diagnostics(relative_package_path)
            .into_iter()
            .map(|diagnostic| PackageEvent::Diagnostic {
              package_id: package_id.clone(),
              diagnostic,
            }),
        );
        report.errors += 1;
      }
    }
    *scanned += 1;
    emit_event(PackageEvent::ScanProgress {
      scanned: *scanned,
      total,
    });
  }
}

fn canonical_scan_root(root: &Path, category: &Path) -> Option<(PathBuf, PathBuf)> {
  let canonical_root = root.canonicalize().ok()?;
  let canonical_category = category.canonicalize().ok()?;
  canonical_category
    .starts_with(&canonical_root)
    .then_some((canonical_root, canonical_category))
}

fn count_package_candidates(request: &ScanRequest) -> usize {
  [
    "scripts/game",
    "scripts/screensaver",
    "data/mod/game",
    "data/mod/screensaver",
  ]
  .into_iter()
  .map(|relative| count_child_dirs(&request.root.join(relative)))
  .sum()
}

fn count_child_dirs(dir: &Path) -> usize {
  std::fs::read_dir(dir)
    .map(|entries| {
      entries
        .flatten()
        .filter(|entry| entry.path().is_dir())
        .count()
    })
    .unwrap_or(0)
}

// 读取并验证单个包的 package.json 文件
fn read_package(
  dir: &Path,
  dir_name: &str,
  expected_type: &PackageType,
  source: &PackageSource,
  request: &ScanRequest,
) -> Result<PackageInfo, PackageReadError> {
  let json_path = dir.join("package.json");
  let content =
    read_utf8_file_limited(&json_path, MAX_PACKAGE_MANIFEST_BYTES).map_err(|error| {
      PackageReadError {
        code: "read_failed",
        field_path: "$".to_string(),
        line: None,
        column: None,
        reason: format!("cannot read package.json: {error}"),
        related: Vec::new(),
      }
    })?;
  let mut watched_files = vec![json_path.clone()];

  let mut deserializer = serde_json::Deserializer::from_str(&content);
  let raw: RawPackageJson =
    serde_path_to_error::deserialize(&mut deserializer).map_err(|error| {
      let field_path = error.path().to_string();
      let inner = error.into_inner();
      PackageReadError {
        code: match inner.classify() {
          serde_json::error::Category::Syntax | serde_json::error::Category::Eof => "json_syntax",
          _ => "schema",
        },
        field_path: if field_path.is_empty() {
          "$".to_string()
        } else {
          field_path
        },
        line: Some(inner.line()),
        column: Some(inner.column()),
        reason: inner.to_string(),
        related: Vec::new(),
      }
    })?;

  let mut manifest_errors = Vec::new();
  PackageReadError::push_if(
    &mut manifest_errors,
    raw.schema_version != PACKAGE_MANIFEST_VERSION,
    "schema_version",
    format!(
      "schema_version {} != host {}",
      raw.schema_version, PACKAGE_MANIFEST_VERSION
    ),
  );

  let pkg_type = match parse_package_type(&raw.package_type) {
    Ok(package_type) => {
      PackageReadError::push_if(
        &mut manifest_errors,
        package_type != *expected_type,
        "type",
        format!(
          "type mismatch: manifest says {:?}, directory expects {:?}",
          package_type, expected_type
        ),
      );
      Some(package_type)
    }
    Err(reason) => {
      manifest_errors.push(PackageReadError::at("type", reason));
      None
    }
  };
  let package_id = pkg_type.and_then(|package_type| {
    match PackageId::new(*source, package_type, raw.mod_id.clone()) {
      Ok(package_id) => Some(package_id),
      Err(reason) => {
        manifest_errors.push(PackageReadError::at("mod_id", reason));
        None
      }
    }
  });

  PackageReadError::push_if(
    &mut manifest_errors,
    raw.version_code == 0,
    "version_code",
    "version_code must be > 0".to_string(),
  );
  PackageReadError::push_if(
    &mut manifest_errors,
    raw.api.min > raw.api.max,
    "api.min",
    format!("api.min ({}) > api.max ({})", raw.api.min, raw.api.max),
  );
  PackageReadError::push_if(
    &mut manifest_errors,
    raw.api.min > HOST_API_VERSION,
    "api.min",
    format!(
      "api.min ({}) > host API ({})",
      raw.api.min, HOST_API_VERSION
    ),
  );
  PackageReadError::push_if(
    &mut manifest_errors,
    raw.api.max < HOST_API_VERSION,
    "api.max",
    format!(
      "api.max ({}) < host API ({})",
      raw.api.max, HOST_API_VERSION
    ),
  );
  if let Some(game) = raw.game.as_ref() {
    let target_fps = game.target_fps.unwrap_or(60);
    PackageReadError::push_if(
      &mut manifest_errors,
      !VALID_TARGET_FPS.contains(&target_fps),
      "game.target_fps",
      format!(
        "target_fps must be one of {:?}, got {}",
        VALID_TARGET_FPS, target_fps
      ),
    );
  }
  if let Some(package_type) = pkg_type {
    PackageReadError::push_if(
      &mut manifest_errors,
      package_type == PackageType::Game && raw.game.is_none(),
      "game",
      "missing 'game' config for game type".to_string(),
    );
    PackageReadError::push_if(
      &mut manifest_errors,
      package_type == PackageType::Game && raw.screensaver.is_some(),
      "screensaver",
      "game package must not contain screensaver configuration".to_string(),
    );
    PackageReadError::push_if(
      &mut manifest_errors,
      package_type == PackageType::Screensaver && raw.screensaver.is_none(),
      "screensaver",
      "missing 'screensaver' config".to_string(),
    );
    PackageReadError::push_if(
      &mut manifest_errors,
      package_type == PackageType::Screensaver && raw.game.is_some(),
      "game",
      "screensaver package must not contain game configuration".to_string(),
    );
  }
  if !manifest_errors.is_empty() {
    return Err(PackageReadError::combine(manifest_errors));
  }

  let Some(pkg_type) = pkg_type else {
    return Err(PackageReadError::at(
      "type",
      "package type validation did not produce a value",
    ));
  };
  let Some(package_id) = package_id else {
    return Err(PackageReadError::at(
      "mod_id",
      "package identity validation did not produce a value",
    ));
  };

  let entry =
    resolve_entry(dir, &raw.entry).map_err(|reason| PackageReadError::at("entry", reason))?;

  let display = raw.display;
  let title = resolve_package_text(
    dir,
    &display.title,
    request,
    &mut watched_files,
    "display.title",
  )?;
  if title.trim().is_empty() {
    return Err(PackageReadError::at(
      "display.title",
      "display.title is empty",
    ));
  }
  let version = resolve_package_text(dir, &raw.version, request, &mut watched_files, "version")?;
  if version.trim().is_empty() {
    return Err(PackageReadError::at("version", "version is empty"));
  }

  let runtime = raw.runtime;

  let game = match pkg_type {
    PackageType::Game => {
      let g = raw
        .game
        .ok_or_else(|| PackageReadError::at("game", "missing 'game' config for game type"))?;
      let target_fps = g.target_fps.unwrap_or(60);
      if !VALID_TARGET_FPS.contains(&target_fps) {
        return Err(PackageReadError::at(
          "game.target_fps",
          format!(
            "target_fps must be one of {:?}, got {}",
            VALID_TARGET_FPS, target_fps
          ),
        ));
      }
      let name = resolve_package_text(dir, &g.name, request, &mut watched_files, "game.name")?;
      if name.trim().is_empty() {
        return Err(PackageReadError::at("game.name", "game.name is empty"));
      }
      let detail =
        resolve_package_text(dir, &g.detail, request, &mut watched_files, "game.detail")?;
      let supported_languages = normalize_language_codes(&g.language)
        .map_err(|reason| PackageReadError::at("game.language", reason))?;
      let mut actions = HashMap::new();
      let mut action_order = Vec::new();
      let raw_actions = g
        .actions
        .as_object()
        .ok_or_else(|| PackageReadError::at("game.actions", "game.actions must be an object"))?;
      for (name, value) in raw_actions {
        let a: RawActionConfig = serde_json::from_value(value.clone()).map_err(|error| {
          PackageReadError::at(
            format!("game.actions.{name}"),
            format!("invalid action configuration: {error}"),
          )
        })?;
        action_order.push(name.clone());
        actions.insert(
          name.clone(),
          ActionConfig {
            description: resolve_package_text(
              dir,
              &a.description,
              request,
              &mut watched_files,
              &format!("game.actions.{name}.description"),
            )?,
            keys: normalize_action_keys(name, a.keys).map_err(|reason| {
              PackageReadError::at(format!("game.actions.{name}.keys"), reason)
            })?,
            lock: a.lock,
          },
        );
      }
      Some(GameConfig {
        name,
        detail,
        high_privilege: g.high_privilege.unwrap_or(false),
        mouse: g.mouse.unwrap_or(false),
        truecolor: g.truecolor.unwrap_or(false),
        target_fps,
        save: g.save.unwrap_or(false),
        supported_languages,
        score: g
          .score
          .map(|s| {
            Ok::<_, String>(ScoreConfig {
              enabled: s.enabled.unwrap_or(false),
              empty_text: match s.empty_text {
                Some(value) => resolve_package_text(
                  dir,
                  &value,
                  request,
                  &mut watched_files,
                  "game.score.empty_text",
                )?,
                None => String::new(),
              },
            })
          })
          .transpose()?,
        actions,
        action_order,
      })
    }
    PackageType::Screensaver => None,
  };

  let screensaver = match pkg_type {
    PackageType::Screensaver => {
      let s = raw
        .screensaver
        .ok_or_else(|| PackageReadError::at("screensaver", "missing 'screensaver' config"))?;
      let name = resolve_package_text(
        dir,
        &s.name,
        request,
        &mut watched_files,
        "screensaver.name",
      )?;
      if name.trim().is_empty() {
        return Err(PackageReadError::at(
          "screensaver.name",
          "screensaver.name is empty",
        ));
      }
      if s.command.trim().is_empty() {
        return Err(PackageReadError::at(
          "screensaver.command",
          "screensaver.command is empty",
        ));
      }
      Some(ScreensaverConfig {
        name,
        truecolor: s.truecolor.unwrap_or(false),
        command: s.command,
      })
    }
    PackageType::Game => None,
  };

  let package = PackageInfo {
    id: package_id,
    source: *source,
    dir_name: dir_name.to_string(),
    mod_id: raw.mod_id,
    package_type: pkg_type,
    version,
    version_code: raw.version_code,
    api_min: raw.api.min,
    api_max: raw.api.max,
    entry,
    display: PackageDisplay {
      title,
      description: resolve_package_text(
        dir,
        &display.description,
        request,
        &mut watched_files,
        "display.description",
      )?,
      author: resolve_package_text(
        dir,
        &display.author,
        request,
        &mut watched_files,
        "display.author",
      )?,
      icon: parse_package_asset(dir, &display.icon, AssetShape::Icon, &mut watched_files)?
        .unwrap_or_else(default_icon_asset),
      banner: parse_package_asset(dir, &display.banner, AssetShape::Banner, &mut watched_files)?
        .unwrap_or_else(default_banner_asset),
    },
    runtime: PackageRuntime {
      min_width: runtime.min_width,
      min_height: runtime.min_height,
    },
    game,
    screensaver,
    path: dir.to_path_buf(),
    watched_files,
  };
  validate_loaded_package(&package)?;
  resolve_package_entry_path(&package)?;
  Ok(package)
}

fn normalize_action_keys(action: &str, keys: Vec<Vec<String>>) -> Result<Vec<Vec<String>>, String> {
  if action.trim().is_empty() {
    return Err("game action name is empty".to_string());
  }
  if keys.len() > 2 {
    return Err(format!(
      "game action '{action}' registers {} key bindings; at most 2 are allowed",
      keys.len()
    ));
  }
  keys
    .into_iter()
    .enumerate()
    .map(|(pattern_index, pattern)| {
      if pattern.is_empty() {
        return Err(format!(
          "game action '{action}' key pattern {pattern_index} is empty"
        ));
      }
      if pattern.len() > 2 {
        return Err(format!(
          "game action '{action}' key pattern {pattern_index} contains {} keys; at most 2 are allowed",
          pattern.len()
        ));
      }
      pattern
        .into_iter()
        .map(|token| {
          canonical_key_token(&token)
            .ok_or_else(|| format!("game action '{action}' contains unknown key token '{token}'"))
        })
        .collect()
    })
    .collect()
}

fn read_manifest_package_id(dir: &Path, source: PackageSource) -> Option<PackageId> {
  let content =
    read_utf8_file_limited(&dir.join("package.json"), MAX_PACKAGE_MANIFEST_BYTES).ok()?;
  let value: serde_json::Value = serde_json::from_str(&content).ok()?;
  let mod_id = value.get("mod_id")?.as_str()?;
  let package_type = value
    .get("type")?
    .as_str()
    .and_then(|value| parse_package_type(value).ok())?;
  PackageId::new(source, package_type, mod_id).ok()
}

fn validate_loaded_package(package: &PackageInfo) -> Result<(), String> {
  let expected_id = PackageId::new(package.source, package.package_type, package.mod_id.clone())?;
  if expected_id != package.id {
    return Err("package identity does not match the loaded manifest".to_string());
  }
  if package.api_min > package.api_max
    || package.api_min > HOST_API_VERSION
    || package.api_max < HOST_API_VERSION
  {
    return Err(format!(
      "package API range {}..={} is incompatible with host API {}",
      package.api_min, package.api_max, HOST_API_VERSION
    ));
  }
  match package.package_type {
    PackageType::Game => {
      let game = package
        .game
        .as_ref()
        .ok_or("game package has no game configuration")?;
      if package.screensaver.is_some() {
        return Err("game package contains screensaver configuration".to_string());
      }
      if !VALID_TARGET_FPS.contains(&game.target_fps) {
        return Err(format!("invalid game target_fps {}", game.target_fps));
      }
      if normalize_language_codes(&game.supported_languages)? != game.supported_languages {
        return Err("game.language contains non-canonical language codes".to_string());
      }
      for (action, config) in &game.actions {
        let normalized = normalize_action_keys(action, config.keys.clone())?;
        if normalized != config.keys {
          return Err(format!(
            "game action '{action}' contains non-canonical keys"
          ));
        }
      }
    }
    PackageType::Screensaver => {
      let screensaver = package
        .screensaver
        .as_ref()
        .ok_or("screensaver package has no screensaver configuration")?;
      if screensaver.command.trim().is_empty() {
        return Err("screensaver.command is empty".to_string());
      }
      if package.game.is_some() {
        return Err("screensaver package contains game configuration".to_string());
      }
    }
  }
  Ok(())
}

fn resolve_package_entry_path(package: &PackageInfo) -> Result<PathBuf, String> {
  let canonical_package = package.path.canonicalize().map_err(|error| {
    format!(
      "Failed to resolve package directory '{}': {error}",
      package.path.display()
    )
  })?;
  if !canonical_package.is_dir() {
    return Err(format!(
      "Package path '{}' is not a directory",
      canonical_package.display()
    ));
  }

  let scripts_dir = package.path.join("scripts");
  let canonical_scripts = scripts_dir.canonicalize().map_err(|error| {
    format!(
      "Failed to resolve scripts directory '{}': {error}",
      scripts_dir.display()
    )
  })?;
  if !canonical_scripts.is_dir() || !canonical_scripts.starts_with(&canonical_package) {
    return Err(format!(
      "Scripts directory '{}' escapes package directory '{}'",
      canonical_scripts.display(),
      canonical_package.display()
    ));
  }

  let normalized_entry = resolve_entry(&package.path, &package.entry)?;
  let entry_path = scripts_dir.join(normalized_entry);
  let canonical_entry = entry_path.canonicalize().map_err(|error| {
    format!(
      "Failed to resolve package entry '{}': {error}",
      entry_path.display()
    )
  })?;
  if !canonical_entry.starts_with(&canonical_scripts) {
    return Err(format!(
      "Package entry '{}' escapes scripts directory '{}'",
      canonical_entry.display(),
      canonical_scripts.display()
    ));
  }
  if !extension_is(canonical_entry.to_string_lossy().as_ref(), &["lua"]) {
    return Err(format!(
      "Package entry '{}' is not a Lua source file",
      canonical_entry.display()
    ));
  }
  if !canonical_entry.is_file() {
    return Err(format!(
      "Package entry '{}' is not a file",
      canonical_entry.display()
    ));
  }
  Ok(canonical_entry)
}

fn insert(snapshot: &mut PackageSnapshot, info: PackageInfo) {
  match info.package_type {
    PackageType::Game => snapshot.games.push(info),
    PackageType::Screensaver => snapshot.screensavers.push(info),
  }
}

fn has_package_id(snapshot: &PackageSnapshot, id: &PackageId) -> bool {
  snapshot.games.iter().any(|package| &package.id == id)
    || snapshot
      .screensavers
      .iter()
      .any(|package| &package.id == id)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawPackageJson {
  mod_id: String,
  schema_version: u32,
  #[serde(rename = "type")]
  package_type: String,
  version: RawPackageText,
  version_code: u32,
  api: RawApiRange,
  entry: String,
  display: RawDisplay,
  runtime: RawRuntime,
  game: Option<RawGameConfig>,
  screensaver: Option<RawScreensaverConfig>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawApiRange {
  min: u32,
  max: u32,
}

#[derive(Clone, Deserialize)]
#[serde(untagged)]
enum RawPackageText {
  Literal(String),
  Object(RawPackageTextObject),
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawPackageTextObject {
  #[serde(rename = "type")]
  text_type: String,
  text: Option<String>,
  path: Option<String>,
  key: Option<String>,
  callback: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawDisplay {
  title: RawPackageText,
  description: RawPackageText,
  author: RawPackageText,
  #[serde(default)]
  icon: Option<RawDisplayAsset>,
  #[serde(default)]
  banner: Option<RawDisplayAsset>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawDisplayAsset {
  #[serde(rename = "type")]
  asset_type: String,
  path: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRuntime {
  min_width: u32,
  min_height: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawGameConfig {
  name: RawPackageText,
  detail: RawPackageText,
  high_privilege: Option<bool>,
  mouse: Option<bool>,
  truecolor: Option<bool>,
  target_fps: Option<u32>,
  save: Option<bool>,
  language: Vec<String>,
  score: Option<RawScoreConfig>,
  actions: serde_json::Value,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawScoreConfig {
  enabled: Option<bool>,
  empty_text: Option<RawPackageText>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawActionConfig {
  description: RawPackageText,
  keys: Vec<Vec<String>>,
  #[serde(default)]
  lock: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawScreensaverConfig {
  name: RawPackageText,
  truecolor: Option<bool>,
  command: String,
}

fn parse_package_type(s: &str) -> Result<PackageType, String> {
  match s {
    "game" => Ok(PackageType::Game),
    "screensaver" => Ok(PackageType::Screensaver),
    other => Err(format!("Unknown package type: '{}'", other)),
  }
}

#[derive(Clone, Copy)]
enum AssetShape {
  Icon,
  Banner,
}

impl AssetShape {
  fn size(self) -> (usize, usize) {
    match self {
      AssetShape::Icon => (8, 4),
      AssetShape::Banner => (60, 14),
    }
  }
}

fn parse_package_asset(
  package_dir: &Path,
  raw: &Option<RawDisplayAsset>,
  shape: AssetShape,
  watched_files: &mut Vec<PathBuf>,
) -> Result<Option<PackageAsset>, String> {
  let Some(raw) = raw.as_ref() else {
    return Ok(None);
  };
  let path = safe_asset_path(&raw.path)
    .ok_or_else(|| format!("display asset path '{}' is invalid", raw.path))?;
  let watched_path = package_dir.join("assets").join(&path);
  let asset = match raw.asset_type.as_str() {
    "image" if is_supported_image_path(&path) => {
      resolve_package_image_path(package_dir, Path::new(&path))
        .ok_or_else(|| format!("display image asset '{path}' is missing or unsafe"))?;
      watched_files.push(watched_path);
      PackageAsset::Image { path }
    }
    "text" if is_supported_text_path(&path) => {
      let asset_path = resolve_package_file(package_dir, Path::new("assets"), Path::new(&path))
        .ok_or_else(|| format!("display text asset '{path}' is missing or unsafe"))?;
      watched_files.push(watched_path);
      let content = read_utf8_file_limited(&asset_path, MAX_PACKAGE_TEXT_BYTES)
        .map_err(|error| format!("cannot read display text asset '{path}': {error}"))?;
      PackageAsset::Text {
        path,
        lines: normalize_asset_text(&content, shape),
      }
    }
    "image" => {
      return Err(format!(
        "display image asset '{path}' has an unsupported extension"
      ));
    }
    "text" => {
      return Err(format!(
        "display text asset '{path}' must use the .txt extension"
      ));
    }
    other => {
      return Err(format!(
        "display asset type '{other}' must be 'image' or 'text'"
      ));
    }
  };
  Ok(Some(asset))
}

pub(crate) fn default_icon_lines() -> Vec<String> {
  normalize_asset_lines(
    ["████████", "██ ██ ██", "   ██   ", "  ████  "],
    AssetShape::Icon,
  )
}

fn default_icon_asset() -> PackageAsset {
  PackageAsset::Text {
    path: String::new(),
    lines: default_icon_lines(),
  }
}

fn default_banner_asset() -> PackageAsset {
  PackageAsset::Text {
    path: String::new(),
    lines: normalize_asset_lines(
      [
        "`7MMM.     ,MMF' .g8\"\"8q. `7MM\"\"\"Yb.   ",
        "  MMMb    dPMM .dP'    `YM. MM    `Yb. ",
        "  M YM   ,M MM dM'      `MM MM     `Mb ",
        "  M  Mb  M' MM MM        MM MM      MM ",
        "  M  YM.P'  MM MM.      ,MP MM     ,MP ",
        "  M  `YM'   MM `Mb.    ,dP' MM    ,dP' ",
        ".JML. `'  .JMML. `\"bmmd\"' .JMMmmmdP'   ",
      ],
      AssetShape::Banner,
    ),
  }
}

fn normalize_asset_lines<const N: usize>(lines: [&str; N], shape: AssetShape) -> Vec<String> {
  normalize_asset_text(&lines.join("\n"), shape)
}

fn safe_asset_path(path: &str) -> Option<String> {
  let trimmed = path.trim();
  if trimmed.is_empty() || trimmed.contains('\\') {
    return None;
  }

  let mut parts = Vec::new();
  for component in Path::new(trimmed).components() {
    match component {
      Component::Normal(part) => parts.push(part.to_string_lossy().to_string()),
      _ => return None,
    }
  }

  (!parts.is_empty()).then(|| parts.join("/"))
}

fn safe_path_segment(value: &str) -> bool {
  let value = value.trim();
  !value.is_empty()
    && value != "."
    && value != ".."
    && !value
      .chars()
      .any(|character| matches!(character, '/' | '\\' | ':'))
    && matches!(
      Path::new(value).components().collect::<Vec<_>>().as_slice(),
      [Component::Normal(_)]
    )
}

fn safe_relative_path(path: &Path) -> bool {
  !path.as_os_str().is_empty()
    && path
      .components()
      .all(|component| matches!(component, Component::Normal(_)))
}

fn resolve_package_file(
  package_dir: &Path,
  allowed_root: &Path,
  relative: &Path,
) -> Option<PathBuf> {
  if !safe_relative_path(allowed_root) || !safe_relative_path(relative) {
    return None;
  }
  let canonical_package = package_dir.canonicalize().ok()?;
  let root = package_dir.join(allowed_root);
  let canonical_root = root.canonicalize().ok()?;
  if !canonical_root.is_dir() || !canonical_root.starts_with(&canonical_package) {
    return None;
  }
  let canonical_file = root.join(relative).canonicalize().ok()?;
  (canonical_file.is_file() && canonical_file.starts_with(&canonical_root))
    .then_some(canonical_file)
}

fn resolve_package_image_path(package_dir: &Path, relative: &Path) -> Option<PathBuf> {
  if !safe_relative_path(relative) {
    return None;
  }
  let canonical_package = package_dir.canonicalize().ok()?;
  let root = package_dir.join("assets");
  let canonical_root = match root.canonicalize() {
    Ok(root) => {
      if !root.is_dir() || !root.starts_with(&canonical_package) {
        return None;
      }
      Some(root)
    }
    Err(_) => None,
  };
  let candidate = root.join(relative);
  match candidate.canonicalize() {
    Ok(file) => {
      let root = canonical_root?;
      (file.is_file() && file.starts_with(root)).then_some(file)
    }
    Err(_) => {
      let parent_is_safe = candidate
        .parent()
        .and_then(|parent| parent.canonicalize().ok())
        .is_none_or(|parent| {
          canonical_root
            .as_ref()
            .is_some_and(|root| parent.starts_with(root))
        });
      parent_is_safe.then_some(candidate)
    }
  }
}

fn read_utf8_file_limited(path: &Path, limit: usize) -> Result<String, String> {
  let file = File::open(path).map_err(|error| error.to_string())?;
  let metadata = file.metadata().map_err(|error| error.to_string())?;
  if !metadata.is_file() {
    return Err("path is not a regular file".to_string());
  }
  if metadata.len() > limit as u64 {
    return Err(format!(
      "file is {} bytes; limit is {limit} bytes",
      metadata.len()
    ));
  }

  let mut bytes = Vec::with_capacity(metadata.len() as usize);
  file
    .take(limit.saturating_add(1) as u64)
    .read_to_end(&mut bytes)
    .map_err(|error| error.to_string())?;
  if bytes.len() > limit {
    return Err(format!(
      "file exceeds the {limit} byte limit while being read"
    ));
  }
  String::from_utf8(bytes).map_err(|error| error.to_string())
}

fn is_supported_image_path(path: &str) -> bool {
  extension_is(path, &["png", "jpg", "jpeg"])
}

fn is_supported_text_path(path: &str) -> bool {
  extension_is(path, &["txt"])
}

fn extension_is(path: &str, allowed: &[&str]) -> bool {
  Path::new(path)
    .extension()
    .and_then(|ext| ext.to_str())
    .map(|ext| allowed.iter().any(|item| ext.eq_ignore_ascii_case(item)))
    .unwrap_or(false)
}

fn normalize_asset_text(content: &str, shape: AssetShape) -> Vec<String> {
  let (width, height) = shape.size();
  let mut lines: Vec<String> = content
    .lines()
    .map(|line| {
      let line = line.trim_end_matches('\r');
      if line.trim_start().starts_with("f%") {
        line.to_string()
      } else {
        fit_line_width(line, width)
      }
    })
    .collect();

  if lines.len() > height {
    let start = (lines.len() - height) / 2;
    lines = lines[start..start + height].to_vec();
  }

  while lines.len() < height {
    if (height - lines.len()) % 2 == 1 {
      lines.insert(0, " ".repeat(width));
    } else {
      lines.push(" ".repeat(width));
    }
  }

  lines
}

fn fit_line_width(line: &str, width: usize) -> String {
  let mut result = String::new();
  let mut used = 0;
  for grapheme in UnicodeSegmentation::graphemes(line.trim_end_matches('\r'), true) {
    let grapheme_width = UnicodeWidthStr::width(grapheme);
    if used + grapheme_width > width {
      break;
    }
    used += grapheme_width;
    result.push_str(grapheme);
  }
  let padding = width.saturating_sub(UnicodeWidthStr::width(result.as_str()));
  let left = padding.div_ceil(2);
  format!(
    "{}{}{}",
    " ".repeat(left),
    result,
    " ".repeat(padding - left)
  )
}

fn resolve_package_text(
  pkg_dir: &Path,
  value: &RawPackageText,
  request: &ScanRequest,
  watched_files: &mut Vec<PathBuf>,
  field: &str,
) -> Result<String, String> {
  match value {
    RawPackageText::Literal(text) => Ok(text.clone()),
    RawPackageText::Object(value) if value.text_type == "text" => value
      .text
      .clone()
      .ok_or_else(|| format!("{field}.text is required when type is 'text'")),
    RawPackageText::Object(value) if value.text_type == "i18n" => {
      let path = value
        .path
        .as_deref()
        .ok_or_else(|| format!("{field}.path is required when type is 'i18n'"))?;
      let path = safe_asset_path(path)
        .filter(|path| extension_is(path, &["json"]))
        .ok_or_else(|| format!("{field}.path must be a safe relative .json path"))?;
      let key = value
        .key
        .as_deref()
        .filter(|key| !key.trim().is_empty())
        .ok_or_else(|| format!("{field}.key is required when type is 'i18n'"))?;
      let callback = value
        .callback
        .as_deref()
        .ok_or_else(|| format!("{field}.callback is required when type is 'i18n'"))?;
      Ok(resolve_package_i18n(
        pkg_dir,
        &path,
        key,
        callback,
        request,
        watched_files,
      ))
    }
    RawPackageText::Object(value) => Err(format!(
      "{field}.type must be 'text' or 'i18n', got '{}'",
      value.text_type
    )),
  }
}

fn resolve_package_i18n(
  pkg_dir: &Path,
  path: &str,
  key: &str,
  callback: &str,
  request: &ScanRequest,
  watched_files: &mut Vec<PathBuf>,
) -> String {
  push_package_i18n_watch_path(pkg_dir, &request.language_code, path, watched_files);
  push_package_i18n_watch_path(pkg_dir, "en_us", path, watched_files);
  load_package_i18n_value(pkg_dir, &request.language_code, path, key)
    .or_else(|| load_package_i18n_value(pkg_dir, "en_us", path, key))
    .unwrap_or_else(|| callback.to_string())
}

fn push_package_i18n_watch_path(
  pkg_dir: &Path,
  language_code: &str,
  path: &str,
  watched_files: &mut Vec<PathBuf>,
) {
  if safe_path_segment(language_code) {
    watched_files.push(
      pkg_dir
        .join("assets/language")
        .join(language_code)
        .join(path),
    );
  }
}

fn load_package_i18n_value(
  pkg_dir: &Path,
  language_code: &str,
  relative_path: &str,
  key: &str,
) -> Option<String> {
  if !safe_path_segment(language_code) {
    return None;
  }
  let relative = Path::new("language")
    .join(language_code)
    .join(relative_path);
  let path = resolve_package_file(pkg_dir, Path::new("assets"), &relative)?;
  let content = read_utf8_file_limited(&path, MAX_PACKAGE_I18N_BYTES).ok()?;
  serde_json::from_str::<HashMap<String, String>>(&content)
    .ok()?
    .get(key)
    .cloned()
}

fn normalize_language_codes(values: &[String]) -> Result<Vec<String>, String> {
  let mut seen = HashSet::new();
  let mut normalized = Vec::with_capacity(values.len());
  for (index, value) in values.iter().enumerate() {
    let code = value.trim().to_ascii_lowercase();
    if code.is_empty()
      || code.len() > 64
      || !code
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
    {
      return Err(format!(
        "game.language[{index}] contains an invalid language code"
      ));
    }
    if seen.insert(code.clone()) {
      normalized.push(code);
    }
  }
  Ok(normalized)
}

// 规范化包入口脚本路径。扫描阶段只验证路径语义，不验证脚本文件是否存在。
fn resolve_entry(pkg_dir: &Path, entry: &str) -> Result<String, String> {
  let _ = pkg_dir;
  let trimmed = entry.trim();
  if trimmed.is_empty() {
    return Err("Entry is empty".to_string());
  }
  if trimmed.contains('\\') || trimmed.contains(':') {
    return Err(format!(
      "Entry '{}' must use a portable relative scripts path",
      entry
    ));
  }
  let mut parts = Vec::new();
  for component in Path::new(trimmed).components() {
    match component {
      Component::Normal(part) => parts.push(part.to_string_lossy().to_string()),
      _ => return Err(format!("Entry '{}' must be a relative scripts path", entry)),
    }
  }
  if parts.is_empty() {
    return Err("Entry is empty".to_string());
  }
  if let Some(last) = parts.last_mut() {
    if !last.ends_with(".lua") {
      last.push_str(".lua");
    }
  }
  Ok(parts.join("/"))
}

impl<E: From<PackageAsyncEvent> + Send + 'static> tg_service_async::AsyncJob<E> for PackageTask {
  fn run(
    self: Box<Self>,
    id: tg_service_async::TaskId,
    events: &crossbeam_channel::Sender<E>,
    cancellation: &tg_service_async::TaskCancellation,
  ) -> Result<(), String> {
    let _ = cancellation;
    run_package_task(id, *self, events)
  }
}

#[cfg(test)]
mod tests {
  use std::io;

  use super::*;
  use tg_service_async::{AsyncRuntime, TaskStatusEvent};

  #[derive(Debug)]
  enum TestEvent {
    Package(PackageAsyncEvent),
    Status,
  }

  impl From<PackageAsyncEvent> for TestEvent {
    fn from(event: PackageAsyncEvent) -> Self {
      Self::Package(event)
    }
  }

  impl From<TaskStatusEvent> for TestEvent {
    fn from(_: TaskStatusEvent) -> Self {
      Self::Status
    }
  }

  const MISSING: &str = "[Missing i18n Key: {value:missing_key}]";

  fn temp_root(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("tg_package_{name}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    root
  }

  #[cfg(unix)]
  fn create_dir_link(target: &Path, link: &Path) -> io::Result<()> {
    std::os::unix::fs::symlink(target, link)
  }

  #[cfg(windows)]
  fn create_dir_link(target: &Path, link: &Path) -> io::Result<()> {
    std::os::windows::fs::symlink_dir(target, link)
  }

  #[cfg(unix)]
  fn create_file_link(target: &Path, link: &Path) -> io::Result<()> {
    std::os::unix::fs::symlink(target, link)
  }

  #[cfg(windows)]
  fn create_file_link(target: &Path, link: &Path) -> io::Result<()> {
    std::os::windows::fs::symlink_file(target, link)
  }

  fn poll_async_package_events(
    runtime: &AsyncRuntime<TestEvent>,
    service: &mut PackageService,
    log: &mut LogService,
  ) -> Vec<PackageEvent> {
    runtime
      .poll_events()
      .into_iter()
      .filter_map(|event| match event {
        TestEvent::Package(event) => Some(service.handle_async_event(event, log)),
        _ => None,
      })
      .collect()
  }

  #[test]
  fn package_id_is_stable_validated_and_separates_source_and_type() {
    let official_game =
      PackageId::new(PackageSource::Official, PackageType::Game, "sample.game-1").unwrap();
    let mod_game = PackageId::new(PackageSource::Mod, PackageType::Game, "sample.game-1").unwrap();
    let official_screensaver = PackageId::new(
      PackageSource::Official,
      PackageType::Screensaver,
      "sample.game-1",
    )
    .unwrap();

    assert_eq!(official_game.storage_key(), "official/game/sample.game-1");
    assert_ne!(official_game, mod_game);
    assert_ne!(official_game, official_screensaver);
    assert!(PackageId::new(PackageSource::Mod, PackageType::Game, "").is_err());
    assert!(PackageId::new(PackageSource::Mod, PackageType::Game, "bad/id").is_err());
    assert!(PackageId::new(PackageSource::Mod, PackageType::Game, "中").is_err());
    assert!(PackageId::new(PackageSource::Mod, PackageType::Game, "a".repeat(129)).is_err());
    assert!(
      serde_json::from_str::<PackageId>(
        r#"{"source":"mod","package_type":"game","mod_id":"../escape"}"#,
      )
      .is_err()
    );
  }

  #[test]
  fn scan_allows_the_same_mod_id_across_sources_and_package_types() {
    let root = temp_root("package_id_scope");
    write_game(&root, "scripts/game", "shared.id", "Official Game");
    write_game(&root, "data/mod/game", "shared.id", "Mod Game");
    write_screensaver(
      &root,
      "data/mod/screensaver",
      "shared.id",
      "Mod Screensaver",
    );

    let mut service = PackageService::new();
    let mut log = LogService::new();
    scan(&mut service, &root, &mut log, "en_us");

    assert_eq!(service.games().len(), 2);
    assert_eq!(service.screensavers().len(), 1);
    assert!(
      service
        .find_by_id(
          &PackageId::new(PackageSource::Official, PackageType::Game, "shared.id").unwrap()
        )
        .is_some()
    );
    assert!(
      service
        .find_by_id(
          &PackageId::new(PackageSource::Mod, PackageType::Screensaver, "shared.id").unwrap()
        )
        .is_some()
    );
    let _ = std::fs::remove_dir_all(root);
  }

  #[test]
  fn action_validation_allows_unbound_actions_and_rejects_corrupt_bindings() {
    assert_eq!(
      normalize_action_keys("idle", Vec::new()).unwrap(),
      Vec::<Vec<String>>::new()
    );
    assert!(normalize_action_keys("move", vec![Vec::new()]).is_err());
    assert!(normalize_action_keys("move", vec![vec!["arrow_right".into()]]).is_err());
    assert!(
      normalize_action_keys(
        "move",
        vec![vec!["a".into()], vec!["d".into()], vec!["right".into()]],
      )
      .is_err()
    );
    assert!(
      normalize_action_keys(
        "move",
        vec![vec!["ctrl".into(), "shift".into(), "right".into()]],
      )
      .is_err()
    );
  }

  #[test]
  fn manifest_semantic_validation_reports_independent_fields_together() {
    let root = temp_root("semantic_diagnostics");
    let dir = root.join("data/mod/game/invalid");
    std::fs::create_dir_all(dir.join("scripts")).unwrap();
    std::fs::write(dir.join("scripts/main.lua"), "-- test").unwrap();
    std::fs::write(
      dir.join("package.json"),
      r#"{
        "mod_id":"invalid/id",
        "schema_version":99,
        "type":"game",
        "version":"1.0.0",
        "version_code":0,
        "api":{"min":2,"max":0},
        "entry":"main",
        "display":{"title":"Invalid","description":"Description","author":"Tester"},
        "runtime":{"min_width":1,"min_height":1},
        "game":{
          "name":"Invalid","detail":"Detail","target_fps":17,
          "language":["en_us"],"actions":{}
        }
      }"#,
    )
    .unwrap();
    let request = ScanRequest {
      root: root.clone(),
      language_code: "en_us".to_string(),
      missing_template: MISSING.to_string(),
    };

    let diagnostics = read_package(
      &dir,
      "invalid",
      &PackageType::Game,
      &PackageSource::Mod,
      &request,
    )
    .unwrap_err()
    .into_diagnostics("data/mod/game/invalid/package.json".to_string());
    let fields = diagnostics
      .iter()
      .map(|diagnostic| diagnostic.field_path.as_str())
      .collect::<Vec<_>>();
    assert!(fields.contains(&"schema_version"));
    assert!(fields.contains(&"mod_id"));
    assert!(fields.contains(&"version_code"));
    assert!(fields.contains(&"api.min"));
    assert!(fields.contains(&"api.max"));
    assert!(fields.contains(&"game.target_fps"));

    let _ = std::fs::remove_dir_all(root);
  }

  #[test]
  fn checked_in_lua_test_packages_have_valid_complete_manifests() {
    // The checked-in test packages live at the workspace root, three levels above this crate.
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../test_package");
    let request = ScanRequest {
      root: root.clone(),
      language_code: "en_us".to_string(),
      missing_template: MISSING.to_string(),
    };

    for (relative, expected_type) in [
      ("game", PackageType::Game),
      ("screensaver", PackageType::Screensaver),
    ] {
      let category = root.join(relative);
      let mut count = 0;
      for entry in std::fs::read_dir(&category).unwrap() {
        let entry = entry.unwrap();
        if !entry.file_type().unwrap().is_dir() {
          continue;
        }
        count += 1;
        let dir_name = entry.file_name().to_string_lossy().into_owned();
        let package = read_package(
          &entry.path(),
          &dir_name,
          &expected_type,
          &PackageSource::Mod,
          &request,
        )
        .unwrap_or_else(|error| panic!("{relative}/{dir_name}: {}", error.reason));
        assert!(package.runtime.min_width > 0);
        assert!(package.runtime.min_height > 0);
        assert!(entry.path().join("scripts/main.lua").is_file());
        match expected_type {
          PackageType::Game => {
            let game = package.game.expect("game configuration");
            assert!(!game.actions.is_empty());
            let action_map = game
              .actions
              .iter()
              .map(
                |(action, config)| tg_core_input::ActionMapEntry {
                  action: action.clone(),
                  description: config.description.clone(),
                  keys: config.keys.clone(),
                },
              )
              .collect::<Vec<_>>();
            tg_core_input::translate_action_map(&action_map)
              .unwrap_or_else(|error| panic!("{relative}/{dir_name}: {error:?}"));
          }
          PackageType::Screensaver => {
            assert!(package.screensaver.is_some());
          }
        }
      }
      assert_eq!(count, 3, "expected three {relative} test packages");
    }
  }

  fn write_game(root: &Path, relative: &str, id: &str, title: &str) {
    let dir = root.join(relative).join(id);
    std::fs::create_dir_all(dir.join("scripts")).unwrap();
    std::fs::write(dir.join("scripts/main.lua"), "-- test").unwrap();
    std::fs::write(
      dir.join("package.json"),
      format!(
        r#"{{
          "mod_id":"{id}",
          "schema_version":1,
          "type":"game",
          "version":"1.0.0",
          "version_code":1,
          "api":{{"min":1,"max":1}},
          "entry":"main",
          "display":{{"title":"{title}","description":"Description","author":"Tester"}},
          "runtime":{{"min_width":0,"min_height":0}},
          "game":{{"name":"{title}","detail":"Detail","target_fps":60,"language":["en_us"],"actions":{{}}}}
        }}"#
      ),
    )
    .unwrap();
  }

  fn write_game_manifest_only(root: &Path, relative: &str, id: &str, title: &str) {
    let dir = root.join(relative).join(id);
    std::fs::create_dir_all(dir.join("scripts")).unwrap();
    std::fs::write(dir.join("scripts/main.lua"), "-- test").unwrap();
    std::fs::write(
      dir.join("package.json"),
      format!(
        r#"{{
          "mod_id":"{id}",
          "schema_version":1,
          "type":"game",
          "version":"1.0.0",
          "version_code":1,
          "api":{{"min":1,"max":1}},
          "entry":"main",
          "display":{{"title":"{title}","description":"Description","author":"Tester"}},
          "runtime":{{"min_width":0,"min_height":0}},
          "game":{{"name":"{title}","detail":"Detail","target_fps":60,"language":["en_us"],"actions":{{}}}}
        }}"#
      ),
    )
    .unwrap();
  }

  fn write_screensaver(root: &Path, relative: &str, id: &str, title: &str) {
    let dir = root.join(relative).join(id);
    std::fs::create_dir_all(dir.join("scripts")).unwrap();
    std::fs::write(dir.join("scripts/main.lua"), "-- test").unwrap();
    std::fs::write(
      dir.join("package.json"),
      format!(
        r#"{{
          "mod_id":"{id}",
          "schema_version":1,
          "type":"screensaver",
          "version":"1.0.0",
          "version_code":1,
          "api":{{"min":1,"max":1}},
          "entry":"main",
          "display":{{"title":"{title}","description":"Description","author":"Tester"}},
          "runtime":{{"min_width":0,"min_height":0}},
          "screensaver":{{"name":"{title}","command":"screen"}}
        }}"#
      ),
    )
    .unwrap();
  }

  fn scan(service: &mut PackageService, root: &Path, log: &mut LogService, language: &str) {
    service.scan_all(root, log, language, MISSING);
  }

  #[test]
  fn package_scan_log_lists_loaded_and_failed_manifests_with_diagnostics() {
    let root = temp_root("scan_log");
    write_game(&root, "data/mod/game", "valid_game", "Valid Game");
    let invalid_dir = root.join("data/mod/game/invalid_game");
    std::fs::create_dir_all(&invalid_dir).unwrap();
    std::fs::write(
      invalid_dir.join("package.json"),
      "{\n  \"mod_id\": \"invalid_game\",\n  \"schema_version\": ]\n}",
    )
    .unwrap();

    let log_dir = root.join("data/log");
    let main_log = log_dir.join("tui_log.log");
    let package_log = log_dir.join("package.log");
    let mut log = LogService::new();
    log.set_output_path(main_log.clone()).unwrap();
    log
      .set_package_scan_output_path(package_log.clone())
      .unwrap();
    log.activate_embedded_english().unwrap();
    let mut service = PackageService::new();

    scan(&mut service, &root, &mut log, "en_us");

    let package_text = std::fs::read_to_string(package_log).unwrap();
    assert!(package_text.contains("mod/game/valid_game"));
    assert!(package_text.contains("data/mod/game/valid_game/package.json"));
    assert!(package_text.contains("data/mod/game/invalid_game/package.json"));
    assert!(package_text.contains("[json_syntax]"));
    assert!(package_text.contains("3:"));

    let main_text = std::fs::read_to_string(main_log).unwrap();
    assert!(main_text.contains("1 loaded, 1 failed"));
    assert!(!main_text.contains("invalid_game/package.json"));
    let _ = std::fs::remove_dir_all(root);
  }

  fn write_package_language(
    root: &Path,
    relative: &str,
    id: &str,
    language: &str,
    file: &str,
    json: &str,
  ) {
    let language_root = root
      .join(relative)
      .join(id)
      .join("assets/language")
      .join(language);
    let path = if file.is_empty() {
      language_root.with_extension("json")
    } else {
      language_root.join(file)
    };
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, json).unwrap();
  }

  #[test]
  fn mod_lists_exclude_official_packages() {
    let root = temp_root("mod_filter");
    write_game(&root, "scripts/game", "official_game", "Official Game");
    write_game(&root, "data/mod/game", "mod_game", "Mod Game");
    write_screensaver(
      &root,
      "data/mod/screensaver",
      "mod_screensaver",
      "Mod Screensaver",
    );

    let mut service = PackageService::new();
    let mut log = LogService::new();
    scan(&mut service, &root, &mut log, "en_us");

    let games = service.mod_games();
    assert_eq!(games.len(), 1);
    assert_eq!(games[0].mod_id, "mod_game");
    assert_eq!(service.mod_screensavers()[0].mod_id, "mod_screensaver");

    let _ = std::fs::remove_dir_all(root);
  }

  #[test]
  fn request_rescan_replaces_snapshot() {
    let root = temp_root("rescan");
    write_game(&root, "data/mod/game", "first", "First");

    let mut service = PackageService::new();
    let mut log = LogService::new();
    scan(&mut service, &root, &mut log, "en_us");
    assert_eq!(service.mod_games()[0].mod_id, "first");

    std::fs::remove_dir_all(root.join("data/mod/game/first")).unwrap();
    write_game(&root, "data/mod/game", "second", "Second");
    let runtime = AsyncRuntime::<TestEvent>::with_worker_count(1);
    assert!(service.request_rescan(&runtime));

    for _ in 0..100 {
      let _ = poll_async_package_events(&runtime, &mut service, &mut log);
      if service
        .mod_games()
        .first()
        .map(|entry| entry.mod_id.as_str())
        == Some("second")
      {
        let _ = std::fs::remove_dir_all(root);
        return;
      }
      std::thread::sleep(std::time::Duration::from_millis(10));
    }
    panic!("rescan did not replace package snapshot");
  }

  #[test]
  fn request_rescan_emits_started_progress_and_finished_events() {
    let root = temp_root("rescan_progress");
    write_game(&root, "data/mod/game", "first", "First");
    write_screensaver(&root, "data/mod/screensaver", "screen", "Screen");

    let mut service = PackageService::new();
    let mut log = LogService::new();
    scan(&mut service, &root, &mut log, "en_us");
    let runtime = AsyncRuntime::<TestEvent>::with_worker_count(1);
    assert!(service.request_rescan(&runtime));

    let mut events = Vec::new();
    for _ in 0..100 {
      events.extend(poll_async_package_events(&runtime, &mut service, &mut log));
      if events
        .iter()
        .any(|event| matches!(event, PackageEvent::ScanFinished { .. }))
      {
        break;
      }
      std::thread::sleep(std::time::Duration::from_millis(10));
    }

    assert!(matches!(
      events.first(),
      Some(PackageEvent::ScanStarted { total: 2 })
    ));
    assert!(events.iter().any(|event| matches!(
      event,
      PackageEvent::ScanProgress { scanned, total: 2 } if *scanned <= 2
    )));
    assert!(
      events
        .iter()
        .any(|event| matches!(event, PackageEvent::ScanFinished { total: 2, .. }))
    );

    let _ = std::fs::remove_dir_all(root);
  }

  #[test]
  fn package_json_only_mod_package_is_scanned() {
    let root = temp_root("manifest_only");
    write_game_manifest_only(&root, "data/mod/game", "manifest_only", "Manifest Only");

    let mut service = PackageService::new();
    let mut log = LogService::new();
    scan(&mut service, &root, &mut log, "en_us");

    let games = service.mod_games();
    assert_eq!(games.len(), 1);
    assert_eq!(games[0].mod_id, "manifest_only");
    assert_eq!(service.games()[0].entry, "main.lua");

    let _ = std::fs::remove_dir_all(root);
  }

  #[test]
  fn exact_lookup_and_entry_resolution_use_package_identity() {
    let root = temp_root("lua_entry");
    write_game(&root, "scripts/game", "official_id", "Official");
    write_game(&root, "data/mod/game", "mod_id", "Mod");

    let mut service = PackageService::new();
    let mut log = LogService::new();
    scan(&mut service, &root, &mut log, "en_us");

    let official = service
      .find_game(&PackageSource::Official, "official_id")
      .unwrap();
    let modified = service.find_game(&PackageSource::Mod, "mod_id").unwrap();
    assert!(
      service
        .find_game(&PackageSource::Mod, "official_id")
        .is_none()
    );
    assert_ne!(official.path, modified.path);
    assert!(
      service
        .resolve_entry_path(&official)
        .unwrap()
        .ends_with("scripts/main.lua")
    );
    assert!(
      service
        .resolve_entry_path(&modified)
        .unwrap()
        .ends_with("scripts/main.lua")
    );

    std::fs::remove_file(modified.path.join("scripts/main.lua")).unwrap();
    assert!(service.resolve_entry_path(&modified).is_err());
    let _ = std::fs::remove_dir_all(root);
  }

  #[test]
  fn package_list_entry_carries_game_action_keys() {
    let root = temp_root("entry_action_keys");
    let dir = root.join("data/mod/game/action_keys");
    std::fs::create_dir_all(dir.join("scripts")).unwrap();
    std::fs::write(dir.join("scripts/main.lua"), "-- test").unwrap();
    std::fs::write(
      dir.join("package.json"),
      r#"{
        "mod_id":"action_keys",
        "schema_version":1,
        "type":"game",
        "version":"1.0.0",
        "version_code":1,
        "api":{"min":1,"max":1},
        "entry":"main",
        "display":{"title":"f%{key:move_up} Move","description":"Description","author":"Tester"},
        "runtime":{"min_width":0,"min_height":0},
        "game":{
          "name":"Move Game","detail":"Detail","language":["en_us"],
          "target_fps":60,
          "actions":{
            "move_up":{"description":"Move up","keys":[["w"],["up"]],"lock":true}
          }
        }
      }"#,
    )
    .unwrap();

    let mut service = PackageService::new();
    let mut log = LogService::new();
    scan(&mut service, &root, &mut log, "en_us");

    assert!(service.games()[0].game.as_ref().unwrap().actions["move_up"].lock);
    let entry = service.mod_games().remove(0);
    assert_eq!(
      entry.key_actions.get("move_up"),
      Some(&vec![vec!["w".to_string()], vec!["up".to_string()]])
    );
    service.set_user_game_key_actions(BTreeMap::from([(
      PackageId::new(PackageSource::Mod, PackageType::Game, "action_keys")
        .unwrap()
        .storage_key(),
      BTreeMap::from([("move_up".into(), vec![vec!["k".into()]])]),
    )]));
    let entry = service.mod_games().remove(0);
    assert_eq!(entry.key_actions["move_up"], vec![vec!["k".to_string()]]);
    assert_eq!(
      entry.key_default_actions["move_up"],
      vec![vec!["w".to_string()], vec!["up".to_string()]]
    );

    let _ = std::fs::remove_dir_all(root);
  }

  #[test]
  fn game_action_may_define_no_keys() {
    let root = temp_root("empty_action_keys");
    let dir = root.join("data/mod/game/empty_action_keys");
    std::fs::create_dir_all(dir.join("scripts")).unwrap();
    std::fs::write(dir.join("scripts/main.lua"), "-- test").unwrap();
    std::fs::write(
      dir.join("package.json"),
      r#"{
        "mod_id":"empty_action_keys",
        "schema_version":1,
        "type":"game",
        "version":"1.0.0",
        "version_code":1,
        "api":{"min":1,"max":1},
        "entry":"main",
        "display":{"title":"Empty Action","description":"Description","author":"Tester"},
        "runtime":{"min_width":0,"min_height":0},
        "game":{
          "name":"Empty Action","detail":"Detail","language":["en_us"],
          "target_fps":60,
          "actions":{"optional":{"description":"Optional","keys":[]}}
        }
      }"#,
    )
    .unwrap();

    let mut service = PackageService::new();
    let mut log = LogService::new();
    scan(&mut service, &root, &mut log, "en_us");

    assert!(
      service.games()[0].game.as_ref().unwrap().actions["optional"]
        .keys
        .is_empty()
    );
    let _ = std::fs::remove_dir_all(root);
  }

  #[test]
  fn package_i18n_fields_are_resolved_during_scan() {
    let root = temp_root("i18n_fields");
    write_game_manifest_only(&root, "data/mod/game", "i18n_game", "@display/title");
    let package_json = root.join("data/mod/game/i18n_game/package.json");
    std::fs::write(
      &package_json,
      r#"{
        "mod_id":"i18n_game",
        "schema_version":1,
        "type":"game",
        "version":{"type":"i18n","path":"meta.json","key":"version","callback":"1.0.0"},
        "version_code":1,
        "api":{"min":1,"max":1},
        "entry":"ui/init",
        "display":{
          "title":{"type":"i18n","path":"display.json","key":"title","callback":"Title"},
          "description":{"type":"i18n","path":"deep/nested/text.json","key":"description","callback":"Description"},
          "author":{"type":"i18n","path":"common.json","key":"author","callback":"Author"}
        },
        "runtime":{"min_width":1,"min_height":1},
        "game":{
          "name":{"type":"i18n","path":"common.json","key":"game.name","callback":"Game"},
          "detail":{"type":"i18n","path":"detail.json","key":"main","callback":"Detail"},
          "target_fps":60,
          "language":["zh_cn"],
          "score":{"enabled":true,"empty_text":{"type":"i18n","path":"common.json","key":"score.empty","callback":"Empty"}},
          "actions":{"move_up":{"description":{"type":"i18n","path":"action.json","key":"move.up","callback":"Move"},"keys":[["w"]]}}
        }
      }"#,
    )
    .unwrap();
    std::fs::create_dir_all(root.join("data/mod/game/i18n_game/scripts/ui")).unwrap();
    std::fs::write(
      root.join("data/mod/game/i18n_game/scripts/ui/init.lua"),
      "-- test",
    )
    .unwrap();
    write_package_language(
      &root,
      "data/mod/game",
      "i18n_game",
      "zh_cn",
      "display.json",
      r#"{"title":"中文标题"}"#,
    );
    write_package_language(
      &root,
      "data/mod/game",
      "i18n_game",
      "zh_cn",
      "meta.json",
      r#"{"version":"版本一"}"#,
    );
    write_package_language(
      &root,
      "data/mod/game",
      "i18n_game",
      "zh_cn",
      "deep/nested/text.json",
      r#"{"description":"多级简介"}"#,
    );
    write_package_language(
      &root,
      "data/mod/game",
      "i18n_game",
      "zh_cn",
      "common.json",
      r#"{"author":"作者","game.name":"游戏名","score.empty":"无记录"}"#,
    );
    write_package_language(
      &root,
      "data/mod/game",
      "i18n_game",
      "zh_cn",
      "detail.json",
      r#"{"main":"游戏详情"}"#,
    );
    write_package_language(
      &root,
      "data/mod/game",
      "i18n_game",
      "zh_cn",
      "action.json",
      r#"{"move.up":"上移"}"#,
    );

    let mut service = PackageService::new();
    let mut log = LogService::new();
    scan(&mut service, &root, &mut log, "zh_cn");

    let game = service.games().remove(0);
    let game_config = game.game.as_ref().unwrap();
    assert_eq!(game.entry, "ui/init.lua");
    assert_eq!(game.version, "版本一");
    assert_eq!(game.display.title, "中文标题");
    assert_eq!(game.display.description, "多级简介");
    assert_eq!(game.display.author, "作者");
    assert_eq!(game_config.name, "游戏名");
    assert_eq!(game_config.detail, "游戏详情");
    assert_eq!(game_config.score.as_ref().unwrap().empty_text, "无记录");
    assert_eq!(game_config.actions["move_up"].description, "上移");
    assert!(!game_config.actions["move_up"].lock);
    assert_eq!(package_list_entry(game).game_name, "游戏名");

    let _ = std::fs::remove_dir_all(root);
  }

  #[test]
  fn package_i18n_dot_file_key_path_is_resolved() {
    let root = temp_root("i18n_dot_path");
    write_game_manifest_only(&root, "data/mod/game", "i18n_dot_game", "Dot Path Game");
    let package_json = root.join("data/mod/game/i18n_dot_game/package.json");
    let content = std::fs::read_to_string(&package_json)
      .unwrap()
      .replace(
        r#""author":"Tester""#,
        r#""author":{"type":"i18n","path":"creator/identity/author.json","key":"name","callback":"Author"}"#,
      )
      .replace(r#""version":"1.0.0""#, r#""version":{"type":"i18n","path":"meta/version.json","key":"text","callback":"1.0.0"}"#)
      .replace(
        r#""title":"Dot Path Game""#,
        r#""title":{"type":"i18n","path":"long/path/display/title.json","key":"text","callback":"Title"}"#,
      );
    std::fs::write(package_json, content).unwrap();
    write_package_language(
      &root,
      "data/mod/game",
      "i18n_dot_game",
      "zh_cn",
      "creator/identity/author.json",
      r#"{"name":"点号作者"}"#,
    );
    write_package_language(
      &root,
      "data/mod/game",
      "i18n_dot_game",
      "zh_cn",
      "meta/version.json",
      r#"{"text":"点号版本"}"#,
    );
    write_package_language(
      &root,
      "data/mod/game",
      "i18n_dot_game",
      "zh_cn",
      "long/path/display/title.json",
      r#"{"text":"点号标题"}"#,
    );

    let mut service = PackageService::new();
    let mut log = LogService::new();
    scan(&mut service, &root, &mut log, "zh_cn");

    let game = service.games().remove(0);
    assert_eq!(game.display.author, "点号作者");
    assert_eq!(game.version, "点号版本");
    assert_eq!(game.display.title, "点号标题");

    let _ = std::fs::remove_dir_all(root);
  }

  #[test]
  fn package_i18n_falls_back_to_en_us_then_missing_template() {
    let root = temp_root("i18n_fallback");
    write_game_manifest_only(&root, "data/mod/game", "fallback_game", "Fallback Title");
    write_package_language(
      &root,
      "data/mod/game",
      "fallback_game",
      "en_us",
      "display.json",
      r#"{"title":"English Title"}"#,
    );
    let package_json = root.join("data/mod/game/fallback_game/package.json");
    let content = std::fs::read_to_string(&package_json)
      .unwrap()
      .replace(
        r#""title":"Fallback Title""#,
        r#""title":{"type":"i18n","path":"display.json","key":"title","callback":"Fallback Title"}"#,
      )
      .replace(
        r#""author":"Tester""#,
        r#""author":{"type":"i18n","path":"missing.json","key":"author","callback":"Fallback Author"}"#,
      );
    std::fs::write(package_json, content).unwrap();

    let mut service = PackageService::new();
    let mut log = LogService::new();
    scan(&mut service, &root, &mut log, "zh_cn");

    let game = service.games().remove(0);
    assert_eq!(game.display.title, "English Title");
    assert_eq!(game.display.author, "Fallback Author");

    let _ = std::fs::remove_dir_all(root);
  }

  #[test]
  fn screensaver_i18n_name_is_resolved() {
    let root = temp_root("screensaver_i18n");
    write_screensaver(&root, "data/mod/screensaver", "screen_i18n", "Screen Title");
    let package_json = root.join("data/mod/screensaver/screen_i18n/package.json");
    let content = std::fs::read_to_string(&package_json).unwrap().replace(
      r#""name":"Screen Title""#,
      r#""name":{"type":"i18n","path":"screen.json","key":"name","callback":"Screen Title"}"#,
    );
    std::fs::write(package_json, content).unwrap();
    write_package_language(
      &root,
      "data/mod/screensaver",
      "screen_i18n",
      "zh_cn",
      "screen.json",
      r#"{"name":"屏保名"}"#,
    );

    let mut service = PackageService::new();
    let mut log = LogService::new();
    scan(&mut service, &root, &mut log, "zh_cn");

    assert_eq!(
      service.screensavers()[0].screensaver.as_ref().unwrap().name,
      "屏保名"
    );

    let _ = std::fs::remove_dir_all(root);
  }

  #[test]
  fn removed_screensaver_mouse_flag_is_rejected() {
    let root = temp_root("screensaver_mouse");
    write_screensaver(
      &root,
      "data/mod/screensaver",
      "mouse_screen",
      "Mouse Screen",
    );
    let package_json = root.join("data/mod/screensaver/mouse_screen/package.json");
    let content = std::fs::read_to_string(&package_json).unwrap().replace(
      r#""screensaver":{"name":"Mouse Screen","command":"screen"}"#,
      r#""screensaver":{"name":"Mouse Screen","mouse":true,"command":"screen"}"#,
    );
    std::fs::write(package_json, content).unwrap();

    let mut service = PackageService::new();
    let mut log = LogService::new();
    scan(&mut service, &root, &mut log, "en_us");

    assert!(service.mod_screensavers().is_empty());

    let _ = std::fs::remove_dir_all(root);
  }

  #[test]
  fn game_high_privilege_and_truecolor_flags_reach_list_entry() {
    let root = temp_root("game_privilege_truecolor");
    write_game(&root, "data/mod/game", "flag_game", "Flag Game");
    let package_json = root.join("data/mod/game/flag_game/package.json");
    let content = std::fs::read_to_string(&package_json).unwrap().replace(
      r#""target_fps":60"#,
      r#""target_fps":60,"high_privilege":true,"truecolor":true"#,
    );
    std::fs::write(package_json, content).unwrap();

    let mut service = PackageService::new();
    let mut log = LogService::new();
    scan(&mut service, &root, &mut log, "en_us");

    let entry = service.mod_games().remove(0);
    assert!(entry.high_privilege_required);
    assert!(entry.truecolor_required);

    let _ = std::fs::remove_dir_all(root);
  }

  #[test]
  fn removed_game_write_field_is_rejected() {
    let root = temp_root("removed_write_field");
    write_game(
      &root,
      "data/mod/game",
      "removed_field_game",
      "Removed Field Game",
    );
    let package_json = root.join("data/mod/game/removed_field_game/package.json");
    let content = std::fs::read_to_string(&package_json)
      .unwrap()
      .replace(r#""target_fps":60"#, r#""target_fps":60,"write":true"#);
    std::fs::write(package_json, content).unwrap();

    let mut service = PackageService::new();
    let mut log = LogService::new();
    scan(&mut service, &root, &mut log, "en_us");

    assert!(service.mod_games().is_empty());

    let _ = std::fs::remove_dir_all(root);
  }

  #[test]
  fn screensaver_truecolor_command_and_i18n_name_are_scanned() {
    let root = temp_root("screensaver_flags");
    write_screensaver(&root, "data/mod/screensaver", "flag_screen", "Flag Screen");
    let package_json = root.join("data/mod/screensaver/flag_screen/package.json");
    let content = std::fs::read_to_string(&package_json).unwrap().replace(
      r#""screensaver":{"name":"Flag Screen","command":"screen"}"#,
      r#""screensaver":{"name":{"type":"i18n","path":"screen.json","key":"screen.name","callback":"Flag Screen"},"truecolor":true,"command":"flag-screen"}"#,
    );
    std::fs::write(package_json, content).unwrap();
    write_package_language(
      &root,
      "data/mod/screensaver",
      "flag_screen",
      "zh_cn",
      "screen.json",
      r#"{"screen.name":"旗标屏保"}"#,
    );

    let mut service = PackageService::new();
    let mut log = LogService::new();
    scan(&mut service, &root, &mut log, "zh_cn");

    let screen = service.screensavers().remove(0);
    let config = screen.screensaver.as_ref().unwrap();
    assert_eq!(config.name, "旗标屏保");
    assert!(config.truecolor);
    assert_eq!(config.command, "flag-screen");

    let entry = service.mod_screensavers().remove(0);
    assert!(!entry.mouse_required);
    assert!(entry.truecolor_required);

    let _ = std::fs::remove_dir_all(root);
  }

  #[test]
  fn display_assets_parse_new_image_and_text_structure() {
    let root = temp_root("display_assets");
    write_game_manifest_only(&root, "data/mod/game", "asset_game", "Asset Game");
    let dir = root.join("data/mod/game/asset_game");
    std::fs::create_dir_all(dir.join("assets/ui")).unwrap();
    std::fs::write(dir.join("assets/ui/icon.txt"), "abc\n一二三四五\nx").unwrap();
    std::fs::write(
      dir.join("assets/ui/banner.txt"),
      [
        "line00", "line01", "line02", "line03", "line04", "line05", "line06", "line07", "line08",
        "line09", "line10",
      ]
      .join("\n"),
    )
    .unwrap();

    let package_json = dir.join("package.json");
    let content = std::fs::read_to_string(&package_json)
      .unwrap()
      .replace(
        r#""author":"Tester""#,
        r#""author":"Tester","icon":{"type":"image","path":"ui/icon.png"},"banner":{"type":"text","path":"ui/banner.txt"}"#,
      );
    std::fs::write(package_json, content).unwrap();

    let mut service = PackageService::new();
    let mut log = LogService::new();
    scan(&mut service, &root, &mut log, "en_us");

    let game = service.games().remove(0);
    assert_eq!(
      game.display.icon,
      PackageAsset::Image {
        path: "ui/icon.png".to_string()
      }
    );
    let PackageAsset::Text { path, lines } = game.display.banner else {
      panic!("banner should be parsed as text asset");
    };
    assert_eq!(path, "ui/banner.txt");
    assert_eq!(lines.len(), 14);
    assert_eq!(lines[0].trim_end(), "");
    assert_eq!(lines[1].trim_end(), "");
    assert_eq!(lines[2].trim(), "line00");
    assert_eq!(lines[12].trim(), "line10");
    assert!(
      lines
        .iter()
        .all(|line| UnicodeWidthStr::width(line.as_str()) == 60)
    );

    let entry = service.mod_games().remove(0);
    let icon_path = entry.icon_path.unwrap();
    assert!(Path::new(&icon_path).ends_with(Path::new("assets").join("ui").join("icon.png")));

    let _ = std::fs::remove_dir_all(root);
  }

  #[test]
  fn text_asset_normalizes_icon_to_four_lines_and_eight_columns() {
    let lines = normalize_asset_text("abcdefghi\n中中中中中\nx", AssetShape::Icon);
    assert_eq!(lines.len(), 4);
    assert!(
      lines
        .iter()
        .all(|line| UnicodeWidthStr::width(line.as_str()) == 8)
    );
    assert_eq!(lines[0], "        ");
    assert_eq!(lines[1], "abcdefghi".chars().take(8).collect::<String>());
    assert_eq!(lines[2], "中中中中");
    assert_eq!(lines[3], "    x   ");
  }

  #[test]
  fn rich_text_asset_lines_are_preserved_for_ui_clipping() {
    let rich_line = "f%<fg:red>RICH</fg> + plain text that is longer than the icon width";
    let lines = normalize_asset_text(&format!("{rich_line}\nx"), AssetShape::Icon);

    assert_eq!(lines.len(), 4);
    assert_eq!(lines[1], rich_line);
    assert_eq!(lines[2], "    x   ");
  }

  #[test]
  fn package_watcher_filters_first_level_package_json_only() {
    let root = PathBuf::from("root/data/mod/game");
    let roots = vec![root.clone()];

    assert_eq!(
      watched_package_dir(&roots, &root.join("alpha/package.json")),
      Some(root.join("alpha"))
    );
    assert_eq!(
      watched_package_dir(&roots, &root.join("alpha")),
      Some(root.join("alpha"))
    );
    assert_eq!(
      watched_package_dir(&roots, &root.join("alpha/nested/package.json")),
      None
    );
    assert_eq!(
      watched_package_dir(&roots, &root.join("alpha/nested")),
      None
    );
    assert_eq!(
      watched_package_dir(&roots, &root.join("alpha/readme.md")),
      None
    );
  }

  #[test]
  fn watched_resource_files_map_back_to_package_dir() {
    let root = PathBuf::from("root/data/mod/game");
    let roots = vec![root.clone()];
    let package_dir = root.join("alpha");
    let files = vec![
      package_dir.join("package.json"),
      package_dir.join("assets/ui/icon.txt"),
      package_dir.join("assets/language/zh_cn.json"),
      root.join("beta/assets/language/zh_cn/display.json"),
      root.join("package.json"),
    ];

    let watched = watched_file_package_dirs(&roots, files);

    assert_eq!(
      watched.get(&package_dir.join("assets/ui/icon.txt")),
      Some(&package_dir)
    );
    assert_eq!(
      watched.get(&package_dir.join("assets/language/zh_cn.json")),
      Some(&package_dir)
    );
    assert_eq!(
      watched.get(&root.join("beta/assets/language/zh_cn/display.json")),
      Some(&root.join("beta"))
    );
    assert!(!watched.contains_key(&root.join("package.json")));
  }

  #[test]
  fn scan_collects_manifest_asset_and_i18n_watch_files() {
    let root = temp_root("watch_files");
    write_game_manifest_only(&root, "data/mod/game", "watch_game", "Watch Game");
    let dir = root.join("data/mod/game/watch_game");
    std::fs::create_dir_all(dir.join("assets/ui")).unwrap();
    std::fs::write(dir.join("assets/ui/icon.txt"), "icon").unwrap();
    write_package_language(
      &root,
      "data/mod/game",
      "watch_game",
      "zh_cn",
      "display.json",
      r#"{"title":"监听标题"}"#,
    );
    let package_json = dir.join("package.json");
    let content = std::fs::read_to_string(&package_json)
      .unwrap()
      .replace(
        r#""title":"Watch Game""#,
        r#""title":{"type":"i18n","path":"display.json","key":"title","callback":"Watch Game"}"#,
      )
      .replace(
        r#""author":"Tester""#,
        r#""author":"Tester","icon":{"type":"text","path":"ui/icon.txt"}"#,
      );
    std::fs::write(&package_json, content).unwrap();

    let request = ScanRequest {
      root: root.clone(),
      language_code: "zh_cn".to_string(),
      missing_template: MISSING.to_string(),
    };
    let total = count_package_candidates(&request);
    let report = scan_all_packages(&request, &mut |_| {}, total);
    let files = report.watched_files;

    assert!(files.contains(&package_json));
    assert!(files.contains(&dir.join("assets/ui/icon.txt")));
    assert!(files.contains(&dir.join("assets/language/zh_cn/display.json")));
    assert!(files.contains(&dir.join("assets/language/en_us/display.json")));

    let _ = std::fs::remove_dir_all(root);
  }

  #[test]
  fn missing_assets_use_defaults_and_invalid_explicit_assets_reject_package() {
    let root = temp_root("default_assets");
    write_game_manifest_only(
      &root,
      "data/mod/game",
      "default_asset_game",
      "Default Asset Game",
    );
    let dir = root.join("data/mod/game/default_asset_game");
    let mut service = PackageService::new();
    let mut log = LogService::new();
    scan(&mut service, &root, &mut log, "en_us");

    let game = service.games().remove(0);
    let PackageAsset::Text { lines: icon, .. } = game.display.icon else {
      panic!("invalid icon should fall back to default text icon");
    };
    let PackageAsset::Text { lines: banner, .. } = game.display.banner else {
      panic!("missing banner should fall back to default text banner");
    };
    assert_eq!(icon, default_icon_lines());
    assert_eq!(banner.len(), 14);
    assert!(
      banner
        .iter()
        .all(|line| UnicodeWidthStr::width(line.as_str()) == 60)
    );
    assert!(service.mod_games()[0].icon_path.is_none());

    let package_json = dir.join("package.json");
    let content = std::fs::read_to_string(&package_json).unwrap().replace(
      r#""author":"Tester""#,
      r#""author":"Tester","icon":{"type":"image","path":"bad/icon.gif"}"#,
    );
    std::fs::write(package_json, content).unwrap();
    scan(&mut service, &root, &mut log, "en_us");
    assert!(service.games().is_empty());

    let _ = std::fs::remove_dir_all(root);
  }

  #[test]
  fn entry_allows_relative_scripts_path_only() {
    assert_eq!(resolve_entry(Path::new("."), "main").unwrap(), "main.lua");
    assert_eq!(
      resolve_entry(Path::new("."), "main.lua").unwrap(),
      "main.lua"
    );
    assert_eq!(
      resolve_entry(Path::new("."), "ui/init").unwrap(),
      "ui/init.lua"
    );
    assert_eq!(
      resolve_entry(Path::new("."), "ui/init.lua").unwrap(),
      "ui/init.lua"
    );
    assert!(resolve_entry(Path::new("."), "").is_err());
    assert!(resolve_entry(Path::new("."), "../main").is_err());
    assert!(resolve_entry(Path::new("."), "/main").is_err());
    assert!(resolve_entry(Path::new("."), r"..\main").is_err());
    assert!(resolve_entry(Path::new("."), r"C:\main").is_err());
  }

  #[test]
  fn package_i18n_paths_reject_directory_traversal() {
    assert!(!safe_path_segment("../outside"));
    assert!(safe_asset_path("../outside.json").is_none());
    assert!(safe_asset_path(r"folder\..\title.json").is_none());
  }

  #[test]
  fn package_file_resolution_stays_inside_assets() {
    let root = temp_root("asset_boundary");
    let package = root.join("package");
    std::fs::create_dir_all(package.join("assets/ui")).unwrap();
    std::fs::write(package.join("assets/ui/icon.txt"), "safe").unwrap();
    std::fs::write(root.join("outside.txt"), "outside").unwrap();

    assert!(
      resolve_package_file(&package, Path::new("assets"), Path::new("ui/icon.txt")).is_some()
    );
    assert!(
      resolve_package_file(&package, Path::new("assets"), Path::new("../outside.txt")).is_none()
    );

    let link = package.join("assets/ui/outside.txt");
    if create_file_link(&root.join("outside.txt"), &link).is_ok() {
      assert!(
        resolve_package_file(&package, Path::new("assets"), Path::new("ui/outside.txt")).is_none()
      );
    }
    let _ = std::fs::remove_dir_all(root);
  }

  #[test]
  fn scripts_directory_link_cannot_escape_package() {
    let root = temp_root("scripts_link_boundary");
    write_game_manifest_only(&root, "data/mod/game", "linked_scripts", "Linked Scripts");
    let package = root.join("data/mod/game/linked_scripts");
    let outside = root.join("outside_scripts");
    std::fs::create_dir_all(&outside).unwrap();
    std::fs::write(outside.join("main.lua"), "-- outside").unwrap();
    std::fs::remove_dir_all(package.join("scripts")).unwrap();
    if create_dir_link(&outside, &package.join("scripts")).is_err() {
      let _ = std::fs::remove_dir_all(root);
      return;
    }

    let mut service = PackageService::new();
    let mut log = LogService::new();
    scan(&mut service, &root, &mut log, "en_us");
    assert!(service.games().is_empty());
    let _ = std::fs::remove_dir_all(root);
  }

  #[test]
  fn package_text_reads_are_size_limited() {
    let root = temp_root("read_limit");
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join("large.txt");
    std::fs::write(&path, "12345").unwrap();
    assert_eq!(read_utf8_file_limited(&path, 5).unwrap(), "12345");
    assert!(read_utf8_file_limited(&path, 4).is_err());
    let _ = std::fs::remove_dir_all(root);
  }
}
