//! Image service: converts png/jpg images into half-block rich text (memory + disk cache),
//! synchronously or as an async job.

use std::collections::HashMap;
use std::fs::{self, File};
use std::hash::{DefaultHasher, Hasher};
use std::io::{self, Cursor, Read};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use crossbeam_channel::Sender;
use image::{GenericImageView, ImageReader, Limits};
use serde::{Deserialize, Serialize};
use tg_service_async::{AsyncJob, AsyncRuntime, TaskCancellation, TaskId, TaskStatusEvent};

/// Parameters of an image conversion.
#[derive(Clone, Debug)]
pub struct ImageConvertParams {
  pub image_path: String,
  pub output_width: Option<u32>,
  pub output_height: Option<u32>,
  pub crop_x: i32,
  pub crop_y: i32,
  pub crop_width: Option<u32>,
  pub crop_height: Option<u32>,
  pub square_crop: bool,
  pub scale: f64,
  pub cache: bool,
}

impl Default for ImageConvertParams {
  fn default() -> Self {
    Self {
      image_path: String::new(),
      output_width: Some(80),
      output_height: Some(24),
      crop_x: 0,
      crop_y: 0,
      crop_width: None,
      crop_height: None,
      square_crop: false,
      scale: 1.0,
      cache: true,
    }
  }
}

#[derive(Clone, Debug)]
pub enum ImageTask {
  Convert {
    params: ImageConvertParams,
    cache_dir: Option<PathBuf>,
  },
}

#[derive(Clone, Debug)]
pub enum ImageEvent {
  ConvertFinished { task_id: TaskId, output: String },
  Failed { task_id: TaskId, error: String },
}

impl<E: From<ImageEvent> + Send + 'static> AsyncJob<E> for ImageTask {
  fn run(
    self: Box<Self>,
    task_id: TaskId,
    events: &Sender<E>,
    _cancellation: &TaskCancellation,
  ) -> Result<(), String> {
    match *self {
      ImageTask::Convert { params, cache_dir } => {
        match ImageService::new(cache_dir).convert(params) {
          Ok(output) => {
            let _ = events.send(ImageEvent::ConvertFinished { task_id, output }.into());
            Ok(())
          }
          Err(error) => {
            let _ = events.send(
              ImageEvent::Failed {
                task_id,
                error: error.clone(),
              }
              .into(),
            );
            Err(error)
          }
        }
      }
    }
  }
}

/// The image service, converting images into terminal half-block character art (with a memory and
/// disk cache).
pub struct ImageService {
  cache: HashMap<u64, String>,
  cache_dir: Option<PathBuf>,
}

const MAX_SOURCE_FILE_BYTES: u64 = 32 * 1024 * 1024;
const MAX_SOURCE_PIXELS: u64 = 16_000_000;
const MAX_SOURCE_DIMENSION: u32 = 16_384;
const MAX_DECODE_ALLOCATION_BYTES: u64 = 80 * 1024 * 1024;
const MAX_OUTPUT_DIMENSION: u32 = 2_048;
const MAX_OUTPUT_CELLS: u64 = 16_384;
const MAX_SCALED_PIXELS: u64 = 16_000_000;

/// The format of a disk cache entry.
#[derive(Serialize, Deserialize)]
struct DiskCacheEntry {
  source_modified: u64,
  rendered: String,
}

impl ImageService {
  /// Creates an image service. The disk cache is disabled when `cache_dir` is `None`.
  pub fn new(cache_dir: Option<PathBuf>) -> Self {
    Self {
      cache: HashMap::new(),
      cache_dir,
    }
  }

  /// Converts an image into terminal character art (with caching).
  ///
  /// # Errors
  ///
  /// Returns an error when the parameters are invalid, the image path cannot be resolved to an
  /// existing png/jpg/jpeg file, the image cannot be opened, or the crop area is empty.
  pub fn convert(&mut self, params: ImageConvertParams) -> Result<String, String> {
    validate(&params)?;

    let resolved = resolve_path(&params.image_path)?;
    let source_bytes = read_source(&resolved)?;
    let hash = compute_hash(&source_bytes, &params);

    if params.cache {
      // 1. Memory cache.
      if let Some(cached) = self.cache.get(&hash) {
        return Ok(cached.clone());
      }
      // 2. Disk cache.
      if let Some(disk) = self.read_disk_cache(hash, &resolved) {
        self.cache.insert(hash, disk.clone());
        return Ok(disk);
      }
    }

    let img = decode_image(&source_bytes, &resolved)?;

    let result = process(&img, &params)?;

    if params.cache {
      self.cache.insert(hash, result.clone());
      self.write_disk_cache(hash, &resolved, &result);
    }
    Ok(result)
  }

