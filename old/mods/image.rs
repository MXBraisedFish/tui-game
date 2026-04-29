// Mod 图像处理模块，支持将 PNG/JPG/WebP 等光栅图像和 ASCII 艺术转换为固定规格的 ASCII/富文本图像（缩略图 4×8，Banner 13×86）。提供从包元数据加载图像的统一入口 image_from_meta

use std::fs; // 文件读写、创建缓存目录
use std::path::{Path}; // 路径处理
use std::hash::{Hash, Hasher}; // 构建图像缓存键

use anyhow::{Context, Result, anyhow}; // 错误处理
use image::GenericImageView; // 光栅图像解码与操作
use ratatui::style::{Color, Style}; // 富文本样式（预热渲染行）
use ratatui::text::Line; // 预渲染行类型
use serde_json::Value as JsonValue; // 解析 JSON 中的图像配置
use unicode_width::UnicodeWidthChar; // 计算字符宽度

use crate::app::rich_text; // 富文本解析（预热渲染）
use crate::mods::types::*; // 导入 ModImage、ImageKind、ImageColorMode、ImageSpec
use crate::mods::{mod_cache_dir, resolve_asset_path, mtime_secs}; // 路径工具和文件时间戳

const ASCII_IMAGE_CHARS: &str = r#"M@N%W$E#RK&FXYI*l]}1/+i>"!~';,`:."#; // 从深到浅排列的 28 个 ASCII 字符
const IMAGE_RENDER_ALGORITHM_VERSION: u8 = 2; // 缓存键的一部分，算法变更时缓存自动失效

const DEFAULT_THUMBNAIL_LINES: [&str; 4] = [
    "\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}",
    "\u{2588}\u{2588} \u{2588}\u{2588} \u{2588}\u{2588}",
    "   \u{2588}\u{2588}   ",
    "  \u{2588}\u{2588}\u{2588}\u{2588}  ",
]; // 当 Mod 未提供缩略图时使用

const DEFAULT_BANNER_ASCII: [&str; 7] = [
    "`7MMM.     ,MMF' .g8\"\"8q. `7MM\"\"\"Yb.   ",
    "  MMMb    dPMM .dP'    `YM. MM    `Yb. ",
    "  M YM   ,M MM dM'      `MM MM     `Mb ",
    "  M  Mb  M' MM MM        MM MM      MM ",
    "  M  YM.P'  MM MM.      ,MP MM     ,MP ",
    "  M  `YM'   MM `Mb.    ,dP' MM    ,dP' ",
    ".JML. `'  .JMML. `\"bmmd\"' .JMMmmmdP'   ",
]; // 当 Mod 未提供 Banner 时使用

// 统一入口，根据 JSON 值类型加载图像（字符串路径/数组 ASCII/默认）
pub fn image_from_meta(namespace: &str, raw: Option<&JsonValue>, kind: ImageKind) -> Result<ModImage> {
    let image = match raw {
        Some(JsonValue::String(value)) => load_image_from_string(namespace, value, kind)?,
        Some(JsonValue::Array(value)) => {
            parse_ascii_image_array(value, kind).unwrap_or_else(|| default_image(kind))
        }
        _ => default_image(kind),
    };
    let mut image = normalize_image(image, kind);
    warm_rendered_image_lines(&mut image);
    Ok(image)
}

// 从字符串描述加载：先尝试解析为光栅路径，失败则作为文本文件读取
fn load_image_from_string(namespace: &str, value: &str, kind: ImageKind) -> Result<ModImage> {
    if let Ok(spec) = parse_image_spec(namespace, value) {
        let asset_path = resolve_asset_path(&spec.namespace, &spec.path)?;
        if !asset_path.exists() {
            return Ok(default_image(kind));
        }

        return match asset_path
            .extension()
            .and_then(|value| value.to_str())
            .map(|value| value.to_ascii_lowercase())
            .as_deref()
        {
            Some("png") | Some("jpg") | Some("jpeg") | Some("webp") => {
                render_cached_raster_image(&asset_path, &spec, kind)
            }
            _ => Ok(default_image(kind)),
        };
    }

    let asset_path = match resolve_asset_path(namespace, value) {
        Ok(path) => path,
        Err(_) => return Ok(default_image(kind)),
    };
    if !asset_path.exists() {
        return Ok(default_image(kind));
    }

    let content = fs::read_to_string(asset_path)?;
    let lines = content
        .trim_start_matches('\u{feff}')
        .lines()
        .map(|line| line.to_string())
        .collect::<Vec<_>>();
    if lines.is_empty() {
        Ok(default_image(kind))
    } else {
        Ok(ModImage {
            lines,
            rendered_lines: Vec::new(),
        })
    }
}

