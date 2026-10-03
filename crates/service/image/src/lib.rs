//! Image-to-terminal block conversion with content-based memory and disk caches.
//!
//! # Examples
//!
//! ```rust
//! use std::{
//!   path::PathBuf,
//!   time::{Duration, Instant},
//! };
//!
//! use tg_service_async::{AsyncRuntime, TaskStatusEvent};
//! use tg_service_image::{ImageConvertParams, ImageEvent, ImageService};
//!
//! #[derive(Debug)]
//! enum Event {
//!   Image(ImageEvent),
//!     Status,
//! }
//!
//! impl From<ImageEvent> for Event {
//!   fn from(event: ImageEvent) -> Self {
//!     Self::Image(event)
//!   }
//! }
//!
//! impl From<TaskStatusEvent> for Event {
//!   fn from(_: TaskStatusEvent) -> Self {
//!     Self::Status
//!   }
//! }
//!
//! fn main() {
//!   let root = create_temp_dir("tg_image_smoke");
//!   let path = root.join("smoke.png");
//!   let mut pixels = image::RgbImage::new(8, 8);
//!   for (x, y, pixel) in pixels.enumerate_pixels_mut() {
//!     *pixel = image::Rgb([(x * 32) as u8, (y * 32) as u8, 128]);
//!   }
//!   pixels.save(&path).expect("write smoke image");
//!
//!   let params = ImageConvertParams {
//!     image_path: path.to_string_lossy().into(),
//!     output_width: Some(8),
//!     output_height: Some(4),
//!     cache: false,
//!     ..Default::default()
//!   };
//!   let mut service = ImageService::new(None);
//!   let rendered = service
//!     .convert(params.clone())
//!     .expect("synchronous conversion");
//!   assert!(rendered.starts_with("f%"));
//!
//!   let runtime = AsyncRuntime::<Event>::with_worker_count(1);
//!   let task = service.convert_async(&runtime, params);
//!   let deadline = Instant::now() + Duration::from_secs(5);
//!   let mut output = None;
//!   while output.is_none() && Instant::now() < deadline {
//!     for event in runtime.poll_events() {
//!       if let Event::Image(ImageEvent::ConvertFinished {
//!         task_id,
//!         output: text,
//!       }) = event
//!       {
//!         assert_eq!(task_id, task);
//!         output = Some(text);
//!       }
//!     }
//!     std::thread::sleep(Duration::from_millis(5));
//!   }
//!   assert_eq!(
//!     output.as_deref(),
//!     Some(rendered.as_str()),
//!     "async job renders the same text"
//!   );
//!
//!   std::fs::remove_dir_all(&root).expect("clean up temporary directory");
//!   println!("image ok: {} bytes of half-block text", rendered.len());
//! }
//!
//! fn create_temp_dir(prefix: &str) -> PathBuf {
//!   let nonce = std::time::SystemTime::now()
//!     .duration_since(std::time::UNIX_EPOCH)
//!     .unwrap_or_default()
//!     .as_nanos();
//!   let root = std::env::temp_dir().join(format!("{prefix}_{}_{nonce}", std::process::id()));
//!   std::fs::create_dir(&root)
//!     .unwrap_or_else(|error| panic!("create temporary directory {}: {error}", root.display()));
//!   root
//! }
//! ```

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use blake3::Hasher;
use crossbeam_channel::Sender;
use image::{GenericImageView, ImageReader, Limits};
use serde::{Deserialize, Serialize};
use tg_core_atomic_fs::atomic_write;
use tg_service_async::{AsyncJob, AsyncRuntime, TaskCancellation, TaskId, TaskStatusEvent};

/// Configuration values controlling image convert behavior.
///
/// # Fields
///
/// * `image_path` - The filesystem path for image.
/// * `mode` - The image convert mode carried by this image convert params.
/// * `background` - The background color.
/// * `output_width` - The output width in terminal columns.
/// * `output_height` - The output height in terminal rows.
/// * `crop_x` - The crop x.
/// * `crop_y` - The crop y.
/// * `crop_width` - The crop width in terminal columns.
/// * `crop_height` - The crop height in terminal rows.
/// * `square_crop` - The square crop.
/// * `scale` - The scale.
/// * `cache` - The cache.
#[derive(Clone, Debug)]
pub struct ImageConvertParams {
  /// The filesystem path for image.
  pub image_path: String,
  /// The image convert mode carried by this image convert params.
  pub mode: ImageConvertMode,

  /// The background color.
  pub background: [u8; 3],
  /// The output width in terminal columns.
  pub output_width: Option<u32>,
  /// The output height in terminal rows.
  pub output_height: Option<u32>,
  /// The crop x.
  pub crop_x: i32,
  /// The crop y.
  pub crop_y: i32,
  /// The crop width in terminal columns.
  pub crop_width: Option<u32>,
  /// The crop height in terminal rows.
  pub crop_height: Option<u32>,
  /// The square crop.
  pub square_crop: bool,
  /// The scale.
  pub scale: f64,
  /// The cache.
  pub cache: bool,
}

/// The half-block or block-mask sampling rule used for image conversion.
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq)]
pub enum ImageConvertMode {
  /// The half block setting for image convert mode.
  #[default]
  HalfBlock,

  /// The mix block setting for image convert mode.
  MixBlock,
}