  pub fn convert_async<E>(
    &self,
    async_runtime: &AsyncRuntime<E>,
    params: ImageConvertParams,
  ) -> TaskId
  where
    E: From<ImageEvent> + From<TaskStatusEvent> + Send + 'static,
  {
    async_runtime.submit(ImageTask::Convert {
      params,
      cache_dir: self.cache_dir.clone(),
    })
  }

  // ─── Disk cache helpers ──────────────────────────────

  fn disk_cache_path(&self, hash: u64) -> Option<PathBuf> {
    self
      .cache_dir
      .as_ref()
      .map(|dir| dir.join(format!("{hash}.json")))
  }

  fn read_disk_cache(&self, hash: u64, source_path: &Path) -> Option<String> {
    let path = self.disk_cache_path(hash)?;
    let data = fs::read_to_string(&path).ok()?;
    let entry: DiskCacheEntry = serde_json::from_str(&data).ok()?;
    let current_mtime = source_modified(source_path).unwrap_or(0);
    if entry.source_modified == current_mtime {
      Some(entry.rendered)
    } else {
      // Stale: delete it.
      let _ = fs::remove_file(&path);
      None
    }
  }

  fn write_disk_cache(&self, hash: u64, source_path: &Path, rendered: &str) {
    let Some(path) = self.disk_cache_path(hash) else {
      return;
    };
    let mtime = match source_modified(source_path) {
      Ok(m) => m,
      Err(_) => return,
    };
    if let Some(parent) = path.parent() {
      // TODO: add log warn when LogService is available
      let _ = fs::create_dir_all(parent);
    }
    let entry = DiskCacheEntry {
      source_modified: mtime,
      rendered: rendered.to_string(),
    };
    if let Ok(json) = serde_json::to_string(&entry) {
      // TODO: add log warn when LogService is available
      let _ = fs::write(&path, json);
    }
  }
}

/// Returns the modification time of the source file (Unix seconds).
///
/// # Errors
///
/// Returns an error when the file metadata cannot be read or the modification time is before the
/// Unix epoch.
fn source_modified(path: &Path) -> io::Result<u64> {
  let meta = fs::metadata(path)?;
  let dur = meta
    .modified()?
    .duration_since(UNIX_EPOCH)
    .map_err(|e| io::Error::other(format!("mtime before epoch: {e:?}")))?;
  Ok(dur.as_secs())
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Rgb(u8, u8, u8);

fn validate(p: &ImageConvertParams) -> Result<(), String> {
  if p.image_path.is_empty() {
    return Err("image_path 不能为空".into());
  }
  if p.output_width == Some(0) {
    return Err("output_width 必须 > 0".into());
  }
  if p.output_height == Some(0) {
    return Err("output_height 必须 > 0".into());
  }
  if p.output_width.is_some_and(|width| width > MAX_OUTPUT_DIMENSION) {
    return Err(format!("output_width 不得超过 {MAX_OUTPUT_DIMENSION}"));
  }
  if p.output_height.is_some_and(|height| height > MAX_OUTPUT_DIMENSION) {
    return Err(format!("output_height 不得超过 {MAX_OUTPUT_DIMENSION}"));
  }
  if !p.scale.is_finite() || p.scale <= 0.0 {
    return Err("scale 必须 > 0".into());
  }
  if p.crop_x < 0 {
    return Err("crop_x 不得小于 0".into());
  }
  if p.crop_y < 0 {
    return Err("crop_y 不得小于 0".into());
  }
  if let Some(0) = p.crop_width {
    return Err("crop_width 必须 > 0".into());
  }
  if let Some(0) = p.crop_height {
    return Err("crop_height 必须 > 0".into());
  }
  Ok(())
}

fn read_source(path: &Path) -> Result<Vec<u8>, String> {
  let file = File::open(path).map_err(|error| format!("无法读取图片 {}: {error}", path.display()))?;
  let mut bytes = Vec::new();
  file
    .take(MAX_SOURCE_FILE_BYTES + 1)
    .read_to_end(&mut bytes)
    .map_err(|error| format!("读取图片 {} 失败: {error}", path.display()))?;
  if bytes.len() as u64 > MAX_SOURCE_FILE_BYTES {
    return Err(format!(
      "图片文件 {} 超过 {} MiB 限制",
      path.display(),
      MAX_SOURCE_FILE_BYTES / (1024 * 1024)
    ));
  }
  Ok(bytes)
}

fn decode_image(bytes: &[u8], path: &Path) -> Result<image::DynamicImage, String> {
  let dimensions_reader = image_reader(bytes, path)?;
  let (width, height) = dimensions_reader
    .into_dimensions()
    .map_err(|error| format!("读取图片尺寸 {} 失败: {error}", path.display()))?;
  validate_source_dimensions(width, height, path)?;

  let mut reader = image_reader(bytes, path)?;
  reader.limits(Limits {
    max_image_width: Some(MAX_SOURCE_DIMENSION),
    max_image_height: Some(MAX_SOURCE_DIMENSION),
    max_alloc: Some(MAX_DECODE_ALLOCATION_BYTES),
  });
  reader
    .decode()
    .map_err(|error| format!("无法解码图片 {}: {error}", path.display()))
}

fn image_reader<'a>(bytes: &'a [u8], path: &Path) -> Result<ImageReader<Cursor<&'a [u8]>>, String> {
  ImageReader::new(Cursor::new(bytes))
    .with_guessed_format()
    .map_err(|error| format!("无法识别图片格式 {}: {error}", path.display()))
}

