use std::collections::HashMap;
use std::fs;
use std::hash::{DefaultHasher, Hasher};
use std::io;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use image::GenericImageView;
use serde::{Deserialize, Serialize};

/// 图片转换参数
#[derive(Clone, Debug)]
pub struct ImageConvertParams {
  pub image_path: String,
  pub output_width: u32,
  pub output_height: u32,
  pub crop_x: i32,
  pub crop_y: i32,
  pub crop_width: Option<u32>,
  pub crop_height: Option<u32>,
  pub scale: f64,
  pub cache: bool,
}

impl Default for ImageConvertParams {
  fn default() -> Self {
    Self {
      image_path: String::new(),
      output_width: 80,
      output_height: 24,
      crop_x: 0,
      crop_y: 0,
      crop_width: None,
      crop_height: None,
      scale: 1.0,
      cache: true,
    }
  }
}

/// 图片服务，将图片转换为终端半块字符画格式（支持内存 + 磁盘缓存）
pub struct ImageService {
  cache: HashMap<u64, String>,
  cache_dir: Option<PathBuf>,
}

/// 磁盘缓存条目格式。
#[derive(Serialize, Deserialize)]
struct DiskCacheEntry {
  source_modified: u64,
  rendered: String,
}

impl ImageService {
  /// `cache_dir` 为 `None` 时不启用磁盘缓存。
  pub fn new(cache_dir: Option<PathBuf>) -> Self {
    Self {
      cache: HashMap::new(),
      cache_dir,
    }
  }

  /// 将图片转换为终端字符画（含缓存支持）
  pub fn convert(&mut self, params: ImageConvertParams) -> Result<String, String> {
    validate(&params)?;

    let resolved = resolve_path(&params.image_path)?;
    let hash = compute_hash(&resolved, &params);

    if params.cache {
      // 1. 内存缓存。
      if let Some(cached) = self.cache.get(&hash) {
        return Ok(cached.clone());
      }
      // 2. 磁盘缓存。
      if let Some(disk) = self.read_disk_cache(hash, &resolved) {
        self.cache.insert(hash, disk.clone());
        return Ok(disk);
      }
    }

    let img =
      image::open(&resolved).map_err(|e| format!("无法打开图片 {}: {}", resolved.display(), e))?;

    let result = process(&img, &params)?;

    if params.cache {
      self.cache.insert(hash, result.clone());
      self.write_disk_cache(hash, &resolved, &result);
    }
    Ok(result)
  }

  // ─── 磁盘缓存辅助方法 ──────────────────────────────

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
      // 过期则删除。
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
      let _ = fs::create_dir_all(parent);
    }
    let entry = DiskCacheEntry {
      source_modified: mtime,
      rendered: rendered.to_string(),
    };
    if let Ok(json) = serde_json::to_string(&entry) {
      let _ = fs::write(&path, json);
    }
  }
}

