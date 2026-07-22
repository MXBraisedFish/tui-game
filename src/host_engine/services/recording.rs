use std::{
  fs,
  io::Read,
  path::{Path, PathBuf},
  time::{Duration, Instant},
};

use chrono::{Local, SecondsFormat};
use crossbeam_channel::Sender;
use serde::{Deserialize, Serialize};

use crate::host_engine::services::{
  AsyncRuntime, CanvasCell, ComposedCell, ComposedFrame, EngineEvent, EngineTask, StorageService,
  TaskId, TerminalColor, TextColor,
};

const RECORDING_SCHEMA_VERSION: u32 = 2;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RecordingState {
  #[default]
  Stopped,
  Recording,
  Paused,
  Finalizing,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RecordingSnapshot {
  pub state: RecordingState,
  pub active_duration: Duration,
  pub wall_duration: Duration,
  pub paused_duration: Duration,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordingPlaybackMetadata {
  pub started_at: String,
  pub frame_rate: Option<u16>,
  pub max_width: u16,
  pub max_height: u16,
  pub duration_us: u64,
}

#[derive(Clone, Debug)]
pub struct RecordingPlayback {
  metadata: RecordingPlaybackMetadata,
  palette: Vec<ComposedCell>,
  initial: PlaybackInitialFrame,
  events: Vec<PlaybackFrameEvent>,
}

#[derive(Clone, Debug, Deserialize)]
struct PlaybackDocument {
  schema_version: u32,
  started_at: String,
  frame_rate: Option<u16>,
  canvas: PlaybackCanvas,
  duration_us: PlaybackDurations,
  palette: Vec<PlaybackCell>,
  initial: PlaybackInitialFrame,
  events: Vec<PlaybackFrameEvent>,
}

#[derive(Clone, Debug, Deserialize)]
struct PlaybackHeader {
  schema_version: u32,
  started_at: String,
  frame_rate: Option<u16>,
  canvas: PlaybackCanvas,
  duration_us: PlaybackDurations,
}

#[derive(Clone, Copy, Debug, Deserialize)]
struct PlaybackCanvas {
  max_width: u16,
  max_height: u16,
}

#[derive(Clone, Copy, Debug, Deserialize)]
struct PlaybackDurations {
  active: u64,
}

#[derive(Clone, Debug, Deserialize)]
struct PlaybackInitialFrame {
  width: u16,
  height: u16,
  rows: Vec<Vec<(u32, u32)>>,
}

#[derive(Clone, Debug, Deserialize)]
struct PlaybackFrameEvent {
  time_us: u64,
  size: [u16; 2],
  changes: Vec<(u16, u16, Vec<u32>)>,
}

#[derive(Clone, Debug, Deserialize)]
struct PlaybackCell {
  text: String,
  foreground: Option<PlaybackColor>,
  background: Option<PlaybackColor>,
  #[serde(default)]
  flags: u16,
  #[serde(default)]
  continuation: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
enum PlaybackColor {
  Terminal(String),
  Rgb([u8; 3]),
  ForceRgb([u8; 3]),
  Transparent,
}

#[derive(Clone, Debug)]
pub struct RecordingTask {
  document: RecordingDocument,
  path: PathBuf,
}

#[derive(Clone, Debug)]
pub enum RecordingAsyncEvent {
  Saved { task_id: TaskId, path: PathBuf },
  Failed { task_id: TaskId, error: String },
}

pub struct RecordingService {
  state: RecordingState,
  session: Option<RecordingSession>,
  finalizing_task: Option<TaskId>,
  last_snapshot: RecordingSnapshot,
  last_presented_frame: Option<ComposedFrame>,
}

struct RecordingSession {
  started_at: String,
  frame_rate: Option<u16>,
  started_instant: Instant,
  active_before_run: Duration,
  run_started: Instant,
  paused_duration: Duration,
  pause_started: Option<Instant>,
  path: PathBuf,
  last_frame: ComposedFrame,
  max_width: u16,
  max_height: u16,
  palette: Vec<RecordedCell>,
  initial: RecordedInitialFrame,
  events: Vec<RecordedFrameEvent>,
}

#[derive(Clone, Debug, Serialize)]
struct RecordingDocument {
  schema_version: u32,
  started_at: String,
  finished_at: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  frame_rate: Option<u16>,
  canvas: RecordedCanvas,
  duration_us: RecordedDurations,
  palette: Vec<RecordedCell>,
  initial: RecordedInitialFrame,
  events: Vec<RecordedFrameEvent>,
}

#[derive(Clone, Debug, Serialize)]
struct RecordedCanvas {
  max_width: u16,
  max_height: u16,
}

#[derive(Clone, Debug, Serialize)]
struct RecordedDurations {
  active: u64,
  paused: u64,
  wall: u64,
}

#[derive(Clone, Debug, Serialize)]
struct RecordedInitialFrame {
  width: u16,
  height: u16,
  rows: Vec<Vec<(u32, u32)>>,
}

#[derive(Clone, Debug, Serialize)]
struct RecordedFrameEvent {
  time_us: u64,
  size: [u16; 2],
  changes: Vec<(u16, u16, Vec<u32>)>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct RecordedCell {
  text: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  foreground: Option<RecordedColor>,
  #[serde(skip_serializing_if = "Option::is_none")]
  background: Option<RecordedColor>,
  #[serde(skip_serializing_if = "is_zero")]
  flags: u16,
  #[serde(skip_serializing_if = "is_false")]
  continuation: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
enum RecordedColor {
  Terminal(&'static str),
  Rgb([u8; 3]),
  ForceRgb([u8; 3]),
  Transparent,
}

fn is_zero(value: &u16) -> bool {
  *value == 0
}

fn is_false(value: &bool) -> bool {
  !*value
}

impl RecordingService {
  pub fn new() -> Self {
    Self {
      state: RecordingState::Stopped,
      session: None,
      finalizing_task: None,
      last_snapshot: RecordingSnapshot::default(),
      last_presented_frame: None,
    }
  }

  pub fn state(&self) -> RecordingState {
    self.state
  }

  pub fn snapshot(&self) -> RecordingSnapshot {
    let Some(session) = &self.session else {
      return RecordingSnapshot {
        state: self.state,
        ..self.last_snapshot
      };
    };
    let now = Instant::now();
    let active = session.active_duration(now, self.state);
    let paused = session.paused_duration(now, self.state);
    RecordingSnapshot {
      state: self.state,
      active_duration: active,
      wall_duration: now.saturating_duration_since(session.started_instant),
      paused_duration: paused,
    }
  }

  pub fn is_recording(&self) -> bool {
    self.state == RecordingState::Recording
  }

  pub fn is_paused(&self) -> bool {
    self.state == RecordingState::Paused
  }

  pub fn start(
    &mut self,
    initial: ComposedFrame,
    frame_rate: Option<u16>,
    storage: &StorageService,
  ) -> bool {
    if self.state != RecordingState::Stopped || initial.width() == 0 || initial.height() == 0 {
      return false;
    }
    let now = Instant::now();
    let started_at = Local::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    let filename = Local::now().format("%Y%m%d_%H%M%S_%3f.json").to_string();
    let mut palette = vec![RecordedCell::empty()];
    let recorded_initial = encode_initial(&initial, &mut palette);
    self.session = Some(RecordingSession {
      started_at,
      frame_rate,
      started_instant: now,
      active_before_run: Duration::ZERO,
      run_started: now,
      paused_duration: Duration::ZERO,
      pause_started: None,
      path: storage.recording_cache_dir_path().join(filename),
      max_width: initial.width(),
      max_height: initial.height(),
      last_frame: initial,
      palette,
      initial: recorded_initial,
      events: Vec::new(),
    });
    self.state = RecordingState::Recording;
    true
  }

  pub fn pause(&mut self) -> bool {
    if self.state != RecordingState::Recording {
      return false;
    }
    let now = Instant::now();
    if let Some(session) = &mut self.session {
      session.active_before_run = session
        .active_before_run
        .saturating_add(now.saturating_duration_since(session.run_started));
      session.pause_started = Some(now);
    }
    self.state = RecordingState::Paused;
    true
  }

  pub fn resume(&mut self) -> bool {
    if self.state != RecordingState::Paused {
      return false;
    }
    let now = Instant::now();
    if let Some(session) = &mut self.session {
      if let Some(paused_at) = session.pause_started.take() {
        session.paused_duration = session
          .paused_duration
          .saturating_add(now.saturating_duration_since(paused_at));
      }
      session.run_started = now;
    }
    self.state = RecordingState::Recording;
    true
  }

  pub fn stop(&mut self, async_runtime: &AsyncRuntime) -> bool {
    if !matches!(
      self.state,
      RecordingState::Recording | RecordingState::Paused
    ) {
      return false;
    }
    let now = Instant::now();
    let previous_state = self.state;
    let Some(session) = self.session.take() else {
      self.state = RecordingState::Stopped;
      return false;
    };
    let active = session.active_duration(now, previous_state);
    let paused = session.paused_duration(now, previous_state);
    let wall = now.saturating_duration_since(session.started_instant);
    self.last_snapshot = RecordingSnapshot {
      state: RecordingState::Finalizing,
      active_duration: active,
      wall_duration: wall,
      paused_duration: paused,
    };
    let document = RecordingDocument {
      schema_version: RECORDING_SCHEMA_VERSION,
      started_at: session.started_at,
      finished_at: Local::now().to_rfc3339_opts(SecondsFormat::Millis, true),
      frame_rate: session.frame_rate,
      canvas: RecordedCanvas {
        max_width: session.max_width,
        max_height: session.max_height,
      },
      duration_us: RecordedDurations {
        active: duration_us(active),
        paused: duration_us(paused),
        wall: duration_us(wall),
      },
      palette: session.palette,
      initial: session.initial,
      events: session.events,
    };
    let task_id = async_runtime.submit(EngineTask::Recording(RecordingTask {
      document,
      path: session.path,
    }));
    self.finalizing_task = Some(task_id);
    self.state = RecordingState::Finalizing;
    true
  }

  pub(crate) fn capture_presented_frame(&mut self, frame: &ComposedFrame) {
    if self.state == RecordingState::Recording {
      let now = Instant::now();
      if let Some(session) = &mut self.session {
        let size_changed = frame.width() != session.last_frame.width()
          || frame.height() != session.last_frame.height();
        session.max_width = session.max_width.max(frame.width());
        session.max_height = session.max_height.max(frame.height());
        let changes = encode_changes(
          &session.last_frame,
          frame,
          session.max_width,
          session.max_height,
          &mut session.palette,
        );
        if size_changed || !changes.is_empty() {
          session.events.push(RecordedFrameEvent {
            time_us: duration_us(session.active_duration(now, RecordingState::Recording)),
            size: [frame.width(), frame.height()],
            changes,
          });
        }
        session.last_frame = frame.clone();
      }
    }
    self.last_presented_frame = Some(frame.clone());
  }

  pub(crate) fn capture_last_frame(&self) -> Option<ComposedFrame> {
    self.last_presented_frame.clone()
  }

  pub(crate) fn handle_engine_event(&mut self, event: &RecordingAsyncEvent) {
    let task_id = match event {
      RecordingAsyncEvent::Saved { task_id, .. } | RecordingAsyncEvent::Failed { task_id, .. } => {
        *task_id
      }
    };
    if self.finalizing_task == Some(task_id) {
      self.finalizing_task = None;
      self.state = RecordingState::Stopped;
    }
  }
}

impl Default for RecordingService {
  fn default() -> Self {
    Self::new()
  }
}

pub fn load_recording_playback_metadata(path: &Path) -> Option<RecordingPlaybackMetadata> {
  const PALETTE_FIELD: &[u8] = b"\"palette\"";

  let mut file = fs::File::open(path).ok()?;
  let mut bytes = Vec::new();
  let mut chunk = [0u8; 4096];
  let palette_index = loop {
    let read = file.read(&mut chunk).ok()?;
    if read == 0 {
      return None;
    }
    bytes.extend_from_slice(&chunk[..read]);
    if let Some(index) = bytes
      .windows(PALETTE_FIELD.len())
      .position(|window| window == PALETTE_FIELD)
    {
      break index;
    }
  };
  bytes.truncate(palette_index);
  while bytes.last().is_some_and(|byte| byte.is_ascii_whitespace()) {
    bytes.pop();
  }
  if bytes.last() == Some(&b',') {
    bytes.pop();
  }
  bytes.push(b'}');

  let header: PlaybackHeader = serde_json::from_slice(&bytes).ok()?;
  playback_metadata(
    header.schema_version,
    header.started_at,
    header.frame_rate,
    header.canvas,
    header.duration_us,
  )
}

pub fn load_recording_playback(path: &Path) -> Option<RecordingPlayback> {
  let document: PlaybackDocument = serde_json::from_reader(fs::File::open(path).ok()?).ok()?;
  let metadata = playback_metadata(
    document.schema_version,
    document.started_at.clone(),
    document.frame_rate,
    document.canvas,
    document.duration_us,
  )?;
  validate_playback_document(&document, &metadata)?;
  let palette = document
    .palette
    .into_iter()
    .map(playback_cell)
    .collect::<Option<Vec<_>>>()?;
  Some(RecordingPlayback {
    metadata,
    palette,
    initial: document.initial,
    events: document.events,
  })
}

impl RecordingPlayback {
  pub fn metadata(&self) -> &RecordingPlaybackMetadata {
    &self.metadata
  }

  pub fn initial_frame(&self) -> ComposedFrame {
    let mut frame = ComposedFrame::new(self.metadata.max_width, self.metadata.max_height);
    for (y, row) in self.initial.rows.iter().enumerate() {
      let mut x = 0u16;
      for &(count, palette_id) in row {
        for _ in 0..count {
          frame.set(x, y as u16, self.palette[palette_id as usize].clone());
          x = x.saturating_add(1);
        }
      }
    }
    frame
  }

  pub fn apply_until(&self, frame: &mut ComposedFrame, cursor: &mut usize, time_us: u64) {
    while let Some(event) = self
      .events
      .get(*cursor)
      .filter(|event| event.time_us <= time_us)
    {
      for (y, x, palette_ids) in &event.changes {
        for (offset, palette_id) in palette_ids.iter().enumerate() {
          frame.set(
            x.saturating_add(offset as u16),
            *y,
            self.palette[*palette_id as usize].clone(),
          );
        }
      }
      *cursor += 1;
    }
  }
}

fn playback_metadata(
  schema_version: u32,
  started_at: String,
  frame_rate: Option<u16>,
  canvas: PlaybackCanvas,
  duration_us: PlaybackDurations,
) -> Option<RecordingPlaybackMetadata> {
  if !matches!(schema_version, 1 | RECORDING_SCHEMA_VERSION)
    || started_at.is_empty()
    || canvas.max_width == 0
    || canvas.max_height == 0
    || frame_rate == Some(0)
  {
    return None;
  }
  Some(RecordingPlaybackMetadata {
    started_at,
    frame_rate,
    max_width: canvas.max_width,
    max_height: canvas.max_height,
    duration_us: duration_us.active,
  })
}

fn validate_playback_document(
  document: &PlaybackDocument,
  metadata: &RecordingPlaybackMetadata,
) -> Option<()> {
  let initial = &document.initial;
  if initial.width == 0
    || initial.height == 0
    || initial.width > metadata.max_width
    || initial.height > metadata.max_height
    || initial.rows.len() != initial.height as usize
  {
    return None;
  }
  for row in &initial.rows {
    let mut width = 0u32;
    for &(count, palette_id) in row {
      if count == 0 || palette_id as usize >= document.palette.len() {
        return None;
      }
      width = width.checked_add(count)?;
    }
    if width != initial.width as u32 {
      return None;
    }
  }

  let mut previous_time = 0;
  for event in &document.events {
    if event.time_us < previous_time
      || event.time_us > metadata.duration_us
      || event.size[0] == 0
      || event.size[1] == 0
      || event.size[0] > metadata.max_width
      || event.size[1] > metadata.max_height
    {
      return None;
    }
    previous_time = event.time_us;
    for (y, x, palette_ids) in &event.changes {
      if *y >= metadata.max_height
        || usize::from(*x).saturating_add(palette_ids.len()) > metadata.max_width as usize
        || palette_ids
          .iter()
          .any(|palette_id| *palette_id as usize >= document.palette.len())
      {
        return None;
      }
    }
  }
  Some(())
}

fn playback_cell(cell: PlaybackCell) -> Option<ComposedCell> {
  if cell.continuation {
    return Some(ComposedCell::Text(CanvasCell::continuation()));
  }
  let foreground = match cell.foreground {
    Some(color) => Some(playback_color(color)?),
    None => None,
  };
  let background = match cell.background {
    Some(color) => Some(playback_color(color)?),
    None => None,
  };
  Some(ComposedCell::Text(CanvasCell::styled(
    cell.text,
    crate::host_engine::services::TextStyle {
      foreground,
      background,
      bold: cell.flags & 1 != 0,
      italic: cell.flags & (1 << 1) != 0,
      underline: cell.flags & (1 << 2) != 0,
      strike: cell.flags & (1 << 3) != 0,
      blink: cell.flags & (1 << 4) != 0,
      reverse: cell.flags & (1 << 5) != 0,
      hidden: cell.flags & (1 << 6) != 0,
      dim: cell.flags & (1 << 7) != 0,
    },
  )))
}

fn playback_color(color: PlaybackColor) -> Option<TextColor> {
  Some(match color {
    PlaybackColor::Terminal(value) => TextColor::Terminal(match value.as_str() {
      "black" => TerminalColor::Black,
      "red" => TerminalColor::Red,
      "green" => TerminalColor::Green,
      "yellow" => TerminalColor::Yellow,
      "blue" => TerminalColor::Blue,
      "magenta" => TerminalColor::Magenta,
      "cyan" => TerminalColor::Cyan,
      "white" => TerminalColor::White,
      "bright_black" => TerminalColor::BrightBlack,
      "bright_red" => TerminalColor::BrightRed,
      "bright_green" => TerminalColor::BrightGreen,
      "bright_yellow" => TerminalColor::BrightYellow,
      "bright_blue" => TerminalColor::BrightBlue,
      "bright_magenta" => TerminalColor::BrightMagenta,
      "bright_cyan" => TerminalColor::BrightCyan,
      "bright_white" => TerminalColor::BrightWhite,
      _ => return None,
    }),
    PlaybackColor::Rgb([r, g, b]) => TextColor::Rgb { r, g, b },
    PlaybackColor::ForceRgb([r, g, b]) => TextColor::ForceRgb { r, g, b },
    PlaybackColor::Transparent => TextColor::Transparent,
  })
}

impl RecordingSession {
  fn active_duration(&self, now: Instant, state: RecordingState) -> Duration {
    if state == RecordingState::Recording {
      self
        .active_before_run
        .saturating_add(now.saturating_duration_since(self.run_started))
    } else {
      self.active_before_run
    }
  }

  fn paused_duration(&self, now: Instant, state: RecordingState) -> Duration {
    if state == RecordingState::Paused {
      self.paused_duration.saturating_add(
        self
          .pause_started
          .map(|started| now.saturating_duration_since(started))
          .unwrap_or_default(),
      )
    } else {
      self.paused_duration
    }
  }
}

impl RecordedCell {
  fn empty() -> Self {
    Self {
      text: " ".to_string(),
      foreground: None,
      background: None,
      flags: 0,
      continuation: false,
    }
  }
}

fn duration_us(duration: Duration) -> u64 {
  duration.as_micros().min(u64::MAX as u128) as u64
}

fn encode_initial(frame: &ComposedFrame, palette: &mut Vec<RecordedCell>) -> RecordedInitialFrame {
  let rows = (0..frame.height())
    .map(|y| {
      let ids = (0..frame.width())
        .map(|x| palette_id(palette, frame.get(x, y)))
        .collect::<Vec<_>>();
      rle(&ids)
    })
    .collect();
  RecordedInitialFrame {
    width: frame.width(),
    height: frame.height(),
    rows,
  }
}

fn rle(ids: &[u32]) -> Vec<(u32, u32)> {
  let mut runs = Vec::new();
  for &id in ids {
    match runs.last_mut() {
      Some((count, previous)) if *previous == id => *count += 1,
      _ => runs.push((1, id)),
    }
  }
  runs
}

fn encode_changes(
  previous: &ComposedFrame,
  current: &ComposedFrame,
  width: u16,
  height: u16,
  palette: &mut Vec<RecordedCell>,
) -> Vec<(u16, u16, Vec<u32>)> {
  let mut changes = Vec::new();
  for y in 0..height {
    let mut x = 0;
    while x < width {
      if frame_cell(previous, x, y) == frame_cell(current, x, y) {
        x += 1;
        continue;
      }
      let start = x;
      let mut ids = Vec::new();
      while x < width && frame_cell(previous, x, y) != frame_cell(current, x, y) {
        ids.push(palette_id(palette, current.get(x, y)));
        x += 1;
      }
      changes.push((y, start, ids));
    }
  }
  changes
}

fn frame_cell(frame: &ComposedFrame, x: u16, y: u16) -> Option<&ComposedCell> {
  frame
    .get(x, y)
    .filter(|cell| !matches!(cell, ComposedCell::Empty))
}

fn palette_id(palette: &mut Vec<RecordedCell>, cell: Option<&ComposedCell>) -> u32 {
  let Some(ComposedCell::Text(cell)) = cell else {
    return 0;
  };
  if cell == &CanvasCell::blank() {
    return 0;
  }
  let recorded = recorded_cell(cell);
  if let Some(index) = palette.iter().position(|entry| entry == &recorded) {
    return index as u32;
  }
  palette.push(recorded);
  palette.len() as u32 - 1
}

fn recorded_cell(cell: &CanvasCell) -> RecordedCell {
  let style = &cell.style;
  let flags = (style.bold as u16)
    | (style.italic as u16) << 1
    | (style.underline as u16) << 2
    | (style.strike as u16) << 3
    | (style.blink as u16) << 4
    | (style.reverse as u16) << 5
    | (style.hidden as u16) << 6
    | (style.dim as u16) << 7;
  RecordedCell {
    text: cell.text.clone(),
    foreground: style.foreground.as_ref().map(recorded_color),
    background: style.background.as_ref().map(recorded_color),
    flags,
    continuation: cell.is_continuation(),
  }
}

fn recorded_color(color: &TextColor) -> RecordedColor {
  match color {
    TextColor::Terminal(color) => RecordedColor::Terminal(terminal_color_name(color)),
    TextColor::Rgb { r, g, b } => RecordedColor::Rgb([*r, *g, *b]),
    TextColor::ForceRgb { r, g, b } => RecordedColor::ForceRgb([*r, *g, *b]),
    TextColor::Transparent => RecordedColor::Transparent,
  }
}

fn terminal_color_name(color: &TerminalColor) -> &'static str {
  match color {
    TerminalColor::Black => "black",
    TerminalColor::Red => "red",
    TerminalColor::Green => "green",
    TerminalColor::Yellow => "yellow",
    TerminalColor::Blue => "blue",
    TerminalColor::Magenta => "magenta",
    TerminalColor::Cyan => "cyan",
    TerminalColor::White => "white",
    TerminalColor::BrightBlack => "bright_black",
    TerminalColor::BrightRed => "bright_red",
    TerminalColor::BrightGreen => "bright_green",
    TerminalColor::BrightYellow => "bright_yellow",
    TerminalColor::BrightBlue => "bright_blue",
    TerminalColor::BrightMagenta => "bright_magenta",
    TerminalColor::BrightCyan => "bright_cyan",
    TerminalColor::BrightWhite => "bright_white",
  }
}

pub fn run_recording_task(
  task_id: TaskId,
  task: RecordingTask,
  event_tx: &Sender<EngineEvent>,
) -> Result<(), String> {
  let result = (|| {
    let parent = task.path.parent().ok_or("recording path has no parent")?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let temporary = task.path.with_extension("json.tmp");
    let bytes = serde_json::to_vec(&task.document).map_err(|error| error.to_string())?;
    fs::write(&temporary, bytes).map_err(|error| error.to_string())?;
    fs::rename(&temporary, &task.path).map_err(|error| error.to_string())
  })();
  match result {
    Ok(()) => {
      let _ = event_tx.send(EngineEvent::Recording(RecordingAsyncEvent::Saved {
        task_id,
        path: task.path,
      }));
      Ok(())
    }
    Err(error) => {
      let _ = event_tx.send(EngineEvent::Recording(RecordingAsyncEvent::Failed {
        task_id,
        error: error.clone(),
      }));
      Err(error)
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::host_engine::services::{TextColor, TextStyle};
  use std::time::{SystemTime, UNIX_EPOCH};

  fn frame(width: u16, height: u16, values: &[(u16, u16, &str)]) -> ComposedFrame {
    let mut frame = ComposedFrame::new(width, height);
    for &(x, y, text) in values {
      frame.set(
        x,
        y,
        ComposedCell::Text(CanvasCell::styled(text, TextStyle::default())),
      );
    }
    frame
  }

  fn playback_file(value: serde_json::Value) -> PathBuf {
    let nonce = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap()
      .as_nanos();
    let path = std::env::temp_dir().join(format!(
      "tui-game-recording-playback-{}-{nonce}.json",
      std::process::id()
    ));
    fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
    path
  }

  fn playback_document(schema_version: u32, frame_rate: Option<u16>) -> serde_json::Value {
    serde_json::json!({
      "schema_version": schema_version,
      "started_at": "2026-07-21T20:20:32.895Z",
      "finished_at": "2026-07-21T20:20:34.895Z",
      "frame_rate": frame_rate,
      "canvas": { "max_width": 2, "max_height": 1 },
      "duration_us": { "active": 2_000_000, "paused": 0, "wall": 2_000_000 },
      "palette": [
        { "text": " " },
        { "text": "x", "foreground": { "type": "rgb", "value": [1, 2, 3] } },
        { "text": "y" }
      ],
      "initial": { "width": 2, "height": 1, "rows": [[[2, 1]]] },
      "events": [{ "time_us": 1_000_000, "size": [2, 1], "changes": [[0, 1, [2]]] }]
    })
  }

  #[test]
  fn initial_frame_uses_rle_and_palette() {
    let mut frame = frame(3, 1, &[(0, 0, "a"), (1, 0, "a")]);
    frame.set(2, 0, ComposedCell::Text(CanvasCell::blank()));
    let mut palette = vec![RecordedCell::empty()];
    let initial = encode_initial(&frame, &mut palette);
    assert_eq!(palette.len(), 2);
    assert_eq!(palette[0], RecordedCell::empty());
    assert_eq!(initial.rows[0], vec![(2, 1), (1, 0)]);
  }

  #[test]
  fn changes_are_grouped_by_adjacent_cells() {
    let previous = frame(4, 1, &[]);
    let current = frame(4, 1, &[(1, 0, "我"), (2, 0, "")]);
    let mut palette = vec![RecordedCell::empty()];
    let changes = encode_changes(&previous, &current, 4, 1, &mut palette);
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].0, 0);
    assert_eq!(changes[0].1, 1);
    assert_eq!(changes[0].2.len(), 2);
  }

  #[test]
  fn resize_compares_against_historical_max_canvas() {
    let previous = frame(3, 2, &[(2, 1, "x")]);
    let current = frame(2, 1, &[]);
    let mut palette = vec![RecordedCell::empty()];
    let changes = encode_changes(&previous, &current, 3, 2, &mut palette);
    assert_eq!(changes, vec![(1, 2, vec![0])]);
  }

  #[test]
  fn palette_preserves_unicode_styles_and_continuations() {
    let mut palette = vec![RecordedCell::empty()];
    let styled = CanvasCell::styled(
      "👨‍👩‍👧‍👦",
      TextStyle {
        foreground: Some(TextColor::Rgb { r: 1, g: 2, b: 3 }),
        bold: true,
        ..Default::default()
      },
    );
    assert_eq!(
      palette_id(&mut palette, Some(&ComposedCell::Text(styled))),
      1
    );
    assert_eq!(palette[1].text, "👨‍👩‍👧‍👦");
    assert_eq!(palette[1].flags & 1, 1);
    assert_eq!(
      palette_id(
        &mut palette,
        Some(&ComposedCell::Text(CanvasCell::continuation()))
      ),
      2
    );
    assert!(palette[2].continuation);
  }

  #[test]
  fn document_keeps_frame_rate_and_event_timing() {
    let document = RecordingDocument {
      schema_version: RECORDING_SCHEMA_VERSION,
      started_at: "2026-07-21T20:20:32.895Z".to_string(),
      finished_at: "2026-07-21T20:20:34.895Z".to_string(),
      frame_rate: Some(60),
      canvas: RecordedCanvas {
        max_width: 120,
        max_height: 30,
      },
      duration_us: RecordedDurations {
        active: 2_000_000,
        paused: 0,
        wall: 2_000_000,
      },
      palette: vec![RecordedCell::empty()],
      initial: RecordedInitialFrame {
        width: 120,
        height: 30,
        rows: Vec::new(),
      },
      events: vec![RecordedFrameEvent {
        time_us: 16_667,
        size: [120, 30],
        changes: Vec::new(),
      }],
    };

    let value = serde_json::to_value(document).unwrap();
    assert_eq!(value["schema_version"], 2);
    assert_eq!(value["frame_rate"], 60);
    assert_eq!(value["duration_us"]["active"], 2_000_000);
    assert_eq!(value["events"][0]["time_us"], 16_667);
  }

  #[test]
  fn playback_loads_header_first_and_applies_events_by_recorded_time() {
    let path = playback_file(playback_document(RECORDING_SCHEMA_VERSION, Some(60)));

    let metadata = load_recording_playback_metadata(&path).unwrap();
    assert_eq!(metadata.frame_rate, Some(60));
    assert_eq!(metadata.duration_us, 2_000_000);

    let playback = load_recording_playback(&path).unwrap();
    let mut frame = playback.initial_frame();
    let mut cursor = 0;
    playback.apply_until(&mut frame, &mut cursor, 999_999);
    assert_eq!(frame.get(1, 0), frame.get(0, 0));
    playback.apply_until(&mut frame, &mut cursor, 1_000_000);
    let Some(ComposedCell::Text(cell)) = frame.get(1, 0) else {
      panic!("event should produce a text cell");
    };
    assert_eq!(cell.text, "y");
    fs::remove_file(path).unwrap();
  }

  #[test]
  fn legacy_playback_without_frame_rate_remains_supported() {
    let path = playback_file(playback_document(1, None));

    assert_eq!(
      load_recording_playback_metadata(&path).unwrap().frame_rate,
      None
    );
    assert!(load_recording_playback(&path).is_some());
    fs::remove_file(path).unwrap();
  }

  #[test]
  fn structurally_invalid_recording_keeps_header_but_rejects_playback() {
    let mut document = playback_document(RECORDING_SCHEMA_VERSION, Some(60));
    document["events"][0]["changes"] = serde_json::json!([[0, 2, [2]]]);
    let path = playback_file(document);

    assert!(load_recording_playback_metadata(&path).is_some());
    assert!(load_recording_playback(&path).is_none());
    fs::remove_file(path).unwrap();
  }
}
