//! Screenshot service: rasterizes composed terminal frames to PNG/JSON and saves them as async jobs.

use std::{
  collections::{HashMap, HashSet},
  env, fs,
  path::{Path, PathBuf},
};

use chrono::Local;
use crossbeam_channel::Sender;
use harfrust::{
  BufferFlags, Direction as ShapeDirection, FontRef as ShapingFontRef, ShapeOptions, ShaperData,
  UnicodeBuffer,
};
use image::{ImageBuffer, Rgba, RgbaImage};
use serde::{Deserialize, Serialize};
use serde_json::json;
use unicode_bidi::{BidiInfo, Level};
use unicode_script::{Script, UnicodeScript};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use tg_core_atomic_fs::{atomic_replace_with, atomic_write};
use tg_core_log::LogSource;
use tg_core_style::{CanvasCell, ComposedCell, ComposedFrame, TerminalColor, TextColor, TextStyle};
use tg_core_version::MEDIA_MANIFEST_VERSION;
use tg_service_async::TaskCancellation;
use tg_service_async::TaskId;
use tg_service_log::LogService;
use tg_service_storage::{RecordingPixelScale, StorageService};

// Reference profile: Maple Mono NF CN at 12 pt / 144 DPI, measured in Windows Terminal.
const REFERENCE_FONT_SIZE: f32 = 24.0;
const REFERENCE_LINE_HEIGHT: f32 = 31.68;
const REFERENCE_CELL_HEIGHT: u32 = 36;
const REFERENCE_CELL_LEADING: f32 = REFERENCE_CELL_HEIGHT as f32 - REFERENCE_LINE_HEIGHT;

#[derive(Clone, Copy)]
struct RasterMetrics {
  cell_width: f32,
  cell_height: u32,
  font_size: f32,
  baseline: f32,
  scale: f32,
}

impl RasterMetrics {
  fn for_font(font: &fontdue::Font) -> Result<Self, String> {
    let font_name = font.name().unwrap_or("unknown font");
    let line_metrics = font
      .horizontal_line_metrics(REFERENCE_FONT_SIZE)
      .ok_or_else(|| format!("Screenshot font '{font_name}' has no horizontal line metrics"))?;
    if !line_metrics.new_line_size.is_finite() || line_metrics.new_line_size <= 0.0 {
      return Err(format!(
        "Screenshot font '{font_name}' has invalid line height {}",
        line_metrics.new_line_size
      ));
    }

    let font_size = REFERENCE_FONT_SIZE * REFERENCE_LINE_HEIGHT / line_metrics.new_line_size;
    let line_metrics = font
      .horizontal_line_metrics(font_size)
      .ok_or_else(|| format!("Screenshot font '{font_name}' has no horizontal line metrics"))?;
    let cell_width = font.metrics('M', font_size).advance_width;
    if !cell_width.is_finite() || cell_width <= 0.0 {
      return Err(format!(
        "Screenshot font '{font_name}' has invalid monospace advance {cell_width}"
      ));
    }

    Ok(Self {
      cell_width,
      cell_height: REFERENCE_CELL_HEIGHT,
      font_size,
      baseline: line_metrics.ascent + REFERENCE_CELL_LEADING / 2.0,
      scale: 1.0,
    })
  }

  fn for_scale(self, scale: RecordingPixelScale) -> Self {
    let (numerator, denominator) = scale.multiplier();
    let multiplier = numerator as f32 / denominator as f32;
    Self {
      cell_width: (self.cell_width * multiplier).max(1.0),
      cell_height: (self.cell_height * numerator / denominator).max(1),
      font_size: self.font_size * multiplier,
      baseline: self.baseline * multiplier,
      scale: self.scale * multiplier,
    }
  }

  fn cell_x(self, column: u32) -> u32 {
    (column as f32 * self.cell_width).round() as u32
  }

  fn cell_span_width(self, column: u32, cells: u32) -> u32 {
    self.cell_x(column.saturating_add(cells)) - self.cell_x(column)
  }

  fn image_width(self, columns: u16) -> u32 {
    even_dimension((f32::from(columns) * self.cell_width).round() as u32)
  }

  fn image_height(self, rows: u16) -> u32 {
    even_dimension(u32::from(rows) * self.cell_height)
  }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScreenshotRect {
  pub x: u16,
  pub y: u16,
  pub width: u16,
  pub height: u16,
}

#[derive(Clone, Debug)]
pub struct ScreenshotTask {
  pub frame: ComposedFrame,
  pub selection: ScreenshotRect,
  pub png_path: PathBuf,
  pub fonts: Vec<String>,
  pub deployment_root: PathBuf,
}

#[derive(Clone, Debug)]
pub enum ScreenshotAsyncEvent {
  Progress {
    task_id: TaskId,
    completed_rows: u16,
    total_rows: u16,
  },
  Saved {
    task_id: TaskId,
    png_path: PathBuf,
  },
  Failed {
    task_id: TaskId,
    error: String,
  },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScreenshotOperationFeedback {
  pub copy_succeeded: Option<bool>,
  pub save_task: Option<TaskId>,
}

#[derive(Default)]
pub struct ScreenshotService {
  last_presented_frame: Option<ComposedFrame>,
  pending_font_preview: Option<Vec<String>>,
  pending_operation_feedback: Option<ScreenshotOperationFeedback>,
  active_export_sources: HashMap<TaskId, Vec<PathBuf>>,
}

impl ScreenshotService {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn request_font_preview(&mut self, fonts: Vec<String>) {
    self.pending_font_preview = Some(fonts);
  }

  pub fn take_font_preview_request(&mut self) -> Option<Vec<String>> {
    self.pending_font_preview.take()
  }

  pub fn report_operation(&mut self, copy_succeeded: Option<bool>, save_task: Option<TaskId>) {
    self.pending_operation_feedback = Some(ScreenshotOperationFeedback {
      copy_succeeded,
      save_task,
    });
  }

  pub fn take_operation_feedback(&mut self) -> Option<ScreenshotOperationFeedback> {
    self.pending_operation_feedback.take()
  }

  pub fn register_source_export(&mut self, task_id: TaskId, source_path: PathBuf) {
    let sources = self.active_export_sources.entry(task_id).or_default();
    if !sources.contains(&source_path) {
      sources.push(source_path);
    }
  }

  pub fn handle_engine_event(&mut self, event: &ScreenshotAsyncEvent) {
    match event {
      ScreenshotAsyncEvent::Saved { task_id, .. }
      | ScreenshotAsyncEvent::Failed { task_id, .. } => {
        self.active_export_sources.remove(task_id);
      }
      ScreenshotAsyncEvent::Progress { .. } => {}
    }
  }

  pub fn is_source_exporting(&self, path: &Path) -> bool {
    self
      .active_export_sources
      .values()
      .flatten()
      .any(|source| source == path)
  }

  pub fn font_preview_frame() -> ComposedFrame {
    let lines = font_preview_lines();
    let width = lines
      .iter()
      .map(|line| preview_line_width(line))
      .max()
      .unwrap_or(1)
      .saturating_add(4)
      .min(u16::MAX as usize) as u16;
    let height = lines.len().saturating_add(4).min(u16::MAX as usize) as u16;
    let mut frame = ComposedFrame::new(width, height);
    for (index, line) in lines.iter().enumerate() {
      write_preview_line(&mut frame, 2, index as u16 + 2, line);
    }
    frame
  }

  pub fn remember_presented_frame(&mut self, frame: ComposedFrame) {
    self.last_presented_frame = Some(frame);
  }

  pub fn capture_last_frame(&self) -> Option<ComposedFrame> {
    self.last_presented_frame.clone()
  }

  pub fn whole_frame_rect(frame: &ComposedFrame) -> Option<ScreenshotRect> {
    (frame.width() > 0 && frame.height() > 0).then_some(ScreenshotRect {
      x: 0,
      y: 0,
      width: frame.width(),
      height: frame.height(),
    })
  }

  pub fn normalize_selection(
    frame: &ComposedFrame,
    rect: ScreenshotRect,
  ) -> Option<ScreenshotRect> {
    if rect.width == 0 || rect.height == 0 || frame.width() == 0 || frame.height() == 0 {
      return None;
    }
    let mut left = rect.x.min(frame.width().saturating_sub(1));
    let mut top = rect.y.min(frame.height().saturating_sub(1));
    let mut right = rect
      .x
      .saturating_add(rect.width.saturating_sub(1))
      .min(frame.width().saturating_sub(1));
    let mut bottom = rect
      .y
      .saturating_add(rect.height.saturating_sub(1))
      .min(frame.height().saturating_sub(1));

    if left > right {
      std::mem::swap(&mut left, &mut right);
    }
    if top > bottom {
      std::mem::swap(&mut top, &mut bottom);
    }

    for y in top..=bottom {
      if is_continuation(frame, left, y) {
        while left > 0 && is_continuation(frame, left, y) {
          left -= 1;
        }
      }
      let mut x = left;
      while x <= right {
        if let Some(ComposedCell::Text(cell)) = frame.get(x, y)
          && !cell.is_continuation()
        {
          let w = cell.text.width().max(1) as u16;
          right = right.max(x.saturating_add(w.saturating_sub(1)).min(frame.width() - 1));
        }
        x = x.saturating_add(1);
      }
    }

    Some(ScreenshotRect {
      x: left,
      y: top,
      width: right.saturating_sub(left).saturating_add(1),
      height: bottom.saturating_sub(top).saturating_add(1),
    })
  }

