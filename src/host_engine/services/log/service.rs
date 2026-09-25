use std::collections::{HashMap, HashSet, VecDeque};
use std::io;
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::host_engine::services::storage::atomic_write;
use crate::host_engine::services::PackageId;

use super::{
  HostLogMessage, LogEntry, LogLabels, LogLevel, LogPrintOptions, LogSource, format_file_log_entry,
  format_log_entry, format_print_log_entry,
};

/// 日志服务：以环形队列存储最近 N 条日志，支持按级别写入与导出。
pub struct LogService {
  queue: VecDeque<LogEntry>,
  next_sequence: u64,
  next_file_sequence: u64,
  max_entries: usize,
  output_path: Option<PathBuf>,
  package_scan_output_path: Option<PathBuf>,
  labels: LogLabels,
  last_file_error: Option<String>,
  next_session_id: u64,
  session_logs: HashMap<LogSessionId, SessionLog>,
  package_file_errors: HashSet<PathBuf>,
  message_templates: HashMap<String, String>,
  pending_messages: Vec<PendingHostLog>,
  run_id: String,
  write_enabled: bool,
}

const MAX_PACKAGE_LOG_BYTES: usize = 8 * 1024 * 1024;
const PACKAGE_LOG_TRUNCATED: &str = "[Log][WARN] Older package log entries were truncated.\n";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LogSessionId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogSessionKind {
  Game,
  Screensaver,
}

struct SessionLog {
  path: PathBuf,
  package_id: PackageId,
  kind: LogSessionKind,
}

struct PendingHostLog {
  timestamp_ms: u128,
  sequence: u64,
  level: LogLevel,
  source: LogSource,
  message: HostLogMessage,
}

impl LogService {
  pub fn new() -> Self {
    Self {
      queue: VecDeque::new(),
      next_sequence: 0,
      next_file_sequence: 0,
      max_entries: 1000,
      output_path: None,
      package_scan_output_path: None,
      labels: LogLabels::new(),
      last_file_error: None,
      next_session_id: 1,
      session_logs: HashMap::new(),
      package_file_errors: HashSet::new(),
      message_templates: HashMap::new(),
      pending_messages: Vec::new(),
      run_id: format!("{}-{}", std::process::id(), now_ms()),
      write_enabled: false,
    }
  }

  pub fn set_output_path(&mut self, path: PathBuf) -> io::Result<()> {
    self.output_path = Some(path);
    if self.write_enabled {
      self.flush_pending_to_file()
    } else {
      Ok(())
    }
  }

  /// 设置包管理器的集中扫描日志。该文件与游戏/屏保运行日志相互独立。
  pub fn set_package_scan_output_path(&mut self, path: PathBuf) -> io::Result<()> {
    if let Some(parent) = path.parent() {
      std::fs::create_dir_all(parent)?;
    }
    self.package_scan_output_path = Some(path);
    Ok(())
  }

  pub fn info_package_scan_message(&mut self, message: HostLogMessage) {
    self.push_package_scan_message(LogLevel::Info, message);
  }

  pub fn warn_package_scan_message(&mut self, message: HostLogMessage) {
    self.push_package_scan_message(LogLevel::Warn, message);
  }

