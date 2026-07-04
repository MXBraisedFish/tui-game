use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};

use crossbeam_channel::Sender;
use serde::Deserialize;

use crate::host_engine::services::{
  async_runtime::{AsyncRuntime, EngineEvent, EngineTask, TaskId},
  log::{LogService, LogSource},
};

/// 宿主机 API 版本号
pub const HOST_API_VERSION: u32 = 1;

/// 包清单 schema 版本号
pub const HOST_SCHEMA_VERSION: u32 = 1;
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
}

/// 面向 UI 列表的轻量包条目快照。
#[derive(Clone, Debug)]
pub struct PackageListEntry {
  pub mod_id: String,
  pub source: PackageSource,
  pub package_type: PackageType,
  pub title: String,
  pub author: String,
  pub version: String,
  pub icon_path: Option<String>,
  pub path: PathBuf,
  pub enabled: bool,
  pub debug: bool,
  pub safe_mode: bool,
}

/// 包显示信息
#[derive(Clone, Debug)]
pub struct PackageDisplay {
  pub title: String,
  pub description: String,
  pub author: String,
  pub icon: IconOrImage,
  pub banner: IconOrImage,
}

/// 图标或图片资源
#[derive(Clone, Debug)]
pub enum IconOrImage {
  Image(String),
  ArtArray,
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
  SnapshotReady {
    snapshot: PackageSnapshot,
    finished: PackageEvent,
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
}

/// 包管理服务，负责扫描和加载游戏/屏保包。
pub struct PackageService {
  snapshot: PackageSnapshot,
  last_scan: Option<ScanRequest>,
}

impl PackageService {
  pub fn new() -> Self {
    Self {
      snapshot: PackageSnapshot::default(),
      last_scan: None,
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
      PackageAsyncEvent::SnapshotReady { snapshot, finished } => {
        self.snapshot = snapshot;
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

fn package_list_entry(info: PackageInfo) -> PackageListEntry {
  PackageListEntry {
    mod_id: info.mod_id,
    source: info.source,
    package_type: info.package_type,
    title: info.display.title,
    author: info.display.author,
    version: info.version,
    icon_path: match info.display.icon {
      IconOrImage::Image(path) => Some(info.path.join(path).to_string_lossy().to_string()),
      IconOrImage::ArtArray => None,
    },
    path: info.path,
    enabled: true,
    debug: false,
    safe_mode: true,
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

  let raw: RawPackageJson =
    serde_json::from_str(&content).map_err(|e| format!("Invalid JSON: {}", e))?;

  if raw.mod_id.trim().is_empty() {
    return Err("mod_id is empty".into());
  }

  if raw.schema_version != HOST_SCHEMA_VERSION {
    return Err(format!(
      "schema_version {} != host {}",
      raw.schema_version, HOST_SCHEMA_VERSION
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
  let title = resolve_package_text(dir, display.title, request);
  if title.trim().is_empty() {
    return Err("display.title is empty".into());
  }
  let version = raw
    .version
    .map(|value| resolve_package_text(dir, value, request))
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
                .map(|value| resolve_package_text(dir, value, request))
                .unwrap_or_default(),
              keys: a.keys,
            },
          );
        }
      }
      Some(GameConfig {
        name: g
          .name
          .map(|value| resolve_package_text(dir, value, request))
          .unwrap_or_default(),
        detail: g
          .detail
          .map(|value| resolve_package_text(dir, value, request))
          .unwrap_or_default(),
        write: g.write.unwrap_or(false),
        mouse: g.mouse.unwrap_or(false),
        target_fps: g.target_fps,
        save: g.save.unwrap_or(false),
        score: g.score.map(|s| ScoreConfig {
          enabled: s.enabled,
          empty_text: s
            .empty_text
            .map(|value| resolve_package_text(dir, value, request))
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
          .map(|value| resolve_package_text(dir, value, request))
          .unwrap_or_default(),
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
        .map(|value| resolve_package_text(dir, value, request))
        .unwrap_or_default(),
      author: display
        .author
        .map(|value| resolve_package_text(dir, value, request))
        .unwrap_or_default(),
      icon: parse_icon_or_image(&display.icon),
      banner: parse_icon_or_image(&display.banner),
    },
    runtime: PackageRuntime {
      min_width: runtime.min_width.unwrap_or(0),
      min_height: runtime.min_height.unwrap_or(0),
    },
    game,
    screensaver,
    path: dir.to_path_buf(),
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
  icon: Option<String>,
  #[serde(default)]
  banner: Option<String>,
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
}

fn parse_package_type(s: &str) -> Result<PackageType, String> {
  match s {
    "game" => Ok(PackageType::Game),
    "screensaver" => Ok(PackageType::Screensaver),
    other => Err(format!("Unknown package type: '{}'", other)),
  }
}

fn parse_icon_or_image(raw: &Option<String>) -> IconOrImage {
  match raw {
    Some(s) if s.starts_with("image:") => {
      IconOrImage::Image(s.strip_prefix("image:").unwrap().to_string())
    }
    Some(_) => IconOrImage::ArtArray,
    None => IconOrImage::ArtArray,
  }
}

fn resolve_package_text(pkg_dir: &Path, value: String, request: &ScanRequest) -> String {
  match package_text(value) {
    PackageText::Literal(text) => text,
    PackageText::I18n(key) => resolve_package_i18n(pkg_dir, &key, request),
  }
}

fn package_text(value: String) -> PackageText {
  value
    .strip_prefix('@')
    .map(|key| PackageText::I18n(key.to_string()))
    .unwrap_or(PackageText::Literal(value))
}

fn resolve_package_i18n(pkg_dir: &Path, key: &str, request: &ScanRequest) -> String {
  load_package_i18n_value(pkg_dir, &request.language_code, key)
    .or_else(|| load_package_i18n_value(pkg_dir, "en_us", key))
    .unwrap_or_else(|| {
      request
        .missing_template
        .replace("{value:missing_key}", &format!("@{key}"))
    })
}

fn load_package_i18n_value(pkg_dir: &Path, language_code: &str, key: &str) -> Option<String> {
  let (path, field) = package_i18n_path(pkg_dir, language_code, key)?;
  let content = std::fs::read_to_string(path).ok()?;
  serde_json::from_str::<HashMap<String, String>>(&content)
    .ok()?
    .get(&field)
    .cloned()
}

fn package_i18n_path(pkg_dir: &Path, language_code: &str, key: &str) -> Option<(PathBuf, String)> {
  let parts: Vec<&str> = key.split('/').filter(|part| !part.is_empty()).collect();
  let language_root = pkg_dir.join("assets").join("language").join(language_code);
  match parts.as_slice() {
    [] => None,
    [field] => Some((language_root.with_extension("json"), (*field).to_string())),
    many => {
      let (field, prefix) = many.split_last()?;
      let (file, dirs) = prefix.split_last()?;
      let mut path = language_root;
      for dir in dirs {
        path = path.join(dir);
      }
      Some((path.join(format!("{file}.json")), (*field).to_string()))
    }
  }
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