// 对图像每行调用富文本解析器，生成预渲染的 ratatui::text::Line 缓存
fn warm_rendered_image_lines(image: &mut ModImage) {
    image.rendered_lines = image
        .lines
        .iter()
        .map(|line| {
            rich_text::parse_rich_text_wrapped(line, usize::MAX / 8, Style::default().fg(Color::White))
                .into_iter()
                .next()
                .unwrap_or_else(|| Line::from(""))
        })
        .collect();
}

// 解析如 color:image:path.png 的图像规格字符串
fn parse_image_spec(namespace: &str, value: &str) -> Result<ImageSpec> {
    let mut color_mode = ImageColorMode::Grayscale;
    let mut parts = value.split(':').collect::<Vec<_>>();
    let mut saw_image_prefix = false;

    while let Some(head) = parts.first().copied() {
        match head {
            "color" => {
                color_mode = ImageColorMode::Color;
                parts.remove(0);
            }
            "image" => {
                parts.remove(0);
                saw_image_prefix = true;
                break;
            }
            _ => break,
        }
    }

    if !saw_image_prefix {
        return Err(anyhow!("invalid image spec"));
    }
    let path = parts.join(":");
    if path.trim().is_empty() {
        return Err(anyhow!("empty image path"));
    }

    Ok(ImageSpec {
        namespace: namespace.to_string(),
        path,
        color_mode,
    })
}

// 光栅图像渲染 + 缓存：先查缓存 JSON，无则调用 render_raster_image 并写入缓存
fn render_cached_raster_image(path: &Path, spec: &ImageSpec, kind: ImageKind) -> Result<ModImage> {
    fs::create_dir_all(mod_cache_dir()?)?;
    let cache_key = build_image_cache_key(path, spec, kind);
    let cache_path = mod_cache_dir()?.join(format!("{cache_key}.json"));

    if let Ok(raw) = fs::read_to_string(&cache_path) {
        if let Ok(image) = serde_json::from_str::<ModImage>(raw.trim_start_matches('\u{feff}')) {
            return Ok(image);
        }
    }

    let rendered = render_raster_image(path, spec, kind)?;
    fs::write(&cache_path, serde_json::to_string(&rendered)?)?;
    Ok(rendered)
}

// 基于文件路径、修改时间、色彩模式、图像类型和算法版本构建唯一缓存键
fn build_image_cache_key(path: &Path, spec: &ImageSpec, kind: ImageKind) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    IMAGE_RENDER_ALGORITHM_VERSION.hash(&mut hasher);
    path.to_string_lossy().hash(&mut hasher);
    mtime_secs(path).hash(&mut hasher);
    spec.namespace.hash(&mut hasher);
    spec.path.hash(&mut hasher);
    match spec.color_mode {
        ImageColorMode::Grayscale => 11_u8.hash(&mut hasher),
        ImageColorMode::Color => 13_u8.hash(&mut hasher),
    }
    match kind {
        ImageKind::Thumbnail => 3_u8.hash(&mut hasher),
        ImageKind::Banner => 4_u8.hash(&mut hasher),
    }
    format!("{:016x}", hasher.finish())
}

// 打开光栅图像文件，委托 render_ascii_image 转换为 ASCII
fn render_raster_image(path: &Path, spec: &ImageSpec, kind: ImageKind) -> Result<ModImage> {
    let dynamic = image::open(path)
        .with_context(|| format!("failed to open raster image: {}", path.display()))?;
    Ok(render_ascii_image(&dynamic, spec.color_mode, kind))
}