fn validate_source_dimensions(width: u32, height: u32, path: &Path) -> Result<(), String> {
  if width == 0 || height == 0 {
    return Err(format!("图片尺寸无效 {}: {width}×{height}", path.display()));
  }
  if width > MAX_SOURCE_DIMENSION || height > MAX_SOURCE_DIMENSION {
    return Err(format!(
      "图片尺寸 {} 超过单边 {} 像素限制: {width}×{height}",
      path.display(),
      MAX_SOURCE_DIMENSION
    ));
  }
  let pixels = u64::from(width) * u64::from(height);
  if pixels > MAX_SOURCE_PIXELS {
    return Err(format!(
      "图片像素数 {} 超过 {} 限制: {pixels}",
      path.display(),
      MAX_SOURCE_PIXELS
    ));
  }
  Ok(())
}

const VALID_EXTS: &[&str] = &["png", "jpg", "jpeg"];

// Resolves the image path; without an extension, looks for a matching png/jpg/jpeg file.
fn resolve_path(raw: &str) -> Result<PathBuf, String> {
  let path = Path::new(raw);

  if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
    let ext = ext.to_ascii_lowercase();
    if !VALID_EXTS.contains(&ext.as_str()) {
      return Err(format!("不支持的图片后缀 '{}'，仅支持 png/jpg/jpeg", ext));
    }
    if !path.is_file() {
      return Err(format!("图片文件不存在: {}", path.display()));
    }
    return Ok(path.to_path_buf());
  }

  let parent = path.parent().unwrap_or(Path::new("."));
  let stem = path
    .file_stem()
    .ok_or_else(|| "无效的图片路径".to_string())?;

  for ext in VALID_EXTS {
    let candidate = parent.join(format!("{}.{}", stem.to_string_lossy(), ext));
    if candidate.is_file() {
      return Ok(candidate);
    }
  }
  Err(format!(
    "未找到与 '{}' 匹配的 png/jpg/jpeg 文件",
    path.display()
  ))
}

// Computes the cache hash from the image path and the conversion parameters.
fn compute_hash(source_bytes: &[u8], p: &ImageConvertParams) -> u64 {
  let mut h = DefaultHasher::new();
  h.write(source_bytes);
  hash_optional_u32(&mut h, p.output_width);
  hash_optional_u32(&mut h, p.output_height);
  h.write_i32(p.crop_x);
  h.write_i32(p.crop_y);
  hash_optional_u32(&mut h, p.crop_width);
  hash_optional_u32(&mut h, p.crop_height);
  h.write_u8(u8::from(p.square_crop));
  h.write_u64(p.scale.to_bits());
  h.write_u8(tg_core_version::IMAGE_CACHE_FORMAT_VERSION);
  h.finish()
}