/// 获取源文件的修改时间（Unix 秒）。
fn source_modified(path: &Path) -> io::Result<u64> {
  let meta = fs::metadata(path)?;
  let dur = meta.modified()?.duration_since(UNIX_EPOCH).map_err(|e| {
    io::Error::new(io::ErrorKind::Other, format!("mtime before epoch: {e:?}"))
  })?;
  Ok(dur.as_secs())
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Rgb(u8, u8, u8);

fn validate(p: &ImageConvertParams) -> Result<(), String> {
  if p.image_path.is_empty() {
    return Err("image_path 不能为空".into());
  }
  if p.output_width == 0 {
    return Err("output_width 必须 > 0".into());
  }
  if p.output_height == 0 {
    return Err("output_height 必须 > 0".into());
  }
  if p.scale <= 0.0 {
    return Err("scale 必须 > 0".into());
  }
  if let Some(0) = p.crop_width {
    return Err("crop_width 必须 > 0".into());
  }
  if let Some(0) = p.crop_height {
    return Err("crop_height 必须 > 0".into());
  }
  Ok(())
}

const VALID_EXTS: &[&str] = &["png", "jpg", "jpeg"];

// 解析图片路径，支持无后缀时自动查找 png/jpg/jpeg 文件
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

// 根据图片路径和转换参数计算缓存哈希值
fn compute_hash(resolved: &Path, p: &ImageConvertParams) -> u64 {
  let mut h = DefaultHasher::new();

  let input = format!(
    "{}\x00{}\x00{}\x00{}\x00{}\x00{:?}\x00{:?}\x00{:.6}\x00{}",
    resolved.display(),
    p.output_width,
    p.output_height,
    p.crop_x,
    p.crop_y,
    p.crop_width,
    p.crop_height,
    p.scale,
    p.cache,
  );
  h.write(input.as_bytes());
  h.finish()
}

// 对图片执行裁剪、缩放并采样为半块字符画
fn process(img: &image::DynamicImage, p: &ImageConvertParams) -> Result<String, String> {
  let (src_w, src_h) = img.dimensions();

  let cx = p.crop_x.max(0) as u32;
  let cy = p.crop_y.max(0) as u32;
  let cw = p
    .crop_width
    .unwrap_or(src_w.saturating_sub(cx))
    .min(src_w.saturating_sub(cx));
  let ch = p
    .crop_height
    .unwrap_or(src_h.saturating_sub(cy))
    .min(src_h.saturating_sub(cy));
  if cw == 0 || ch == 0 {
    return Err("裁剪区域为空".into());
  }

  let rgba = img.to_rgba8();
  let cropped = image::imageops::crop_imm(&rgba, cx, cy, cw, ch).to_image();

  let scaled = if (p.scale - 1.0).abs() > f64::EPSILON {
    let sw = ((cw as f64) * p.scale).round().max(1.0) as u32;
    let sh = ((ch as f64) * p.scale).round().max(1.0) as u32;
    image::imageops::resize(&cropped, sw, sh, image::imageops::FilterType::Lanczos3)
  } else {
    cropped
  };

  let pw = p.output_width;
  let ph = p.output_height * 2;
  let resized = image::imageops::resize(&scaled, pw, ph, image::imageops::FilterType::Lanczos3);

  Ok(sample_halfblock(&resized, pw, ph))
}

// 将 RGBA 图像采样为终端半块字符 + 前景/背景色标签字符串
fn sample_halfblock(rgba: &image::RgbaImage, w: u32, h: u32) -> String {
  let char_rows = h / 2;
  let cap = (w as usize * char_rows as usize) * 18 + 2;
  let mut out = String::with_capacity(cap);
  out.push_str("f%");

  let mut prev_fg: Option<Rgb> = None;
  let mut prev_bg: Option<Rgb> = None;

  for cy in 0..char_rows {
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
    let abs = std::path::absolute("assets/images/test/test.jpg").expect("test image should exist");
    let p = ImageConvertParams {
      image_path: abs.to_string_lossy().into(),
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
    let abs = std::path::absolute("assets/images/test/test.jpg").expect("test image should exist");
    let p = ImageConvertParams {
      image_path: abs.to_string_lossy().into(),
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
    let abs = std::path::absolute("assets/images/test/test.jpg").expect("test image should exist");
    let p = ImageConvertParams {
      image_path: abs.to_string_lossy().into(),
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
    let abs = std::path::absolute("assets/images/test/test.jpg").expect("test image should exist");
    let (w, h) = (15u32, 8u32);
    let p = ImageConvertParams {
      image_path: abs.to_string_lossy().into(),
      output_width: w,
      output_height: h,
      scale: 0.3,
      ..Default::default()
    };
    let result = svc.convert(p).expect("conversion should succeed");

    let block_count = result.chars().filter(|&c| c == '\u{2585}').count();
    assert_eq!(block_count, (w * h) as usize);
  }

  #[test]
  fn disk_cache_writes_and_reads() {
    let tmp = std::env::temp_dir().join(format!("tg_image_test_{}", std::process::id()));
    let _ = fs::create_dir_all(&tmp);

    let abs = std::path::absolute("assets/images/test/test.jpg").expect("test image should exist");
    let p = ImageConvertParams {
      image_path: abs.to_string_lossy().into(),
      output_width: 10,
      output_height: 5,
      scale: 0.3,
      cache: true,
      ..Default::default()
    };

    // 第一次调用：渲染并写入磁盘缓存。
    let r1 = {
      let mut svc = ImageService::new(Some(tmp.clone()));
      svc.convert(p.clone()).expect("first convert")
    };

    // 第二次调用：新实例应从磁盘缓存读取。
    let r2 = {
      let mut svc = ImageService::new(Some(tmp.clone()));
      svc.convert(p.clone()).expect("second convert (from disk)")
    };

    assert_eq!(r1, r2, "disk-cached result must match");

    // 清理。
    let _ = fs::remove_dir_all(&tmp);
  }

  #[test]
  fn disk_cache_stale_when_source_changed() {
    let tmp = std::env::temp_dir().join(format!("tg_image_test2_{}", std::process::id()));
    let _ = fs::create_dir_all(&tmp);

    let abs = std::path::absolute("assets/images/test/test.jpg").expect("test image should exist");
    let p = ImageConvertParams {
      image_path: abs.to_string_lossy().into(),
      output_width: 10,
      output_height: 5,
      scale: 0.3,
      cache: true,
      ..Default::default()
    };

    let mut svc = ImageService::new(Some(tmp.clone()));
    let _r1 = svc.convert(p.clone()).expect("first convert");

    // 篡改磁盘缓存文件的 mtime 以制造过期。
    let hash = {
      let resolved = resolve_path(&p.image_path).unwrap();
      compute_hash(&resolved, &p)
    };
    let cache_file = tmp.join(format!("{hash}.json"));
    assert!(cache_file.exists(), "cache file should exist");

    // 将 source_modified 改为 0，使其与源文件 mtime 不匹配。
    let raw = fs::read_to_string(&cache_file).unwrap();
    let stale = raw.replace(
      &format!("\"source_modified\":{}", {
        let meta = fs::metadata(&abs).unwrap();
        meta.modified().unwrap().duration_since(UNIX_EPOCH).unwrap().as_secs()
      }),
      "\"source_modified\":0",
    );
    fs::write(&cache_file, stale).unwrap();

    // 新实例读取时发现过期，应重新渲染。
    let mut svc2 = ImageService::new(Some(tmp.clone()));
    let r2 = svc2.convert(p).expect("stale cache should re-render");
    assert!(r2.starts_with("f%"));

    let _ = fs::remove_dir_all(&tmp);
  }
}