// 核心转换函数：缩放图像 → 逐像素映射为 ASCII 字符 → 生成带 f% 富文本标记的行
fn render_ascii_image(
    image: &image::DynamicImage,
    color_mode: ImageColorMode,
    kind: ImageKind,
) -> ModImage {
    let (target_h, target_w) = image_target_size(kind);
    let resized = resize_ascii_image(image, target_w as u32, target_h as u32);

    let mut lines = Vec::with_capacity(target_h);
    for y in 0..target_h {
        let mut line = if matches!(
            color_mode,
            ImageColorMode::Grayscale | ImageColorMode::Color
        ) {
            String::from("f%")
        } else {
            String::new()
        };
        let mut current_color: Option<String> = None;

        for x in 0..target_w {
            let pixel = resized.get_pixel(x as u32, y as u32).0;
            let alpha = pixel[3];
            let ch = if alpha == 0 {
                ' '
            } else {
                let gray = image_luma([pixel[0], pixel[1], pixel[2]]);
                let index = (((255.0 - gray as f32) / 255.0)
                    * (ASCII_IMAGE_CHARS.chars().count().saturating_sub(1)) as f32)
                    .round() as usize;
                ASCII_IMAGE_CHARS
                    .chars()
                    .nth(index.min(ASCII_IMAGE_CHARS.chars().count().saturating_sub(1)))
                    .unwrap_or('.')
            };

            if ch != ' ' {
                if let Some(color) = image_output_color(color_mode, [pixel[0], pixel[1], pixel[2]]) {
                    if current_color.as_deref() != Some(color.as_str()) {
                        line.push_str(&format!("{{tc:{color}}}"));
                        current_color = Some(color);
                    }
                }
            }

            push_rich_text_safe_char(&mut line, ch);
        }

        if matches!(
            color_mode,
            ImageColorMode::Grayscale | ImageColorMode::Color
        ) && current_color.is_some()
        {
            line.push_str("{tc:clear}");
        }
        lines.push(line);
    }

    ModImage {
        lines,
        rendered_lines: Vec::new(),
    }
}

// 根据色彩模式生成输出颜色字符串：彩色返回 #RRGGBB，灰度返回等灰度的 #XXXXXX
fn image_output_color(color_mode: ImageColorMode, rgb: [u8; 3]) -> Option<String> {
    match color_mode {
        ImageColorMode::Color => Some(format!("#{:02x}{:02x}{:02x}", rgb[0], rgb[1], rgb[2])),
        ImageColorMode::Grayscale => {
            let gray = image_luma(rgb);
            Some(format!("#{:02x}{:02x}{:02x}", gray, gray, gray))
        }
    }
}

// 将字符安全加入富文本行，转义 {、}、\
fn push_rich_text_safe_char(out: &mut String, ch: char) {
    if matches!(ch, '{' | '}' | '\\') {
        out.push('\\');
    }
    out.push(ch);
}

// 计算 RGB 像素的亮度值（加权灰度公式）
fn image_luma(rgb: [u8; 3]) -> u8 {
    ((0.2126 * rgb[0] as f32) + (0.7152 * rgb[1] as f32) + (0.0722 * rgb[2] as f32))
        .round()
        .clamp(0.0, 255.0) as u8
}

// 按终端字符宽高比（1:2）裁剪并缩放图像到目标尺寸
fn resize_ascii_image(
    image: &image::DynamicImage,
    target_w: u32,
    target_h: u32,
) -> image::RgbaImage {
    use image::imageops::FilterType;

    let (src_w, src_h) = image.dimensions();
    let src_ratio = src_w as f32 / src_h as f32;
    let cell_width = 1.0_f32;
    let cell_height = 2.0_f32;
    let dst_ratio = (target_w as f32 * cell_width) / (target_h as f32 * cell_height);

    let cropped = if src_ratio > dst_ratio {
        let new_w = (src_h as f32 * dst_ratio).round().max(1.0) as u32;
        let start_x = (src_w.saturating_sub(new_w)) / 2;
        image.crop_imm(start_x, 0, new_w, src_h)
    } else {
        let new_h = (src_w as f32 / dst_ratio).round().max(1.0) as u32;
        let start_y = (src_h.saturating_sub(new_h)) / 2;
        image.crop_imm(0, start_y, src_w, new_h)
    };

    cropped
        .resize_exact(target_w, target_h, FilterType::Lanczos3)
        .to_rgba8()
}

// 解析 JSON 数组形式的 ASCII 图像数据
fn parse_ascii_image_array(raw: &[JsonValue], _kind: ImageKind) -> Option<ModImage> {
    let mut lines = Vec::new();
    for row in raw {
        let mut line = String::new();
        flatten_ascii_row(row, &mut line)?;
        lines.push(line);
    }
    if lines.is_empty() {
        None
    } else {
        Some(ModImage {
            lines,
            rendered_lines: Vec::new(),
        })
    }
}