  /// Applies translated labels and `log_info` message templates, then enables file output.
  pub fn refresh_labels(
    &mut self,
    translate: impl Fn(&'static str) -> Option<String>,
    message_templates: HashMap<String, String>,
  ) -> io::Result<()> {
    self.labels.refresh(translate);
    self.message_templates = message_templates;
    self.materialize_pending_messages();
    self.write_enabled = true;
    self.flush_pending_to_file()
  }

  /// Enables file output with the embedded English labels when i18n is unavailable.
  pub fn activate_embedded_english(&mut self) -> io::Result<()> {
    self.materialize_pending_messages();
    self.write_enabled = true;
    self.flush_pending_to_file()
  }

  pub fn run_id(&self) -> &str {
    &self.run_id
  }

  pub fn trace_message(&mut self, source: LogSource, message: HostLogMessage) {
    self.push_message(LogLevel::Trace, source, message);
  }

  pub fn debug_message(&mut self, source: LogSource, message: HostLogMessage) {
    self.push_message(LogLevel::Debug, source, message);
  }

  pub fn info_message(&mut self, source: LogSource, message: HostLogMessage) {
    self.push_message(LogLevel::Info, source, message);
  }

  pub fn warn_message(&mut self, source: LogSource, message: HostLogMessage) {
    self.push_message(LogLevel::Warn, source, message);
  }

  pub fn error_message(&mut self, source: LogSource, message: HostLogMessage) {
    self.push_message(LogLevel::Error, source, message);
  }

  pub fn fatal_message(&mut self, source: LogSource, message: HostLogMessage) {
    self.push_message(LogLevel::Fatal, source, message);
  }

  pub fn warn_operation_failed(
    &mut self,
    source: LogSource,
    operation: impl Into<String>,
    target: impl Into<String>,
    error: impl Into<String>,
  ) {
    self.warn_message(
      source,
      HostLogMessage::new(
        "log_info.operation.failed",
        "Host operation {operation} failed for {target}: {error}",
      )
      .param("operation", operation.into())
      .param("target", target.into())
      .param("error", error.into()),
    );
  }

  pub fn error_operation_failed(
    &mut self,
    source: LogSource,
    operation: impl Into<String>,
    target: impl Into<String>,
    error: impl Into<String>,
  ) {
    self.error_message(
      source,
      HostLogMessage::new(
        "log_info.operation.failed",
        "Host operation {operation} failed for {target}: {error}",
      )
      .param("operation", operation.into())
      .param("target", target.into())
      .param("error", error.into()),
    );
  }

  pub fn open_session(
    &mut self,
    kind: LogSessionKind,
    package_id: &PackageId,
  ) -> io::Result<LogSessionId> {
    let path = self.package_log_path(package_id)?;
    let id = LogSessionId(self.next_session_id);
    self.next_session_id = self.next_session_id.saturating_add(1).max(1);
    let kind_name = match kind {
      LogSessionKind::Game => "game",
      LogSessionKind::Screensaver => "screensaver",
    };
    if let Err(error) = append_capped(
      &path,
      &format!(
        "[Session][id={:06}][kind={kind_name}][package={package_id}][started={}]\n",
        id.0,
        now_ms(),
      ),
      MAX_PACKAGE_LOG_BYTES,
    ) {
      self.record_package_file_error(path, io::Error::new(error.kind(), error.to_string()));
      return Err(error);
    }
    self.package_file_errors.remove(&path);
    self.session_logs.insert(
      id,
      SessionLog {
        path,
        package_id: package_id.clone(),
        kind,
      },
    );
    self.info_message(
      LogSource::Runtime,
      HostLogMessage::new(
        "log_info.session.started",
        "{kind} session started for {package}.",
      )
      .param("kind", kind_name)
      .param("package", package_id.storage_key()),
    );
    Ok(id)
  }

  pub fn close_session(&mut self, id: LogSessionId) {
    if let Some(session) = self.session_logs.remove(&id) {
      let kind_name = match session.kind {
        LogSessionKind::Game => "game",
        LogSessionKind::Screensaver => "screensaver",
      };
      self.write_package_entry(
        &session.path,
        &format!(
          "[Session][id={:06}][package={}][ended={}]\n",
          id.0,
          session.package_id,
          now_ms(),
        ),
      );
      self.info_message(
        LogSource::Runtime,
        HostLogMessage::new(
          "log_info.session.stopped",
          "{kind} session stopped for {package}.",
        )
        .param("kind", kind_name)
        .param("package", session.package_id.storage_key()),
      );
    }
  }

  pub fn package_log_path(&self, package_id: &PackageId) -> io::Result<PathBuf> {
    let directory = self
      .output_path
      .as_deref()
      .and_then(std::path::Path::parent)
      .ok_or_else(|| io::Error::other("main log output path is not configured"))?;
    Ok(
      directory
        .join(package_id.package_type.as_str())
        .join(format!(
          "{}_{}.log",
          package_id.source.as_str(),
          package_id.mod_id
        )),
    )
  }

  pub fn error_package(
    &mut self,
    package_id: &PackageId,
    source: LogSource,
    message: impl Into<String>,
  ) {
    self.push_package(package_id, LogLevel::Error, source, message);
  }

  pub fn warn_package(
    &mut self,
    package_id: &PackageId,
    source: LogSource,
    message: impl Into<String>,
  ) {
    self.push_package(package_id, LogLevel::Warn, source, message);
  }

  pub fn warn_package_message(
    &mut self,
    package_id: &PackageId,
    source: LogSource,
    message: HostLogMessage,
  ) {
    let rendered = message.render(self.message_templates.get(message.key).map(String::as_str));
    self.push_package(package_id, LogLevel::Warn, source, rendered);
  }

  pub fn info_package(
    &mut self,
    package_id: &PackageId,
    source: LogSource,
    message: impl Into<String>,
  ) {
    self.push_package(package_id, LogLevel::Info, source, message);
  }

  pub fn debug_package(
    &mut self,
    package_id: &PackageId,
    source: LogSource,
    message: impl Into<String>,
  ) {
    self.push_package(package_id, LogLevel::Debug, source, message);
  }

  pub fn trace_session(&mut self, id: LogSessionId, source: LogSource, message: impl Into<String>) {
    self.push_session(id, LogLevel::Trace, source, message);
  }

  pub fn debug_session(&mut self, id: LogSessionId, source: LogSource, message: impl Into<String>) {
    self.push_session(id, LogLevel::Debug, source, message);
  }

  pub fn info_session(&mut self, id: LogSessionId, source: LogSource, message: impl Into<String>) {
    self.push_session(id, LogLevel::Info, source, message);
  }

  pub fn warn_session(&mut self, id: LogSessionId, source: LogSource, message: impl Into<String>) {
    self.push_session(id, LogLevel::Warn, source, message);
  }

  pub fn error_session(&mut self, id: LogSessionId, source: LogSource, message: impl Into<String>) {
    self.push_session(id, LogLevel::Error, source, message);
  }

  pub fn print_session(
    &mut self,
    id: LogSessionId,
    source: LogSource,
    message: impl Into<String>,
    options: LogPrintOptions,
  ) {
    let entry = self.make_entry(
      options.level.unwrap_or(LogLevel::Info),
      source,
      message.into(),
    );
    if let Some(session) = self.session_logs.get(&id) {
      let text = format_print_log_entry(&entry, &self.labels, options);
      let path = session.path.clone();
      self.write_package_entry(&path, &text);
    }
  }

  pub fn print_package(
    &mut self,
    package_id: &PackageId,
    source: LogSource,
    message: impl Into<String>,
    options: LogPrintOptions,
  ) {
    let entry = self.make_entry(
      options.level.unwrap_or(LogLevel::Info),
      source,
      message.into(),
    );
    let text = format_print_log_entry(&entry, &self.labels, options);
    match self.package_log_path(package_id) {
      Ok(path) => self.write_package_entry(&path, &text),
      Err(error) => self.record_package_file_error(PathBuf::from(package_id.storage_key()), error),
    }
  }

  fn push_session(
    &mut self,
    id: LogSessionId,
    level: LogLevel,
    source: LogSource,
    message: impl Into<String>,
  ) {
    let entry = self.make_entry(level, source, message.into());
    if let Some(session) = self.session_logs.get(&id) {
      let text = format_file_log_entry(&entry, &self.labels);
      let path = session.path.clone();
      self.write_package_entry(&path, &text);
    }
  }

  fn push_package(
    &mut self,
    package_id: &PackageId,
    level: LogLevel,
    source: LogSource,
    message: impl Into<String>,
  ) {
    let entry = self.make_entry(level, source, message.into());
    let text = format_file_log_entry(&entry, &self.labels);
    match self.package_log_path(package_id) {
      Ok(path) => self.write_package_entry(&path, &text),
      Err(error) => self.record_package_file_error(PathBuf::from(package_id.storage_key()), error),
    }
  }

  fn write_package_entry(&mut self, path: &std::path::Path, text: &str) {
    match append_capped(path, text, MAX_PACKAGE_LOG_BYTES) {
      Ok(()) => {
        self.package_file_errors.remove(path);
      }
      Err(error) => {
        self.record_package_file_error(path.to_path_buf(), error);
      }
    }
  }

  fn push_package_scan_message(&mut self, level: LogLevel, message: HostLogMessage) {
    let rendered = message.render(self.message_templates.get(message.key).map(String::as_str));
    let entry = self.make_entry(level, LogSource::Pack, rendered);
    let text = format_file_log_entry(&entry, &self.labels);
    let Some(path) = self.package_scan_output_path.clone() else {
      return;
    };
    self.write_package_entry(&path, &text);
  }

  fn record_package_file_error(&mut self, path: PathBuf, error: io::Error) {
    if self.package_file_errors.insert(path.clone()) {
      self.error_operation_failed(
        LogSource::Storage,
        "write_package_log",
        path.display().to_string(),
        error.to_string(),
      );
    }
  }

  fn push(&mut self, level: LogLevel, source: LogSource, message: impl Into<String>) {
    let entry = self.make_entry(level, source, message.into());
    self.store_entry(entry);
  }

  fn push_message(&mut self, level: LogLevel, source: LogSource, message: HostLogMessage) {
    if !self.write_enabled {
      self.pending_messages.push(PendingHostLog {
        timestamp_ms: now_ms(),
        sequence: self.next_sequence,
        level,
        source,
        message,
      });
      self.next_sequence = self.next_sequence.saturating_add(1);
      return;
    }
    let rendered = message.render(self.message_templates.get(message.key).map(String::as_str));
    self.push(level, source, rendered);
  }

  fn materialize_pending_messages(&mut self) {
    for pending in self.pending_messages.drain(..) {
      let rendered = pending.message.render(
        self
          .message_templates
          .get(pending.message.key)
          .map(String::as_str),
      );
      self.queue.push_back(LogEntry {
        timestamp_ms: pending.timestamp_ms,
        sequence: pending.sequence,
        level: pending.level,
        source: pending.source,
        message: rendered,
      });
    }
    self
      .queue
      .make_contiguous()
      .sort_by_key(|entry| entry.sequence);
    while self.queue.len() > self.max_entries {
      self.queue.pop_front();
    }
  }

  fn make_entry(&mut self, level: LogLevel, source: LogSource, message: String) -> LogEntry {
    let entry = LogEntry {
      timestamp_ms: now_ms(),
      sequence: self.next_sequence,
      level,
      source,
      message,
    };
    self.next_sequence = self.next_sequence.saturating_add(1);
    entry
  }

  fn store_entry(&mut self, entry: LogEntry) {
    self.queue.push_back(entry);
    while self.queue.len() > self.max_entries {
      self.queue.pop_front();
    }

    if self.write_enabled {
      let _ = self.flush_pending_to_file();
    }
  }

  pub fn entries(&self) -> &VecDeque<LogEntry> {
    &self.queue
  }

  /// 取出队列中所有日志并清空。
  pub fn drain(&mut self) -> Vec<LogEntry> {
    self.queue.drain(..).collect()
  }
  pub fn is_empty(&self) -> bool {
    self.queue.is_empty()
  }

  /// 设置最大存储条数（至少为 1），超出时截断旧条目。
  pub fn set_max_entries(&mut self, max_entries: usize) {
    self.max_entries = max_entries.max(1);

    while self.queue.len() > self.max_entries {
      self.queue.pop_front();
    }
  }

  /// 将当前所有日志输出到控制台（stdout）。
  pub fn flush_to_console(&self) {
    for entry in &self.queue {
      let line = format_log_entry(entry);
      if writeln!(std::io::stdout(), "{}", line).is_err() {
        break; // broken pipe, stop
      }
    }
  }

  pub fn flush_pending_to_file(&mut self) -> io::Result<()> {
    if !self.write_enabled {
      return Ok(());
    }
    let Some(path) = self.output_path.as_ref() else {
      return Ok(());
    };

    let text = self
      .queue
      .iter()
      .filter(|entry| entry.sequence >= self.next_file_sequence)
      .map(|entry| format_file_log_entry(entry, &self.labels))
      .collect::<String>();

    if text.is_empty() {
      return Ok(());
    }

    match append_text(path, &text) {
      Ok(()) => {
        self.next_file_sequence = self
          .queue
          .back()
          .map(|entry| entry.sequence.saturating_add(1))
          .unwrap_or(self.next_file_sequence);
        self.last_file_error = None;
        Ok(())
      }
      Err(error) => {
        self.last_file_error = Some(error.to_string());
        Err(error)
      }
    }
  }

  pub fn last_file_error(&self) -> Option<&str> {
    self.last_file_error.as_deref()
  }
}

fn append_text(path: &std::path::Path, text: &str) -> io::Result<()> {
  if let Some(parent) = path.parent() {
    std::fs::create_dir_all(parent)?;
  }

  std::fs::OpenOptions::new()
    .create(true)
    .append(true)
    .open(path)?
    .write_all(text.as_bytes())
}

fn append_capped(path: &std::path::Path, text: &str, max_bytes: usize) -> io::Result<()> {
  if let Some(parent) = path.parent() {
    std::fs::create_dir_all(parent)?;
  }
  let current_len = std::fs::metadata(path)
    .map(|metadata| metadata.len() as usize)
    .unwrap_or_default();
  if current_len.saturating_add(text.len()) <= max_bytes {
    return append_text(path, text);
  }

  let current = std::fs::read(path).unwrap_or_default();
  let marker = PACKAGE_LOG_TRUNCATED.as_bytes();
  let available = max_bytes.saturating_sub(marker.len());
  let mut combined = Vec::with_capacity(current.len().saturating_add(text.len()).min(available));
  combined.extend_from_slice(&current);
  combined.extend_from_slice(text.as_bytes());
  let start = combined.len().saturating_sub(available);
  let start = combined[start..]
    .iter()
    .position(|byte| *byte == b'\n')
    .map(|offset| start + offset + 1)
    .unwrap_or(start);
  let mut output = Vec::with_capacity(marker.len() + combined.len().saturating_sub(start));
  output.extend_from_slice(marker);
  output.extend_from_slice(&combined[start..]);
  atomic_write(path, &output, false)
}

impl Default for LogService {
  fn default() -> Self {
    Self::new()
  }
}

// 获取当前 Unix 毫秒时间戳，失败时回退为 0。
fn now_ms() -> u128 {
  SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map(|duration| duration.as_millis())
    .unwrap_or(0)
}

#[cfg(test)]
mod tests {
  use std::fs;

