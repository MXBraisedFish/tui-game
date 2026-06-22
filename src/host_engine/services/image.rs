//! 图片转半块富文本——将 PNG/JPEG 图片转换为终端可用的 f% 格式富文本字符串。
//! 每个字符格 = 2 个纵向子像素（上半=背景色，下半=前景色用 ▅ 呈现）。

use std::collections::HashMap;
use std::hash::{DefaultHasher, Hasher};
use std::path::{Path, PathBuf};

use image::GenericImageView;

// ── 公开类型 ──

/// 半块转换参数。除 image_path / output_width / output_height 外均有默认值。
#[derive(Clone, Debug)]
pub struct ImageConvertParams {
    pub image_path: String,
    pub output_width: u32,
    pub output_height: u32, // 终端字符行数，内部像素行 = 此值 * 2
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

/// 图片服务——持有半块转换结果的内存缓存。
pub struct ImageService {
    cache: HashMap<u64, String>,
}

impl ImageService {
    pub fn new() -> Self {
        Self { cache: HashMap::new() }
    }

    /// 将图片转为 f% 开头的富文本字符串。
    /// 流程：校验→解析路径→哈希→查缓存→加载→裁剪→缩放→重采样→半块采样→存缓存→返回
    pub fn convert(&mut self, params: ImageConvertParams) -> Result<String, String> {
        validate(&params)?;

        let resolved = resolve_path(&params.image_path)?;
        let hash = compute_hash(&resolved, &params);

        if params.cache {
            if let Some(cached) = self.cache.get(&hash) {
                return Ok(cached.clone());
            }
        }

        let img = image::open(&resolved)
            .map_err(|e| format!("无法打开图片 {}: {}", resolved.display(), e))?;

        let result = process(&img, &params)?;

        if params.cache {
            self.cache.insert(hash, result.clone());
        }
        Ok(result)
    }
}

// ── 内部类型 ──

#[derive(Clone, Copy, PartialEq, Eq)]
struct Rgb(u8, u8, u8);

// ── 步骤 1：参数校验 ──

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

// ── 步骤 2：路径解析（含无后缀自动查找） ──

const VALID_EXTS: &[&str] = &["png", "jpg", "jpeg"];

fn resolve_path(raw: &str) -> Result<PathBuf, String> {
    let path = Path::new(raw);

    // 有后缀 → 校验格式 + 存在性
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

    // 无后缀 → 在所在目录查找 stem.{png,jpg,jpeg}
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

// ── 步骤 3：参数哈希 ──

fn compute_hash(resolved: &Path, p: &ImageConvertParams) -> u64 {
    let mut h = DefaultHasher::new();
    // 用 \x00 分隔各字段，防止不同参数组合碰撞
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

// ── 步骤 5-7：图片处理（裁剪→缩放→重采样→半块采样+标签合并） ──

fn process(img: &image::DynamicImage, p: &ImageConvertParams) -> Result<String, String> {
    let (src_w, src_h) = img.dimensions();

    // 裁剪参数计算
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

    // 转 rgba8 → 裁剪 → 缩放/resize 均使用 imageops 自由函数保证类型一致
    let rgba = img.to_rgba8();
    let cropped = image::imageops::crop_imm(&rgba, cx, cy, cw, ch).to_image();

    let scaled = if (p.scale - 1.0).abs() > f64::EPSILON {
        let sw = ((cw as f64) * p.scale).round().max(1.0) as u32;
        let sh = ((ch as f64) * p.scale).round().max(1.0) as u32;
        image::imageops::resize(&cropped, sw, sh, image::imageops::FilterType::Lanczos3)
    } else {
        cropped
    };

    // 重采样到目标像素网格：宽 = output_width，高 = output_height * 2
    let pw = p.output_width;
    let ph = p.output_height * 2;
    let resized = image::imageops::resize(&scaled, pw, ph, image::imageops::FilterType::Lanczos3);

    Ok(sample_halfblock(&resized, pw, ph))
}

/// 将像素网格转为 f% 富文本——每字符格取上下两像素，仅颜色变化时输出标签。
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
            out.push('\u{2585}'); // ▅
        }
    }
    out
}

fn get_rgb(rgba: &image::RgbaImage, x: u32, y: u32) -> Rgb {
    let p = rgba.get_pixel(x, y).0;
    Rgb(p[0], p[1], p[2])
}

// ── 测试 ──

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_rejects_empty_path() {
        let p = ImageConvertParams { image_path: String::new(), ..Default::default() };
        assert!(validate(&p).is_err());
    }

    #[test]
    fn validate_rejects_zero_width() {
        let p = ImageConvertParams { image_path: "x".into(), output_width: 0, ..Default::default() };
        assert!(validate(&p).is_err());
    }

    #[test]
    fn validate_rejects_zero_height() {
        let p = ImageConvertParams { image_path: "x".into(), output_height: 0, ..Default::default() };
        assert!(validate(&p).is_err());
    }

    #[test]
    fn validate_rejects_zero_crop_width() {
        let p = ImageConvertParams {
            image_path: "x".into(), crop_width: Some(0), ..Default::default()
        };
        assert!(validate(&p).is_err());
    }

    #[test]
    fn validate_rejects_negative_scale() {
        let p = ImageConvertParams { image_path: "x".into(), scale: -1.0, ..Default::default() };
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
            image_path: "/a/b.png".into(), output_width: 10, ..Default::default()
        };
        let p2 = ImageConvertParams {
            image_path: "/a/b.png".into(), output_width: 20, ..Default::default()
        };
        let path = PathBuf::from("/a/b.png");
        assert_ne!(compute_hash(&path, &p1), compute_hash(&path, &p2));
    }

    #[test]
    fn convert_returns_f_percent_prefix() {
        let mut svc = ImageService::new();
        let abs = std::path::absolute("assets/images/test/test.jpg")
            .expect("test image should exist");
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
        let mut svc = ImageService::new();
        let abs = std::path::absolute("assets/images/test/test.jpg")
            .expect("test image should exist");
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
        let mut svc = ImageService::new();
        let abs = std::path::absolute("assets/images/test/test.jpg")
            .expect("test image should exist");
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
        let mut svc = ImageService::new();
        let abs = std::path::absolute("assets/images/test/test.jpg")
            .expect("test image should exist");
        let (w, h) = (15u32, 8u32);
        let p = ImageConvertParams {
            image_path: abs.to_string_lossy().into(),
            output_width: w,
            output_height: h,
            scale: 0.3,
            ..Default::default()
        };
        let result = svc.convert(p).expect("conversion should succeed");
        // 每个字符格产生一个 ▅，w * h 个字符
        let block_count = result.chars().filter(|&c| c == '\u{2585}').count();
        assert_eq!(block_count, (w * h) as usize);
    }
}
