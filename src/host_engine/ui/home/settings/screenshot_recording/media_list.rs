use std::{
  fs, io,
  marker::PhantomData,
  path::{Path, PathBuf},
  sync::mpsc::{self, Receiver},
  thread,
  time::{Duration, SystemTime},
};

use serde_json::Value;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::host_engine::services::{
  ActionMapEntry, BorderStyle, CanvasCell, CanvasService, ComposedCell, ComposedFrame,
  DrawTextParams, HitAreaEvent, HitAreaId, HitAreaOptions, HitAreaService, I18nService, KeyState,
  LayoutService, MouseButton, Overflow, Rect, RenderService, RichTextParams, RichTextService,
  RuntimeObjectPool, RuntimeObjectPoolOwner, ScreenshotRect, ScrollBoxId, ScrollBoxOptions,
  ScrollBoxService, ScrollbarLayout, ScrollbarPolicy, ScrollbarVisibility, TerminalColor,
  TextColor, TextInputEvent, TextInputId, TextInputMode, TextInputOptions, TextInputRenderParams,
  TextInputService, TextStyle, UiEvent, UiObjectPool, UiObjectPoolOwner,
};

const HINT_COLOR: TextColor = TextColor::Rgb {
  r: 85,
  g: 87,
  b: 83,
};