  use super::*;

  #[test]
  fn log_service_appends_structured_entries_to_file() {
    let path = std::env::temp_dir().join(format!(
      "tui_game_log_service_{}_{}.log",
      std::process::id(),
      now_ms()
    ));
    let mut log = LogService::new();

    log.set_output_path(path.clone()).unwrap();
    log.activate_embedded_english().unwrap();
    log.push(LogLevel::Info, LogSource::Storage, "storage ready");

    let text = fs::read_to_string(&path).unwrap();
    assert!(text.contains("[Runtime][Storage]"));
    assert!(text.contains("[INFO] storage ready"));

    let _ = fs::remove_file(path);
  }

  #[test]
  fn typed_boot_messages_are_translated_before_first_write() {
    let path = std::env::temp_dir().join(format!(
      "tui_game_log_boot_cache_{}_{}.log",
      std::process::id(),
      now_ms()
    ));
    let mut log = LogService::new();
    log.set_output_path(path.clone()).unwrap();
    log.info_message(
      LogSource::Boot,
      HostLogMessage::new("log_info.boot.stage_complete", "Ready: {name}").param("name", "engine"),
    );
    assert!(!path.exists());

    log
      .message_templates
      .insert("log_info.boot.stage_complete".into(), "就绪：{name}".into());
    log.activate_embedded_english().unwrap();
    let text = fs::read_to_string(&path).unwrap();
    assert!(text.contains("就绪：engine"));
    assert!(!text.contains("Ready: engine"));

    let _ = fs::remove_file(path);
  }