// 递归将 JSON 值（字符串/数字/数组）展平为字符串
fn flatten_ascii_row(value: &JsonValue, out: &mut String) -> Option<()> {
    match value {
        JsonValue::String(value) => {
            out.push_str(value);
            Some(())
        }
        JsonValue::Number(value) => {
            out.push_str(&value.to_string());
            Some(())
        }
        JsonValue::Array(items) => {
            for item in items {
                flatten_ascii_row(item, out)?;
            }
            Some(())
        }
        _ => None,
    }
}

// 返回对应图像类型的默认图像（缩略图或 Banner）
fn default_image(kind: ImageKind) -> ModImage {
    let lines = match kind {
        ImageKind::Thumbnail => DEFAULT_THUMBNAIL_LINES
            .iter()
            .map(|line| (*line).to_string())
            .collect(),
        ImageKind::Banner => DEFAULT_BANNER_ASCII
            .iter()
            .map(|line| (*line).to_string())
            .collect(),
    };
    ModImage {
        lines,
        rendered_lines: Vec::new(),
    }
}

// 将任意尺寸的图像裁剪/填充到目标尺寸
fn normalize_image(image: ModImage, kind: ImageKind) -> ModImage {
    let (target_h, target_w) = image_target_size(kind);

    let mut lines = image.lines;
    if lines.is_empty() {
        lines = default_image(kind).lines;
    }

    lines = center_crop_or_pad_vertical(lines, target_h);
    lines = lines
        .into_iter()
        .map(|line| center_crop_or_pad_horizontal(&line, target_w))
        .collect();

    ModImage {
        lines,
        rendered_lines: Vec::new(),
    }
}

// 垂直方向居中裁剪或交替上下填充
fn center_crop_or_pad_vertical(mut lines: Vec<String>, target_h: usize) -> Vec<String> {
    if lines.len() > target_h {
        let start = (lines.len() - target_h) / 2;
        lines = lines.into_iter().skip(start).take(target_h).collect();
    }
    while lines.len() < target_h {
        if lines.len() % 2 == 0 {
            lines.insert(0, String::new());
        } else {
            lines.push(String::new());
        }
    }
    lines
}

// 水平方向居中裁剪或两侧平衡填充
fn center_crop_or_pad_horizontal(line: &str, target_w: usize) -> String {
    let current_w = visible_text_width(line);
    if current_w > target_w && !line.starts_with("f%") && !line.contains('{') {
        let chars: Vec<char> = line.chars().collect();
        let start = (chars.len().saturating_sub(target_w)) / 2;
        return chars
            .into_iter()
            .skip(start)
            .take(target_w)
            .collect::<String>();
    }
    if current_w >= target_w {
        return line.to_string();
    }
    pad_line_balanced(line, target_w - current_w)
}

// 返回图像类型的目标尺寸：Thumbnail (4,8)、Banner (13,86)
fn image_target_size(kind: ImageKind) -> (usize, usize) {
    match kind {
        ImageKind::Thumbnail => (4, 8),
        ImageKind::Banner => (13, 86),
    }
}

// 在字符串两侧交替添加空格以实现平衡填充
fn pad_line_balanced(line: &str, pad: usize) -> String {
    let mut left = 0usize;
    let mut right = 0usize;
    let mut add_left = true;
    for _ in 0..pad {
        if add_left {
            left += 1;
        } else {
            right += 1;
        }
        add_left = !add_left;
    }
    format!("{}{}{}", " ".repeat(left), line, " ".repeat(right))
}

// 计算富文本去除标记后的可见字符宽度
fn visible_text_width(text: &str) -> usize {
    let text = text.strip_prefix("f%").unwrap_or(text);
    let chars = text.chars().collect::<Vec<_>>();
    let mut i = 0usize;
    let mut width = 0usize;

    while i < chars.len() {
        match chars[i] {
            '\\' => {
                if i + 1 < chars.len() {
                    if chars[i + 1] != 'n' {
                        width += chars[i + 1].width().unwrap_or(0);
                    }
                    i += 2;
                } else {
                    width += 1;
                    i += 1;
                }
            }
            '{' => {
                if let Some(end) = chars[i + 1..].iter().position(|ch| *ch == '}') {
                    i += end + 2;
                } else {
                    width += 1;
                    i += 1;
                }
            }
            '\n' => {
                i += 1;
            }
            ch => {
                width += ch.width().unwrap_or(0);
                i += 1;
            }
        }
    }

    width
}