fn hash_optional_u32(hasher: &mut impl Hasher, value: Option<u32>) {
  match value {
    Some(value) => {
      hasher.write_u8(1);
      hasher.write_u32(value);
    }
    None => hasher.write_u8(0),
  }
}

// Crops and scales the image, then samples it into half-block character art.
fn process(img: &image::DynamicImage, p: &ImageConvertParams) -> Result<String, String> {
  let (src_w, src_h) = img.dimensions();
  let (cx, cy, cw, ch) = crop_area(src_w, src_h, p)?;
  let (output_width, output_height) = output_dimensions(src_w, src_h, p)?;
  let (scaled_width, scaled_height) = scaled_dimensions(cw, ch, p.scale)?;

  let rgba = img.to_rgba8();
  let cropped = image::imageops::crop_imm(&rgba, cx, cy, cw, ch).to_image();

  let scaled = if (p.scale - 1.0).abs() > f64::EPSILON {
    image::imageops::resize(
      &cropped,
      scaled_width,
      scaled_height,
      image::imageops::FilterType::Lanczos3,
    )
  } else {
    cropped
  };

  let pw = output_width;
  let ph = output_height
    .checked_mul(2)
    .ok_or_else(|| "output_height 超出支持范围".to_string())?;
  let resized = image::imageops::resize(&scaled, pw, ph, image::imageops::FilterType::Lanczos3);

  Ok(sample_halfblock(&resized, pw, ph))
}

fn crop_area(
  source_width: u32,
  source_height: u32,
  params: &ImageConvertParams,
) -> Result<(u32, u32, u32, u32), String> {
  if params.square_crop {
    let side = source_width.min(source_height);
    return Ok(((source_width - side) / 2, (source_height - side) / 2, side, side));
  }

  let x = u32::try_from(params.crop_x).map_err(|_| "crop_x 不得小于 0".to_string())?;
  let y = u32::try_from(params.crop_y).map_err(|_| "crop_y 不得小于 0".to_string())?;
  if x >= source_width || y >= source_height {
    return Err(format!(
      "裁剪起点 ({x}, {y}) 超出图片范围 {source_width}×{source_height}"
    ));
  }

  let width = params.crop_width.unwrap_or(source_width - x);
  let height = params.crop_height.unwrap_or(source_height - y);
  if x.checked_add(width).is_none_or(|right| right > source_width)
    || y
      .checked_add(height)
      .is_none_or(|bottom| bottom > source_height)
  {
    return Err(format!(
      "裁剪矩形 ({x}, {y}, {width}, {height}) 超出图片范围 {source_width}×{source_height}"
    ));
  }
  if width == 0 || height == 0 {
    return Err("裁剪区域为空".into());
  }
  Ok((x, y, width, height))
}

fn output_dimensions(
  source_width: u32,
  source_height: u32,
  params: &ImageConvertParams,
) -> Result<(u32, u32), String> {
  let width = params.output_width.unwrap_or((source_width / 100).max(1));
  let height = params.output_height.unwrap_or((source_height / 200).max(1));
  if width == 0 || height == 0 {
    return Err("输出宽高必须 > 0".into());
  }
  if width > MAX_OUTPUT_DIMENSION || height > MAX_OUTPUT_DIMENSION {
    return Err(format!(
      "输出尺寸不得超过单边 {MAX_OUTPUT_DIMENSION} 格: {width}×{height}"
    ));
  }
  let cells = u64::from(width) * u64::from(height);
  if cells > MAX_OUTPUT_CELLS {
    return Err(format!(
      "输出单元数 {cells} 超过 {MAX_OUTPUT_CELLS} 限制"
    ));
  }
  Ok((width, height))
}