const ACTIVE_BORDER: TextColor = TextColor::Rgb {
  r: 95,
  g: 215,
  b: 105,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MediaListCommand {
  Back,
  FocusSearch,
  BlurSearch,
  SelectList(i32),
  ScrollList(i32),
  ScrollInfo {
    dx: i32,
    dy: i32,
  },
  BeginRename,
  CancelRename,
  CommitRename {
    old_name: String,
    new_name: String,
  },
  CopyScreenshot {
    frame: ComposedFrame,
    rect: ScreenshotRect,
    rich: bool,
  },
  SaveScreenshot {
    frame: ComposedFrame,
    rect: ScreenshotRect,
    copy: bool,
  },
}

fn display_timestamp(timestamp: &str) -> String {
  let bytes = timestamp.as_bytes();
  if bytes.len() >= 15 && bytes.get(8) == Some(&b'_') {
    return format!(
      "{}.{}.{} {}:{}:{}",
      &timestamp[0..4],
      &timestamp[4..6],
      &timestamp[6..8],
      &timestamp[9..11],
      &timestamp[11..13],
      &timestamp[13..15]
    );
  }
  if bytes.len() >= 19
    && bytes.get(4) == Some(&b'-')
    && bytes.get(7) == Some(&b'-')
    && bytes.get(10) == Some(&b'T')
  {
    return format!(
      "{}.{}.{} {}:{}:{}",
      &timestamp[0..4],
      &timestamp[5..7],
      &timestamp[8..10],
      &timestamp[11..13],
      &timestamp[14..16],
      &timestamp[17..19]
    );
  }
  timestamp.to_string()
}

fn screenshot_size_text(width: u16, height: u16) -> String {
  format!("w-{width} x h-{height}")
}

fn frame_rate_text(frame_rate: Option<u16>) -> String {
  frame_rate
    .map(|value| format!("FPS {value}"))
    .unwrap_or_else(|| "FPS --".to_string())
}

fn truncate_text(text: &str, width: u16) -> String {
  let width = width as usize;
  if text.width() <= width {
    return text.to_string();
  }
  if width <= 3 {
    return ".".repeat(width);
  }
  let mut output = String::new();
  let mut used = 0;
  for ch in text.chars() {
    let ch_width = ch.width().unwrap_or(0);
    if used + ch_width > width - 3 {
      break;
    }
    output.push(ch);
    used += ch_width;
  }
  output.push_str("...");
  output
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ActivePanel {
  List,
  Info,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SortField {
  Name,
  Time,
  Duration,
}

#[derive(Clone, Debug)]
struct MediaEntry {
  name: String,
  path: PathBuf,
  modified: SystemTime,
  duration_us: u64,
  info: Option<MediaInfo>,
  preview: Option<ScreenshotPreview>,
  valid: Option<bool>,
}

struct MediaLoadResult {
  path: PathBuf,
  duration_us: u64,
  info: Option<MediaInfo>,
  preview: Option<ScreenshotPreview>,
  valid: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct MediaInfo {
  width: u16,
  height: u16,
  timestamp: String,
  frame_rate: Option<u16>,
}

#[derive(Clone, Debug)]
pub(super) struct ScreenshotPreview {
  width: u16,
  height: u16,
  timestamp: String,
  cells: Vec<PreviewCell>,
}

impl ScreenshotPreview {
  fn frame_and_rect(&self) -> (ComposedFrame, ScreenshotRect) {
    let mut frame = ComposedFrame::new(self.width, self.height);
    for cell in &self.cells {
      frame.set(
        cell.x,
        cell.y,
        ComposedCell::Text(CanvasCell::styled(cell.text.clone(), cell.style.clone())),
      );
    }
    (
      frame,
      ScreenshotRect {
        x: 0,
        y: 0,
        width: self.width,
        height: self.height,
      },
    )
  }
}

pub(super) fn load_screenshot_preview(path: &Path) -> Option<ScreenshotPreview> {
  let document: Value = serde_json::from_slice(&fs::read(path).ok()?).ok()?;
  let selection = document.get("selection")?;
  let width = json_u16(selection.get("width")?)?;
  let height = json_u16(selection.get("height")?)?;
  let timestamp = document.get("timestamp")?.as_str()?.to_string();
  let mut cells = Vec::new();
  for (row_index, row) in document.get("rich_text")?.as_array()?.iter().enumerate() {
    let y = u16::try_from(row_index).ok()?;
    for value in row.as_array()? {
      cells.push(PreviewCell {
        x: json_u16(value.get("x")?)?,
        y,
        text: value.get("text")?.as_str()?.to_string(),
        style: parse_style(value.get("style")?),
      });
    }
  }
  Some(ScreenshotPreview {
    width,
    height,
    timestamp,
    cells,
  })
}

fn json_u16(value: &Value) -> Option<u16> {
  u16::try_from(value.as_u64()?).ok()
}

fn recording_info(document: &Value) -> Option<MediaInfo> {
  let canvas = document.get("canvas")?;
  Some(MediaInfo {
    width: json_u16(canvas.get("max_width")?)?,
    height: json_u16(canvas.get("max_height")?)?,
    timestamp: document.get("started_at")?.as_str()?.to_string(),
    frame_rate: document.get("frame_rate").and_then(json_u16),
  })
}

fn parse_style(value: &Value) -> TextStyle {
  TextStyle {
    foreground: value
      .get("fg")
      .and_then(Value::as_str)
      .and_then(parse_color),
    background: value
      .get("bg")
      .and_then(Value::as_str)
      .and_then(parse_color),
    bold: value.get("bold").and_then(Value::as_bool).unwrap_or(false),
    italic: value
      .get("italic")
      .and_then(Value::as_bool)
      .unwrap_or(false),
    underline: value
      .get("underline")
      .and_then(Value::as_bool)
      .unwrap_or(false),
    strike: value
      .get("strike")
      .and_then(Value::as_bool)
      .unwrap_or(false),
    reverse: value
      .get("reverse")
      .and_then(Value::as_bool)
      .unwrap_or(false),
    dim: value.get("dim").and_then(Value::as_bool).unwrap_or(false),
    ..Default::default()
  }
}

fn parse_color(value: &str) -> Option<TextColor> {
  if value == "transparent" {
    return Some(TextColor::Transparent);
  }
  if let Some(hex) = value.strip_prefix('#')
    && hex.len() == 6
  {
    return Some(TextColor::Rgb {
      r: u8::from_str_radix(&hex[0..2], 16).ok()?,
      g: u8::from_str_radix(&hex[2..4], 16).ok()?,
      b: u8::from_str_radix(&hex[4..6], 16).ok()?,
    });
  }
  let color = match value {
    "black" => TerminalColor::Black,
    "red" => TerminalColor::Red,
    "green" => TerminalColor::Green,
    "yellow" => TerminalColor::Yellow,
    "blue" => TerminalColor::Blue,
    "magenta" => TerminalColor::Magenta,
    "cyan" => TerminalColor::Cyan,
    "white" => TerminalColor::White,
    "brightblack" | "bright_black" => TerminalColor::BrightBlack,
    "brightred" | "bright_red" => TerminalColor::BrightRed,
    "brightgreen" | "bright_green" => TerminalColor::BrightGreen,
    "brightyellow" | "bright_yellow" => TerminalColor::BrightYellow,
    "brightblue" | "bright_blue" => TerminalColor::BrightBlue,
    "brightmagenta" | "bright_magenta" => TerminalColor::BrightMagenta,
    "brightcyan" | "bright_cyan" => TerminalColor::BrightCyan,
    "brightwhite" | "bright_white" => TerminalColor::BrightWhite,
    _ => return None,
  };
  Some(TextColor::Terminal(color))
}

#[derive(Clone, Debug)]
struct PreviewCell {
  x: u16,
  y: u16,
  text: String,
  style: TextStyle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MediaRenameError {
  Invalid,
  Duplicate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MediaListNotice {
  RenameError {
    namespace: &'static str,
    error: MediaRenameError,
  },
  ClearRenameError,
}

#[derive(Clone, Debug)]
struct MediaListLayout {
  left: Rect,
  right: Rect,
  search: Rect,
  sort_y: u16,
  list: Rect,
  hint_y: u16,
  hint_lines: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct MediaInfoHeaderLayout {
  name_x: u16,
  name_width: u16,
  frame_rate_x: Option<u16>,
  size_x: u16,
  time_x: u16,
}

fn media_info_header_layout(
  panel: Rect,
  frame_rate_width: Option<u16>,
  size_width: u16,
  timestamp_width: u16,
) -> MediaInfoHeaderLayout {
  let name_x = panel.x.saturating_add(1);
  let time_x = panel
    .x
    .saturating_add(panel.width.saturating_sub(1 + timestamp_width));
  let size_x = time_x.saturating_sub(2 + size_width);
  let frame_rate_x = frame_rate_width.map(|width| size_x.saturating_sub(2 + width));
  let metadata_x = frame_rate_x.unwrap_or(size_x);
  let name_width = metadata_x.saturating_sub(name_x.saturating_add(2));
  MediaInfoHeaderLayout {
    name_x,
    name_width,
    frame_rate_x,
    size_x,
    time_x,
  }
}

pub trait MediaListSpec: Send + Sync + 'static {
  const NS: &'static str;
  const SUPPORTS_DURATION: bool;
  fn action_map() -> Vec<ActionMapEntry>;
  fn left_hint_keys() -> &'static [&'static str];
  fn right_hint_keys() -> &'static [&'static str];
  fn load_preview(_path: &Path) -> Option<ScreenshotPreview> {
    None
  }
}

pub struct MediaListUi<S: MediaListSpec> {
  objects: UiObjectPool,
  runtime_objects: RuntimeObjectPool,
  search_input: TextInputId,
  rename_input: TextInputId,
  list_scroll: ScrollBoxId,
  info_scroll: ScrollBoxId,
  list_panel_area: HitAreaId,
  info_panel_area: HitAreaId,
  order_area: HitAreaId,
  sort_area: HitAreaId,
  item_areas: Vec<HitAreaId>,
  entries: Vec<MediaEntry>,
  search: String,
  selected: usize,
  active: ActivePanel,
  ascending: bool,
  sort_field: SortField,
  renaming: Option<String>,
  pending_notice: Option<MediaListNotice>,
  last_list_scroll_y: u16,
  zoomed: bool,
  directory: Option<PathBuf>,
  scan_rx: Option<Receiver<io::Result<Vec<MediaEntry>>>>,
  load_rx: Option<Receiver<MediaLoadResult>>,
  loading_path: Option<PathBuf>,
  reload_elapsed: Duration,
  marker: PhantomData<S>,
}

impl<S: MediaListSpec> UiObjectPoolOwner for MediaListUi<S> {
  fn objects(&self) -> &UiObjectPool {
    &self.objects
  }

  fn objects_mut(&mut self) -> &mut UiObjectPool {
    &mut self.objects
  }
}

impl<S: MediaListSpec> RuntimeObjectPoolOwner for MediaListUi<S> {
  fn runtime_objects(&self) -> &RuntimeObjectPool {
    &self.runtime_objects
  }

  fn runtime_objects_mut(&mut self) -> &mut RuntimeObjectPool {
    &mut self.runtime_objects
  }
}

impl<S: MediaListSpec> MediaListUi<S> {
  pub fn init(
    hit_area: &HitAreaService,
    text_input: &TextInputService,
    scroll_box: &ScrollBoxService,
  ) -> Self {
    let mut objects = UiObjectPool::new();
    let search_input = text_input.create(
      &mut objects,
      TextInputOptions {
        max_chars: Some(128),
        mode: TextInputMode::SingleLine,
        mouse: true,
        ..Default::default()
      },
    );
    let rename_input = text_input.create(
      &mut objects,
      TextInputOptions {
        max_chars: Some(128),
        mode: TextInputMode::SingleLine,
        mouse: true,
        ..Default::default()
      },
    );
    let list_scroll = scroll_box
      .create(
        &mut objects,
        ScrollBoxOptions {
          rect: Rect::default(),
          content_width: 1,
          content_height: 1,
          overflow_x: Overflow::Hidden,
          overflow_y: Overflow::Auto,
          scrollbar: ScrollbarPolicy {
            vertical: ScrollbarVisibility::Auto,
            horizontal: ScrollbarVisibility::Never,
          },
          scrollbar_layout: ScrollbarLayout::Inside,
          emit_scroll_events: true,
          ..Default::default()
        },
      )
      .expect("failed to create media list scroll box");
    let info_scroll = scroll_box
      .create(
        &mut objects,
        ScrollBoxOptions {
          rect: Rect::default(),
          content_width: 1,
          content_height: 1,
          overflow_x: Overflow::Auto,
          overflow_y: Overflow::Auto,
          scrollbar: ScrollbarPolicy {
            vertical: ScrollbarVisibility::Auto,
            horizontal: ScrollbarVisibility::Auto,
          },
          scrollbar_layout: ScrollbarLayout::Inside,
          ..Default::default()
        },
      )
      .expect("failed to create media info scroll box");
    Self {
      search_input,
      rename_input,
      list_scroll,
      info_scroll,
      list_panel_area: hit_area.create(&mut objects, HitAreaOptions::default()),
      info_panel_area: hit_area.create(&mut objects, HitAreaOptions::default()),
      order_area: hit_area.create(&mut objects, HitAreaOptions::default()),
      sort_area: hit_area.create(&mut objects, HitAreaOptions::default()),
      item_areas: Vec::new(),
      entries: Vec::new(),
      search: String::new(),
      selected: 0,
      active: ActivePanel::List,
      ascending: true,
      sort_field: SortField::Name,
      renaming: None,
      pending_notice: None,
      last_list_scroll_y: 0,
      zoomed: false,
      directory: None,
      scan_rx: None,
      load_rx: None,
      loading_path: None,
      reload_elapsed: Duration::ZERO,
      objects,
      runtime_objects: RuntimeObjectPool::new(),
      marker: PhantomData,
    }
  }

  pub fn reload(&mut self, directory: &Path) -> io::Result<()> {
    self.directory = Some(directory.to_path_buf());
    self.start_scan();
    Ok(())
  }

  fn start_scan(&mut self) {
    let Some(directory) = self.directory.clone() else {
      return;
    };
    if self.scan_rx.is_some() {
      return;
    }
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
      let result = (|| {
        let mut entries = Vec::new();
        for item in fs::read_dir(directory)? {
          let item = item?;
          let path = item.path();
          if !item.file_type()?.is_file()
            || path.extension().and_then(|v| v.to_str()) != Some("json")
          {
            continue;
          }
          let metadata = item.metadata()?;
          entries.push(MediaEntry {
            name: path
              .file_stem()
              .and_then(|v| v.to_str())
              .unwrap_or_default()
              .to_string(),
            path,
            modified: metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
            duration_us: 0,
            info: None,
            preview: None,
            valid: None,
          });
        }
        Ok(entries)
      })();
      let _ = tx.send(result);
    });
    self.scan_rx = Some(rx);
  }

  fn request_selected_load(&mut self) {
    if self.load_rx.is_some() {
      return;
    }
    let Some(entry) = self.filtered_entries().get(self.selected).copied() else {
      return;
    };
    if entry.valid.is_some() {
      return;
    }
    let path = entry.path.clone();
    let worker_path = path.clone();
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
      let document = fs::read_to_string(&worker_path)
        .ok()
        .and_then(|v| serde_json::from_str::<Value>(&v).ok());
      let preview = S::load_preview(&worker_path);
      let valid = if S::SUPPORTS_DURATION {
        document.as_ref().is_some_and(|v| {
          v.get("schema_version").is_some()
            && v.get("initial").is_some()
            && v.get("events").is_some()
        })
      } else {
        preview.is_some()
      };
      let duration_us = document
        .as_ref()
        .and_then(|v| v.get("duration_us"))
        .and_then(|v| v.get("active"))
        .and_then(Value::as_u64)
        .unwrap_or(0);
      let info = if S::SUPPORTS_DURATION {
        document.as_ref().and_then(recording_info)
      } else {
        preview.as_ref().map(|preview| MediaInfo {
          width: preview.width,
          height: preview.height,
          timestamp: preview.timestamp.clone(),
          frame_rate: None,
        })
      };
      let _ = tx.send(MediaLoadResult {
        path: worker_path,
        duration_us,
        info,
        preview,
        valid,
      });
    });
    self.loading_path = Some(path);
    self.load_rx = Some(rx);
  }

  pub fn reset_for_entry(
    &mut self,
    text_input: &mut TextInputService,
    scroll_box: &ScrollBoxService,
    layout: &LayoutService,
  ) {
    self.search.clear();
    self.renaming = None;
    self.selected = 0;
    self.active = ActivePanel::List;
    self.ascending = true;
    self.sort_field = SortField::Name;
    self.last_list_scroll_y = 0;
    self.zoomed = false;
    self.pending_notice = None;
    let _ = text_input.clear(&mut self.objects, self.search_input);
    let _ = text_input.clear(&mut self.objects, self.rename_input);
    let _ = text_input.blur(&mut self.objects);
    let _ = scroll_box.scroll_to(&mut self.objects, self.list_scroll, 0, 0, layout);
    let _ = scroll_box.scroll_to(&mut self.objects, self.info_scroll, 0, 0, layout);
  }

  pub fn action_map() -> Vec<ActionMapEntry> {
    S::action_map()
  }

  pub fn focus_search(&mut self, text_input: &mut TextInputService) {
    self.active = ActivePanel::List;
    let _ = text_input.focus(&mut self.objects, self.search_input);
  }

  pub fn blur_search(&mut self, text_input: &mut TextInputService) {
    let _ = text_input.blur(&mut self.objects);
  }

  pub fn begin_rename(&mut self, text_input: &mut TextInputService) {
    let Some(name) = self
      .filtered_entries()
      .get(self.selected)
      .map(|entry| entry.name.clone())
    else {
      return;
    };
    self.renaming = Some(name.clone());
    self.pending_notice = Some(MediaListNotice::ClearRenameError);
    let _ = text_input.set_text(&mut self.objects, self.rename_input, name);
    let _ = text_input.focus(&mut self.objects, self.rename_input);
  }

  pub fn cancel_rename(&mut self, text_input: &mut TextInputService) {
    self.renaming = None;
    self.pending_notice = Some(MediaListNotice::ClearRenameError);
    let _ = text_input.blur(&mut self.objects);
  }

  pub fn commit_rename(
    &mut self,
    directory: &Path,
    old_name: &str,
    new_name: &str,
    text_input: &mut TextInputService,
  ) -> io::Result<()> {
    let old_path = media_json_path(directory, old_name);
    let new_path = media_json_path(directory, new_name);
    if old_name != new_name {
      if new_path.exists() {
        return Err(io::Error::new(
          io::ErrorKind::AlreadyExists,
          "target media name already exists",
        ));
      }
      fs::rename(&old_path, &new_path)?;
    }
    if let Some(entry) = self.entries.iter_mut().find(|entry| entry.name == old_name) {
      entry.name = new_name.to_string();
      entry.path = new_path.clone();
      entry.modified = fs::metadata(&new_path)
        .and_then(|metadata| metadata.modified())
        .unwrap_or(entry.modified);
    }
    self.selected = self
      .filtered_entries()
      .iter()
      .position(|entry| entry.name == new_name)
      .unwrap_or(0);
    self.cancel_rename(text_input);
    Ok(())
  }

  pub fn rename_io_failed(&mut self) {
    self.pending_notice = Some(MediaListNotice::RenameError {
      namespace: S::NS,
      error: MediaRenameError::Invalid,
    });
  }

  pub fn take_notice(&mut self) -> Option<MediaListNotice> {
    self.pending_notice.take()
  }

  pub fn scroll_list(&mut self, service: &ScrollBoxService, layout: &LayoutService, dy: i32) {
    if dy == 0 || self.filtered_entries().is_empty() {
      return;
    }
    let _ = service.scroll_by(&mut self.objects, self.list_scroll, 0, dy, layout);
    self.clamp_selection_to_list_view(service, layout);
    self.last_list_scroll_y = service
      .scroll_y(&self.objects, self.list_scroll)
      .unwrap_or(self.last_list_scroll_y);
  }

  pub fn select_list(&mut self, service: &ScrollBoxService, layout: &LayoutService, dy: i32) {
    self.move_selection(dy as isize);
    self.ensure_selection_visible(service, layout);
    self.last_list_scroll_y = service
      .scroll_y(&self.objects, self.list_scroll)
      .unwrap_or(self.last_list_scroll_y);
  }

  pub fn scroll_info(
    &mut self,
    service: &ScrollBoxService,
    layout: &LayoutService,
    dx: i32,
    dy: i32,
  ) {
    let _ = service.scroll_by(&mut self.objects, self.info_scroll, dx, dy, layout);
  }

  pub fn update(&mut self, dt: Duration, service: &ScrollBoxService, layout: &LayoutService) {
    self.reload_elapsed += dt;
    if self.reload_elapsed >= Duration::from_millis(500) {
      self.reload_elapsed = Duration::ZERO;
      self.start_scan();
    }
    if let Some(result) = self.scan_rx.as_ref().and_then(|rx| rx.try_recv().ok()) {
      self.scan_rx = None;
      if let Ok(mut scanned) = result {
        let selected = self
          .filtered_entries()
          .get(self.selected)
          .map(|v| v.name.clone());
        for entry in &mut scanned {
          if let Some(old) = self
            .entries
            .iter()
            .find(|old| old.path == entry.path && old.modified == entry.modified)
          {
            entry.duration_us = old.duration_us;
            entry.info = old.info.clone();
            entry.preview = old.preview.clone();
            entry.valid = old.valid;
          }
        }
        self.entries = scanned;
        self.selected = selected
          .and_then(|name| self.filtered_entries().iter().position(|v| v.name == name))
          .unwrap_or(0);
      }
    }
    if let Some(result) = self.load_rx.as_ref().and_then(|rx| rx.try_recv().ok()) {
      self.load_rx = None;
      self.loading_path = None;
      if let Some(entry) = self
        .entries
        .iter_mut()
        .find(|entry| entry.path == result.path)
      {
        entry.duration_us = result.duration_us;
        entry.info = result.info;
        entry.preview = result.preview;
        entry.valid = Some(result.valid);
      }
    }
    self.request_selected_load();
    for event in service.drain_scroll_events(&mut self.objects) {
      let crate::host_engine::services::ScrollBoxEvent::Scrolled { id, y, .. } = event;
      if id != self.list_scroll || y == self.last_list_scroll_y {
        continue;
      }
      let delta = i32::from(y) - i32::from(self.last_list_scroll_y);
      self.last_list_scroll_y = y;
      if delta != 0 {
        self.clamp_selection_to_list_view(service, layout);
      }
    }
  }

  pub fn prepare_surfaces(
    &mut self,
    layout: &LayoutService,
    i18n: &I18nService,
    text_input: &TextInputService,
    scroll_box: &ScrollBoxService,
  ) {
    let pos = self.compute_layout(layout, i18n, text_input);
    self.prepare_scroll_box(scroll_box, layout, &pos);
  }

  pub fn handle_event(&mut self, event: &UiEvent) -> Option<MediaListCommand> {
    if let Some(original) = self.renaming.clone() {
      return match event {
        UiEvent::TextInput(TextInputEvent::Submit { id, value }) if *id == self.rename_input => {
          let value = value.to_string();
          match self.rename_error_for(&value, &original) {
            Some(error) => {
              self.pending_notice = Some(MediaListNotice::RenameError {
                namespace: S::NS,
                error,
              });
              None
            }
            None => Some(MediaListCommand::CommitRename {
              old_name: original,
              new_name: value,
            }),
          }
        }
        UiEvent::TextInput(TextInputEvent::Cancel { id, .. }) if *id == self.rename_input => {
          Some(MediaListCommand::CancelRename)
        }
        _ => None,
      };
    }
    match event {
      UiEvent::TextInput(TextInputEvent::Pressed { id }) if *id == self.search_input => {
        self.active = ActivePanel::List;
        return Some(MediaListCommand::FocusSearch);
      }
      UiEvent::TextInput(TextInputEvent::PressedOutside { id }) if *id == self.search_input => {
        return Some(MediaListCommand::BlurSearch);
      }
      UiEvent::TextInput(TextInputEvent::Cancel { id, .. }) if *id == self.search_input => {
        return Some(MediaListCommand::BlurSearch);
      }
      UiEvent::TextInput(TextInputEvent::Changed { id, value }) if *id == self.search_input => {
        self.search = value.clone();
        self.selected = 0;
        return None;
      }
      UiEvent::HitArea(HitAreaEvent::HoverEnter { id, .. }) => {
        self.handle_pointer_target(*id, false);
        return None;
      }
      UiEvent::HitArea(HitAreaEvent::Click {
        id,
        button: MouseButton::Left,
        ..
      }) => {
        self.handle_pointer_target(*id, true);
        return None;
      }
      UiEvent::HitArea(HitAreaEvent::Press {
        button: MouseButton::Right,
        ..
      }) => {
        if self.zoomed {
          self.zoomed = false;
          return None;
        }
        return Some(MediaListCommand::Back);
      }
      _ => {}
    }
    let UiEvent::Action(event) = event else {
      return None;
    };
    if event.state != KeyState::Pressed {
      return None;
    }
    let prefix = S::NS;
    match event.action.strip_prefix(prefix).unwrap_or_default() {
      ".back" if !self.zoomed => Some(MediaListCommand::Back),
      ".switch" if !self.zoomed => {
        self.active = match self.active {
          ActivePanel::List => ActivePanel::Info,
          ActivePanel::Info => ActivePanel::List,
        };
        None
      }
      ".search" if self.active == ActivePanel::List => Some(MediaListCommand::FocusSearch),
      ".focus_up" if self.active == ActivePanel::List => Some(MediaListCommand::SelectList(-1)),
      ".focus_down" if self.active == ActivePanel::List => Some(MediaListCommand::SelectList(1)),
      ".scroll_up" if self.active == ActivePanel::List => Some(MediaListCommand::ScrollList(-3)),
      ".scroll_down" if self.active == ActivePanel::List => Some(MediaListCommand::ScrollList(3)),
      ".scroll_up" if self.active == ActivePanel::Info => {
        Some(MediaListCommand::ScrollInfo { dx: 0, dy: -3 })
      }
      ".scroll_down" if self.active == ActivePanel::Info => {
        Some(MediaListCommand::ScrollInfo { dx: 0, dy: 3 })
      }
      ".scroll_left" if self.active == ActivePanel::Info => {
        Some(MediaListCommand::ScrollInfo { dx: -3, dy: 0 })
      }
      ".scroll_right" if self.active == ActivePanel::Info => {
        Some(MediaListCommand::ScrollInfo { dx: 3, dy: 0 })
      }
      ".order" if self.active == ActivePanel::List => {
        self.ascending = !self.ascending;
        None
      }
      ".sort" if self.active == ActivePanel::List => {
        self.sort_field = match self.sort_field {
          SortField::Name => SortField::Time,
          SortField::Time if S::SUPPORTS_DURATION => SortField::Duration,
          _ => SortField::Name,
        };
        None
      }
      ".modify" if self.active == ActivePanel::List && !self.filtered_entries().is_empty() => {
        Some(MediaListCommand::BeginRename)
      }
      ".copy" if self.active == ActivePanel::Info => self.screenshot_command(false, false),
      ".copy_rich_text" if self.active == ActivePanel::Info => self.screenshot_command(true, false),
      ".save_image" if self.active == ActivePanel::Info => self.screenshot_command(false, true),
      ".all" if self.active == ActivePanel::Info => self.screenshot_command(true, true),
      ".order" | ".zoom"
        if self.active == ActivePanel::Info && self.selected_preview().is_some() =>
      {
        self.zoomed = !self.zoomed;
        None
      }
      _ => None,
    }
  }

  fn screenshot_command(&self, flag: bool, save: bool) -> Option<MediaListCommand> {
    let (frame, rect) = self.selected_preview()?.frame_and_rect();
    Some(if save {
      MediaListCommand::SaveScreenshot {
        frame,
        rect,
        copy: flag,
      }
    } else {
      MediaListCommand::CopyScreenshot {
        frame,
        rect,
        rich: flag,
      }
    })
  }

  pub fn render(
    &mut self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    i18n: &I18nService,
    hit_area: &HitAreaService,
    text_input: &TextInputService,
    scroll_box: &ScrollBoxService,
  ) -> Option<(u16, u16)> {
    let pos = self.compute_layout(layout, i18n, text_input);
    self.draw_frames(render, canvas, i18n, &pos);

    if self.zoomed {
      hit_area.render_host(&mut self.objects, self.info_panel_area, pos.right, canvas);
      self.draw_info(render, canvas, i18n, &pos);
      self.draw_hints(render, canvas, layout, i18n, text_input, &pos);
      return None;
    }

    hit_area.render_host(&mut self.objects, self.list_panel_area, pos.left, canvas);
    hit_area.render_host(&mut self.objects, self.info_panel_area, pos.right, canvas);

    let cursor = text_input.render_host(
      &mut self.objects,
      self.search_input,
      &TextInputRenderParams {
        rect: pos.search,
        placeholder: i18n.get_runtime_text(S::NS, &format!("{}.list.search.placeholder", S::NS)),
        placeholder_fg: Some(HINT_COLOR.clone()),
        ..Default::default()
      },
      canvas,
    );
    self.draw_sort_bar(render, canvas, i18n, hit_area, &pos);
    self.draw_entries(render, canvas, layout, i18n, hit_area, scroll_box, &pos);
    self.draw_info(render, canvas, i18n, &pos);
    self.draw_hints(render, canvas, layout, i18n, text_input, &pos);
    if self.renaming.is_some() {
      self.draw_rename_dialog(render, canvas, layout, i18n, text_input)
    } else {
      cursor
    }
  }

  fn compute_layout(
    &self,
    layout: &LayoutService,
    i18n: &I18nService,
    text_input: &TextInputService,
  ) -> MediaListLayout {
    let viewport = layout.developer_viewport_rect();
    let hint_lines = self.hint_lines(i18n, text_input, viewport.width);
    let hint_h = hint_lines.len().max(1) as u16;
    let content_h = viewport.height.saturating_sub(hint_h);
    if self.zoomed {
      return MediaListLayout {
        left: Rect {
          x: viewport.x,
          y: viewport.y,
          width: 0,
          height: 0,
        },
        right: Rect {
          x: viewport.x,
          y: viewport.y,
          width: viewport.width,
          height: content_h,
        },
        search: Rect {
          x: 0,
          y: 0,
          width: 0,
          height: 0,
        },
        sort_y: 0,
        list: Rect {
          x: 0,
          y: 0,
          width: 0,
          height: 0,
        },
        hint_y: viewport.y.saturating_add(content_h),
        hint_lines,
      };
    }
    let left_w = viewport.width / 3;
    let left = Rect {
      x: viewport.x,
      y: viewport.y,
      width: left_w,
      height: content_h,
    };
    let right = Rect {
      x: viewport.x.saturating_add(left_w),
      y: viewport.y,
      width: viewport.width.saturating_sub(left_w),
      height: content_h,
    };
    MediaListLayout {
      search: Rect {
        x: left.x.saturating_add(1),
        y: left.y.saturating_add(1),
        width: left.width.saturating_sub(2),
        height: 1,
      },
      sort_y: left.y.saturating_add(2),
      list: Rect {
        x: left.x.saturating_add(1),
        y: left.y.saturating_add(3),
        width: left.width.saturating_sub(2),
        height: left.height.saturating_sub(4),
      },
      left,
      right,
      hint_y: viewport.y.saturating_add(content_h),
      hint_lines,
    }
  }

  fn prepare_scroll_box(
    &mut self,
    service: &ScrollBoxService,
    layout: &LayoutService,
    pos: &MediaListLayout,
  ) {
    let viewport = layout.developer_viewport_rect();
    let rect = Rect {
      x: pos.list.x.saturating_sub(viewport.x),
      y: pos.list.y.saturating_sub(viewport.y),
      width: pos.list.width,
      height: pos.list.height,
    };
    let len = self.filtered_entries().len() as u16;
    let _ = service.set_rect(&mut self.objects, self.list_scroll, rect, layout);
    let _ = service.set_content_size(
      &mut self.objects,
      self.list_scroll,
      rect.width.saturating_sub(1).max(1),
      len.max(rect.height).max(1),
      layout,
    );
    if self.active == ActivePanel::List {
      self.ensure_selection_visible(service, layout);
    }

    let info = self.selected_preview();
    let info_rect = if self.zoomed {
      Rect {
        x: pos.right.x.saturating_sub(viewport.x),
        y: pos.right.y.saturating_sub(viewport.y),
        width: pos.right.width,
        height: pos.right.height,
      }
    } else {
      Rect {
        x: pos.right.x.saturating_add(1).saturating_sub(viewport.x),
        y: pos.right.y.saturating_add(3).saturating_sub(viewport.y),
        width: pos.right.width.saturating_sub(2),
        height: pos.right.height.saturating_sub(4),
      }
    };
    let (content_width, content_height) = info
      .map(|preview| (preview.width, preview.height))
      .unwrap_or((1, 1));
    let _ = service.set_rect(&mut self.objects, self.info_scroll, info_rect, layout);
    let _ = service.set_content_size(
      &mut self.objects,
      self.info_scroll,
      content_width.max(info_rect.width).max(1),
      content_height.max(info_rect.height).max(1),
      layout,
    );
  }

  fn ensure_selection_visible(&mut self, service: &ScrollBoxService, layout: &LayoutService) {
    let height = service
      .visible_content_height(&self.objects, self.list_scroll, layout)
      .unwrap_or(0) as usize;
    if height == 0 {
      return;
    }
    let top = service
      .scroll_y(&self.objects, self.list_scroll)
      .unwrap_or(0) as usize;
    let target = if self.selected < top {
      Some(self.selected)
    } else if self.selected >= top.saturating_add(height) {
      Some(self.selected + 1 - height)
    } else {
      None
    };
    if let Some(y) = target {
      let _ = service.scroll_to(&mut self.objects, self.list_scroll, 0, y as u16, layout);
    }
  }

  fn clamp_selection_to_list_view(&mut self, service: &ScrollBoxService, layout: &LayoutService) {
    let len = self.filtered_entries().len();
    let height = service
      .visible_content_height(&self.objects, self.list_scroll, layout)
      .unwrap_or(0) as usize;
    if len == 0 || height == 0 {
      return;
    }
    let top = service
      .scroll_y(&self.objects, self.list_scroll)
      .unwrap_or(0) as usize;
    self.selected = self.selected.clamp(
      top.min(len - 1),
      top.saturating_add(height - 1).min(len - 1),
    );
  }

  fn draw_frames(
    &self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    i18n: &I18nService,
    pos: &MediaListLayout,
  ) {
    if self.zoomed {
      return;
    }
    for (rect, active) in [
      (pos.left, self.active == ActivePanel::List),
      (pos.right, self.active == ActivePanel::Info),
    ] {
      render.draw_host_border_rect(
        canvas,
        rect.x,
        rect.y,
        rect.width,
        rect.height,
        &BorderStyle::Line,
        Some(if active {
          ACTIVE_BORDER.clone()
        } else {
          TextColor::Terminal(TerminalColor::BrightWhite)
        }),
        None,
        None,
        None,
      );
    }
    for (rect, key) in [
      (pos.left, format!("{}.list", S::NS)),
      (pos.right, format!("{}.info", S::NS)),
    ] {
      render.draw_host_text(
        canvas,
        &DrawTextParams {
          x: rect.x.saturating_add(1),
          y: rect.y,
          text: format!(
            "f%<fg:bright_magenta><b>{}</b></fg>",
            i18n.get_runtime_text(S::NS, &key)
          ),
          max_width: Some(rect.width.saturating_sub(2)),
          ..Default::default()
        },
      );
    }
    if pos.right.width >= 2 && pos.right.height >= 4 {
      let edge = if self.active == ActivePanel::Info {
        "rgb(95,215,105)"
      } else {
        "bright_white"
      };
      render.draw_host_text(
        canvas,
        &DrawTextParams {
          x: pos.right.x,
          y: pos.right.y.saturating_add(2),
          text: format!(
            "f%<fg:{edge}>├{}┤</fg>",
            "─".repeat(pos.right.width.saturating_sub(2) as usize)
          ),
          max_width: Some(pos.right.width),
          max_height: Some(1),
          ..Default::default()
        },
      );
    }
  }

  fn draw_sort_bar(
    &mut self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    i18n: &I18nService,
    hit_area: &HitAreaService,
    pos: &MediaListLayout,
  ) {
    if pos.left.width < 2 {
      return;
    }
    let order = i18n.get_runtime_text(
      S::NS,
      &format!(
        "{}.list.order.{}",
        S::NS,
        if self.ascending {
          "ascending"
        } else {
          "descending"
        }
      ),
    );
    let sort = i18n.get_runtime_text(
      S::NS,
      &format!(
        "{}.list.sort.{}",
        S::NS,
        match self.sort_field {
          SortField::Name => "name",
          SortField::Time => "time",
          SortField::Duration => "duration",
        }
      ),
    );
    let label_width = format!("[{order}]{sort}")
      .width()
      .min(pos.left.width.saturating_sub(2) as usize);
    let line = "─".repeat(pos.left.width.saturating_sub(2 + label_width as u16) as usize);
    let edge = if self.active == ActivePanel::List {
      "rgb(95,215,105)"
    } else {
      "bright_black"
    };
    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: pos.left.x,
        y: pos.sort_y,
        text: format!(
          "f%<fg:{edge}>├[</fg><fg:bright_yellow>{order}</fg><fg:{edge}>]</fg><fg:bright_green>{sort}</fg><fg:{edge}>{line}┤</fg>"
        ),
        max_width: Some(pos.left.width),
        max_height: Some(1),
        ..Default::default()
      },
    );
    let order_width = format!("[{order}]").width() as u16;
    hit_area.render_host(
      &mut self.objects,
      self.order_area,
      Rect {
        x: pos.left.x.saturating_add(1),
        y: pos.sort_y,
        width: order_width.min(pos.left.width.saturating_sub(2)),
        height: 1,
      },
      canvas,
    );
    hit_area.render_host(
      &mut self.objects,
      self.sort_area,
      Rect {
        x: pos.left.x.saturating_add(1).saturating_add(order_width),
        y: pos.sort_y,
        width: sort.width().min(u16::MAX as usize) as u16,
        height: 1,
      },
      canvas,
    );
  }

  fn draw_entries(
    &mut self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    i18n: &I18nService,
    hit_area: &HitAreaService,
    scroll_box: &ScrollBoxService,
    pos: &MediaListLayout,
  ) {
    let entries: Vec<_> = self.filtered_entries().into_iter().cloned().collect();
    self.resize_item_areas(hit_area, entries.len());
    if entries.is_empty() {
      let text = i18n.get_runtime_text(S::NS, &format!("{}.no.image", S::NS));
      render.draw_host_text(
        canvas,
        &DrawTextParams {
          x: pos.list.x
            + pos
              .list
              .width
              .saturating_sub(layout.get_text_width(&text, None))
              / 2,
          y: pos.list.y + pos.list.height / 2,
          text: format!("f%<fg:rgb(85,87,83)>{text}</fg>"),
          ..Default::default()
        },
      );
      return;
    }
    for (index, entry) in entries.iter().enumerate() {
      if index == self.selected {
        render.draw_text_in_scroll_box(
          canvas,
          self.list_scroll,
          &DrawTextParams {
            x: 0,
            y: index as u16,
            text: "f%<fg:bright_cyan>▌</fg>".to_string(),
            ..Default::default()
          },
        );
      }
      render.draw_text_in_scroll_box(
        canvas,
        self.list_scroll,
        &DrawTextParams {
          x: 2,
          y: index as u16,
          text: entry.name.clone(),
          max_width: Some(pos.list.width.saturating_sub(4)),
          max_height: Some(1),
          overflow_marker: Some("...".to_string()),
          ..Default::default()
        },
      );
    }
    let top = scroll_box
      .scroll_y(&self.objects, self.list_scroll)
      .unwrap_or(0) as usize;
    let height = scroll_box
      .visible_content_height(&self.objects, self.list_scroll, layout)
      .unwrap_or(0) as usize;
    for index in top..entries.len().min(top.saturating_add(height)) {
      hit_area.render_host(
        &mut self.objects,
        self.item_areas[index],
        Rect {
          x: pos.list.x,
          y: pos.list.y.saturating_add((index - top) as u16),
          width: pos.list.width.saturating_sub(1),
          height: 1,
        },
        canvas,
      );
    }
  }

  fn selected_preview(&self) -> Option<&ScreenshotPreview> {
    self
      .filtered_entries()
      .get(self.selected)
      .and_then(|entry| entry.preview.as_ref())
  }

  fn draw_info(
    &self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    i18n: &I18nService,
    pos: &MediaListLayout,
  ) {
    let selected = self.filtered_entries().get(self.selected).copied();
    if selected.is_none() || selected.is_some_and(|entry| entry.valid == Some(false)) {
      let text = i18n.get_runtime_text(S::NS, &format!("{}.no.info", S::NS));
      render.draw_host_text(
        canvas,
        &DrawTextParams {
          x: pos
            .right
            .x
            .saturating_add(pos.right.width.saturating_sub(text.width() as u16) / 2),
          y: pos.right.y.saturating_add(pos.right.height / 2),
          text: format!("f%<fg:rgb(85,87,83)>{text}</fg>"),
          max_width: Some(pos.right.width),
          max_height: Some(1),
          ..Default::default()
        },
      );
      return;
    }
    if self.zoomed {
      let Some(preview) = self.selected_preview() else {
        return;
      };
      for cell in &preview.cells {
        canvas.styled_text_in_scroll_box(
          self.info_scroll,
          cell.x,
          cell.y,
          &cell.text,
          cell.style.clone(),
        );
      }
      return;
    }
    let Some(selected) = selected else {
      return;
    };
    let Some(info) = selected.info.as_ref() else {
      return;
    };
    let inner_width = pos.right.width.saturating_sub(2);
    if inner_width == 0 {
      return;
    }
    let timestamp = display_timestamp(&info.timestamp);
    let size = screenshot_size_text(info.width, info.height);
    let frame_rate = S::SUPPORTS_DURATION.then(|| frame_rate_text(info.frame_rate));
    let timestamp_width = timestamp.width().min(inner_width as usize) as u16;
    let size_width = size.width().min(inner_width as usize) as u16;
    let frame_rate_width = frame_rate
      .as_ref()
      .map(|text| text.width().min(inner_width as usize) as u16);
    let header = media_info_header_layout(pos.right, frame_rate_width, size_width, timestamp_width);
    let name = truncate_text(&selected.name, header.name_width);
    let mut fields = vec![(header.name_x, name.as_str(), header.name_width)];
    if let (Some(x), Some(text), Some(width)) =
      (header.frame_rate_x, frame_rate.as_deref(), frame_rate_width)
    {
      fields.push((x, text, width));
    }
    fields.extend([
      (header.size_x, size.as_str(), size_width),
      (header.time_x, timestamp.as_str(), timestamp_width),
    ]);
    for (x, text, width) in fields {
      render.draw_host_text(
        canvas,
        &DrawTextParams {
          x,
          y: pos.right.y.saturating_add(1),
          text: text.to_string(),
          max_width: Some(width),
          max_height: Some(1),
          ..Default::default()
        },
      );
    }
    if let Some(preview) = self.selected_preview() {
      for cell in &preview.cells {
        canvas.styled_text_in_scroll_box(
          self.info_scroll,
          cell.x,
          cell.y,
          &cell.text,
          cell.style.clone(),
        );
      }
    }
  }

  fn draw_hints(
    &self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    _i18n: &I18nService,
    _text_input: &TextInputService,
    pos: &MediaListLayout,
  ) {
    let params = RichTextParams::from_action_map(&S::action_map(), &format!("{}.", S::NS));
    for (index, line) in pos.hint_lines.iter().enumerate() {
      let width = layout.get_text_width(line, Some(&params));
      render.draw_host_text(
        canvas,
        &DrawTextParams {
          x: pos.left.x + (pos.left.width + pos.right.width).saturating_sub(width) / 2,
          y: pos.hint_y.saturating_add(index as u16),
          text: format!("f%<fg:rgb(85,87,83)>{line}</fg>"),
          params: Some(params.clone()),
          ..Default::default()
        },
      );
    }
  }

  fn hint_lines(
    &self,
    i18n: &I18nService,
    text_input: &TextInputService,
    width: u16,
  ) -> Vec<String> {
    let zoom_keys = [
      "action.scroll.info",
      "action.copy",
      "action.copy_rich_text",
      "action.save_image",
      "action.all",
      "action.zoom.out",
    ];
    let keys = if self.zoomed {
      &zoom_keys[..]
    } else if self.renaming.is_some() {
      return vec![String::new()];
    } else if text_input.is_focused(&self.objects, self.search_input) {
      &["action.cancel", "action.confirm"][..]
    } else if self.active == ActivePanel::List {
      S::left_hint_keys()
    } else {
      S::right_hint_keys()
    };
    let params = RichTextParams::from_action_map(&S::action_map(), &format!("{}.", S::NS));
    let rich = RichTextService::new();
    let mut lines = vec![String::new()];
    let mut current_width = 0usize;
    for suffix in keys {
      let key = format!("{}.{}", S::NS, suffix);
      let item = i18n.get_runtime_text(S::NS, &key);
      let item_width = rich.visible_text(&item, Some(&params)).width();
      let gap = usize::from(current_width > 0) * 2;
      if current_width > 0 && current_width + gap + item_width > width as usize {
        lines.push(String::new());
        current_width = 0;
      }
      if current_width > 0 {
        lines.last_mut().unwrap().push_str("  ");
        current_width += 2;
      }
      lines.last_mut().unwrap().push_str(&item);
      current_width += item_width;
    }
    lines
  }

  fn filtered_entries(&self) -> Vec<&MediaEntry> {
    let query = self.search.to_lowercase();
    let mut entries: Vec<_> = self
      .entries
      .iter()
      .filter(|entry| query.is_empty() || entry.name.to_lowercase().contains(&query))
      .collect();
    entries.sort_by(|a, b| {
      let order = match self.sort_field {
        SortField::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        SortField::Time => a.modified.cmp(&b.modified),
        SortField::Duration => a.duration_us.cmp(&b.duration_us),
      };
      if self.ascending {
        order
      } else {
        order.reverse()
      }
    });
    entries
  }

  fn move_selection(&mut self, delta: isize) {
    let len = self.filtered_entries().len();
    if len == 0 {
      self.selected = 0;
      return;
    }
    self.selected = (self.selected as isize + delta).clamp(0, len as isize - 1) as usize;
  }

  fn handle_pointer_target(&mut self, id: HitAreaId, clicked: bool) {
    if id == self.list_panel_area {
      self.active = ActivePanel::List;
    } else if id == self.info_panel_area {
      self.active = ActivePanel::Info;
    } else if id == self.order_area && clicked {
      self.ascending = !self.ascending;
    } else if id == self.sort_area && clicked {
      self.sort_field = match self.sort_field {
        SortField::Name => SortField::Time,
        SortField::Time if S::SUPPORTS_DURATION => SortField::Duration,
        _ => SortField::Name,
      };
    } else if let Some(index) = self.item_areas.iter().position(|area| *area == id) {
      self.active = ActivePanel::List;
      self.selected = index;
    }
  }

  fn resize_item_areas(&mut self, service: &HitAreaService, len: usize) {
    while self.item_areas.len() < len {
      self
        .item_areas
        .push(service.create(&mut self.objects, HitAreaOptions::default()));
    }
    while self.item_areas.len() > len {
      if let Some(id) = self.item_areas.pop() {
        let _ = service.remove(&mut self.objects, id);
      }
    }
  }

  fn rename_error_for(&self, value: &str, original: &str) -> Option<MediaRenameError> {
    if !valid_media_name(value) {
      return Some(MediaRenameError::Invalid);
    }
    if value != original
      && self
        .entries
        .iter()
        .any(|entry| entry.name.eq_ignore_ascii_case(value))
    {
      return Some(MediaRenameError::Duplicate);
    }
    None
  }

  fn draw_rename_dialog(
    &mut self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    i18n: &I18nService,
    text_input: &TextInputService,
  ) -> Option<(u16, u16)> {
    let viewport = layout.developer_viewport_rect();
    let original = self.renaming.clone()?;
    let value = text_input
      .get_text(&self.objects, self.rename_input)
      .unwrap_or_default()
      .to_string();
    let invalid = self.rename_error_for(&value, &original).is_some();
    let border = if invalid {
      TextColor::Terminal(TerminalColor::BrightRed)
    } else {
      TextColor::Terminal(TerminalColor::BrightBlue)
    };
    let params = RichTextParams::from_action_map(&S::action_map(), &format!("{}.", S::NS));
    let actions = format!(
      "{}  {}",
      i18n.get_runtime_text(S::NS, &format!("{}.action.cancel", S::NS)),
      i18n.get_runtime_text(S::NS, &format!("{}.action.confirm", S::NS))
    );
    let placeholder = i18n.get_runtime_text(S::NS, &format!("{}.list.modify.placeholder", S::NS));
    let desired = layout
      .get_text_width(&actions, Some(&params))
      .max(layout.get_text_width(&placeholder, None))
      .max(layout.get_text_width(&value, None))
      .saturating_add(4)
      .max(32);
    let width = desired.min(viewport.width.saturating_sub(4)).max(8);
    let height = 5.min(viewport.height);
    let rect = Rect {
      x: viewport.x + viewport.width.saturating_sub(width) / 2,
      y: viewport.y + viewport.height.saturating_sub(height) / 2,
      width,
      height,
    };
    render.draw_host_filled_rect(
      canvas,
      rect.x,
      rect.y,
      rect.width,
      rect.height,
      Some(" ".to_string()),
      None,
      Some(TextColor::Terminal(TerminalColor::Black)),
    );
    render.draw_host_border_rect(
      canvas,
      rect.x,
      rect.y,
      rect.width,
      rect.height,
      &BorderStyle::Line,
      Some(border.clone()),
      None,
      None,
      None,
    );
    let separator = format!("├{}┤", "─".repeat(rect.width.saturating_sub(2) as usize));
    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: rect.x,
        y: rect.y.saturating_add(2),
        text: separator,
        fg: Some(border),
        max_width: Some(rect.width),
        max_height: Some(1),
        ..Default::default()
      },
    );
    let cursor = text_input.render_host(
      &mut self.objects,
      self.rename_input,
      &TextInputRenderParams {
        rect: Rect {
          x: rect.x.saturating_add(1),
          y: rect.y.saturating_add(1),
          width: rect.width.saturating_sub(2),
          height: 1,
        },
        placeholder,
        placeholder_fg: Some(HINT_COLOR.clone()),
        ..Default::default()
      },
      canvas,
    );
    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: rect.x
          + rect
            .width
            .saturating_sub(layout.get_text_width(&actions, Some(&params)))
            / 2,
        y: rect.y.saturating_add(3),
        text: format!("f%<fg:rgb(85,87,83)>{actions}</fg>"),
        params: Some(params),
        max_width: Some(rect.width.saturating_sub(2)),
        max_height: Some(1),
        ..Default::default()
      },
    );
    cursor
  }
}