  #[test]
  fn game_and_screensaver_packages_use_stable_separate_log_files() {
    let directory = std::env::temp_dir().join(format!(
      "tui_game_session_logs_{}_{}",
      std::process::id(),
      now_ms()
    ));
    fs::create_dir_all(&directory).unwrap();
    let mut log = LogService::new();
    log.set_output_path(directory.join("tui_log.log")).unwrap();
    log.activate_embedded_english().unwrap();

    let game_id = PackageId::new(
      crate::host_engine::services::PackageSource::Mod,
      crate::host_engine::services::PackageType::Game,
      "sample.game",
    )
    .unwrap();
    let screensaver_id = PackageId::new(
      crate::host_engine::services::PackageSource::Official,
      crate::host_engine::services::PackageType::Screensaver,
      "sample.screensaver",
    )
    .unwrap();
    let official_game_id = PackageId::new(
      crate::host_engine::services::PackageSource::Official,
      crate::host_engine::services::PackageType::Game,
      "sample.game",
    )
    .unwrap();
    let game = log.open_session(LogSessionKind::Game, &game_id).unwrap();
    let game_again = log.open_session(LogSessionKind::Game, &game_id).unwrap();
    let screensaver = log
      .open_session(LogSessionKind::Screensaver, &screensaver_id)
      .unwrap();
    assert_ne!(game, screensaver);
    log.info_session(game, LogSource::Lua, "game initialized");
    log.warn_session(screensaver, LogSource::Lua, "screensaver warning");
    log.close_session(game);
    log.close_session(game_again);
    log.close_session(screensaver);

    let game_file = log.package_log_path(&game_id).unwrap();
    let official_game_file = log.package_log_path(&official_game_id).unwrap();
    let screensaver_file = log.package_log_path(&screensaver_id).unwrap();
    assert_ne!(game_file, official_game_file);
    assert!(
      fs::read_to_string(&game_file)
        .unwrap()
        .contains("game initialized")
    );
    assert!(
      fs::read_to_string(&screensaver_file)
        .unwrap()
        .contains("screensaver warning")
    );
    assert!(
      fs::read_to_string(directory.join("tui_log.log"))
        .map(|text| !text.contains("game initialized"))
        .unwrap_or(true)
    );
    let game_dir = game_file.parent().unwrap();
    assert_eq!(fs::read_dir(game_dir).unwrap().count(), 1);
    assert_eq!(game_dir, directory.join("game"));
    assert_eq!(
      screensaver_file.parent().unwrap(),
      directory.join("screensaver")
    );

    let _ = fs::remove_dir_all(directory);
  }

  #[test]
  fn capped_package_log_keeps_complete_recent_lines() {
    let directory = std::env::temp_dir().join(format!(
      "tui_game_capped_log_{}_{}",
      std::process::id(),
      now_ms()
    ));
    let path = directory.join("sample.log");
    append_capped(
      &path,
      "old-one-xxxxxxxxxxxx\nold-two-xxxxxxxxxxxx\nold-three-xxxxxxxxxxxx\n",
      96,
    )
    .unwrap();
    append_capped(
      &path,
      "recent-one-xxxxxxxx\nrecent-two-xxxxxxxx\nrecent-three\n",
      96,
    )
    .unwrap();
    let bytes = fs::read(&path).unwrap();
    let text = String::from_utf8(bytes.clone()).unwrap();
    assert!(bytes.len() <= 96);
    assert!(text.starts_with(PACKAGE_LOG_TRUNCATED));
    assert!(text.ends_with("recent-three\n"));
    let _ = fs::remove_dir_all(directory);
  }
}
