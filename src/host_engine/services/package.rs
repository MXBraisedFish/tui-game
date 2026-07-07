use std::collections::{HashMap, HashSet};
use std::path::{Component, Path, PathBuf};
use std::sync::{
  Arc,
  atomic::{AtomicBool, Ordering},
};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use crossbeam_channel::{Receiver, RecvTimeoutError, Sender, unbounded};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::Deserialize;

use crate::host_engine::services::{
  async_runtime::{AsyncRuntime, EngineEvent, EngineTask, ManagedThreadId, TaskId},
  log::{LogService, LogSource},
  version::{HOST_API_VERSION, PACKAGE_MANIFEST_VERSION},
};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

const VALID_TARGET_FPS: &[u32] = &[30, 60, 120];

/// 包来源（官方或模组）
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PackageSource {
  Official,
  Mod,
}

/// 包类型（游戏或屏保）
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PackageType {
  Game,
  Screensaver,
}

/// 包文本字段：普通字符串直接使用，@ 开头的字符串在扫描时解析包内 i18n。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PackageText {
  Literal(String),
  I18n(String),
}

/// 包完整信息
#[derive(Clone, Debug)]
pub struct PackageInfo {
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
  pub mod_id: String,
  pub source: PackageSource,
  pub package_type: PackageType,
  pub key_actions: HashMap<String, Vec<Vec<String>>>,
  pub title: String,
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
  pub write_required: bool,
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
  pub write: bool,
  pub mouse: bool,
  pub target_fps: u32,
  pub save: bool,
  pub score: Option<ScoreConfig>,
  pub actions: HashMap<String, ActionConfig>,
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
}

/// 屏保配置
#[derive(Clone, Debug)]
pub struct ScreensaverConfig {
  pub name: String,
  pub mouse: bool,
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
  Info(String),
  Warn(String),
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
  pub fn request_rescan(&self, async_runtime: &AsyncRuntime) -> bool {
    let Some(request) = self.last_scan.clone() else {
      return false;
    };
    async_runtime.submit(EngineTask::Package(PackageTask::Scan(request)));
    true
  }

  /// 启动 package.json 热更新监听。监听线程只产生事件；快照仍由主线程替换。
  pub fn start_watcher(&mut self, async_runtime: &mut AsyncRuntime) -> bool {
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
  pub fn request_rescan_for_language(
    &mut self,
    async_runtime: &AsyncRuntime,
    language_code: &str,
    missing_template: &str,
  ) -> bool {
    let Some(mut request) = self.last_scan.clone() else {
      return false;
    };
    request.language_code = language_code.to_string();
    request.missing_template = missing_template.to_string();
    self.last_scan = Some(request.clone());
    async_runtime.submit(EngineTask::Package(PackageTask::Scan(request)));
    true
  }

  /// 兼容旧调用；统一异步架构下扫描事件从 EngineEventQueue 输入。
  pub fn poll_events(&mut self, log: &mut LogService) -> Vec<PackageEvent> {
    let _ = log;
    Vec::new()
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

  pub fn mod_games(&self) -> Vec<PackageListEntry> {
    self
      .games()
      .into_iter()
      .filter(|info| info.source == PackageSource::Mod)
      .map(package_list_entry)
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

  pub fn total_count(&self) -> usize {
    self.snapshot.games.len() + self.snapshot.screensavers.len()
  }
}

pub(crate) fn run_package_task(
  task_id: TaskId,
  task: PackageTask,
  event_tx: &Sender<EngineEvent>,
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
      let report = scan_all_packages(
        &request,
        &mut |event| send_package_event(event_tx, event),
        total_candidates,
      );
      let finished = scan_finished_event(&report);
      let _ = event_tx.send(EngineEvent::Package(PackageAsyncEvent::SnapshotReady {
        snapshot: report.snapshot,
        finished,
        watched_files: report.watched_files,
      }));
      let _ = task_id;
      Ok(())
    }
  }
}

fn send_package_event(event_tx: &Sender<EngineEvent>, event: PackageEvent) {
  let _ = event_tx.send(EngineEvent::Package(PackageAsyncEvent::Event(event)));
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
    PackageEvent::Info(message) => log.info(LogSource::Pack, message),
    PackageEvent::Warn(message) => log.warn(LogSource::Pack, message),
    PackageEvent::ScanStarted { total } => log.info(
      LogSource::Pack,
      format!("Started package scan ({} candidates)", total),
    ),
    PackageEvent::ScanProgress { .. } => {}
    PackageEvent::WatchChanged { folders } => log.info(
      LogSource::Pack,
      format!(
        "Package manifest changed in {} folder(s), requesting rescan",
        folders
      ),
    ),
    PackageEvent::ScanFinished {
      total,
      games,
      screensavers,
      errors,
      duplicates,
    } => log.info(
      LogSource::Pack,
      format!(
        "Scanned {} packages ({} games, {} screensavers), {} errors, {} duplicates skipped",
        total, games, screensavers, errors, duplicates,
      ),
    ),
  }
}