fn media_json_path(directory: &Path, name: &str) -> PathBuf {
  directory.join(format!("{name}.json"))
}

fn valid_media_name(name: &str) -> bool {
  if name.is_empty()
    || name != name.trim()
    || matches!(name, "." | "..")
    || name.ends_with('.')
    || name.chars().any(|ch| {
      ch.is_control() || matches!(ch, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*')
    })
  {
    return false;
  }
  let stem = name
    .split('.')
    .next()
    .unwrap_or_default()
    .to_ascii_uppercase();
  !matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
    && !(stem.len() == 4
      && (stem.starts_with("COM") || stem.starts_with("LPT"))
      && matches!(stem.as_bytes()[3], b'1'..=b'9'))
}

fn action(name: &str, key: &str) -> ActionMapEntry {
  ActionMapEntry {
    action: name.to_string(),
    description: name.to_string(),
    keys: vec![vec![key.to_string()]],
  }
}

pub fn actions(entries: &[(&str, &str)]) -> Vec<ActionMapEntry> {
  entries
    .iter()
    .map(|(name, key)| action(name, key))
    .collect()
}

fn read_duration_us(path: &Path) -> Option<u64> {
  let value: serde_json::Value = serde_json::from_slice(&fs::read(path).ok()?).ok()?;
  value.get("duration_us")?.get("active")?.as_u64()
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::host_engine::services::{InputActionEvent, InputEventType};

  struct TestSpec;

  impl MediaListSpec for TestSpec {
    const NS: &'static str = "test";
    const SUPPORTS_DURATION: bool = false;

    fn action_map() -> Vec<ActionMapEntry> {
      actions(&[
        ("test.order", "z"),
        ("test.zoom", "z"),
        ("test.back", "esc"),
        ("test.switch", "tab"),
        ("test.copy", "1"),
      ])
    }

    fn left_hint_keys() -> &'static [&'static str] {
      &[]
    }

    fn right_hint_keys() -> &'static [&'static str] {
      &[]
    }
  }

  fn action_event(action: &str) -> UiEvent {
    UiEvent::Action(InputActionEvent {
      event_type: InputEventType::Keyboard,
      action: action.to_string(),
      state: KeyState::Pressed,
    })
  }

  fn ui_with_preview() -> MediaListUi<TestSpec> {
    let hit_area = HitAreaService::new();
    let text_input = TextInputService::new();
    let scroll_box = ScrollBoxService::new();
    let mut ui = MediaListUi::init(&hit_area, &text_input, &scroll_box);
    ui.entries.push(MediaEntry {
      name: "capture".to_string(),
      path: PathBuf::from("capture.json"),
      modified: SystemTime::UNIX_EPOCH,
      duration_us: 0,
      info: Some(MediaInfo {
        width: 8,
        height: 1,
        timestamp: "2026-07-14T22:23:57.641".to_string(),
        frame_rate: None,
      }),
      preview: Some(ScreenshotPreview {
        width: 8,
        height: 1,
        timestamp: "2026-07-14T22:23:57.641".to_string(),
        cells: Vec::new(),
      }),
      valid: Some(true),
    });
    ui.active = ActivePanel::Info;
    ui
  }

  #[test]
  fn media_names_follow_cross_platform_file_rules() {
    for name in ["capture", "截图 01", "capture.final"] {
      assert!(valid_media_name(name), "expected valid name: {name}");
    }
    for name in [
      "", " capture", "capture ", "a/b", "a:b", "CON", "com1.txt", ".", "..",
    ] {
      assert!(!valid_media_name(name), "expected invalid name: {name}");
    }
  }

  #[test]
  fn screenshot_size_uses_explicit_width_and_height_labels() {
    assert_eq!(screenshot_size_text(8, 1), "w-8 x h-1");
  }

  #[test]
  fn screenshot_size_has_two_cells_of_spacing_on_both_sides() {
    let size_width = screenshot_size_text(8, 1).width() as u16;
    let timestamp_width = "2026.07.14 22:23:57".width() as u16;
    let header = media_info_header_layout(
      Rect {
        x: 10,
        y: 0,
        width: 80,
        height: 3,
      },
      None,
      size_width,
      timestamp_width,
    );

    assert_eq!(header.size_x - (header.name_x + header.name_width), 2);
    assert_eq!(header.time_x - (header.size_x + size_width), 2);
  }

  #[test]
  fn recording_header_separates_each_field_by_two_cells() {
    let frame_rate_width = "FPS 60".width() as u16;
    let size_width = screenshot_size_text(120, 30).width() as u16;
    let timestamp_width = "2026.07.21 20:20:32".width() as u16;
    let header = media_info_header_layout(
      Rect {
        x: 10,
        y: 0,
        width: 100,
        height: 3,
      },
      Some(frame_rate_width),
      size_width,
      timestamp_width,
    );

    let frame_rate_x = header.frame_rate_x.expect("recording has an FPS field");
    assert_eq!(frame_rate_x - (header.name_x + header.name_width), 2);
    assert_eq!(header.size_x - (frame_rate_x + frame_rate_width), 2);
    assert_eq!(header.time_x - (header.size_x + size_width), 2);
  }

  #[test]
  fn recording_frame_rate_uses_the_requested_display_format() {
    assert_eq!(frame_rate_text(Some(60)), "FPS 60");
    assert_eq!(frame_rate_text(None), "FPS --");
  }

  #[test]
  fn recording_info_reads_optional_frame_rate_and_keeps_timestamp() {
    let current = serde_json::json!({
      "started_at": "2026-07-21T20:20:32.895Z",
      "frame_rate": 60,
      "canvas": { "max_width": 120, "max_height": 30 }
    });
    let legacy = serde_json::json!({
      "started_at": "2026-07-20T09:17:05.441Z",
      "canvas": { "max_width": 80, "max_height": 24 }
    });

    assert_eq!(recording_info(&current).unwrap().frame_rate, Some(60));
    assert_eq!(recording_info(&legacy).unwrap().frame_rate, None);
    assert_eq!(
      display_timestamp(&recording_info(&current).unwrap().timestamp),
      "2026.07.21 20:20:32"
    );
  }

  #[test]
  fn shared_z_binding_toggles_zoom_from_the_info_panel() {
    let mut ui = ui_with_preview();

    assert!(ui.handle_event(&action_event("test.order")).is_none());
    assert!(ui.zoomed);

    assert!(ui.handle_event(&action_event("test.order")).is_none());
    assert!(!ui.zoomed);
  }

  #[test]
  fn zoom_mode_ignores_back_and_panel_switch_actions() {
    let mut ui = ui_with_preview();
    ui.zoomed = true;

    assert!(ui.handle_event(&action_event("test.back")).is_none());
    assert!(ui.handle_event(&action_event("test.switch")).is_none());
    assert!(ui.zoomed);
    assert_eq!(ui.active, ActivePanel::Info);
  }

  #[test]
  fn entering_media_list_resets_child_view_state() {
    let mut ui = ui_with_preview();
    let mut text_input = TextInputService::new();
    let scroll_box = ScrollBoxService::new();
    let layout = LayoutService::new();
    ui.zoomed = true;
    ui.ascending = false;
    ui.sort_field = SortField::Time;

    ui.reset_for_entry(&mut text_input, &scroll_box, &layout);

    assert_eq!(ui.active, ActivePanel::List);
    assert!(!ui.zoomed);
    assert!(ui.ascending);
    assert_eq!(ui.sort_field, SortField::Name);
    assert_eq!(ui.selected, 0);
    assert_eq!(
      scroll_box.scroll_position(&ui.objects, ui.list_scroll),
      Some((0, 0))
    );
    assert_eq!(
      scroll_box.scroll_position(&ui.objects, ui.info_scroll),
      Some((0, 0))
    );
  }

  #[test]
  fn zoom_mode_uses_the_full_base_area_above_its_dynamic_hints() {
    let hit_area = HitAreaService::new();
    let text_input = TextInputService::new();
    let scroll_box = ScrollBoxService::new();
    let i18n = I18nService::new();
    let mut layout = LayoutService::new();
    layout.resize_physical(120, 50);
    layout.set_developer_viewport(Rect {
      x: 4,
      y: 2,
      width: 100,
      height: 40,
    });
    let mut ui = MediaListUi::<TestSpec>::init(&hit_area, &text_input, &scroll_box);
    ui.entries.push(MediaEntry {
      name: "capture".to_string(),
      path: PathBuf::from("capture.json"),
      modified: SystemTime::UNIX_EPOCH,
      duration_us: 0,
      info: Some(MediaInfo {
        width: 100,
        height: 50,
        timestamp: String::new(),
        frame_rate: None,
      }),
      preview: Some(ScreenshotPreview {
        width: 100,
        height: 50,
        timestamp: String::new(),
        cells: Vec::new(),
      }),
      valid: Some(true),
    });
    ui.zoomed = true;

    let pos = ui.compute_layout(&layout, &i18n, &text_input);
    ui.prepare_scroll_box(&scroll_box, &layout, &pos);

    assert_eq!((pos.right.x, pos.right.y, pos.right.width), (4, 2, 100));
    assert_eq!(
      pos.right.height + pos.hint_lines.len() as u16,
      layout.developer_height()
    );
    assert_eq!(
      scroll_box.rect(&ui.objects, ui.info_scroll),
      Some(Rect {
        x: 0,
        y: 0,
        width: pos.right.width,
        height: pos.right.height,
      })
    );
    assert_eq!(
      scroll_box.max_scroll_x(&ui.objects, ui.info_scroll, &layout),
      Some(1)
    );
  }

  #[test]
  fn copy_action_rebuilds_the_cached_screenshot_frame() {
    let mut ui = ui_with_preview();

    let Some(MediaListCommand::CopyScreenshot { frame, rect, rich }) =
      ui.handle_event(&action_event("test.copy"))
    else {
      panic!("copy action should emit a screenshot copy command");
    };

    assert!(!rich);
    assert_eq!((frame.width(), frame.height()), (8, 1));
    assert_eq!((rect.width, rect.height), (8, 1));
  }
}