fn scaled_dimensions(width: u32, height: u32, scale: f64) -> Result<(u32, u32), String> {
  let scaled_width = (f64::from(width) * scale).round().max(1.0);
  let scaled_height = (f64::from(height) * scale).round().max(1.0);
  if !scaled_width.is_finite()
    || !scaled_height.is_finite()
    || scaled_width > f64::from(MAX_SOURCE_DIMENSION)
    || scaled_height > f64::from(MAX_SOURCE_DIMENSION)
  {
    return Err("缩放后的图片单边超过像素限制".into());
  }
  let scaled_width = scaled_width as u32;
  let scaled_height = scaled_height as u32;
  if u64::from(scaled_width) * u64::from(scaled_height) > MAX_SCALED_PIXELS {
    return Err("缩放后的图片像素数超过限制".into());
  }
  Ok((scaled_width, scaled_height))
}

// Samples an RGBA image into a string of terminal half-block characters with foreground/background
// color tags.
fn sample_halfblock(rgba: &image::RgbaImage, w: u32, h: u32) -> String {
  let char_rows = h / 2;
  let cap = (w as usize * char_rows as usize) * 18 + 2;
  let mut out = String::with_capacity(cap);
  out.push_str("f%");

  for cy in 0..char_rows {
    if cy > 0 {
      out.push('\n');
    }
    let mut prev_fg: Option<Rgb> = None;
    let mut prev_bg: Option<Rgb> = None;
    for cx in 0..w {
      let top = get_rgb(rgba, cx, cy * 2);
      let bot = get_rgb(rgba, cx, cy * 2 + 1);

      let fg_changed = prev_fg != Some(bot);
      let bg_changed = prev_bg != Some(top);

      if fg_changed || bg_changed {
        if bg_changed {
          prev_bg = Some(top);
          out.push_str(&format!("<bg:#{:02x}{:02x}{:02x}>", top.0, top.1, top.2));
        }
        if fg_changed {
          prev_fg = Some(bot);
          out.push_str(&format!("<fg:#{:02x}{:02x}{:02x}>", bot.0, bot.1, bot.2));
        }
      }
      out.push('\u{2585}');
    }
  }
  out
}

