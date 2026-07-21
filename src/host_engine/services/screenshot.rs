use std::{
  env, fs,
  path::{Path, PathBuf},
};

use chrono::Local;
use crossbeam_channel::Sender;
use image::{ImageBuffer, Rgba, RgbaImage};
use serde::{Deserialize, Serialize};
use serde_json::json;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::host_engine::services::{
  CanvasCell, ComposedCell, ComposedFrame, EngineEvent, LogService, LogSource, StorageService,
  TaskId, TerminalColor, TextColor, TextStyle,
};

const CELL_WIDTH: u32 = 12;
const CELL_HEIGHT: u32 = 24;
const FONT_SIZE: f32 = 18.0;

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

pub struct ScreenshotService {
  last_presented_frame: Option<ComposedFrame>,
  pending_font_preview: Option<Vec<String>>,
}

impl ScreenshotService {
  pub fn new() -> Self {
    Self {
      last_presented_frame: None,
      pending_font_preview: None,
    }
  }

  pub fn request_font_preview(&mut self, fonts: Vec<String>) {
    self.pending_font_preview = Some(fonts);
  }

  pub fn take_font_preview_request(&mut self) -> Option<Vec<String>> {
    self.pending_font_preview.take()
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
        if let Some(ComposedCell::Text(cell)) = frame.get(x, y) {
          if !cell.is_continuation() {
            let w = cell.text.width().max(1) as u16;
            right = right.max(x.saturating_add(w.saturating_sub(1)).min(frame.width() - 1));
          }
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
      "timestamp": timestamp,
      "frame": { "width": frame.width(), "height": frame.height() },
      "selection": rect,
      "plain_text": Self::plain_text(frame, rect),
      "png_path": png_path.map(|p| p.to_string_lossy().to_string()),
      "rich_text": rich_text_json(frame, rect),
    });
    if let Err(error) = fs::create_dir_all(storage.screenshot_cache_dir_path()).and_then(|_| {
      fs::write(
        &path,
        serde_json::to_vec_pretty(&document).unwrap_or_default(),
      )
    }) {
      log.warn(
        LogSource::Storage,
        format!("Failed to write screenshot JSON: {error}"),
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

pub fn run_screenshot_task(
  task_id: TaskId,
  task: ScreenshotTask,
  event_tx: &Sender<EngineEvent>,
) -> Result<(), String> {
  match save_png(
    task_id,
    &task.frame,
    task.selection,
    &task.png_path,
    &task.fonts,
    event_tx,
  ) {
    Ok(()) => {
      let _ = event_tx.send(EngineEvent::Screenshot(ScreenshotAsyncEvent::Saved {
        task_id,
        png_path: task.png_path,
      }));
      Ok(())
    }
    Err(error) => {
      let _ = event_tx.send(EngineEvent::Screenshot(ScreenshotAsyncEvent::Failed {
        task_id,
        error: error.clone(),
      }));
      Err(error)
    }
  }
}

fn save_png(
  task_id: TaskId,
  frame: &ComposedFrame,
  rect: ScreenshotRect,
  path: &PathBuf,
  preferred_fonts: &[String],
  event_tx: &Sender<EngineEvent>,
) -> Result<(), String> {
  fs::create_dir_all(path.parent().ok_or("PNG path has no parent directory")?)
    .map_err(|error| error.to_string())?;
  let fonts = FontSet::load(preferred_fonts)?;
  let width = rect.width as u32 * CELL_WIDTH;
  let height = rect.height as u32 * CELL_HEIGHT;
  let mut image = ImageBuffer::from_pixel(width.max(1), height.max(1), Rgba([0, 0, 0, 255]));

  for y in 0..rect.height {
    for x in 0..rect.width {
      let Some(ComposedCell::Text(cell)) = frame.get(rect.x + x, rect.y + y) else {
        continue;
      };
      let (fg, bg) = resolved_colors(&cell.style);
      fill_rect(
        &mut image,
        x as u32 * CELL_WIDTH,
        y as u32 * CELL_HEIGHT,
        CELL_WIDTH,
        CELL_HEIGHT,
        bg,
      );
      if cell.style.underline {
        draw_underline(&mut image, x, y, fg);
      }
    }
    send_progress(
      event_tx,
      task_id,
      y.saturating_add(1),
      rect.height.saturating_mul(2),
    );
  }

  for y in 0..rect.height {
    for x in 0..rect.width {
      let Some(ComposedCell::Text(cell)) = frame.get(rect.x + x, rect.y + y) else {
        continue;
      };
      if !cell.is_continuation() {
        draw_cell_text(&mut image, &fonts, x, y, cell);
      }
    }
    send_progress(
      event_tx,
      task_id,
      rect.height.saturating_add(y).saturating_add(1),
      rect.height.saturating_mul(2),
    );
  }

  image.save(path).map_err(|error| error.to_string())
}

fn send_progress(
  event_tx: &Sender<EngineEvent>,
  task_id: TaskId,
  completed_rows: u16,
  total_rows: u16,
) {
  let _ = event_tx.send(EngineEvent::Screenshot(ScreenshotAsyncEvent::Progress {
    task_id,
    completed_rows,
    total_rows,
  }));
}

struct FontSet {
  fonts: Vec<fontdue::Font>,
}

impl FontSet {
  fn load(preferred: &[String]) -> Result<Self, String> {
    let mut fonts = Vec::new();
    let mut database = fontdb::Database::new();
    database.load_system_fonts();

    for value in preferred {
      let path = Path::new(value);
      if path.is_file() {
        let _ = load_font_file(path, &mut fonts);
      } else if let Some(id) = database.query(&fontdb::Query {
        families: &[fontdb::Family::Name(value)],
        ..fontdb::Query::default()
      }) {
        load_database_font(&database, id, &mut fonts);
      }
    }

    for path in [
      Path::new("assets/fonts/mnf.ttf"),
      Path::new("assets/fonts/asmn.otf"),
      Path::new("assets/fonts/nsscvf.ttf"),
    ] {
      if path.is_file() {
        let _ = load_font_file(path, &mut fonts);
      }
    }

    if let Some(paths) = env::var_os("TUI_CAPTURE_FONTS") {
      for path in env::split_paths(&paths) {
        let _ = load_font_file(&path, &mut fonts);
      }
    }

    let mut ids = Vec::new();
    const CANDIDATE_FAMILIES: &[&str] = &[
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
      "Segoe UI Emoji",
      "Apple Color Emoji",
      "Noto Color Emoji",
      "Noto Emoji",
      "Symbola",
    ];

    for family_name in CANDIDATE_FAMILIES {
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
      return Err(
        "No usable screenshot font found. Set TUI_CAPTURE_FONTS to TTF/OTF/TTC paths.".to_string(),
      );
    }

    Ok(Self { fonts })
  }

  fn font_for(&self, character: char) -> Option<&fontdue::Font> {
    self.fonts.iter().find(|font| font.has_glyph(character))
  }
}

fn load_database_font(database: &fontdb::Database, id: fontdb::ID, fonts: &mut Vec<fontdue::Font>) {
  if let Some(result) = database.with_face_data(id, |data, face_index| {
    fontdue::Font::from_bytes(
      data.to_vec(),
      fontdue::FontSettings {
        collection_index: face_index,
        ..fontdue::FontSettings::default()
      },
    )
  }) && let Ok(font) = result
  {
    fonts.push(font);
  }
}

fn load_font_file(path: &Path, fonts: &mut Vec<fontdue::Font>) -> Result<(), String> {
  let bytes =
    fs::read(path).map_err(|error| format!("Failed to read font {}: {error}", path.display()))?;
  let font = fontdue::Font::from_bytes(bytes, fontdue::FontSettings::default())
    .map_err(|error| format!("Failed to parse font {}: {error}", path.display()))?;
  fonts.push(font);
  Ok(())
}

fn draw_cell_text(image: &mut RgbaImage, fonts: &FontSet, x: u16, y: u16, cell: &CanvasCell) {
  if cell.style.hidden {
    return;
  }
  let (fg, _bg) = resolved_colors(&cell.style);
  let px = x as u32 * CELL_WIDTH;
  let py = y as u32 * CELL_HEIGHT;
  let span_width = cell.text.width().max(1) as u32 * CELL_WIDTH;
  draw_grapheme(
    image,
    fonts,
    &cell.text,
    px,
    py,
    span_width,
    &cell.style,
    fg,
  );
}

fn draw_underline(image: &mut RgbaImage, x: u16, y: u16, color: (u8, u8, u8)) {
  let px = x as u32 * CELL_WIDTH;
  let py = y as u32 * CELL_HEIGHT + CELL_HEIGHT.saturating_sub(4);
  for xx in px..px.saturating_add(CELL_WIDTH).min(image.width()) {
    composite_pixel(
      image,
      xx,
      py.min(image.height().saturating_sub(1)),
      color,
      255,
    );
  }
}

#[allow(clippy::too_many_arguments)]
fn draw_grapheme(
  image: &mut RgbaImage,
  fonts: &FontSet,
  grapheme: &str,
  origin_x: u32,
  origin_y: u32,
  span_width: u32,
  style: &TextStyle,
  fg: (u8, u8, u8),
) {
  let visible_width_sum: usize = grapheme
    .chars()
    .map(|character| UnicodeWidthChar::width(character).unwrap_or(0))
    .sum();
  let complex_cluster = visible_width_sum > UnicodeWidthStr::width(grapheme);
  let mut pen_x = origin_x;
  let mut last_base_origin_x = origin_x as i32;
  let clip_left = origin_x;
  let clip_right = (origin_x + span_width).min(image.width());
  let clip_top = origin_y;
  let clip_bottom = (origin_y + CELL_HEIGHT).min(image.height());

  for character in grapheme.chars() {
    if character == '\u{200d}' || character == '\u{fe0f}' {
      continue;
    }
    let char_width = UnicodeWidthChar::width(character).unwrap_or(0).min(2);
    if complex_cluster && char_width > 0 && pen_x != origin_x {
      break;
    }

    let Some(font) = fonts.font_for(character) else {
      continue;
    };
    let font_size = if is_probably_emoji(character) {
      FONT_SIZE * 0.86
    } else {
      FONT_SIZE
    };
    let (metrics, bitmap) = font.rasterize(character, font_size);
    let allocated_width = if char_width == 0 {
      span_width
    } else {
      (char_width as u32 * CELL_WIDTH).min(span_width)
    };
    let glyph_origin_x = if char_width == 0 {
      last_base_origin_x
    } else {
      let origin =
        pen_x as i32 + ((allocated_width as f32 - metrics.advance_width) / 2.0).round() as i32;
      last_base_origin_x = origin;
      origin
    };

    let destination_x = glyph_origin_x + metrics.xmin;
    let baseline = origin_y as i32 + (CELL_HEIGHT as f32 * 0.78) as i32;
    let top = baseline - metrics.height as i32 - metrics.ymin;
    draw_glyph_bitmap(
      image,
      &bitmap,
      metrics.width,
      metrics.height,
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
        &bitmap,
        metrics.width,
        metrics.height,
        destination_x + 1,
        top,
        clip_left,
        clip_top,
        clip_right,
        clip_bottom,
        fg,
      );
    }

    if char_width > 0 {
      pen_x = pen_x.saturating_add(char_width as u32 * CELL_WIDTH);
    }
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

fn fill_rect(
  image: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
  x: u32,
  y: u32,
  width: u32,
  height: u32,
  color: (u8, u8, u8),
) {
  for yy in y..y.saturating_add(height).min(image.height()) {
    for xx in x..x.saturating_add(width).min(image.width()) {
      image.put_pixel(xx, yy, Rgba([color.0, color.1, color.2, 255]));
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

#[cfg(test)]
mod tests {
  use super::ScreenshotService;

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
  fn font_preview_request_is_consumed_once() {
    let mut service = ScreenshotService::new();
    service.request_font_preview(vec!["test.ttf".to_string()]);
    assert_eq!(
      service.take_font_preview_request(),
      Some(vec!["test.ttf".to_string()])
    );
    assert_eq!(service.take_font_preview_request(), None);
  }
}