fn run_package_watcher(
  request: ScanRequest,
  command_rx: Receiver<PackageWatcherCommand>,
  event_tx: Sender<EngineEvent>,
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
          PackageEvent::Warn(format!("Cannot start package watcher: {}", error)),
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
          PackageEvent::Warn(format!(
            "Package watch root does not exist: {}",
            root.display()
          )),
        );
      }
    }
    send_package_event(
      &event_tx,
      PackageEvent::Info(format!(
        "Package watcher started on {} root(s)",
        roots.len()
      )),
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
        let _ = event_tx.send(EngineEvent::Package(PackageAsyncEvent::WatchChanged {
          package_dirs,
        }));
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

fn sync_package_watch_dirs<'a>(
  watcher: &mut RecommendedWatcher,
  watched_dirs: &mut HashSet<PathBuf>,
  roots: &[PathBuf],
  files: impl Iterator<Item = &'a PathBuf>,
  event_tx: &Sender<EngineEvent>,
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

fn watch_package_dir(
  watcher: &mut RecommendedWatcher,
  watched_dirs: &mut HashSet<PathBuf>,
  dir: &Path,
  event_tx: &Sender<EngineEvent>,
) {
  let dir = dir.to_path_buf();
  if !watched_dirs.insert(dir.clone()) {
    return;
  }

  if let Err(error) = watcher.watch(&dir, RecursiveMode::NonRecursive) {
    watched_dirs.remove(&dir);
    send_package_event(
      event_tx,
      PackageEvent::Warn(format!(
        "Cannot watch package path {}: {}",
        dir.display(),
        error
      )),
    );
  }
}

fn queue_package_watch_event(
  watcher: &mut RecommendedWatcher,
  watched_dirs: &mut HashSet<PathBuf>,
  roots: &[PathBuf],
  watched_files: &HashMap<PathBuf, PathBuf>,
  event: Event,
  event_tx: &Sender<EngineEvent>,
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
    PackageAsset::Image { path } => Some(
      info
        .path
        .join("assets")
        .join(path)
        .to_string_lossy()
        .to_string(),
    ),
    _ => None,
  };
  let icon = icon_path
    .as_ref()
    .map(|path| PackageAsset::Image { path: path.clone() })
    .unwrap_or_else(|| info.display.icon.clone());
  let banner = match &info.display.banner {
    PackageAsset::Image { path } => PackageAsset::Image {
      path: info
        .path
        .join("assets")
        .join(path)
        .to_string_lossy()
        .to_string(),
    },
    PackageAsset::Text { .. } => info.display.banner.clone(),
  };
  let mouse_required = info.game.as_ref().is_some_and(|game| game.mouse)
    || info
      .screensaver
      .as_ref()
      .is_some_and(|screensaver| screensaver.mouse);
  let write_required = info.game.as_ref().is_some_and(|game| game.write);
  let key_actions = info
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

  PackageListEntry {
    mod_id: info.mod_id,
    source: info.source,
    package_type: info.package_type,
    key_actions,
    title: info.display.title,
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
    write_required,
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

    match read_package(&path, &dir_name, &expected_type, &source, request) {
      Ok(info) => {
        if has_mod_id(&report.snapshot, &info.mod_id) {
          report.events.push(PackageEvent::Warn(format!(
            "Duplicate mod_id '{}' in '{}', keeping first",
            info.mod_id, dir_name,
          )));
          report.duplicates += 1;
          continue;
        }
        report.watched_files.extend(info.watched_files.clone());
        insert(&mut report.snapshot, info);
      }
      Err(msg) => {
        report.events.push(PackageEvent::Warn(format!(
          "Skipping '{}/{}': {}",
          relative, dir_name, msg
        )));
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
) -> Result<PackageInfo, String> {
  let json_path = dir.join("package.json");
  let content =
    std::fs::read_to_string(&json_path).map_err(|e| format!("Cannot read package.json: {}", e))?;
  let mut watched_files = vec![json_path.clone()];

  let raw: RawPackageJson =
    serde_json::from_str(&content).map_err(|e| format!("Invalid JSON: {}", e))?;

  if raw.mod_id.trim().is_empty() {
    return Err("mod_id is empty".into());
  }

  if raw.schema_version != PACKAGE_MANIFEST_VERSION {
    return Err(format!(
      "schema_version {} != host {}",
      raw.schema_version, PACKAGE_MANIFEST_VERSION
    ));
  }

  let pkg_type = parse_package_type(&raw.package_type)?;
  if pkg_type != *expected_type {
    return Err(format!(
      "Type mismatch: manifest says {:?}, directory expects {:?}",
      pkg_type, expected_type
    ));
  }

  if raw.version_code == 0 {
    return Err("version_code must be > 0".into());
  }

  if raw.api.min > raw.api.max {
    return Err(format!(
      "api.min ({}) > api.max ({})",
      raw.api.min, raw.api.max
    ));
  }
  if raw.api.min > HOST_API_VERSION {
    return Err(format!(
      "api.min ({}) > host API ({})",
      raw.api.min, HOST_API_VERSION
    ));
  }
  if raw.api.max < HOST_API_VERSION {
    return Err(format!(
      "api.max ({}) < host API ({})",
      raw.api.max, HOST_API_VERSION
    ));
  }

  let entry = resolve_entry(dir, &raw.entry)?;

  let display = raw.display.unwrap_or_default();
  let title = resolve_package_text(dir, display.title, request, &mut watched_files);
  if title.trim().is_empty() {
    return Err("display.title is empty".into());
  }
  let version = raw
    .version
    .map(|value| resolve_package_text(dir, value, request, &mut watched_files))
    .unwrap_or_default();

  let runtime = raw.runtime.unwrap_or_default();

  let game = match pkg_type {
    PackageType::Game => {
      let g = raw.game.ok_or("Missing 'game' config for game type")?;
      if !VALID_TARGET_FPS.contains(&g.target_fps) {
        return Err(format!(
          "target_fps must be one of {:?}, got {}",
          VALID_TARGET_FPS, g.target_fps
        ));
      }
      let mut actions = HashMap::new();
      if let Some(raw_actions) = g.actions {
        for (name, a) in raw_actions {
          if a.keys.is_empty() {
            return Err(format!("action '{}' has empty keys", name));
          }
          actions.insert(
            name,
            ActionConfig {
              description: a
                .description
                .map(|value| resolve_package_text(dir, value, request, &mut watched_files))
                .unwrap_or_default(),
              keys: a.keys,
            },
          );
        }
      }
      Some(GameConfig {
        name: g
          .name
          .map(|value| resolve_package_text(dir, value, request, &mut watched_files))
          .unwrap_or_default(),
        detail: g
          .detail
          .map(|value| resolve_package_text(dir, value, request, &mut watched_files))
          .unwrap_or_default(),
        write: g.write.unwrap_or(false),
        mouse: g.mouse.unwrap_or(false),
        target_fps: g.target_fps,
        save: g.save.unwrap_or(false),
        score: g.score.map(|s| ScoreConfig {
          enabled: s.enabled,
          empty_text: s
            .empty_text
            .map(|value| resolve_package_text(dir, value, request, &mut watched_files))
            .unwrap_or_default(),
        }),
        actions,
      })
    }
    PackageType::Screensaver => None,
  };

  let screensaver = match pkg_type {
    PackageType::Screensaver => {
      let s = raw.screensaver.ok_or("Missing 'screensaver' config")?;
      Some(ScreensaverConfig {
        name: s
          .name
          .map(|value| resolve_package_text(dir, value, request, &mut watched_files))
          .unwrap_or_default(),
        mouse: s.mouse.unwrap_or(false),
      })
    }
    PackageType::Game => None,
  };

  Ok(PackageInfo {
    source: source.clone(),
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
      description: display
        .description
        .map(|value| resolve_package_text(dir, value, request, &mut watched_files))
        .unwrap_or_default(),
      author: display
        .author
        .map(|value| resolve_package_text(dir, value, request, &mut watched_files))
        .unwrap_or_default(),
      icon: parse_package_asset(dir, &display.icon, AssetShape::Icon, &mut watched_files)
        .unwrap_or_else(default_icon_asset),
      banner: parse_package_asset(dir, &display.banner, AssetShape::Banner, &mut watched_files)
        .unwrap_or_else(default_banner_asset),
    },
    runtime: PackageRuntime {
      min_width: runtime.min_width.unwrap_or(0),
      min_height: runtime.min_height.unwrap_or(0),
    },
    game,
    screensaver,
    path: dir.to_path_buf(),
    watched_files,
  })
}

fn insert(snapshot: &mut PackageSnapshot, info: PackageInfo) {
  match info.package_type {
    PackageType::Game => snapshot.games.push(info),
    PackageType::Screensaver => snapshot.screensavers.push(info),
  }
}

fn has_mod_id(snapshot: &PackageSnapshot, id: &str) -> bool {
  snapshot.games.iter().any(|p| p.mod_id == id)
    || snapshot.screensavers.iter().any(|p| p.mod_id == id)
}

#[derive(Deserialize)]
struct RawPackageJson {
  mod_id: String,
  schema_version: u32,
  #[serde(rename = "type")]
  package_type: String,
  version: Option<String>,
  version_code: u32,
  api: RawApiRange,
  entry: String,
  display: Option<RawDisplay>,
  runtime: Option<RawRuntime>,
  game: Option<RawGameConfig>,
  screensaver: Option<RawScreensaverConfig>,
}

#[derive(Deserialize)]
struct RawApiRange {
  min: u32,
  max: u32,
}

#[derive(Deserialize, Default)]
struct RawDisplay {
  title: String,
  description: Option<String>,
  author: Option<String>,
  #[serde(default)]
  icon: Option<RawDisplayAsset>,
  #[serde(default)]
  banner: Option<RawDisplayAsset>,
}

#[derive(Deserialize)]
struct RawDisplayAsset {
  #[serde(rename = "type")]
  asset_type: String,
  path: String,
}

#[derive(Deserialize, Default)]
struct RawRuntime {
  min_width: Option<u32>,
  min_height: Option<u32>,
}

#[derive(Deserialize)]
struct RawGameConfig {
  name: Option<String>,
  detail: Option<String>,
  write: Option<bool>,
  mouse: Option<bool>,
  target_fps: u32,
  save: Option<bool>,
  score: Option<RawScoreConfig>,
  actions: Option<HashMap<String, RawActionConfig>>,
}

#[derive(Deserialize)]
struct RawScoreConfig {
  enabled: bool,
  empty_text: Option<String>,
}

#[derive(Deserialize)]
struct RawActionConfig {
  description: Option<String>,
  keys: Vec<Vec<String>>,
}

#[derive(Deserialize)]
struct RawScreensaverConfig {
  name: Option<String>,
  mouse: Option<bool>,
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
) -> Option<PackageAsset> {
  let raw = raw.as_ref()?;
  let path = safe_asset_path(&raw.path)?;
  let asset_path = package_dir.join("assets").join(&path);
  match raw.asset_type.as_str() {
    "image" if is_supported_image_path(&path) => {
      watched_files.push(asset_path);
      Some(PackageAsset::Image { path })
    }
    "text" if is_supported_text_path(&path) => {
      watched_files.push(asset_path.clone());
      let content = std::fs::read_to_string(asset_path).ok()?;
      Some(PackageAsset::Text {
        path,
        lines: normalize_asset_text(&content, shape),
      })
    }
    _ => None,
  }
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
  if trimmed.is_empty() {
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
  value: String,
  request: &ScanRequest,
  watched_files: &mut Vec<PathBuf>,
) -> String {
  match package_text(value) {
    PackageText::Literal(text) => text,
    PackageText::I18n(key) => resolve_package_i18n(pkg_dir, &key, request, watched_files),
  }
}

fn package_text(value: String) -> PackageText {
  value
    .strip_prefix('@')
    .map(|key| PackageText::I18n(key.to_string()))
    .unwrap_or(PackageText::Literal(value))
}

fn resolve_package_i18n(
  pkg_dir: &Path,
  key: &str,
  request: &ScanRequest,
  watched_files: &mut Vec<PathBuf>,
) -> String {
  push_package_i18n_watch_path(pkg_dir, &request.language_code, key, watched_files);
  push_package_i18n_watch_path(pkg_dir, "en_us", key, watched_files);
  load_package_i18n_value(pkg_dir, &request.language_code, key)
    .or_else(|| load_package_i18n_value(pkg_dir, "en_us", key))
    .unwrap_or_else(|| {
      request
        .missing_template
        .replace("{value:missing_key}", &format!("@{key}"))
    })
}

fn push_package_i18n_watch_path(
  pkg_dir: &Path,
  language_code: &str,
  key: &str,
  watched_files: &mut Vec<PathBuf>,
) {
  for (path, _) in package_i18n_paths(pkg_dir, language_code, key) {
    watched_files.push(path);
  }
}

fn load_package_i18n_value(pkg_dir: &Path, language_code: &str, key: &str) -> Option<String> {
  package_i18n_paths(pkg_dir, language_code, key)
    .into_iter()
    .find_map(|(path, field)| {
      let content = std::fs::read_to_string(path).ok()?;
      serde_json::from_str::<HashMap<String, String>>(&content)
        .ok()?
        .get(&field)
        .cloned()
    })
}

fn package_i18n_paths(pkg_dir: &Path, language_code: &str, key: &str) -> Vec<(PathBuf, String)> {
  let parts: Vec<&str> = key.split('/').filter(|part| !part.is_empty()).collect();
  let language_root = pkg_dir.join("assets").join("language").join(language_code);
  match parts.as_slice() {
    [] => Vec::new(),
    [field] => vec![(language_root.with_extension("json"), (*field).to_string())],
    many => {
      let Some((last, dirs)) = many.split_last() else {
        return Vec::new();
      };
      let mut paths = Vec::new();

      if let Some((file, field)) = last.rsplit_once('.') {
        paths.push(package_i18n_file_path(&language_root, dirs, file, field));
      }

      if let Some((field, prefix)) = many.split_last() {
        if let Some((file, old_dirs)) = prefix.split_last() {
          let old_path = package_i18n_file_path(&language_root, old_dirs, file, field);
          if !paths.contains(&old_path) {
            paths.push(old_path);
          }
        }
      }

      paths
    }
  }
}

fn package_i18n_file_path(
  language_root: &Path,
  dirs: &[&str],
  file: &str,
  field: &str,
) -> (PathBuf, String) {
  let mut path = language_root.to_path_buf();
  for dir in dirs {
    path = path.join(dir);
  }
  (path.join(format!("{file}.json")), field.to_string())
}

// 规范化包入口脚本路径。扫描阶段只验证路径语义，不验证脚本文件是否存在。
fn resolve_entry(pkg_dir: &Path, entry: &str) -> Result<String, String> {
  let _ = pkg_dir;
  let trimmed = entry.trim();
  if trimmed.is_empty() {
    return Err("Entry is empty".to_string());
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

#[cfg(test)]
mod tests {
  use super::*;
  use crate::host_engine::services::async_runtime::{AsyncRuntime, EngineEvent};

  const MISSING: &str = "[Missing i18n Key: {value:missing_key}]";

  fn temp_root(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("tg_package_{name}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    root
  }

  fn poll_async_package_events(
    runtime: &AsyncRuntime,
    service: &mut PackageService,
    log: &mut LogService,
  ) -> Vec<PackageEvent> {
    runtime
      .poll_events()
      .into_iter()
      .filter_map(|event| match event {
        EngineEvent::Package(event) => Some(service.handle_async_event(event, log)),
        _ => None,
      })
      .collect()
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
          "display":{{"title":"{title}","author":"Tester"}},
          "game":{{"target_fps":60}}
        }}"#
      ),
    )
    .unwrap();
  }

  fn write_game_manifest_only(root: &Path, relative: &str, id: &str, title: &str) {
    let dir = root.join(relative).join(id);
    std::fs::create_dir_all(&dir).unwrap();
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
          "display":{{"title":"{title}","author":"Tester"}},
          "game":{{"target_fps":60}}
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
          "display":{{"title":"{title}","author":"Tester"}},
          "screensaver":{{"name":"{title}"}}
        }}"#
      ),
    )
    .unwrap();
  }

  fn scan(service: &mut PackageService, root: &Path, log: &mut LogService, language: &str) {
    service.scan_all(root, log, language, MISSING);
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
    let runtime = AsyncRuntime::with_worker_count(1);
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
    let runtime = AsyncRuntime::with_worker_count(1);
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
  fn package_list_entry_carries_game_action_keys() {
    let root = temp_root("entry_action_keys");
    let dir = root.join("data/mod/game/action_keys");
    std::fs::create_dir_all(&dir).unwrap();
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
        "display":{"title":"f%{key:move_up} Move","author":"Tester"},
        "game":{
          "target_fps":60,
          "actions":{
            "move_up":{"description":"Move up","keys":[["w"],["arrow_up"]]}
          }
        }
      }"#,
    )
    .unwrap();

    let mut service = PackageService::new();
    let mut log = LogService::new();
    scan(&mut service, &root, &mut log, "en_us");

    let entry = service.mod_games().remove(0);
    assert_eq!(
      entry.key_actions.get("move_up"),
      Some(&vec![vec!["w".to_string()], vec!["arrow_up".to_string()]])
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
        "version":"@meta/version",
        "version_code":1,
        "api":{"min":1,"max":1},
        "entry":"ui/init",
        "display":{
          "title":"@display/title",
          "description":"@deep/nested/text/description",
          "author":"@author"
        },
        "game":{
          "name":"@game.name",
          "detail":"@detail/main",
          "target_fps":60,
          "score":{"enabled":true,"empty_text":"@score.empty"},
          "actions":{"move_up":{"description":"@action/move.up","keys":[["w"]]}}
        }
      }"#,
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
      "",
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
        r#""author":"@creator/identity/author.name""#,
      )
      .replace(r#""version":"1.0.0""#, r#""version":"@meta/version.text""#)
      .replace(
        r#""title":"Dot Path Game""#,
        r#""title":"@long/path/display/title.text""#,
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
    write_game_manifest_only(&root, "data/mod/game", "fallback_game", "@display/title");
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
      .replace(r#""author":"Tester""#, r#""author":"@missing.author""#);
    std::fs::write(package_json, content).unwrap();

    let mut service = PackageService::new();
    let mut log = LogService::new();
    scan(&mut service, &root, &mut log, "zh_cn");

    let game = service.games().remove(0);
    assert_eq!(game.display.title, "English Title");
    assert_eq!(game.display.author, "[Missing i18n Key: @missing.author]");

    let _ = std::fs::remove_dir_all(root);
  }

  #[test]
  fn screensaver_i18n_name_is_resolved() {
    let root = temp_root("screensaver_i18n");
    write_screensaver(&root, "data/mod/screensaver", "screen_i18n", "Screen Title");
    let package_json = root.join("data/mod/screensaver/screen_i18n/package.json");
    let content = std::fs::read_to_string(&package_json)
      .unwrap()
      .replace(r#""name":"Screen Title""#, r#""name":"@screen.name""#);
    std::fs::write(package_json, content).unwrap();
    write_package_language(
      &root,
      "data/mod/screensaver",
      "screen_i18n",
      "zh_cn",
      "",
      r#"{"screen.name":"屏保名"}"#,
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
  fn screensaver_mouse_flag_reaches_list_entry() {
    let root = temp_root("screensaver_mouse");
    write_screensaver(
      &root,
      "data/mod/screensaver",
      "mouse_screen",
      "Mouse Screen",
    );
    let package_json = root.join("data/mod/screensaver/mouse_screen/package.json");
    let content = std::fs::read_to_string(&package_json).unwrap().replace(
      r#""screensaver":{"name":"Mouse Screen"}"#,
      r#""screensaver":{"name":"Mouse Screen","mouse":true}"#,
    );
    std::fs::write(package_json, content).unwrap();

    let mut service = PackageService::new();
    let mut log = LogService::new();
    scan(&mut service, &root, &mut log, "en_us");

    assert!(service.mod_screensavers()[0].mouse_required);

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
    write_game_manifest_only(&root, "data/mod/game", "watch_game", "@display/title");
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
    let content = std::fs::read_to_string(&package_json).unwrap().replace(
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
  fn missing_or_invalid_assets_fall_back_to_default_text_assets() {
    let root = temp_root("default_assets");
    write_game_manifest_only(
      &root,
      "data/mod/game",
      "default_asset_game",
      "Default Asset Game",
    );
    let dir = root.join("data/mod/game/default_asset_game");
    let package_json = dir.join("package.json");
    let content = std::fs::read_to_string(&package_json)
      .unwrap()
      .replace(
        r#""author":"Tester""#,
        r#""author":"Tester","icon":{"type":"image","path":"bad/icon.gif"},"banner":{"type":"text","path":"missing/banner.txt"}"#,
      );
    std::fs::write(package_json, content).unwrap();

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
  }
}