fn get_rgb(rgba: &image::RgbaImage, x: u32, y: u32) -> Rgb {
  let p = rgba.get_pixel(x, y).0;
  Rgb(p[0], p[1], p[2])
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::sync::atomic::{AtomicU64, Ordering};
  use std::time::{Duration, Instant};

  static NEXT_TEST_IMAGE_ID: AtomicU64 = AtomicU64::new(1);

  struct TestImage {
    root: PathBuf,
    path: PathBuf,
  }

  impl TestImage {
    fn new() -> Self {
      let root = std::env::temp_dir().join(format!(
        "tg_image_source_{}_{}",
        std::process::id(),
        NEXT_TEST_IMAGE_ID.fetch_add(1, Ordering::Relaxed)
      ));
      fs::create_dir_all(&root).expect("create image test directory");
      let path = root.join("source.png");
      let mut image = image::RgbImage::new(4, 4);
      for (x, y, pixel) in image.enumerate_pixels_mut() {
        *pixel = image::Rgb([(x * 63) as u8, (y * 63) as u8, 160]);
      }
      image.save(&path).expect("write image test fixture");
      Self { root, path }
    }
  }

  impl Drop for TestImage {
    fn drop(&mut self) {
      let _ = fs::remove_dir_all(&self.root);
    }
  }

  #[test]
  fn validate_rejects_empty_path() {
    let p = ImageConvertParams {
      image_path: String::new(),
      ..Default::default()
    };
    assert!(validate(&p).is_err());
  }

  #[test]
  fn validate_rejects_zero_width() {
    let p = ImageConvertParams {
      image_path: "x".into(),
      output_width: 0,
      ..Default::default()
    };
    assert!(validate(&p).is_err());
  }

  #[test]
  fn validate_rejects_zero_height() {
    let p = ImageConvertParams {
      image_path: "x".into(),
      output_height: 0,
      ..Default::default()
    };
    assert!(validate(&p).is_err());
  }

  #[test]
  fn validate_rejects_zero_crop_width() {
    let p = ImageConvertParams {
      image_path: "x".into(),
      crop_width: Some(0),
      ..Default::default()
    };
    assert!(validate(&p).is_err());
  }

  #[test]
  fn validate_rejects_negative_scale() {
    let p = ImageConvertParams {
      image_path: "x".into(),
      scale: -1.0,
      ..Default::default()
    };
    assert!(validate(&p).is_err());
  }

  #[test]
  fn validate_accepts_valid_params() {
    let p = ImageConvertParams {
      image_path: "test.jpg".into(),
      output_width: 40,
      output_height: 20,
      ..Default::default()
    };
    assert!(validate(&p).is_ok());
  }

  #[test]
  fn resolve_rejects_unsupported_extension() {
    assert!(resolve_path("/nonexistent/image.gif").is_err());
  }

  #[test]
  fn hash_same_params_same_hash() {
    let p = ImageConvertParams {
      image_path: "/a/b.png".into(),
      output_width: 10,
      output_height: 5,
      ..Default::default()
    };
    let path = PathBuf::from("/a/b.png");
    assert_eq!(compute_hash(&path, &p), compute_hash(&path, &p));
  }

  #[test]
  fn hash_diff_params_diff_hash() {
    let p1 = ImageConvertParams {
      image_path: "/a/b.png".into(),
      output_width: 10,
      ..Default::default()
    };
    let p2 = ImageConvertParams {
      image_path: "/a/b.png".into(),
      output_width: 20,
      ..Default::default()
    };
    let path = PathBuf::from("/a/b.png");
    assert_ne!(compute_hash(&path, &p1), compute_hash(&path, &p2));
  }

  #[test]
  fn convert_returns_f_percent_prefix() {
    let mut svc = ImageService::new(None);
    let source = TestImage::new();
    let p = ImageConvertParams {
      image_path: source.path.to_string_lossy().into(),
      output_width: 20,
      output_height: 10,
      scale: 0.5,
      ..Default::default()
    };
    let result = svc.convert(p).expect("conversion should succeed");
    assert!(result.starts_with("f%"), "output must start with f%");
    assert!(result.contains('\u{2585}'), "output must contain ▅");
    assert!(result.contains("<fg:#"), "output must contain fg tags");
    assert!(result.contains("<bg:#"), "output must contain bg tags");
  }

  #[test]
  fn cache_returns_same_result() {
    let mut svc = ImageService::new(None);
    let source = TestImage::new();
    let p = ImageConvertParams {
      image_path: source.path.to_string_lossy().into(),
      output_width: 10,
      output_height: 5,
      scale: 0.3,
      cache: true,
      ..Default::default()
    };
    let r1 = svc.convert(p.clone()).expect("first call should succeed");
    let r2 = svc.convert(p).expect("second call (cached) should succeed");
    assert_eq!(r1, r2, "cached result must equal first result");
  }

  #[test]
  fn no_cache_returns_different_call() {
    let mut svc = ImageService::new(None);
    let source = TestImage::new();
    let p = ImageConvertParams {
      image_path: source.path.to_string_lossy().into(),
      output_width: 10,
      output_height: 5,
      scale: 0.3,
      cache: false,
      ..Default::default()
    };
    let r = svc.convert(p).expect("conversion should succeed");
    assert!(r.starts_with("f%"));
  }

  #[test]
  fn output_dimensions_match_params() {
    let mut svc = ImageService::new(None);
    let source = TestImage::new();
    let (w, h) = (15u32, 8u32);
    let p = ImageConvertParams {
      image_path: source.path.to_string_lossy().into(),
      output_width: w,
      output_height: h,
      scale: 0.3,
      ..Default::default()
    };
    let result = svc.convert(p).expect("conversion should succeed");

    let block_count = result.chars().filter(|&c| c == '\u{2585}').count();
    assert_eq!(block_count, (w * h) as usize);
    assert_eq!(result.lines().count(), h as usize);
  }

  #[test]
  fn disk_cache_writes_and_reads() {
    let tmp = std::env::temp_dir().join(format!("tg_image_test_{}", std::process::id()));
    let _ = fs::create_dir_all(&tmp);

    let source = TestImage::new();
    let p = ImageConvertParams {
      image_path: source.path.to_string_lossy().into(),
      output_width: 10,
      output_height: 5,
      scale: 0.3,
      cache: true,
      ..Default::default()
    };

    // First call: renders and writes the disk cache.
    let r1 = {
      let mut svc = ImageService::new(Some(tmp.clone()));
      svc.convert(p.clone()).expect("first convert")
    };

    // Second call: a new instance must read from the disk cache.
    let r2 = {
      let mut svc = ImageService::new(Some(tmp.clone()));
      svc.convert(p.clone()).expect("second convert (from disk)")
    };

    assert_eq!(r1, r2, "disk-cached result must match");

    // Clean up.
    let _ = fs::remove_dir_all(&tmp);
  }

  #[test]
  fn disk_cache_stale_when_source_changed() {
    let tmp = std::env::temp_dir().join(format!("tg_image_test2_{}", std::process::id()));
    let _ = fs::create_dir_all(&tmp);

    let source = TestImage::new();
    let p = ImageConvertParams {
      image_path: source.path.to_string_lossy().into(),
      output_width: 10,
      output_height: 5,
      scale: 0.3,
      cache: true,
      ..Default::default()
    };

    let mut svc = ImageService::new(Some(tmp.clone()));
    let _r1 = svc.convert(p.clone()).expect("first convert");

    // Tamper with the mtime recorded in the disk cache file to make the entry stale.
    let hash = {
      let resolved = resolve_path(&p.image_path).unwrap();
      compute_hash(&resolved, &p)
    };
    let cache_file = tmp.join(format!("{hash}.json"));
    assert!(cache_file.exists(), "cache file should exist");

    // Set source_modified to 0 so it no longer matches the source file's mtime.
    let raw = fs::read_to_string(&cache_file).unwrap();
    let stale = raw.replace(
      &format!("\"source_modified\":{}", {
        let meta = fs::metadata(&source.path).unwrap();
        meta
          .modified()
          .unwrap()
          .duration_since(UNIX_EPOCH)
          .unwrap()
          .as_secs()
      }),
      "\"source_modified\":0",
    );
    fs::write(&cache_file, stale).unwrap();

    // A new instance must detect the stale entry and render again.
    let mut svc2 = ImageService::new(Some(tmp.clone()));
    let r2 = svc2.convert(p).expect("stale cache should re-render");
    assert!(r2.starts_with("f%"));

    let _ = fs::remove_dir_all(&tmp);
  }

  #[derive(Debug)]
  enum TestEvent {
    Image(ImageEvent),
    Status(TaskStatusEvent),
  }

  impl From<ImageEvent> for TestEvent {
    fn from(event: ImageEvent) -> Self {
      Self::Image(event)
    }
  }

  impl From<TaskStatusEvent> for TestEvent {
    fn from(event: TaskStatusEvent) -> Self {
      Self::Status(event)
    }
  }

  fn wait_for_events(runtime: &AsyncRuntime<TestEvent>, count: usize) -> Vec<TestEvent> {
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut events = Vec::new();
    while events.len() < count && Instant::now() < deadline {
      events.extend(runtime.poll_events());
      std::thread::sleep(Duration::from_millis(2));
    }
    events
  }

  #[test]
  fn convert_async_reports_the_synchronous_output_then_finishes() {
    let source = TestImage::new();
    let params = ImageConvertParams {
      image_path: source.path.to_string_lossy().into(),
      output_width: 20,
      output_height: 10,
      ..Default::default()
    };
    let expected = ImageService::new(None)
      .convert(params.clone())
      .expect("synchronous conversion");
    let runtime = AsyncRuntime::<TestEvent>::with_worker_count(1);

    let id = ImageService::new(None).convert_async(&runtime, params);

    let events = wait_for_events(&runtime, 2);
    assert!(
      matches!(
        &events[..],
        [
          TestEvent::Image(ImageEvent::ConvertFinished { task_id, output }),
          TestEvent::Status(TaskStatusEvent::Finished { id: finished }),
        ] if *task_id == id && *output == expected && *finished == id
      ),
      "unexpected events: {events:?}"
    );
  }

  #[test]
  fn convert_async_reports_invalid_params_as_image_and_task_failure() {
    let runtime = AsyncRuntime::<TestEvent>::with_worker_count(1);

    let id = ImageService::new(None).convert_async(&runtime, ImageConvertParams::default());

    let events = wait_for_events(&runtime, 2);
    assert!(
      matches!(
        &events[..],
        [
          TestEvent::Image(ImageEvent::Failed { task_id, error }),
          TestEvent::Status(TaskStatusEvent::Failed { id: failed, error: status_error }),
        ] if *task_id == id && *failed == id && error == status_error
      ),
      "unexpected events: {events:?}"
    );
  }
}