impl Default for ImageConvertParams {
  fn default() -> Self {
    Self {
      image_path: String::new(),
      mode: ImageConvertMode::HalfBlock,
      background: [0, 0, 0],
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

/// The inputs of an asynchronous image operation.
#[derive(Clone, Debug)]
pub enum ImageTask {
  /// The convert setting for image task.
  Convert {
    /// The formatting or rendering parameters.
    params: ImageConvertParams,
    /// The cache dir.
    cache_dir: Option<PathBuf>,
  },
}

/// A image event payload queued for its owning consumer.
#[derive(Clone, Debug)]
pub enum ImageEvent {
  /// A convert finished notification delivered to the owning consumer.
  ConvertFinished {
    /// The identifier of the asynchronous task.
    task_id: TaskId,
    /// The output.
    output: String,
  },
  /// A failed notification delivered to the owning consumer.
  Failed {
    /// The identifier of the asynchronous task.
    task_id: TaskId,
    /// The error.
    error: String,
  },
}

impl<E: From<ImageEvent> + Send + 'static> AsyncJob<E> for ImageTask {
  fn run(
    self: Box<Self>,
    task_id: TaskId,
    events: &Sender<E>,
    cancellation: &TaskCancellation,
  ) -> Result<(), String> {
    match *self {
      ImageTask::Convert { params, cache_dir } => {
        let mut service = ImageService::new(cache_dir);
        match service.convert_cancellable(params, Some(cancellation)) {
          Ok(Some(output)) if !cancellation.is_cancelled() => {
            let _ = events.send(ImageEvent::ConvertFinished { task_id, output }.into());
            Ok(())
          }
          Ok(Some(_)) | Ok(None) => Ok(()),
          Err(error) => {
            if cancellation.is_cancelled() {
              return Ok(());
            }
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

/// The public entry point for image operations.
pub struct ImageService {
  cache: HashMap<[u8; 32], String>,
  cache_dir: Option<PathBuf>,
}

const MAX_SOURCE_FILE_BYTES: u64 = 32 * 1024 * 1024;
const MAX_SOURCE_PIXELS: u64 = 16_000_000;
const MAX_SOURCE_DIMENSION: u32 = 16_384;
const MAX_DECODE_ALLOCATION_BYTES: u64 = 80 * 1024 * 1024;
const MAX_OUTPUT_DIMENSION: u32 = 2_048;
const MAX_OUTPUT_CELLS: u64 = 16_384;
const MAX_SCALED_PIXELS: u64 = 16_000_000;
const MAX_CACHE_ENTRY_BYTES: u64 = 2 * 1024 * 1024;
// Serialize cache commits because the atomic writer uses one temporary sibling per destination.

static CACHE_WRITE_LOCK: Mutex<()> = Mutex::new(());

#[derive(Serialize, Deserialize)]
struct DiskCacheEntry {
  format_version: u8,
  rendered: String,
}

impl ImageService {
  /// Create an image service initialized from `cache_dir`.
  pub fn new(cache_dir: Option<PathBuf>) -> Self {
    Self {
      cache: HashMap::new(),
      cache_dir,
    }
  }

  /// Convert an image into terminal block art using validated dimensions, crop settings, and
  /// content caches.
  ///
  /// # Errors
  ///
  /// Return an error for invalid dimensions or crop bounds, an unresolved PNG/JPEG path,
  /// unreadable image data, or an empty crop. Disposable cache-write failures do not invalidate a
  /// successful conversion.
  pub fn convert(&mut self, params: ImageConvertParams) -> Result<String, String> {
    self
      .convert_cancellable(params, None)?
      .ok_or_else(|| "image conversion was cancelled".to_string())
  }

  fn convert_cancellable(
    &mut self,
    params: ImageConvertParams,
    cancellation: Option<&TaskCancellation>,
  ) -> Result<Option<String>, String> {
    if is_cancelled(cancellation) {
      return Ok(None);
    }
    validate(&params)?;

    let resolved = resolve_path(&params.image_path)?;
    if is_cancelled(cancellation) {
      return Ok(None);
    }
    let source_bytes = read_source(&resolved)?;
    if is_cancelled(cancellation) {
      return Ok(None);
    }
    let cache_key = compute_cache_key(&source_bytes, &params);

    if params.cache {
      if let Some(cached) = self.cache.get(&cache_key) {
        return Ok((!is_cancelled(cancellation)).then(|| cached.clone()));
      }

      if let Some(disk) = self.read_disk_cache(&cache_key) {
        self.cache.insert(cache_key, disk.clone());
        return Ok((!is_cancelled(cancellation)).then_some(disk));
      }
    }

    if is_cancelled(cancellation) {
      return Ok(None);
    }
    let img = decode_image(&source_bytes, &resolved)?;
    if is_cancelled(cancellation) {
      return Ok(None);
    }

    let Some(result) = process_cancellable(&img, &params, cancellation)? else {
      return Ok(None);
    };

    if params.cache {
      if is_cancelled(cancellation) {
        return Ok(None);
      }
      self.cache.insert(cache_key, result.clone());
      self.write_disk_cache(&cache_key, &result, cancellation);
    }
    Ok((!is_cancelled(cancellation)).then_some(result))
  }

  /// Queue image conversion and return the task identifier used by completion events.
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

  fn disk_cache_path(&self, cache_key: &[u8; 32]) -> Option<PathBuf> {
    self.cache_dir.as_ref().map(|dir| {
      dir.join(format!(
        "v{}-{}.json",
        tg_core_version::IMAGE_CACHE_FORMAT_VERSION,
        digest_hex(cache_key)
      ))
    })
  }

  fn read_disk_cache(&self, cache_key: &[u8; 32]) -> Option<String> {
    let path = self.disk_cache_path(cache_key)?;
    let mut bytes = Vec::new();
    File::open(&path)
      .ok()?
      .take(MAX_CACHE_ENTRY_BYTES + 1)
      .read_to_end(&mut bytes)
      .ok()?;
    if bytes.len() as u64 > MAX_CACHE_ENTRY_BYTES {
      return None;
    }
    let data = std::str::from_utf8(&bytes).ok()?;
    let entry: DiskCacheEntry = serde_json::from_str(data).ok()?;
    (entry.format_version == tg_core_version::IMAGE_CACHE_FORMAT_VERSION).then_some(entry.rendered)
  }

  fn write_disk_cache(
    &self,
    cache_key: &[u8; 32],
    rendered: &str,
    cancellation: Option<&TaskCancellation>,
  ) {
    let Some(path) = self.disk_cache_path(cache_key) else {
      return;
    };
    if let Some(parent) = path.parent()
      && fs::create_dir_all(parent).is_err()
    {
      return;
    }
    let entry = DiskCacheEntry {
      format_version: tg_core_version::IMAGE_CACHE_FORMAT_VERSION,
      rendered: rendered.to_string(),
    };
    if let Ok(json) = serde_json::to_vec(&entry) {
      // Cache persistence is optional; a valid conversion still succeeds when its cache write
      // fails.

      let _guard = CACHE_WRITE_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
      if is_cancelled(cancellation) {
        return;
      }
      let _ = atomic_write(&path, &json, false);
    }
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
  if p
    .output_width
    .is_some_and(|width| width > MAX_OUTPUT_DIMENSION)
  {
    return Err(format!("output_width 不得超过 {MAX_OUTPUT_DIMENSION}"));
  }
  if p
    .output_height
    .is_some_and(|height| height > MAX_OUTPUT_DIMENSION)
  {
    return Err(format!("output_height 不得超过 {MAX_OUTPUT_DIMENSION}"));
  }
  if !p.scale.is_finite() || p.scale <= 0.0 {
    return Err("scale 必须是有限正数".into());
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
  let file =
    File::open(path).map_err(|error| format!("无法读取图片 {}: {error}", path.display()))?;
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
  let mut limits = Limits::default();
  limits.max_image_width = Some(MAX_SOURCE_DIMENSION);
  limits.max_image_height = Some(MAX_SOURCE_DIMENSION);
  limits.max_alloc = Some(MAX_DECODE_ALLOCATION_BYTES);
  reader.limits(limits);
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

fn compute_cache_key(source_bytes: &[u8], p: &ImageConvertParams) -> [u8; 32] {
  let mut hasher = Hasher::new();
  hasher.update(b"tg-service-image-cache\0");
  hasher.update(&[tg_core_version::IMAGE_CACHE_FORMAT_VERSION]);
  hasher.update(&(source_bytes.len() as u64).to_le_bytes());
  hasher.update(source_bytes);
  hasher.update(&[match p.mode {
    ImageConvertMode::HalfBlock => 0,
    ImageConvertMode::MixBlock => 1,
  }]);
  hasher.update(&p.background);
  hash_optional_u32(&mut hasher, p.output_width);
  hash_optional_u32(&mut hasher, p.output_height);
  hasher.update(&p.crop_x.to_le_bytes());
  hasher.update(&p.crop_y.to_le_bytes());
  hash_optional_u32(&mut hasher, p.crop_width);
  hash_optional_u32(&mut hasher, p.crop_height);
  hasher.update(&[u8::from(p.square_crop)]);
  hasher.update(&p.scale.to_bits().to_le_bytes());
  *hasher.finalize().as_bytes()
}

fn hash_optional_u32(hasher: &mut Hasher, value: Option<u32>) {
  match value {
    Some(value) => {
      hasher.update(&[1]);
      hasher.update(&value.to_le_bytes());
    }
    None => {
      hasher.update(&[0]);
    }
  }
}

fn digest_hex(digest: &[u8; 32]) -> String {
  let mut output = String::with_capacity(64);
  for byte in digest {
    use std::fmt::Write as _;
    let _ = write!(output, "{byte:02x}");
  }
  output
}

#[cfg(test)]
fn process(img: &image::DynamicImage, p: &ImageConvertParams) -> Result<String, String> {
  process_cancellable(img, p, None)?.ok_or_else(|| "image conversion was cancelled".to_string())
}

fn is_cancelled(cancellation: Option<&TaskCancellation>) -> bool {
  cancellation.is_some_and(TaskCancellation::is_cancelled)
}

fn process_cancellable(
  img: &image::DynamicImage,
  p: &ImageConvertParams,
  cancellation: Option<&TaskCancellation>,
) -> Result<Option<String>, String> {
  if is_cancelled(cancellation) {
    return Ok(None);
  }
  let (src_w, src_h) = img.dimensions();
  let (cx, cy, cw, ch) = crop_area(src_w, src_h, p)?;
  let (output_width, output_height) = output_dimensions(src_w, src_h, p)?;
  let (scaled_width, scaled_height) = scaled_dimensions(cw, ch, p.scale)?;

  let mut rgba = img.to_rgba8();
  if !composite_on_background_cancellable(&mut rgba, p.background, cancellation) {
    return Ok(None);
  }
  if is_cancelled(cancellation) {
    return Ok(None);
  }
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
  if is_cancelled(cancellation) {
    return Ok(None);
  }

  let (samples_per_cell_x, samples_per_cell_y) = match p.mode {
    ImageConvertMode::HalfBlock => (1, 2),
    // Sample an 8-by-8 glyph mask with twice the vertical resolution to match terminal-cell
    // proportions.
    ImageConvertMode::MixBlock => (8, 16),
  };
  let pixel_width = output_width
    .checked_mul(samples_per_cell_x)
    .ok_or_else(|| "输出图片宽度超出支持范围".to_string())?;
  let pixel_height = output_height
    .checked_mul(samples_per_cell_y)
    .ok_or_else(|| "输出图片高度超出支持范围".to_string())?;
  let resized = image::imageops::resize(
    &scaled,
    pixel_width,
    pixel_height,
    image::imageops::FilterType::Lanczos3,
  );
  if is_cancelled(cancellation) {
    return Ok(None);
  }

  Ok(match p.mode {
    ImageConvertMode::HalfBlock => {
      sample_halfblock(&resized, pixel_width, pixel_height, cancellation)
    }
    ImageConvertMode::MixBlock => {
      sample_mixblock(&resized, pixel_width, pixel_height, cancellation)
    }
  })
}

#[cfg(test)]
fn composite_on_background(rgba: &mut image::RgbaImage, background: [u8; 3]) {
  let _ = composite_on_background_cancellable(rgba, background, None);
}

fn composite_on_background_cancellable(
  rgba: &mut image::RgbaImage,
  background: [u8; 3],
  cancellation: Option<&TaskCancellation>,
) -> bool {
  for (x, _, pixel) in rgba.enumerate_pixels_mut() {
    if x == 0 && is_cancelled(cancellation) {
      return false;
    }
    let alpha = u32::from(pixel[3]);
    for channel in 0..3 {
      let source = u32::from(pixel[channel]);
      let backdrop = u32::from(background[channel]);
      pixel[channel] = ((source * alpha + backdrop * (255 - alpha) + 127) / 255) as u8;
    }
    pixel[3] = 255;
  }
  true
}

fn crop_area(
  source_width: u32,
  source_height: u32,
  params: &ImageConvertParams,
) -> Result<(u32, u32, u32, u32), String> {
  if params.square_crop {
    let side = source_width.min(source_height);
    return Ok((
      (source_width - side) / 2,
      (source_height - side) / 2,
      side,
      side,
    ));
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
  if x
    .checked_add(width)
    .is_none_or(|right| right > source_width)
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
    return Err(format!("输出单元数 {cells} 超过 {MAX_OUTPUT_CELLS} 限制"));
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

fn sample_halfblock(
  rgba: &image::RgbaImage,
  w: u32,
  h: u32,
  cancellation: Option<&TaskCancellation>,
) -> Option<String> {
  let char_rows = h / 2;
  let cap = (w as usize * char_rows as usize) * 18 + 2;
  let mut out = String::with_capacity(cap);
  out.push_str("f%");

  for cy in 0..char_rows {
    if is_cancelled(cancellation) {
      return None;
    }
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
  Some(out)
}

const MIX_BLOCK_CANDIDATES: [(char, u64); 29] = [
  ('▀', block_glyph_mask('▀')),
  ('▁', block_glyph_mask('▁')),
  ('▂', block_glyph_mask('▂')),
  ('▃', block_glyph_mask('▃')),
  ('▄', block_glyph_mask('▄')),
  ('▅', block_glyph_mask('▅')),
  ('▆', block_glyph_mask('▆')),
  ('▇', block_glyph_mask('▇')),
  ('█', block_glyph_mask('█')),
  ('▉', block_glyph_mask('▉')),
  ('▊', block_glyph_mask('▊')),
  ('▋', block_glyph_mask('▋')),
  ('▌', block_glyph_mask('▌')),
  ('▍', block_glyph_mask('▍')),
  ('▎', block_glyph_mask('▎')),
  ('▏', block_glyph_mask('▏')),
  ('▐', block_glyph_mask('▐')),
  ('▔', block_glyph_mask('▔')),
  ('▕', block_glyph_mask('▕')),
  ('▖', block_glyph_mask('▖')),
  ('▗', block_glyph_mask('▗')),
  ('▘', block_glyph_mask('▘')),
  ('▙', block_glyph_mask('▙')),
  ('▚', block_glyph_mask('▚')),
  ('▛', block_glyph_mask('▛')),
  ('▜', block_glyph_mask('▜')),
  ('▝', block_glyph_mask('▝')),
  ('▞', block_glyph_mask('▞')),
  ('▟', block_glyph_mask('▟')),
];

#[derive(Clone, Copy)]
struct MixCandidate {
  glyph: char,
  foreground: Rgb,
  background: Rgb,
  error: u64,
}

fn sample_mixblock(
  rgba: &image::RgbaImage,
  width: u32,
  height: u32,
  cancellation: Option<&TaskCancellation>,
) -> Option<String> {
  const CELL_SAMPLE_WIDTH: u32 = 8;
  const CELL_SAMPLE_HEIGHT: u32 = 16;

  let columns = width / CELL_SAMPLE_WIDTH;
  let rows = height / CELL_SAMPLE_HEIGHT;
  let capacity = columns as usize * rows as usize * 32 + 2;
  let mut output = String::with_capacity(capacity);
  output.push_str("f%");

  for cell_y in 0..rows {
    if is_cancelled(cancellation) {
      return None;
    }
    if cell_y > 0 {
      output.push('\n');
    }
    let mut previous_foreground = None;
    let mut previous_background = None;

    for cell_x in 0..columns {
      let samples = sample_mix_cell(rgba, cell_x, cell_y);
      let chosen = best_mix_candidate(&samples);

      if previous_background != Some(chosen.background) {
        previous_background = Some(chosen.background);
        output.push_str(&format!(
          "<bg:#{:02x}{:02x}{:02x}>",
          chosen.background.0, chosen.background.1, chosen.background.2
        ));
      }
      if previous_foreground != Some(chosen.foreground) {
        previous_foreground = Some(chosen.foreground);
        output.push_str(&format!(
          "<fg:#{:02x}{:02x}{:02x}>",
          chosen.foreground.0, chosen.foreground.1, chosen.foreground.2
        ));
      }
      output.push(chosen.glyph);
    }
  }
  Some(output)
}

fn sample_mix_cell(rgba: &image::RgbaImage, cell_x: u32, cell_y: u32) -> [Rgb; 64] {
  let mut samples = [Rgb(0, 0, 0); 64];
  for sub_y in 0..8 {
    for sub_x in 0..8 {
      let x = cell_x * 8 + sub_x;
      let y = cell_y * 16 + sub_y * 2;
      let first = get_rgb(rgba, x, y);
      let second = get_rgb(rgba, x, y + 1);
      samples[(sub_y * 8 + sub_x) as usize] = Rgb(
        mean_pair(first.0, second.0),
        mean_pair(first.1, second.1),
        mean_pair(first.2, second.2),
      );
    }
  }
  samples
}

fn mean_pair(first: u8, second: u8) -> u8 {
  (u16::from(first) + u16::from(second)).div_ceil(2) as u8
}

fn best_mix_candidate(samples: &[Rgb; 64]) -> MixCandidate {
  let all_color = mean_rgb(samples, |_| true).expect("each cell has 64 color samples");
  let mut best = None;
  for &(glyph, mask) in &MIX_BLOCK_CANDIDATES {
    let candidate = score_mix_candidate(glyph, mask, samples, all_color);
    if best.is_none_or(|current: MixCandidate| candidate.error < current.error) {
      // Retain the first equal-scoring glyph in codepoint order for reproducible
      // platform-independent choices.

      best = Some(candidate);
    }
  }
  best.expect("the mix-block candidate table must not be empty")
}

fn score_mix_candidate(
  glyph: char,
  mask: u64,
  samples: &[Rgb; 64],
  all_color: Rgb,
) -> MixCandidate {
  let mut foreground_sum = [0u32; 3];
  let mut background_sum = [0u32; 3];
  let mut foreground_count = 0u32;
  let mut background_count = 0u32;
  for (index, sample) in samples.iter().enumerate() {
    let (sum, count) = if mask & (1u64 << index) != 0 {
      (&mut foreground_sum, &mut foreground_count)
    } else {
      (&mut background_sum, &mut background_count)
    };
    sum[0] += u32::from(sample.0);
    sum[1] += u32::from(sample.1);
    sum[2] += u32::from(sample.2);
    *count += 1;
  }
  let foreground = rounded_mean(foreground_sum, foreground_count).unwrap_or(all_color);
  let background = rounded_mean(background_sum, background_count).unwrap_or(all_color);
  let mut error = 0u64;
  for (index, sample) in samples.iter().enumerate() {
    let representative = if mask & (1u64 << index) != 0 {
      foreground
    } else {
      background
    };
    for (value, mean) in [
      (sample.0, representative.0),
      (sample.1, representative.1),
      (sample.2, representative.2),
    ] {
      let delta = i32::from(value) - i32::from(mean);
      error += (delta * delta) as u64;
    }
  }
  MixCandidate {
    glyph,
    foreground,
    background,
    error,
  }
}

fn mean_rgb(samples: &[Rgb; 64], mut include: impl FnMut(usize) -> bool) -> Option<Rgb> {
  let (mut red, mut green, mut blue, mut count) = (0u32, 0u32, 0u32, 0u32);
  for (index, sample) in samples.iter().enumerate() {
    if include(index) {
      red += u32::from(sample.0);
      green += u32::from(sample.1);
      blue += u32::from(sample.2);
      count += 1;
    }
  }
  rounded_mean([red, green, blue], count)
}

fn rounded_mean([red, green, blue]: [u32; 3], count: u32) -> Option<Rgb> {
  (count > 0).then(|| {
    let round = count / 2;
    Rgb(
      ((red + round) / count) as u8,
      ((green + round) / count) as u8,
      ((blue + round) / count) as u8,
    )
  })
}

const fn rectangle_mask(x: u32, y: u32, width: u32, height: u32) -> u64 {
  let mut mask = 0u64;
  let mut row = y;
  while row < y + height {
    let mut column = x;
    while column < x + width {
      mask |= 1u64 << (row * 8 + column);
      column += 1;
    }
    row += 1;
  }
  mask
}

const fn block_glyph_mask(glyph: char) -> u64 {
  match glyph {
    '▀' => rectangle_mask(0, 0, 8, 4),
    '▁' => rectangle_mask(0, 7, 8, 1),
    '▂' => rectangle_mask(0, 6, 8, 2),
    '▃' => rectangle_mask(0, 5, 8, 3),
    '▄' => rectangle_mask(0, 4, 8, 4),
    '▅' => rectangle_mask(0, 3, 8, 5),
    '▆' => rectangle_mask(0, 2, 8, 6),
    '▇' => rectangle_mask(0, 1, 8, 7),
    '█' => rectangle_mask(0, 0, 8, 8),
    '▉' => rectangle_mask(0, 0, 7, 8),
    '▊' => rectangle_mask(0, 0, 6, 8),
    '▋' => rectangle_mask(0, 0, 5, 8),
    '▌' => rectangle_mask(0, 0, 4, 8),
    '▍' => rectangle_mask(0, 0, 3, 8),
    '▎' => rectangle_mask(0, 0, 2, 8),
    '▏' => rectangle_mask(0, 0, 1, 8),
    '▐' => rectangle_mask(4, 0, 4, 8),
    '▔' => rectangle_mask(0, 0, 8, 1),
    '▕' => rectangle_mask(7, 0, 1, 8),
    '▖' => rectangle_mask(0, 4, 4, 4),
    '▗' => rectangle_mask(4, 4, 4, 4),
    '▘' => rectangle_mask(0, 0, 4, 4),
    '▙' => rectangle_mask(0, 0, 4, 8) | rectangle_mask(4, 4, 4, 4),
    '▚' => rectangle_mask(0, 0, 4, 4) | rectangle_mask(4, 4, 4, 4),
    '▛' => rectangle_mask(0, 0, 4, 8) | rectangle_mask(4, 0, 4, 4),
    '▜' => rectangle_mask(0, 0, 8, 4) | rectangle_mask(4, 4, 4, 4),
    '▝' => rectangle_mask(4, 0, 4, 4),
    '▞' => rectangle_mask(4, 0, 4, 4) | rectangle_mask(0, 4, 4, 4),
    '▟' => rectangle_mask(4, 0, 4, 8) | rectangle_mask(0, 4, 4, 4),
    _ => 0,
  }
}

fn get_rgb(rgba: &image::RgbaImage, x: u32, y: u32) -> Rgb {
  let p = rgba.get_pixel(x, y).0;
  Rgb(p[0], p[1], p[2])
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::fs::OpenOptions;
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
  fn alpha_compositing_uses_rounded_rgb_channel_blending() {
    let mut rgba = image::RgbaImage::new(3, 1);
    rgba.put_pixel(0, 0, image::Rgba([255, 12, 4, 0]));
    rgba.put_pixel(1, 0, image::Rgba([200, 100, 0, 128]));
    rgba.put_pixel(2, 0, image::Rgba([200, 100, 0, 255]));

    composite_on_background(&mut rgba, [12, 34, 56]);

    assert_eq!(rgba.get_pixel(0, 0).0, [12, 34, 56, 255]);
    assert_eq!(rgba.get_pixel(1, 0).0, [106, 67, 28, 255]);
    assert_eq!(rgba.get_pixel(2, 0).0, [200, 100, 0, 255]);
  }

  #[test]
  fn transparent_source_renders_with_the_selected_background() {
    let img = image::DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(
      2,
      2,
      image::Rgba([240, 16, 32, 0]),
    ));
    let params = ImageConvertParams {
      output_width: Some(1),
      output_height: Some(1),
      background: [12, 34, 56],
      ..Default::default()
    };

    let output = process(&img, &params).expect("transparent image should render");

    assert!(output.contains("<bg:#0c2238>"));
    assert!(output.contains("<fg:#0c2238>"));
    assert!(
      output.ends_with('▅'),
      "half-block output remains the baseline"
    );

    let mix_params = ImageConvertParams {
      mode: ImageConvertMode::MixBlock,
      ..params
    };
    let mix_output = process(&img, &mix_params).expect("mix-block image should render");
    assert!(mix_output.contains("<bg:#0c2238>"));
    assert!(
      mix_output.ends_with('▀'),
      "solid cells use stable tie ordering"
    );
  }

  #[test]
  fn cancelled_image_task_does_not_emit_a_completion_event() {
    let fixture = TestImage::new();
    let cancellation = TaskCancellation::new(TaskId(42));
    cancellation.cancel();
    let (sender, receiver) = crossbeam_channel::unbounded::<ImageEvent>();
    let task = Box::new(ImageTask::Convert {
      params: ImageConvertParams {
        image_path: fixture.path.to_string_lossy().into_owned(),
        ..Default::default()
      },
      cache_dir: None,
    });

    task
      .run(TaskId(42), &sender, &cancellation)
      .expect("cancelled conversions finish without reporting a failure");
    assert!(receiver.try_recv().is_err());
  }

  #[test]
  fn mix_block_masks_are_supported_sorted_and_include_the_half_baseline() {
    assert_eq!(MIX_BLOCK_CANDIDATES.len(), 29);
    assert!(
      MIX_BLOCK_CANDIDATES
        .iter()
        .all(|(_, mask)| mask.count_ones() > 0)
    );
    assert!(
      MIX_BLOCK_CANDIDATES
        .windows(2)
        .all(|pair| pair[0].0 < pair[1].0)
    );
    let unique_masks = MIX_BLOCK_CANDIDATES
      .iter()
      .map(|(_, mask)| *mask)
      .collect::<std::collections::HashSet<_>>();
    assert_eq!(unique_masks.len(), MIX_BLOCK_CANDIDATES.len());
    assert_eq!(block_glyph_mask('▅').count_ones(), 40);
    assert_eq!(block_glyph_mask('▘').count_ones(), 16);
    assert_eq!(block_glyph_mask('▘') & 1, 1);
    assert_eq!(block_glyph_mask('▘') & (1 << 63), 0);
    assert!(MIX_BLOCK_CANDIDATES.iter().any(|(glyph, _)| *glyph == '▅'));

    let solid = [Rgb(17, 28, 39); 64];
    let selected = best_mix_candidate(&solid);
    assert_eq!(selected.glyph, '▀', "equal scores use codepoint order");
    assert_eq!(selected.foreground, Rgb(17, 28, 39));
    assert_eq!(selected.background, Rgb(17, 28, 39));
    assert_eq!(selected.error, 0);
  }

  #[test]
  fn mix_block_selects_exact_quadrant_and_is_no_worse_than_half_baseline() {
    let quadrant = std::array::from_fn(|index| {
      let x = index % 8;
      let y = index / 8;
      if x < 4 && y < 4 {
        Rgb(250, 20, 30)
      } else {
        Rgb(0, 0, 200)
      }
    });
    let selected = best_mix_candidate(&quadrant);
    assert_eq!(selected.glyph, '▘');
    assert_eq!(selected.foreground, Rgb(250, 20, 30));
    assert_eq!(selected.background, Rgb(0, 0, 200));
    assert_eq!(selected.error, 0);

    let varied = std::array::from_fn(|index| {
      let x = (index % 8) as u8;
      let y = (index / 8) as u8;
      Rgb(x * 31, y * 29, x.wrapping_mul(y).wrapping_mul(7))
    });
    let mix_error = best_mix_candidate(&varied).error;
    let all_color = mean_rgb(&varied, |_| true).unwrap();
    let half_error = score_mix_candidate('▅', block_glyph_mask('▅'), &varied, all_color).error;
    assert!(mix_error <= half_error);
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
      output_width: Some(0),
      ..Default::default()
    };
    assert!(validate(&p).is_err());
  }

  #[test]
  fn validate_rejects_zero_height() {
    let p = ImageConvertParams {
      image_path: "x".into(),
      output_height: Some(0),
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
  fn validate_rejects_non_finite_scale() {
    for scale in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
      let params = ImageConvertParams {
        image_path: "x".into(),
        scale,
        ..Default::default()
      };
      assert!(validate(&params).is_err(), "scale {scale} must be rejected");
    }
  }

  #[test]
  fn validate_rejects_negative_crop_offsets() {
    for (crop_x, crop_y) in [(-1, 0), (0, -1)] {
      let params = ImageConvertParams {
        image_path: "x".into(),
        crop_x,
        crop_y,
        ..Default::default()
      };
      assert!(validate(&params).is_err());
    }
  }

  #[test]
  fn validate_accepts_valid_params() {
    let p = ImageConvertParams {
      image_path: "test.jpg".into(),
      output_width: Some(40),
      output_height: Some(20),
      ..Default::default()
    };
    assert!(validate(&p).is_ok());
  }

  #[test]
  fn output_defaults_follow_documented_ratio_and_never_become_zero() {
    let params = ImageConvertParams {
      output_width: None,
      output_height: None,
      ..Default::default()
    };
    assert_eq!(output_dimensions(250, 400, &params), Ok((2, 2)));
    assert_eq!(output_dimensions(1, 1, &params), Ok((1, 1)));
  }

  #[test]
  fn crop_defaults_to_the_remaining_source_region() {
    let params = ImageConvertParams {
      crop_x: 2,
      crop_y: 1,
      crop_width: None,
      crop_height: None,
      ..Default::default()
    };
    assert_eq!(crop_area(10, 8, &params), Ok((2, 1, 8, 7)));
  }

  #[test]
  fn crop_rejects_out_of_bounds_origin_and_rectangle() {
    let outside_origin = ImageConvertParams {
      crop_x: 10,
      ..Default::default()
    };
    assert!(crop_area(10, 8, &outside_origin).is_err());

    let outside_width = ImageConvertParams {
      crop_x: 8,
      crop_width: Some(3),
      ..Default::default()
    };
    assert!(crop_area(10, 8, &outside_width).is_err());

    let outside_height = ImageConvertParams {
      crop_y: 7,
      crop_height: Some(2),
      ..Default::default()
    };
    assert!(crop_area(10, 8, &outside_height).is_err());
  }

  #[test]
  fn output_dimensions_reject_oversized_cell_count() {
    let params = ImageConvertParams {
      output_width: Some(2_048),
      output_height: Some(16),
      ..Default::default()
    };
    assert!(output_dimensions(100, 100, &params).is_err());
  }

  #[test]
  fn source_dimensions_reject_pixel_and_axis_budgets() {
    let path = Path::new("large.png");
    assert!(validate_source_dimensions(4_001, 4_000, path).is_err());
    assert!(validate_source_dimensions(MAX_SOURCE_DIMENSION + 1, 1, path).is_err());
    assert!(validate_source_dimensions(0, 1, path).is_err());
  }

  #[test]
  fn resolve_rejects_unsupported_extension() {
    assert!(resolve_path("/nonexistent/image.gif").is_err());
  }

  #[test]
  fn cache_key_is_stable_for_same_source_and_params() {
    let p = ImageConvertParams {
      image_path: "/a/b.png".into(),
      output_width: Some(10),
      output_height: Some(5),
      ..Default::default()
    };
    assert_eq!(
      compute_cache_key(b"image-bytes", &p),
      compute_cache_key(b"image-bytes", &p)
    );
  }

  #[test]
  fn cache_key_changes_for_output_params_or_source_bytes() {
    let p1 = ImageConvertParams {
      image_path: "/a/b.png".into(),
      output_width: Some(10),
      ..Default::default()
    };
    assert_ne!(
      compute_cache_key(b"first", &p1),
      compute_cache_key(b"other", &p1)
    );

    let variants = [
      ImageConvertParams {
        output_width: Some(20),
        ..p1.clone()
      },
      ImageConvertParams {
        output_height: Some(25),
        ..p1.clone()
      },
      ImageConvertParams {
        crop_x: 1,
        ..p1.clone()
      },
      ImageConvertParams {
        crop_y: 1,
        ..p1.clone()
      },
      ImageConvertParams {
        crop_width: Some(3),
        ..p1.clone()
      },
      ImageConvertParams {
        crop_height: Some(3),
        ..p1.clone()
      },
      ImageConvertParams {
        square_crop: true,
        ..p1.clone()
      },
      ImageConvertParams {
        mode: ImageConvertMode::MixBlock,
        ..p1.clone()
      },
      ImageConvertParams {
        background: [12, 34, 56],
        ..p1.clone()
      },
      ImageConvertParams {
        scale: 1.000_000_1,
        ..p1.clone()
      },
      ImageConvertParams {
        scale: 1.000_000_2,
        ..p1.clone()
      },
    ];
    let baseline = compute_cache_key(b"image-bytes", &p1);
    for variant in variants {
      assert_ne!(
        baseline,
        compute_cache_key(b"image-bytes", &variant),
        "each output parameter must contribute to the cache key"
      );
    }
  }

  #[test]
  fn convert_returns_f_percent_prefix() {
    let mut svc = ImageService::new(None);
    let source = TestImage::new();
    let p = ImageConvertParams {
      image_path: source.path.to_string_lossy().into(),
      output_width: Some(20),
      output_height: Some(10),
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
      output_width: Some(10),
      output_height: Some(5),
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
      output_width: Some(10),
      output_height: Some(5),
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
      output_width: Some(w),
      output_height: Some(h),
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
    let tmp = std::env::temp_dir().join(format!(
      "tg_image_cache_{}_{}",
      std::process::id(),
      NEXT_TEST_IMAGE_ID.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::create_dir_all(&tmp);

    let source = TestImage::new();
    let p = ImageConvertParams {
      image_path: source.path.to_string_lossy().into(),
      output_width: Some(10),
      output_height: Some(5),
      scale: 0.3,
      cache: true,
      ..Default::default()
    };

    let (r1, cache_path) = {
      let mut svc = ImageService::new(Some(tmp.clone()));
      let rendered = svc.convert(p.clone()).expect("first convert");
      let source_bytes = fs::read(&source.path).unwrap();
      let key = compute_cache_key(&source_bytes, &p);
      (rendered, svc.disk_cache_path(&key).unwrap())
    };
    assert!(cache_path.is_file(), "disk cache entry should be written");

    let r2 = {
      let mut svc = ImageService::new(Some(tmp.clone()));
      svc.convert(p.clone()).expect("second convert (from disk)")
    };

    assert_eq!(r1, r2, "disk-cached result must match");

    let _ = fs::remove_dir_all(&tmp);
  }

  #[test]
  fn disk_cache_uses_content_when_same_path_size_and_mtime_change() {
    let tmp = std::env::temp_dir().join(format!(
      "tg_image_content_cache_{}_{}",
      std::process::id(),
      NEXT_TEST_IMAGE_ID.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::create_dir_all(&tmp);

    let source = TestImage::new();
    image::RgbImage::from_pixel(4, 4, image::Rgb([255, 0, 0]))
      .save(&source.path)
      .expect("write initial source image");
    let p = ImageConvertParams {
      image_path: source.path.to_string_lossy().into(),
      output_width: Some(10),
      output_height: Some(5),
      scale: 0.3,
      cache: true,
      ..Default::default()
    };

    let original_bytes = fs::read(&source.path).unwrap();
    let original_mtime = fs::metadata(&source.path).unwrap().modified().unwrap();
    let r1 = ImageService::new(Some(tmp.clone()))
      .convert(p.clone())
      .expect("first convert");

    image::RgbImage::from_pixel(4, 4, image::Rgb([0, 0, 255]))
      .save(&source.path)
      .expect("replace source image");
    let replacement_bytes = fs::read(&source.path).unwrap();
    assert_eq!(
      original_bytes.len(),
      replacement_bytes.len(),
      "fixture replacement must keep the same byte length"
    );
    OpenOptions::new()
      .write(true)
      .open(&source.path)
      .unwrap()
      .set_times(std::fs::FileTimes::new().set_modified(original_mtime))
      .expect("restore source mtime");
    assert_eq!(
      fs::metadata(&source.path).unwrap().modified().unwrap(),
      original_mtime,
      "fixture replacement must keep the same mtime"
    );

    // Replacement content must invalidate the cache even when filename, byte count, and
    // modification time match.

    let mut svc2 = ImageService::new(Some(tmp.clone()));
    let r2 = svc2.convert(p).expect("replacement image should render");
    assert_ne!(
      r1, r2,
      "changed image content must not use the old cache entry"
    );

    let _ = fs::remove_dir_all(&tmp);
  }

  #[test]
  fn disk_cache_rebuilds_damaged_and_old_format_entries() {
    let source = TestImage::new();
    let params = ImageConvertParams {
      image_path: source.path.to_string_lossy().into(),
      output_width: Some(10),
      output_height: Some(5),
      ..Default::default()
    };
    let key = compute_cache_key(&fs::read(&source.path).unwrap(), &params);

    for (suffix, bytes) in [
      ("damaged", b"{broken".to_vec()),
      (
        "old_version",
        serde_json::to_vec(&DiskCacheEntry {
          format_version: 2,
          rendered: "stale-output".into(),
        })
        .unwrap(),
      ),
    ] {
      let cache_dir = std::env::temp_dir().join(format!(
        "tg_image_cache_{suffix}_{}_{}",
        std::process::id(),
        NEXT_TEST_IMAGE_ID.fetch_add(1, Ordering::Relaxed)
      ));
      fs::create_dir_all(&cache_dir).unwrap();
      let cache_path = ImageService::new(Some(cache_dir.clone()))
        .disk_cache_path(&key)
        .unwrap();
      fs::write(&cache_path, bytes).unwrap();

      let mut service = ImageService::new(Some(cache_dir.clone()));
      let rendered = service
        .convert(params.clone())
        .expect("a bad cache entry should trigger conversion");
      assert!(rendered.starts_with("f%"));
      let repaired: DiskCacheEntry = serde_json::from_slice(&fs::read(&cache_path).unwrap())
        .expect("repaired cache entry should be valid JSON");
      assert_eq!(
        repaired.format_version,
        tg_core_version::IMAGE_CACHE_FORMAT_VERSION
      );
      assert_eq!(repaired.rendered, rendered);
      let _ = fs::remove_dir_all(cache_dir);
    }
  }

  #[test]
  fn cache_false_skips_memory_and_disk_cache() {
    let source = TestImage::new();
    let cache_dir = std::env::temp_dir().join(format!(
      "tg_image_cache_disabled_{}_{}",
      std::process::id(),
      NEXT_TEST_IMAGE_ID.fetch_add(1, Ordering::Relaxed)
    ));
    let params = ImageConvertParams {
      image_path: source.path.to_string_lossy().into(),
      cache: false,
      ..Default::default()
    };
    let mut service = ImageService::new(Some(cache_dir.clone()));

    service
      .convert(params.clone())
      .expect("conversion should succeed");
    service
      .convert(params)
      .expect("uncached repeat should succeed");

    assert!(
      service.cache.is_empty(),
      "cache=false must skip memory cache"
    );
    assert!(
      !cache_dir.exists(),
      "cache=false must not read or create the disk cache directory"
    );
  }

  #[test]
  fn disk_cache_write_failure_does_not_fail_conversion() {
    let source = TestImage::new();
    let blocking_file = std::env::temp_dir().join(format!(
      "tg_image_cache_blocker_{}_{}",
      std::process::id(),
      NEXT_TEST_IMAGE_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::write(&blocking_file, b"file blocks cache directory").unwrap();
    let params = ImageConvertParams {
      image_path: source.path.to_string_lossy().into(),
      ..Default::default()
    };
    let mut service = ImageService::new(Some(blocking_file.clone()));

    let rendered = service
      .convert(params)
      .expect("cache write failure must not fail the conversion");

    assert!(rendered.starts_with("f%"));
    let _ = fs::remove_file(blocking_file);
  }

  #[test]
  fn concurrent_disk_cache_writes_leave_a_valid_entry() {
    let source = TestImage::new();
    let cache_dir = std::env::temp_dir().join(format!(
      "tg_image_cache_concurrent_{}_{}",
      std::process::id(),
      NEXT_TEST_IMAGE_ID.fetch_add(1, Ordering::Relaxed)
    ));
    let image_path = source.path.clone();
    let params = ImageConvertParams {
      image_path: image_path.to_string_lossy().into(),
      output_width: Some(10),
      output_height: Some(5),
      ..Default::default()
    };
    let workers = (0..4)
      .map(|_| {
        let cache_dir = cache_dir.clone();
        let params = params.clone();
        std::thread::spawn(move || {
          ImageService::new(Some(cache_dir))
            .convert(params)
            .expect("concurrent image conversion should succeed")
        })
      })
      .collect::<Vec<_>>();
    let outputs = workers
      .into_iter()
      .map(|worker| worker.join().expect("conversion worker should not panic"))
      .collect::<Vec<_>>();

    assert!(outputs.iter().all(|output| output == &outputs[0]));
    let source_bytes = fs::read(&image_path).unwrap();
    let key = compute_cache_key(&source_bytes, &params);
    let service = ImageService::new(Some(cache_dir.clone()));
    let cached = service
      .read_disk_cache(&key)
      .expect("concurrent writes should leave a complete cache entry");
    assert_eq!(cached, outputs[0]);
    let _ = fs::remove_dir_all(cache_dir);
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
      output_width: Some(20),
      output_height: Some(10),
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