  pub fn plain_text(frame: &ComposedFrame, rect: ScreenshotRect) -> String {
    let mut lines = Vec::new();
    for y in rect.y..rect.y.saturating_add(rect.height) {
      let mut line = String::new();
      for x in rect.x..rect.x.saturating_add(rect.width) {
        match frame.get(x, y) {
          Some(ComposedCell::Text(cell)) if cell.is_continuation() => {}
          Some(ComposedCell::Text(cell)) => line.push_str(&cell.text),
          _ => line.push(' '),
        }
      }
      lines.push(line.trim_end().to_string());
    }
    lines.join("\n")
  }

  pub fn rich_text(frame: &ComposedFrame, rect: ScreenshotRect) -> String {
    let mut output = String::from("f%");
    for y in rect.y..rect.y.saturating_add(rect.height) {
      if y != rect.y {
        output.push('\n');
      }
      for x in rect.x..rect.x.saturating_add(rect.width) {
        match frame.get(x, y) {
          Some(ComposedCell::Text(cell)) if cell.is_continuation() => {}
          Some(ComposedCell::Text(cell)) => push_rich_cell(&mut output, cell),
          _ => output.push(' '),
        }
      }
    }
    output
  }

  pub fn write_json(
    &self,
    storage: &StorageService,
    frame: &ComposedFrame,
    rect: ScreenshotRect,
    png_path: Option<&PathBuf>,
    log: &mut LogService,
  ) -> Option<PathBuf> {
    let timestamp = timestamp();
    let path = storage
      .screenshot_cache_dir_path()
      .join(format!("{timestamp}.json"));
    let document = json!({
      "schema_version": MEDIA_MANIFEST_VERSION,
      "timestamp": timestamp,
      "frame": { "width": frame.width(), "height": frame.height() },
      "selection": rect,
      "plain_text": Self::plain_text(frame, rect),
      "png_path": png_path.map(|p| p.to_string_lossy().to_string()),
      "rich_text": rich_text_json(frame, rect),
    });
    if let Err(error) = fs::create_dir_all(storage.screenshot_cache_dir_path()).and_then(|_| {
      atomic_write(
        &path,
        &serde_json::to_vec_pretty(&document).unwrap_or_default(),
        true,
      )
    }) {
      log.warn_operation_failed(
        LogSource::Storage,
        "write_screenshot_record",
        path.display().to_string(),
        error.to_string(),
      );
      return None;
    }
    Some(path)
  }

  pub fn next_png_path(storage: &StorageService) -> PathBuf {
    storage
      .screenshot_dir_path()
      .join(format!("{}.png", timestamp()))
  }
}

fn font_preview_lines() -> &'static [&'static str] {
  &[
    "ASCII: !\"#$%&'()*+,-./ 0123456789 :;<=>?@ ABC xyz [\\]^_` {|}~",
    "Latin: ÀÁÂÃÄÅ Æ Ç ÈÉÊË ÌÍÎÏ Ñ ÒÓÔÕÖ Ø Œ ÙÚÛÜ Ý ß ẞ",
    "Combining: e\u{301} a\u{308} n\u{303} A\u{30a}  ZWJ: 👩‍💻 👨‍👩‍👧‍👦",
    "Zero width: AB A\u{200c}B A\u{200d}B AB  VS: ✈︎ ✈️",
    "RTL: עברית العربية فارسی اردو  | controls: ABC العربية\u{202c}",
    "",
    "CJK: 中文繁體 日本語かなカナ 한글 漢字 〇々〆〄〓〈〉《》「」『』【】",
    "Kana/Bopomofo: あいうえお アイウエオ ｱｲｳｴｵ ㄅㄆㄇㄈ ㆠㆡㆢ",
    "Indic/SEA: हिन्दी বাংলা ਪੰਜਾਬੀ ગુજરાતી தமிழ் తెలుగు ಕನ್ನಡ മലയാളം ไทย ລາວ မြန်မာ",
    "Greek/Cyrillic: ΑΒΓΔ αβγδ  Ελληνικά  АБВГ абвг Русский Українська",
    "Semitic/African: אבגדה العربية ሀሁሂ ትግርኛ ꦗꦮ ꧋ ߒߞߏ",
    "",
    "Symbols: ←↑→↓ ↔↕ ⇐⇒ ∀∂∃∅∇∈∉∑√∞∧∨∩∪≈≠≤≥ ⌘⌥⌫⏎",
    "Box: ─│┌┐└┘├┤┬┴┼ ═║╔╗╚╝╠╣╦╩╬ ╭╮╰╯ ┏┓┗┛┣┫┳┻╋",
    "Blocks: ▀▁▂▃▄▅▆▇█ ▏▎▍▌▋▊▉ ░▒▓ ■□▪▫●○◆◇◢◣◤◥",
    "Braille: ⠀⠁⠃⠇⠏⠟⠿⡿⣿  Music: ♩♪♫♬♭♮♯  Cards: ♠♥♦♣",
    "Emoji: 😀🥹🫠🚀🌍🔥✨⚙️🧪🏳️‍🌈🇨🇳👍🏽  Keycap: 1️⃣ #️⃣ *️⃣",
    "Historic/rare: 𓀀𓂀 𐀀 𐎀 𐤀 ᚠᚢᚦᚨᚱᚲ ⰀⰁ ⸘ ※ ⁂ ‽",
    "",
    "Full/Half width: ＡＢＣ１２３！ ａｂｃ ﾊﾝｶｸ ｡｢｣､･  Tab→\t←Tab",
    "Space widths: [ ] [\u{a0}] [\u{2002}] [\u{2003}] [\u{2009}] [　] end",
  ]
}

fn preview_line_width(line: &str) -> usize {
  let mut column = 0;
  for grapheme in line.graphemes(true) {
    column += if grapheme == "\t" {
      4 - column % 4
    } else {
      UnicodeWidthStr::width(grapheme)
    };
  }
  column
}

fn write_preview_line(frame: &mut ComposedFrame, start_x: u16, y: u16, line: &str) {
  let mut x = start_x as usize;
  for grapheme in line.graphemes(true) {
    if grapheme == "\t" {
      x += 4 - (x - start_x as usize) % 4;
      continue;
    }
    let width = UnicodeWidthStr::width(grapheme);
    if width == 0 || x >= frame.width() as usize {
      continue;
    }
    frame.set(x as u16, y, ComposedCell::Text(CanvasCell::new(grapheme)));
    for offset in 1..width {
      if x + offset < frame.width() as usize {
        frame.set(
          (x + offset) as u16,
          y,
          ComposedCell::Text(CanvasCell::continuation()),
        );
      }
    }
    x += width;
  }
}

fn timestamp() -> String {
  Local::now().format("%Y%m%d_%H%M%S_%3f").to_string()
}

fn is_continuation(frame: &ComposedFrame, x: u16, y: u16) -> bool {
  matches!(frame.get(x, y), Some(ComposedCell::Text(cell)) if cell.is_continuation())
}

fn rich_text_json(frame: &ComposedFrame, rect: ScreenshotRect) -> Vec<Vec<serde_json::Value>> {
  (rect.y..rect.y.saturating_add(rect.height))
    .map(|y| {
      (rect.x..rect.x.saturating_add(rect.width))
        .filter_map(|x| match frame.get(x, y) {
          Some(ComposedCell::Text(cell)) if !cell.is_continuation() => Some(json!({
            "x": x - rect.x,
            "text": cell.text,
            "style": style_json(&cell.style),
          })),
          _ => None,
        })
        .collect()
    })
    .collect()
}

fn push_rich_cell(output: &mut String, cell: &CanvasCell) {
  let tags = style_open_tags(&cell.style);
  if tags.is_empty() {
    output.push_str(&escape_rich_text(&cell.text));
    return;
  }
  for tag in &tags {
    output.push_str(tag);
  }
  output.push_str(&escape_rich_text(&cell.text));
  output.push_str("<reset>");
}

fn style_open_tags(style: &TextStyle) -> Vec<String> {
  let mut tags = Vec::new();
  if let Some(color) = &style.foreground {
    tags.push(format!("<fg:{}>", rich_color_name(color)));
  }
  if let Some(color) = &style.background
    && !matches!(color, TextColor::Transparent)
  {
    tags.push(format!("<bg:{}>", rich_color_name(color)));
  }
  for (enabled, tag) in [
    (style.bold, "b"),
    (style.italic, "i"),
    (style.underline, "u"),
    (style.strike, "s"),
    (style.blink, "l"),
    (style.reverse, "r"),
    (style.hidden, "h"),
    (style.dim, "d"),
  ] {
    if enabled {
      tags.push(format!("<{tag}>"));
    }
  }
  tags
}

fn escape_rich_text(text: &str) -> String {
  let mut output = String::new();
  for ch in text.chars() {
    if matches!(ch, '\\' | '<' | '{') {
      output.push('\\');
    }
    output.push(ch);
  }
  output
}

fn rich_color_name(color: &TextColor) -> String {
  match color {
    TextColor::Terminal(color) => terminal_color_name(color).to_string(),
    TextColor::Rgb { r, g, b } | TextColor::ForceRgb { r, g, b } => {
      format!("#{r:02X}{g:02X}{b:02X}")
    }
    TextColor::Transparent => "transparent".to_string(),
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

fn style_json(style: &TextStyle) -> serde_json::Value {
  json!({
    "fg": style.foreground.as_ref().map(color_name),
    "bg": style.background.as_ref().map(color_name),
    "bold": style.bold,
    "italic": style.italic,
    "underline": style.underline,
    "strike": style.strike,
    "reverse": style.reverse,
    "dim": style.dim,
  })
}

fn color_name(color: &TextColor) -> String {
  match color {
    TextColor::Terminal(color) => format!("{color:?}").to_lowercase(),
    TextColor::Rgb { r, g, b } | TextColor::ForceRgb { r, g, b } => {
      format!("#{r:02X}{g:02X}{b:02X}")
    }
    TextColor::Transparent => "transparent".to_string(),
  }
}

pub fn run_screenshot_task<E: From<ScreenshotAsyncEvent>>(
  task_id: TaskId,
  task: ScreenshotTask,
  event_tx: &Sender<E>,
  cancellation: &TaskCancellation,
) -> Result<(), String> {
  if cancellation.is_cancelled() {
    return Err("screenshot export cancelled".to_string());
  }
  match save_png(task_id, &task, event_tx, cancellation) {
    Ok(()) => {
      let _ = event_tx.send(E::from(ScreenshotAsyncEvent::Saved {
        task_id,
        png_path: task.png_path,
      }));
      Ok(())
    }
    Err(error) => {
      let _ = event_tx.send(E::from(ScreenshotAsyncEvent::Failed {
        task_id,
        error: error.clone(),
      }));
      Err(error)
    }
  }
}

fn save_png<E: From<ScreenshotAsyncEvent>>(
  task_id: TaskId,
  task: &ScreenshotTask,
  event_tx: &Sender<E>,
  cancellation: &TaskCancellation,
) -> Result<(), String> {
  fs::create_dir_all(
    task
      .png_path
      .parent()
      .ok_or("PNG path has no parent directory")?,
  )
  .map_err(|error| error.to_string())?;
  let rasterizer = TerminalFrameRasterizer::load(&task.fonts, &task.deployment_root)?;
  let image = rasterizer.render(
    &task.frame,
    task.selection,
    RecordingPixelScale::Original,
    |completed, total| {
      send_progress(event_tx, task_id, completed, total);
    },
  );
  if cancellation.is_cancelled() {
    return Err("screenshot export cancelled".to_string());
  }

  atomic_replace_with(&task.png_path, true, |temporary| {
    image
      .save_with_format(temporary, image::ImageFormat::Png)
      .map_err(std::io::Error::other)
  })
  .map_err(|error| error.to_string())
}

fn send_progress<E: From<ScreenshotAsyncEvent>>(
  event_tx: &Sender<E>,
  task_id: TaskId,
  completed_rows: u16,
  total_rows: u16,
) {
  let _ = event_tx.send(E::from(ScreenshotAsyncEvent::Progress {
    task_id,
    completed_rows,
    total_rows,
  }));
}

pub struct TerminalFrameRasterizer {
  fonts: FontSet,
  metrics: RasterMetrics,
}

impl TerminalFrameRasterizer {
  pub fn load(preferred: &[String], deployment_root: &Path) -> Result<Self, String> {
    let fonts = FontSet::load(preferred, deployment_root)?;
    let metrics = RasterMetrics::for_font(fonts.primary_font()?)?;
    Ok(Self { fonts, metrics })
  }

  pub fn dimensions(&self, width: u16, height: u16, scale: RecordingPixelScale) -> (u32, u32) {
    let metrics = self.metrics.for_scale(scale);
    (
      metrics.image_width(width).max(1),
      metrics.image_height(height).max(1),
    )
  }

  pub fn render(
    &self,
    frame: &ComposedFrame,
    rect: ScreenshotRect,
    scale: RecordingPixelScale,
    mut progress: impl FnMut(u16, u16),
  ) -> RgbaImage {
    // 字符、样式与颜色一直保留为结构化数据，直到确定最终导出尺寸后，
    // 才按目标单元格和字号直接栅格化，避免先生成低分辨率位图再缩放。
    let metrics = self.metrics.for_scale(scale);
    let width = metrics.image_width(rect.width);
    let height = metrics.image_height(rect.height);
    let mut image = ImageBuffer::from_pixel(width.max(1), height.max(1), Rgba([0, 0, 0, 255]));

    for y in 0..rect.height {
      let spans = shape_row(frame, rect, y, &self.fonts);
      let mut cell_x = 0u32;
      for span in &spans {
        let span_width = metrics.cell_span_width(cell_x, span.cell_width);
        let (fg, bg) = resolved_colors(&span.style);
        fill_rect(
          &mut image,
          metrics.cell_x(cell_x),
          u32::from(y) * metrics.cell_height,
          span_width,
          metrics.cell_height,
          bg,
        );
        if span.style.underline {
          draw_underline_span(&mut image, metrics, cell_x, y, span.cell_width, fg);
        }
        cell_x = cell_x.saturating_add(span.cell_width);
      }

      let mut cell_x = 0u32;
      for span in &spans {
        draw_shaped_span(&mut image, &self.fonts, metrics, cell_x, y, span);
        cell_x = cell_x.saturating_add(span.cell_width);
      }
      progress(
        y.saturating_add(1).saturating_mul(2),
        rect.height.saturating_mul(2),
      );
    }

    image
  }
}

fn even_dimension(value: u32) -> u32 {
  value.saturating_add(value % 2)
}

struct CachedGlyph {
  metrics: fontdue::Metrics,
  bitmap: Vec<u8>,
}

struct LoadedFont {
  raster: fontdue::Font,
  source: FontSource,
  face_index: u32,
  shaper_data: std::cell::RefCell<Option<ShaperData>>,
}

enum FontSource {
  Owned(Vec<u8>),
  Database(fontdb::ID),
}

#[derive(Debug)]
struct ShapedGlyph {
  glyph_id: u16,
  x_advance: f32,
  x_offset: f32,
  y_offset: f32,
}

struct FontSet {
  fonts: Vec<LoadedFont>,
  database: fontdb::Database,
  glyph_cache:
    std::cell::RefCell<std::collections::HashMap<(usize, u16, u32), std::rc::Rc<CachedGlyph>>>,
}

const SYSTEM_FONT_FALLBACK_FAMILIES: &[&str] = &[
  "Cascadia Mono",
  "Cascadia Code",
  "Consolas",
  "JetBrains Mono",
  "DejaVu Sans Mono",
  "Sarasa Mono SC",
  "Noto Sans Mono CJK SC",
  "Noto Sans CJK SC",
  "Microsoft YaHei",
  "Microsoft YaHei UI",
  "Yu Gothic",
  "PingFang SC",
  "Nirmala UI",
  "Noto Sans Devanagari",
  "Noto Sans Arabic",
  "Noto Sans Hebrew",
  "Noto Sans Thai",
  "Segoe UI",
  "Segoe UI Emoji",
  "Segoe UI Symbol",
  "Symbola",
];

impl FontSet {
  fn load(preferred: &[String], deployment_root: &Path) -> Result<Self, String> {
    let mut database = fontdb::Database::new();
    database.load_system_fonts();
    let extra_font_paths = env::var_os("TUI_CAPTURE_FONTS")
      .map(|paths| env::split_paths(&paths).collect::<Vec<_>>())
      .unwrap_or_default();
    Self::load_with_sources(preferred, deployment_root, &extra_font_paths, database)
  }

  fn load_with_sources(
    preferred: &[String],
    deployment_root: &Path,
    extra_font_paths: &[PathBuf],
    database: fontdb::Database,
  ) -> Result<Self, String> {
    let mut fonts = Vec::new();
    let mut attempted = Vec::new();
    let mut loaded_font_files = HashSet::new();
    for value in preferred {
      let path = resolve_font_path(Path::new(value), deployment_root);
      if path.is_file() {
        attempted.push(path.display().to_string());
        if loaded_font_files.insert(path.clone())
          && let Err(error) = load_font_file(&path, &mut fonts)
        {
          attempted.push(error);
        }
      } else if let Some(id) = database.query(&fontdb::Query {
        families: &[fontdb::Family::Name(value)],
        ..fontdb::Query::default()
      }) {
        load_database_font(&database, id, &mut fonts);
      } else {
        attempted.push(format!("font family '{value}' was not found"));
      }
    }

    for path in extra_font_paths {
      let path = resolve_font_path(path, deployment_root);
      if !loaded_font_files.insert(path.clone()) {
        continue;
      }
      attempted.push(path.display().to_string());
      if let Err(error) = load_font_file(&path, &mut fonts) {
        attempted.push(error);
      }
    }

    for path in bundled_font_paths(deployment_root) {
      if path.is_file() {
        if !loaded_font_files.insert(path.clone()) {
          continue;
        }
        if let Err(error) = load_font_file(&path, &mut fonts) {
          attempted.push(error);
        }
      } else {
        attempted.push(format!("bundled font is missing: {}", path.display()));
      }
    }

    let mut ids = Vec::new();
    for family_name in SYSTEM_FONT_FALLBACK_FAMILIES {
      if let Some(id) = database.query(&fontdb::Query {
        families: &[fontdb::Family::Name(family_name)],
        ..fontdb::Query::default()
      }) && !ids.contains(&id)
      {
        ids.push(id);
      }
    }
    for family in [fontdb::Family::Monospace, fontdb::Family::SansSerif] {
      if let Some(id) = database.query(&fontdb::Query {
        families: &[family],
        ..fontdb::Query::default()
      }) && !ids.contains(&id)
      {
        ids.push(id);
      }
    }

    for id in ids.into_iter().take(16) {
      load_database_font(&database, id, &mut fonts);
    }

    if fonts.is_empty() {
      let attempts = if attempted.is_empty() {
        "no preferred, bundled, or system fonts were available".to_string()
      } else {
        attempted.join("; ")
      };
      return Err(format!(
        "No usable screenshot font found after preferred, bundled, and system fallbacks; attempted: {attempts}"
      ));
    }

    Ok(Self {
      fonts,
      database,
      glyph_cache: std::cell::RefCell::new(std::collections::HashMap::new()),
    })
  }

  fn primary_font(&self) -> Result<&fontdue::Font, String> {
    self
      .fonts
      .first()
      .map(|font| &font.raster)
      .ok_or_else(|| "No usable primary screenshot font was loaded".to_string())
  }

  fn glyph_by_index(
    &self,
    font_index: usize,
    glyph_id: u16,
    font_size: f32,
  ) -> Option<std::rc::Rc<CachedGlyph>> {
    let key = (font_index, glyph_id, font_size.to_bits());
    if let Some(cached) = self.glyph_cache.borrow().get(&key) {
      return Some(std::rc::Rc::clone(cached));
    }
    let (metrics, bitmap) = self
      .fonts
      .get(font_index)?
      .raster
      .rasterize_indexed(glyph_id, font_size);
    let cached = std::rc::Rc::new(CachedGlyph { metrics, bitmap });
    self
      .glyph_cache
      .borrow_mut()
      .insert(key, std::rc::Rc::clone(&cached));
    Some(cached)
  }

  fn font_index_for_grapheme(&self, grapheme: &str) -> Option<usize> {
    let characters = grapheme
      .chars()
      .filter(|character| !is_shaping_ignorable(*character))
      .collect::<Vec<_>>();
    if characters.is_empty() {
      return (!self.fonts.is_empty()).then_some(0);
    }

    if let Some(primary) = self.fonts.first()
      && characters
        .iter()
        .all(|character| font_has_character(&primary.raster, *character))
    {
      return Some(0);
    }

    self
      .fonts
      .iter()
      .position(|font| {
        characters
          .iter()
          .all(|character| font_has_character(&font.raster, *character))
      })
      .or_else(|| {
        self
          .fonts
          .iter()
          .enumerate()
          .max_by_key(|(_, font)| {
            characters
              .iter()
              .filter(|character| font_has_character(&font.raster, **character))
              .count()
          })
          .map(|(index, _)| index)
      })
  }

  fn shape_text(
    &self,
    font_index: usize,
    text: &str,
    direction: ShapeDirection,
    font_size: f32,
  ) -> Option<Vec<ShapedGlyph>> {
    let font = self.fonts.get(font_index)?;
    let shape = |data: &[u8], face_index: u32| {
      let shaping_font = ShapingFontRef::from_index(data, face_index).ok()?;
      let mut cache = font.shaper_data.borrow_mut();
      let shaper_data = cache.get_or_insert_with(|| ShaperData::new(&shaping_font));
      let shaper = shaper_data.shaper(&shaping_font).build();
      let mut buffer = UnicodeBuffer::new();
      buffer.push_str(text);
      buffer.set_direction(direction);
      buffer.set_flags(BufferFlags::BEGINNING_OF_TEXT | BufferFlags::END_OF_TEXT);
      buffer.guess_segment_properties();
      let scale = (font_size * 64.0).round() as i32;
      let glyph_buffer = shaper.shape(buffer, ShapeOptions::new().scale(Some(scale)));
      glyph_buffer
        .glyph_infos()
        .iter()
        .zip(glyph_buffer.glyph_positions())
        .map(|(info, position)| {
          Some(ShapedGlyph {
            glyph_id: u16::try_from(info.glyph_id).ok()?,
            x_advance: position.x_advance as f32 / 64.0,
            x_offset: position.x_offset as f32 / 64.0,
            y_offset: position.y_offset as f32 / 64.0,
          })
        })
        .collect()
    };

    match &font.source {
      FontSource::Owned(data) => shape(data, font.face_index),
      FontSource::Database(id) => self
        .database
        .with_face_data(*id, |data, face_index| shape(data, face_index))?,
    }
  }
}

fn font_has_character(font: &fontdue::Font, character: char) -> bool {
  font.lookup_glyph_index(character) != 0
}

fn resolve_font_path(path: &Path, deployment_root: &Path) -> PathBuf {
  if path.is_absolute() {
    path.to_path_buf()
  } else {
    deployment_root.join(path)
  }
}

fn bundled_font_paths(deployment_root: &Path) -> [PathBuf; 4] {
  [
    deployment_root.join("assets/fonts/mmo.ttf"),
    deployment_root.join("assets/fonts/mnf.ttf"),
    deployment_root.join("assets/fonts/asmn.otf"),
    deployment_root.join("assets/fonts/nsscvf.ttf"),
  ]
}

fn load_database_font(database: &fontdb::Database, id: fontdb::ID, fonts: &mut Vec<LoadedFont>) {
  if let Some(Ok(font)) =
    database.with_face_data(id, |data, face_index| -> Result<LoadedFont, String> {
      let raster = fontdue::Font::from_bytes(
        data,
        fontdue::FontSettings {
          collection_index: face_index,
          ..fontdue::FontSettings::default()
        },
      )
      .map_err(|error| error.to_string())?;
      Ok(LoadedFont {
        raster,
        source: FontSource::Database(id),
        face_index,
        shaper_data: std::cell::RefCell::new(None),
      })
    })
  {
    fonts.push(font);
  }
}

fn load_font_file(path: &Path, fonts: &mut Vec<LoadedFont>) -> Result<(), String> {
  let bytes =
    fs::read(path).map_err(|error| format!("Failed to read font {}: {error}", path.display()))?;
  let font = load_font_bytes(bytes, 0)
    .map_err(|error| format!("Failed to parse font {}: {error}", path.display()))?;
  fonts.push(font);
  Ok(())
}

fn load_font_bytes(bytes: Vec<u8>, face_index: u32) -> Result<LoadedFont, String> {
  let raster = fontdue::Font::from_bytes(
    bytes.clone(),
    fontdue::FontSettings {
      collection_index: face_index,
      ..fontdue::FontSettings::default()
    },
  )
  .map_err(|error| error.to_string())?;
  Ok(LoadedFont {
    raster,
    source: FontSource::Owned(bytes),
    face_index,
    shaper_data: std::cell::RefCell::new(None),
  })
}

struct RowTextCell {
  text: String,
  style: TextStyle,
  cell_width: u32,
  byte_start: usize,
  font_index: Option<usize>,
  script: Script,
  level: Level,
  emoji: bool,
}

struct ShapedTextSpan {
  text: String,
  visual_cells: Vec<RowTextCell>,
  style: TextStyle,
  cell_width: u32,
  font_index: Option<usize>,
  direction: ShapeDirection,
  emoji: bool,
}

fn shape_row(
  frame: &ComposedFrame,
  rect: ScreenshotRect,
  y: u16,
  fonts: &FontSet,
) -> Vec<ShapedTextSpan> {
  let mut cells = Vec::new();
  let mut source = String::new();
  for x in 0..rect.width {
    let Some(composed) = frame.get(rect.x + x, rect.y + y) else {
      continue;
    };
    let (text, style) = match composed {
      ComposedCell::Empty => (" ".to_string(), TextStyle::default()),
      ComposedCell::Text(cell) if cell.is_continuation() => continue,
      ComposedCell::Text(cell) => {
        let text = if cell.text.is_empty() {
          " ".to_string()
        } else {
          cell.text.clone()
        };
        (text, cell.style.clone())
      }
    };
    let byte_start = source.len();
    source.push_str(&text);
    let cell_width = UnicodeWidthStr::width(text.as_str())
      .max(1)
      .min(u32::MAX as usize) as u32;
    let script = text
      .chars()
      .map(|character| character.script())
      .find(|script| !matches!(script, Script::Common | Script::Inherited | Script::Unknown))
      .unwrap_or(Script::Common);
    cells.push(RowTextCell {
      font_index: fonts.font_index_for_grapheme(&text),
      emoji: text.chars().any(is_probably_emoji),
      text,
      style,
      cell_width,
      byte_start,
      script,
      level: Level::ltr(),
    });
  }

  if cells.is_empty() {
    return Vec::new();
  }
  let bidi = BidiInfo::new(&source, None);
  let Some(paragraph) = bidi.paragraphs.first() else {
    return Vec::new();
  };
  let levels = bidi.reordered_levels(paragraph, paragraph.range.clone());
  let mut inherited_script = Script::Common;
  for cell in &mut cells {
    cell.level = levels[cell.byte_start];
    if matches!(
      cell.script,
      Script::Common | Script::Inherited | Script::Unknown
    ) {
      cell.script = inherited_script;
    } else {
      inherited_script = cell.script;
    }
  }

  let visual_order =
    BidiInfo::reorder_visual(&cells.iter().map(|cell| cell.level).collect::<Vec<_>>());
  let mut spans = Vec::new();
  let mut cursor = 0;
  while cursor < visual_order.len() {
    let first = &cells[visual_order[cursor]];
    let key_level = first.level;
    let key_script = first.script;
    let key_font = first.font_index;
    let key_emoji = first.emoji;
    let key_style = first.style.clone();
    let mut end = cursor + 1;
    while end < visual_order.len() {
      let cell = &cells[visual_order[end]];
      if cell.level != key_level
        || cell.script != key_script
        || cell.font_index != key_font
        || cell.emoji != key_emoji
        || cell.style != key_style
      {
        break;
      }
      end += 1;
    }

    let mut visual_cells = visual_order[cursor..end]
      .iter()
      .map(|index| cells[*index].clone_for_span())
      .collect::<Vec<_>>();
    let direction = if key_level.is_rtl() {
      ShapeDirection::RightToLeft
    } else {
      ShapeDirection::LeftToRight
    };
    let mut logical_cells = visual_cells.iter().collect::<Vec<_>>();
    if key_level.is_rtl() {
      logical_cells.reverse();
    }
    let text = logical_cells
      .into_iter()
      .map(|cell| cell.text.as_str())
      .collect::<String>();
    let cell_width = visual_cells
      .iter()
      .fold(0u32, |width, cell| width.saturating_add(cell.cell_width));
    spans.push(ShapedTextSpan {
      text,
      visual_cells: std::mem::take(&mut visual_cells),
      style: key_style,
      cell_width,
      font_index: key_font,
      direction,
      emoji: key_emoji,
    });
    cursor = end;
  }
  spans
}

impl RowTextCell {
  fn clone_for_span(&self) -> Self {
    Self {
      text: self.text.clone(),
      style: self.style.clone(),
      cell_width: self.cell_width,
      byte_start: self.byte_start,
      font_index: self.font_index,
      script: self.script,
      level: self.level,
      emoji: self.emoji,
    }
  }
}

fn draw_shaped_span(
  image: &mut RgbaImage,
  fonts: &FontSet,
  metrics: RasterMetrics,
  origin_cell_x: u32,
  y: u16,
  span: &ShapedTextSpan,
) {
  if span.style.hidden {
    return;
  }
  let Some(font_index) = span.font_index else {
    draw_fallback_span(image, fonts, metrics, origin_cell_x, y, span);
    return;
  };
  let font_size = metrics.font_size * if span.emoji { 0.86 } else { 1.0 };
  let Some(glyphs) = fonts.shape_text(font_index, &span.text, span.direction, font_size) else {
    draw_fallback_span(image, fonts, metrics, origin_cell_x, y, span);
    return;
  };
  if glyphs.is_empty() {
    return;
  }

  let origin_x = metrics.cell_x(origin_cell_x);
  let span_width = metrics.cell_span_width(origin_cell_x, span.cell_width);
  let clip_right = origin_x.saturating_add(span_width).min(image.width());
  let baseline = u32::from(y) * metrics.cell_height + metrics.baseline.round() as u32;
  let (fg, _) = resolved_colors(&span.style);
  let natural_advance: f32 = glyphs.iter().map(|glyph| glyph.x_advance).sum();
  let horizontal_scale = if natural_advance > 0.0 {
    span_width as f32 / natural_advance
  } else {
    1.0
  };
  let mut pen_x = origin_x as f32;
  let clip_top = u32::from(y) * metrics.cell_height;
  let clip_bottom = clip_top
    .saturating_add(metrics.cell_height)
    .min(image.height());
  for glyph in glyphs {
    let Some(rasterized) = fonts.glyph_by_index(font_index, glyph.glyph_id, font_size) else {
      pen_x += glyph.x_advance * horizontal_scale;
      continue;
    };
    let destination_x =
      (pen_x + glyph.x_offset * horizontal_scale).round() as i32 + rasterized.metrics.xmin;
    let glyph_baseline = baseline as f32 - glyph.y_offset;
    let destination_y =
      glyph_baseline.round() as i32 - rasterized.metrics.height as i32 - rasterized.metrics.ymin;
    draw_glyph_bitmap(
      image,
      &rasterized.bitmap,
      rasterized.metrics.width,
      rasterized.metrics.height,
      destination_x,
      destination_y,
      origin_x,
      clip_top,
      clip_right,
      clip_bottom,
      fg,
    );
    if span.style.bold {
      draw_glyph_bitmap(
        image,
        &rasterized.bitmap,
        rasterized.metrics.width,
        rasterized.metrics.height,
        destination_x + 1,
        destination_y,
        origin_x,
        clip_top,
        clip_right,
        clip_bottom,
        fg,
      );
    }
    pen_x += glyph.x_advance * horizontal_scale;
  }
}

fn draw_fallback_span(
  image: &mut RgbaImage,
  fonts: &FontSet,
  metrics: RasterMetrics,
  origin_cell_x: u32,
  y: u16,
  span: &ShapedTextSpan,
) {
  let (fg, _) = resolved_colors(&span.style);
  let mut cell_x = origin_cell_x;
  for cell in &span.visual_cells {
    draw_grapheme(
      image,
      fonts,
      metrics,
      &cell.text,
      cell_x,
      u32::from(y) * metrics.cell_height,
      cell.cell_width,
      &span.style,
      fg,
    );
    cell_x = cell_x.saturating_add(cell.cell_width);
  }
}

fn draw_underline_span(
  image: &mut RgbaImage,
  metrics: RasterMetrics,
  x: u32,
  y: u16,
  width_cells: u32,
  color: (u8, u8, u8),
) {
  let px = metrics.cell_x(x);
  let py = u32::from(y) * metrics.cell_height;
  let width = metrics.cell_span_width(x, width_cells);
  let thickness = metrics.scale.round().max(1.0) as u32;
  let offset = (4.0 * metrics.scale).round() as u32;
  let underline_y = py + metrics.cell_height.saturating_sub(offset);
  for yy in underline_y..underline_y.saturating_add(thickness) {
    for xx in px..px.saturating_add(width).min(image.width()) {
      composite_pixel(
        image,
        xx,
        yy.min(image.height().saturating_sub(1)),
        color,
        255,
      );
    }
  }
}

#[allow(clippy::too_many_arguments)]
fn draw_grapheme(
  image: &mut RgbaImage,
  fonts: &FontSet,
  metrics: RasterMetrics,
  grapheme: &str,
  origin_cell_x: u32,
  origin_y: u32,
  span_cells: u32,
  style: &TextStyle,
  fg: (u8, u8, u8),
) {
  let origin_x = metrics.cell_x(origin_cell_x);
  let span_width = metrics.cell_span_width(origin_cell_x, span_cells);
  if let Some(character) = grapheme.chars().next()
    && grapheme.chars().count() == 1
    && draw_block_element(
      image,
      character,
      origin_x,
      origin_y,
      span_width,
      metrics.cell_height,
      fg,
    )
  {
    return;
  }
  let visible_width_sum: usize = grapheme
    .chars()
    .map(|character| UnicodeWidthChar::width(character).unwrap_or(0))
    .sum();
  let complex_cluster = visible_width_sum > UnicodeWidthStr::width(grapheme);
  let complex_base_count = if complex_cluster {
    grapheme
      .chars()
      .filter(|character| UnicodeWidthChar::width(*character).unwrap_or(0) > 0)
      .count()
      .max(1)
  } else {
    0
  };
  let font_index = fonts.font_index_for_grapheme(grapheme);
  let mut pen_cell_x = origin_cell_x;
  let mut pen_x = origin_x;
  let mut last_base_origin_x = origin_x as i32;
  let mut complex_base_index = 0usize;
  let clip_left = origin_x;
  let clip_right = (origin_x + span_width).min(image.width());
  let clip_top = origin_y;
  let clip_bottom = (origin_y + metrics.cell_height).min(image.height());

  for character in grapheme.chars() {
    if character == '\u{200d}' || character == '\u{fe0f}' {
      continue;
    }
    let char_width = UnicodeWidthChar::width(character).unwrap_or(0).min(2);

    let font_size = if is_probably_emoji(character) {
      metrics.font_size * 0.86
    } else {
      metrics.font_size
    };
    let Some(font_index) = font_index else {
      continue;
    };
    let glyph_id = fonts.fonts[font_index].raster.lookup_glyph_index(character);
    let Some(glyph) = fonts.glyph_by_index(font_index, glyph_id, font_size) else {
      continue;
    };
    let glyph_metrics = &glyph.metrics;
    let bitmap = &glyph.bitmap;
    let allocated_width = if complex_cluster && char_width > 0 {
      let component_left = span_width as usize * complex_base_index / complex_base_count;
      let component_right = span_width as usize * (complex_base_index + 1) / complex_base_count;
      complex_base_index += 1;
      (component_right - component_left) as u32
    } else if char_width == 0 {
      span_width
    } else {
      metrics
        .cell_span_width(pen_cell_x, char_width as u32)
        .min(span_width)
    };
    let glyph_origin_x = if complex_cluster && char_width > 0 {
      let component_left =
        (span_width as usize * (complex_base_index - 1) / complex_base_count) as i32;
      origin_x as i32
        + component_left
        + ((allocated_width as f32 - glyph_metrics.advance_width) / 2.0).round() as i32
    } else if char_width == 0 {
      last_base_origin_x
    } else {
      let origin = pen_x as i32
        + ((allocated_width as f32 - glyph_metrics.advance_width) / 2.0).round() as i32;
      last_base_origin_x = origin;
      origin
    };

    let destination_x = glyph_origin_x + glyph_metrics.xmin;
    let baseline = origin_y as i32 + metrics.baseline.round() as i32;
    let top = baseline - glyph_metrics.height as i32 - glyph_metrics.ymin;
    draw_glyph_bitmap(
      image,
      bitmap,
      glyph_metrics.width,
      glyph_metrics.height,
      destination_x,
      top,
      clip_left,
      clip_top,
      clip_right,
      clip_bottom,
      fg,
    );
    if style.bold {
      draw_glyph_bitmap(
        image,
        bitmap,
        glyph_metrics.width,
        glyph_metrics.height,
        destination_x + 1,
        top,
        clip_left,
        clip_top,
        clip_right,
        clip_bottom,
        fg,
      );
    }

    if char_width > 0 && !complex_cluster {
      pen_cell_x = pen_cell_x.saturating_add(char_width as u32);
      pen_x = metrics.cell_x(pen_cell_x);
    }
  }

  if let Some(character) = grapheme.chars().next()
    && grapheme.chars().count() == 1
    && let Some(connections) = box_connections(character)
  {
    draw_box_connections(
      image,
      metrics,
      origin_x,
      origin_y,
      span_width,
      fg,
      connections,
    );
  }
}

fn draw_block_element(
  image: &mut RgbaImage,
  character: char,
  x: u32,
  y: u32,
  width: u32,
  height: u32,
  color: (u8, u8, u8),
) -> bool {
  let eighth_w = width.div_ceil(8);
  let eighth_h = height.div_ceil(8);
  let rects: &[(u32, u32, u32, u32)] = match character {
    '█' => &[(0, 0, 8, 8)],
    '▀' => &[(0, 0, 8, 4)],
    '▄' => &[(0, 4, 8, 4)],
    '▌' => &[(0, 0, 4, 8)],
    '▐' => &[(4, 0, 4, 8)],
    '▁' => &[(0, 7, 8, 1)],
    '▂' => &[(0, 6, 8, 2)],
    '▃' => &[(0, 5, 8, 3)],
    '▅' => &[(0, 3, 8, 5)],
    '▆' => &[(0, 2, 8, 6)],
    '▇' => &[(0, 1, 8, 7)],
    '▉' => &[(0, 0, 7, 8)],
    '▊' => &[(0, 0, 6, 8)],
    '▋' => &[(0, 0, 5, 8)],
    '▍' => &[(0, 0, 3, 8)],
    '▎' => &[(0, 0, 2, 8)],
    '▏' => &[(0, 0, 1, 8)],
    '▔' => &[(0, 0, 8, 1)],
    '▕' => &[(7, 0, 1, 8)],
    '▖' => &[(0, 4, 4, 4)],
    '▗' => &[(4, 4, 4, 4)],
    '▘' => &[(0, 0, 4, 4)],
    '▙' => &[(0, 0, 4, 8), (4, 4, 4, 4)],
    '▚' => &[(0, 0, 4, 4), (4, 4, 4, 4)],
    '▛' => &[(0, 0, 4, 8), (4, 0, 4, 4)],
    '▜' => &[(0, 0, 8, 4), (4, 4, 4, 4)],
    '▝' => &[(4, 0, 4, 4)],
    '▞' => &[(4, 0, 4, 4), (0, 4, 4, 4)],
    '▟' => &[(4, 0, 4, 8), (0, 4, 4, 4)],
    _ => return false,
  };
  for &(rx, ry, rw, rh) in rects {
    let left = x.saturating_add(rx * eighth_w).min(x + width);
    let top = y.saturating_add(ry * eighth_h).min(y + height);
    let right = if rx + rw == 8 {
      x + width
    } else {
      x.saturating_add((rx + rw) * eighth_w).min(x + width)
    };
    let bottom = if ry + rh == 8 {
      y + height
    } else {
      y.saturating_add((ry + rh) * eighth_h).min(y + height)
    };
    fill_rect(
      image,
      left,
      top,
      right.saturating_sub(left),
      bottom.saturating_sub(top),
      color,
    );
  }
  true
}

#[derive(Clone, Copy)]
struct BoxConnections {
  left: bool,
  right: bool,
  up: bool,
  down: bool,
}

fn box_connections(character: char) -> Option<BoxConnections> {
  let code = character as u32;
  let directions = match code {
    0x2500..=0x2501 | 0x2504..=0x2505 | 0x2508..=0x2509 | 0x254c..=0x254d | 0x257c | 0x257e => {
      (true, true, false, false)
    }
    0x2502..=0x2503 | 0x2506..=0x2507 | 0x250a..=0x250b | 0x254e..=0x254f | 0x257d | 0x257f => {
      (false, false, true, true)
    }
    0x250c..=0x250f | 0x256d => (false, true, false, true),
    0x2510..=0x2513 | 0x256e => (true, false, false, true),
    0x2514..=0x2517 | 0x2570 => (false, true, true, false),
    0x2518..=0x251b | 0x256f => (true, false, true, false),
    0x251c..=0x2523 => (false, true, true, true),
    0x2524..=0x252b => (true, false, true, true),
    0x252c..=0x2533 => (true, true, false, true),
    0x2534..=0x253b => (true, true, true, false),
    0x253c..=0x254b => (true, true, true, true),
    0x2574 | 0x2578 => (true, false, false, false),
    0x2575 | 0x2579 => (false, false, true, false),
    0x2576 | 0x257a => (false, true, false, false),
    0x2577 | 0x257b => (false, false, false, true),
    _ => return None,
  };
  Some(BoxConnections {
    left: directions.0,
    right: directions.1,
    up: directions.2,
    down: directions.3,
  })
}

fn draw_box_connections(
  image: &mut RgbaImage,
  metrics: RasterMetrics,
  x: u32,
  y: u32,
  width: u32,
  color: (u8, u8, u8),
  connections: BoxConnections,
) {
  let center_x = x.saturating_add(width / 2);
  let center_y = y.saturating_add(metrics.cell_height / 2);
  let thickness = (metrics.cell_width / 9.0).round().max(1.0) as u32;
  if connections.left {
    fill_rect(image, x, center_y, width / 2 + 1, thickness, color);
  }
  if connections.right {
    fill_rect(
      image,
      center_x,
      center_y,
      x.saturating_add(width).saturating_sub(center_x),
      thickness,
      color,
    );
  }
  if connections.up {
    fill_rect(
      image,
      center_x,
      y,
      thickness,
      metrics.cell_height / 2 + 1,
      color,
    );
  }
  if connections.down {
    fill_rect(
      image,
      center_x,
      center_y,
      thickness,
      y.saturating_add(metrics.cell_height)
        .saturating_sub(center_y),
      color,
    );
  }
}

#[allow(clippy::too_many_arguments)]
fn draw_glyph_bitmap(
  image: &mut RgbaImage,
  bitmap: &[u8],
  bitmap_width: usize,
  bitmap_height: usize,
  destination_x: i32,
  destination_y: i32,
  clip_left: u32,
  clip_top: u32,
  clip_right: u32,
  clip_bottom: u32,
  color: (u8, u8, u8),
) {
  for source_y in 0..bitmap_height {
    for source_x in 0..bitmap_width {
      let coverage = bitmap[source_y * bitmap_width + source_x];
      if coverage == 0 {
        continue;
      }
      let x = destination_x + source_x as i32;
      let y = destination_y + source_y as i32;
      if x < 0 || y < 0 {
        continue;
      }
      let x = x as u32;
      let y = y as u32;
      if x < clip_left
        || x >= clip_right
        || y < clip_top
        || y >= clip_bottom
        || x >= image.width()
        || y >= image.height()
      {
        continue;
      }
      composite_pixel(image, x, y, color, coverage);
    }
  }
}

fn composite_pixel(image: &mut RgbaImage, x: u32, y: u32, color: (u8, u8, u8), coverage: u8) {
  let destination = image.get_pixel(x, y).0;
  let source_alpha = f32::from(coverage) / 255.0;
  let destination_alpha = f32::from(destination[3]) / 255.0;
  let output_alpha = source_alpha + destination_alpha * (1.0 - source_alpha);
  if output_alpha <= f32::EPSILON {
    image.put_pixel(x, y, Rgba([0, 0, 0, 0]));
    return;
  }

  let blend_channel = |source: u8, destination: u8| -> u8 {
    let source = f32::from(source) / 255.0;
    let destination = f32::from(destination) / 255.0;
    let output = (source * source_alpha + destination * destination_alpha * (1.0 - source_alpha))
      / output_alpha;
    (output.clamp(0.0, 1.0) * 255.0).round() as u8
  };
  image.put_pixel(
    x,
    y,
    Rgba([
      blend_channel(color.0, destination[0]),
      blend_channel(color.1, destination[1]),
      blend_channel(color.2, destination[2]),
      (output_alpha.clamp(0.0, 1.0) * 255.0).round() as u8,
    ]),
  );
}

fn is_probably_emoji(character: char) -> bool {
  matches!(
    character as u32,
    0x1F000..=0x1FAFF | 0x2600..=0x27BF | 0x2300..=0x23FF
  )
}

fn is_shaping_ignorable(character: char) -> bool {
  matches!(
    character as u32,
    0x200C..=0x200D | 0xFE00..=0xFE0F | 0xE0020..=0xE007F | 0xE0100..=0xE01EF
  )
}

fn fill_rect(
  image: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
  x: u32,
  y: u32,
  width: u32,
  height: u32,
  color: (u8, u8, u8),
) {
  let image_width = image.width();
  let image_height = image.height();
  let right = x.saturating_add(width).min(image_width);
  let bottom = y.saturating_add(height).min(image_height);
  if right <= x || bottom <= y {
    return;
  }
  let pixel = [color.0, color.1, color.2, 255u8];
  let mut samples = image.as_flat_samples_mut();
  let raw = samples.as_mut_slice::<u8>();
  for yy in y..bottom {
    let row_start = (yy * image_width + x) as usize;
    let row_end = (yy * image_width + right) as usize;
    for chunk in raw[row_start * 4..row_end * 4].chunks_exact_mut(4) {
      chunk.copy_from_slice(&pixel);
    }
  }
}

fn resolved_colors(style: &TextStyle) -> ((u8, u8, u8), (u8, u8, u8)) {
  let mut fg = style
    .foreground
    .as_ref()
    .map(color_rgb)
    .unwrap_or((222, 214, 207));
  let mut bg = style
    .background
    .as_ref()
    .map(color_rgb)
    .unwrap_or((0, 0, 0));
  if style.reverse {
    std::mem::swap(&mut fg, &mut bg);
  }
  (fg, bg)
}

fn color_rgb(color: &TextColor) -> (u8, u8, u8) {
  match color {
    TextColor::Rgb { r, g, b } | TextColor::ForceRgb { r, g, b } => (*r, *g, *b),
    TextColor::Transparent => (0, 0, 0),
    TextColor::Terminal(color) => terminal_rgb(color),
  }
}

fn terminal_rgb(color: &TerminalColor) -> (u8, u8, u8) {
  match color {
    TerminalColor::Black => (0, 0, 0),
    TerminalColor::Red => (170, 0, 0),
    TerminalColor::Green => (0, 170, 0),
    TerminalColor::Yellow => (170, 170, 0),
    TerminalColor::Blue => (0, 0, 170),
    TerminalColor::Magenta => (170, 0, 170),
    TerminalColor::Cyan => (0, 170, 170),
    TerminalColor::White => (222, 214, 207),
    TerminalColor::BrightBlack => (85, 87, 83),
    TerminalColor::BrightRed => (255, 85, 85),
    TerminalColor::BrightGreen => (85, 255, 85),
    TerminalColor::BrightYellow => (255, 255, 85),
    TerminalColor::BrightBlue => (85, 85, 255),
    TerminalColor::BrightMagenta => (255, 85, 255),
    TerminalColor::BrightCyan => (85, 255, 255),
    TerminalColor::BrightWhite => (255, 255, 255),
  }
}

impl<E: From<ScreenshotAsyncEvent> + Send + 'static> tg_service_async::AsyncJob<E>
  for ScreenshotTask
{
  fn run(
    self: Box<Self>,
    id: tg_service_async::TaskId,
    events: &crossbeam_channel::Sender<E>,
    cancellation: &tg_service_async::TaskCancellation,
  ) -> Result<(), String> {
    run_screenshot_task(id, *self, events, cancellation)
  }

  fn write_target(&self, _id: tg_service_async::TaskId) -> Option<(PathBuf, PathBuf)> {
    let temporary = tg_core_atomic_fs::temporary_path(&self.png_path);
    Some((self.png_path.clone(), temporary))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use tg_service_async::TaskId;

  #[test]
  fn font_preview_contains_representative_unicode_groups() {
    let frame = ScreenshotService::font_preview_frame();
    let rect = ScreenshotService::whole_frame_rect(&frame).unwrap();
    let text = ScreenshotService::plain_text(&frame, rect);
    assert!(text.contains("中文繁體"));
    assert!(text.contains("👩‍💻"));
    assert!(text.contains("עברית"));
    assert!(text.contains("▀▁▂▃▄"));
    assert!(frame.width() > 60);
    assert!(frame.height() > 20);
  }

  #[test]
  fn invalid_preferred_font_falls_back_to_a_bundled_font_under_the_deployment_root() {
    let root = std::env::temp_dir().join(format!(
      "tui-font-root-{}-{}",
      std::process::id(),
      timestamp()
    ));
    let fonts_dir = root.join("assets/fonts");
    std::fs::create_dir_all(&fonts_dir).unwrap();
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../assets/fonts/mnf.ttf");
    std::fs::copy(&fixture, fonts_dir.join("mnf.ttf")).unwrap();
    let invalid_font = root.join("custom-invalid.ttf");
    std::fs::write(&invalid_font, b"not a font").unwrap();
    let preferred = vec![invalid_font.to_string_lossy().into_owned()];

    let fonts =
      FontSet::load_with_sources(&preferred, &root, &[], fontdb::Database::new()).unwrap();

    assert!(!fonts.fonts.is_empty());
    std::fs::remove_dir_all(root).unwrap();
  }

  #[test]
  fn relative_preferred_font_path_is_resolved_from_the_deployment_root() {
    let root = std::env::temp_dir().join(format!(
      "tui-font-relative-{}-{}",
      std::process::id(),
      timestamp()
    ));
    let fonts_dir = root.join("assets/fonts");
    std::fs::create_dir_all(&fonts_dir).unwrap();
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../assets/fonts/mnf.ttf");
    std::fs::copy(&fixture, fonts_dir.join("custom.ttf")).unwrap();
    let preferred = vec!["assets/fonts/custom.ttf".to_string()];

    let fonts =
      FontSet::load_with_sources(&preferred, &root, &[], fontdb::Database::new()).unwrap();

    assert!(!fonts.fonts.is_empty());
    std::fs::remove_dir_all(root).unwrap();
  }

  #[test]
  fn font_failure_reports_attempted_deployment_font_paths() {
    let root = std::env::temp_dir().join(format!(
      "tui-font-missing-{}-{}",
      std::process::id(),
      timestamp()
    ));
    std::fs::create_dir_all(root.join("assets/fonts")).unwrap();

    let error = match FontSet::load_with_sources(&[], &root, &[], fontdb::Database::new()) {
      Ok(_) => panic!("font loading succeeded without any available source"),
      Err(error) => error,
    };

    assert!(error.contains(&root.join("assets/fonts/mnf.ttf").display().to_string()));
    std::fs::remove_dir_all(root).unwrap();
  }

  #[test]
  fn font_preview_request_is_consumed_once() {
    let mut service = ScreenshotService::new();
    service.request_font_preview(vec!["test.ttf".to_string()]);
    assert_eq!(
      service.take_font_preview_request(),
      Some(vec!["test.ttf".to_string()])
    );
    assert_eq!(service.take_font_preview_request(), None);
  }

  #[test]
  fn operation_feedback_is_consumed_once() {
    let mut service = ScreenshotService::new();
    service.report_operation(Some(true), Some(TaskId(7)));

    let feedback = service.take_operation_feedback().unwrap();
    assert_eq!(feedback.copy_succeeded, Some(true));
    assert_eq!(feedback.save_task, Some(TaskId(7)));
    assert_eq!(service.take_operation_feedback(), None);
  }

  #[test]
  fn source_export_lock_is_removed_on_terminal_event() {
    let mut service = ScreenshotService::new();
    let path = PathBuf::from("data/screenshot/cache/example.json");
    service.register_source_export(TaskId(8), path.clone());
    assert!(service.is_source_exporting(&path));
    service.handle_engine_event(&ScreenshotAsyncEvent::Failed {
      task_id: TaskId(8),
      error: "test".to_string(),
    });
    assert!(!service.is_source_exporting(&path));
  }

  #[test]
  fn full_block_fills_the_entire_export_cell_without_font_margins() {
    let rasterizer = maple_rasterizer();
    let metrics = rasterizer.metrics;
    let cell_width = metrics.cell_span_width(0, 1);
    let mut image = RgbaImage::new(cell_width, metrics.cell_height);
    assert!(draw_block_element(
      &mut image,
      '█',
      0,
      0,
      cell_width,
      metrics.cell_height,
      (1, 2, 3)
    ));
    assert!(image.pixels().all(|pixel| pixel.0 == [1, 2, 3, 255]));
  }

  #[test]
  fn raster_metrics_follow_the_primary_font_advance_and_terminal_line_height() {
    let rasterizer = maple_rasterizer();
    assert!((rasterizer.metrics.cell_width - 14.4).abs() < 0.01);
    assert_eq!(rasterizer.metrics.cell_height, 36);
    assert!((rasterizer.metrics.font_size - 24.0).abs() < 0.01);
    assert!((rasterizer.metrics.baseline - 26.64).abs() < 0.01);
    assert_eq!(
      rasterizer.dimensions(73, 25, RecordingPixelScale::Original),
      (1052, 900)
    );
  }

  #[test]
  fn rendered_glyph_uses_the_derived_cell_center_and_baseline() {
    let rasterizer = maple_rasterizer();
    let metrics = rasterizer.metrics;
    let mut frame = ComposedFrame::new(1, 1);
    frame.set(0, 0, ComposedCell::Text(CanvasCell::new("M")));
    let image = rasterizer.render(
      &frame,
      ScreenshotRect {
        x: 0,
        y: 0,
        width: 1,
        height: 1,
      },
      RecordingPixelScale::Original,
      |_, _| {},
    );
    let font_index = rasterizer.fonts.font_index_for_grapheme("M").unwrap();
    let glyph_id = rasterizer.fonts.fonts[font_index]
      .raster
      .lookup_glyph_index('M');
    let glyph = rasterizer
      .fonts
      .glyph_by_index(font_index, glyph_id, metrics.font_size)
      .unwrap();
    let expected_left = ((metrics.cell_span_width(0, 1) as f32 - glyph.metrics.advance_width) / 2.0)
      .round() as i32
      + glyph.metrics.xmin;
    let expected_top =
      metrics.baseline.round() as i32 - glyph.metrics.height as i32 - glyph.metrics.ymin;
    let ink_bounds: Option<(u32, u32, u32, u32)> = image
      .enumerate_pixels()
      .filter(|(_, _, pixel)| pixel.0[..3] != [0, 0, 0])
      .fold(None, |bounds, (x, y, _)| {
        Some(match bounds {
          Some((left, top, right, bottom)) => {
            (left.min(x), top.min(y), right.max(x), bottom.max(y))
          }
          None => (x, y, x, y),
        })
      });
    assert_eq!(
      ink_bounds,
      Some((
        expected_left as u32,
        expected_top as u32,
        (expected_left + glyph.metrics.width as i32 - 1) as u32,
        (expected_top + glyph.metrics.height as i32 - 1) as u32,
      ))
    );
  }

  #[test]
  fn arabic_shaping_uses_contextual_forms_and_rtl_direction() {
    let word = "سلام";
    let Some(fonts) = system_font_set_for_text(word) else {
      eprintln!("Skipping Arabic shaping assertion: no installed font covers the sample");
      return;
    };
    let font_index = fonts.font_index_for_grapheme(word).unwrap();
    let font = &fonts.fonts[font_index].raster;
    let nominal_rtl = word
      .chars()
      .rev()
      .map(|character| font.lookup_glyph_index(character))
      .collect::<Vec<_>>();
    let shaped = fonts
      .shape_text(
        font_index,
        word,
        ShapeDirection::RightToLeft,
        REFERENCE_FONT_SIZE,
      )
      .expect("the bundled reference font should be readable by HarfRust");
    let shaped_ids = shaped
      .iter()
      .map(|glyph| glyph.glyph_id)
      .collect::<Vec<_>>();

    assert_ne!(shaped_ids, nominal_rtl);
    assert!(shaped.iter().all(|glyph| glyph.x_advance >= 0.0));
  }

  #[test]
  fn combining_sequence_is_shaped_with_mark_positioning() {
    let rasterizer = maple_rasterizer();
    let sequence = "e\u{301}";
    let font_index = rasterizer.fonts.font_index_for_grapheme(sequence).unwrap();
    let shaped = rasterizer
      .fonts
      .shape_text(
        font_index,
        sequence,
        ShapeDirection::LeftToRight,
        rasterizer.metrics.font_size,
      )
      .expect("the bundled reference font should be readable by HarfRust");
    assert_eq!(shaped.len(), 1);
    assert_ne!(
      shaped[0].glyph_id,
      rasterizer.fonts.fonts[font_index]
        .raster
        .lookup_glyph_index('e')
    );
  }

  #[test]
  fn zwj_emoji_shaping_retains_both_components_in_its_grapheme() {
    let sequence = "👩‍💻";
    let Some(fonts) = system_font_set_for_text(sequence) else {
      eprintln!("Skipping ZWJ shaping assertion: no installed font covers both emoji components");
      return;
    };
    let font_index = fonts.font_index_for_grapheme(sequence).unwrap();
    let shaped = fonts
      .shape_text(
        font_index,
        sequence,
        ShapeDirection::LeftToRight,
        REFERENCE_FONT_SIZE,
      )
      .expect("the selected emoji font should be readable by HarfRust");
    let woman = fonts
      .shape_text(
        font_index,
        "👩",
        ShapeDirection::LeftToRight,
        REFERENCE_FONT_SIZE,
      )
      .expect("the selected emoji font should be readable by HarfRust");
    let laptop = fonts
      .shape_text(
        font_index,
        "💻",
        ShapeDirection::LeftToRight,
        REFERENCE_FONT_SIZE,
      )
      .expect("the selected emoji font should be readable by HarfRust");

    assert!(!shaped.is_empty());
    assert!(shaped.len() <= woman.len() + laptop.len());
    assert!(shaped.iter().all(|glyph| glyph.glyph_id != 0));
    assert!(shaped.iter().all(|glyph| {
      fonts
        .glyph_by_index(font_index, glyph.glyph_id, REFERENCE_FONT_SIZE * 0.86)
        .is_some_and(|rasterized| rasterized.metrics.width > 0 && rasterized.metrics.height > 0)
    }));
  }

  #[test]
  fn font_coverage_does_not_treat_notdef_as_a_supported_character() {
    let rasterizer = maple_rasterizer();
    let font = &rasterizer.fonts.fonts[0].raster;
    assert!(font_has_character(font, 'M'));
    assert!(!font_has_character(font, '😀'));
  }

  #[test]
  fn indic_shaping_reorders_prebase_vowel_marks() {
    let sequence = "कि";
    let Some(fonts) = system_font_set_for_text(sequence) else {
      eprintln!("Skipping Devanagari shaping assertion: no installed font covers the sample");
      return;
    };
    let font_index = fonts.font_index_for_grapheme(sequence).unwrap();
    let font = &fonts.fonts[font_index].raster;
    let nominal = sequence
      .chars()
      .map(|character| font.lookup_glyph_index(character))
      .collect::<Vec<_>>();
    let shaped = fonts
      .shape_text(
        font_index,
        sequence,
        ShapeDirection::LeftToRight,
        REFERENCE_FONT_SIZE,
      )
      .expect("Nirmala UI should be readable by HarfRust");

    assert_ne!(
      shaped
        .iter()
        .map(|glyph| glyph.glyph_id)
        .collect::<Vec<_>>(),
      nominal
    );
  }

  fn system_font_set_for_text(text: &str) -> Option<FontSet> {
    let mut database = fontdb::Database::new();
    database.load_system_fonts();
    for family in SYSTEM_FONT_FALLBACK_FAMILIES {
      let Some(id) = database.query(&fontdb::Query {
        families: &[fontdb::Family::Name(family)],
        ..fontdb::Query::default()
      }) else {
        continue;
      };
      let mut fonts = Vec::new();
      load_database_font(&database, id, &mut fonts);
      if fonts.first().is_some_and(|font| {
        text
          .chars()
          .filter(|character| !is_shaping_ignorable(*character))
          .all(|character| font_has_character(&font.raster, character))
      }) {
        return Some(FontSet {
          fonts,
          database,
          glyph_cache: std::cell::RefCell::new(std::collections::HashMap::new()),
        });
      }
    }
    None
  }

  #[test]
  fn bidi_keeps_latin_segments_left_to_right_around_hebrew() {
    let text = "ABC אבג 123";
    let bidi = BidiInfo::new(text, None);
    let paragraph = &bidi.paragraphs[0];
    assert_eq!(
      bidi.reorder_line(paragraph, paragraph.range.clone()),
      "ABC 123 גבא"
    );

    let mut frame = ComposedFrame::new(text.chars().count() as u16, 1);
    for (x, grapheme) in text.graphemes(true).enumerate() {
      frame.set(x as u16, 0, ComposedCell::Text(CanvasCell::new(grapheme)));
    }
    let spans = shape_row(
      &frame,
      ScreenshotRect {
        x: 0,
        y: 0,
        width: frame.width(),
        height: 1,
      },
      0,
      &maple_rasterizer().fonts,
    );
    assert!(
      spans
        .iter()
        .any(|span| span.direction == ShapeDirection::RightToLeft)
    );
    assert!(
      spans
        .iter()
        .any(|span| span.direction == ShapeDirection::LeftToRight)
    );
  }

  #[test]
  fn fractional_cell_advance_uses_stable_rounded_boundaries() {
    let metrics = maple_rasterizer().metrics;
    assert_eq!(metrics.cell_x(0), 0);
    assert_eq!(metrics.cell_x(1), 14);
    assert_eq!(metrics.cell_x(2), 29);
    assert_eq!(metrics.cell_x(3), 43);
    assert_eq!(metrics.cell_x(5), 72);
  }

  #[test]
  fn default_fallback_font_starts_with_the_reference_terminal_face() {
    let rasterizer = bundled_rasterizer(&[]);
    assert!(
      rasterizer
        .fonts
        .primary_font()
        .unwrap()
        .name()
        .unwrap()
        .starts_with("Maple Mono NF CN")
    );
  }

  #[test]
  fn export_scale_dimensions_follow_font_metrics() {
    let rasterizer = maple_rasterizer();
    assert_eq!(
      rasterizer.dimensions(3, 5, RecordingPixelScale::Half),
      (22, 90)
    );
    assert_eq!(
      rasterizer.dimensions(3, 5, RecordingPixelScale::Original),
      (44, 180)
    );
    assert_eq!(
      rasterizer.dimensions(3, 5, RecordingPixelScale::Double),
      (86, 360)
    );
  }

  fn maple_rasterizer() -> TerminalFrameRasterizer {
    bundled_rasterizer(&["assets/fonts/mmo.ttf".to_string()])
  }

  fn bundled_rasterizer(preferred: &[String]) -> TerminalFrameRasterizer {
    let deployment_root = test_deployment_root();
    let font_path = preferred
      .first()
      .map(PathBuf::from)
      .unwrap_or_else(|| bundled_font_paths(&deployment_root)[0].clone());
    let mut loaded_fonts = Vec::new();
    load_font_file(
      &resolve_font_path(&font_path, &deployment_root),
      &mut loaded_fonts,
    )
    .unwrap();
    let fonts = FontSet {
      fonts: loaded_fonts,
      database: fontdb::Database::new(),
      glyph_cache: std::cell::RefCell::new(std::collections::HashMap::new()),
    };
    let metrics = RasterMetrics::for_font(fonts.primary_font().unwrap()).unwrap();
    TerminalFrameRasterizer { fonts, metrics }
  }

  fn test_deployment_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..")
  }

  #[test]
  fn screenshot_record_uses_the_shared_media_manifest_version() {
    let root = std::env::temp_dir().join(format!(
      "tg_screenshot_manifest_{}_{}",
      std::process::id(),
      timestamp()
    ));
    let storage = StorageService::from_root_for_test(root.clone());
    let mut log = LogService::new();
    let mut frame = ComposedFrame::new(1, 1);
    frame.set(0, 0, ComposedCell::Text(CanvasCell::new("x")));
    let path = ScreenshotService::new()
      .write_json(
        &storage,
        &frame,
        ScreenshotRect {
          x: 0,
          y: 0,
          width: 1,
          height: 1,
        },
        None,
        &mut log,
      )
      .unwrap();

    let document: serde_json::Value =
      serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    assert_eq!(document["schema_version"], MEDIA_MANIFEST_VERSION);
    let _ = std::fs::remove_dir_all(root);
  }
}
