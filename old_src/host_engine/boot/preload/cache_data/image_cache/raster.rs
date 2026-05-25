//! 光栅图片到终端 ASCII 行转换
// TODO: 迁移至 storage::CacheStore

use image::{DynamicImage, GenericImageView, RgbaImage, imageops::FilterType};

use super::types::{ASCII_GRADIENT, GameImageColorMode, GameImageSlot};

/// 将图片转换为固定终端字符尺寸的 ASCII 行。
pub fn render_image_to_ascii(
    image: &DynamicImage,
    slot: GameImageSlot,
    color_mode: GameImageColorMode,
) -> Vec<String> {
    let (columns, rows) = slot.target_size();
    let resized_image = crop_and_resize_image(image, columns, rows);
    let gradient_chars = ASCII_GRADIENT.chars().collect::<Vec<_>>();
    let mut lines = Vec::with_capacity(rows as usize);

    for y in 0..rows {
        let mut line = String::new();
        let mut current_color: Option<String> = None;

        for x in 0..columns {
            let pixel = resized_image.get_pixel(x, y).0;
            let alpha = pixel[3];
            if alpha == 0 {
                line.push(' ');
                continue;
            }

            let gray = pixel_luma(pixel[0], pixel[1], pixel[2]);
            let character = brightness_to_character(gray, &gradient_chars);

            if color_mode == GameImageColorMode::Color {
                let color = format!("#{:02x}{:02x}{:02x}", pixel[0], pixel[1], pixel[2]);
                if current_color.as_deref() != Some(color.as_str()) {
                    line.push_str(&format!("{{tc:{color}}}"));
                    current_color = Some(color);
                }
            }

            if color_mode == GameImageColorMode::Color {
                push_rich_text_safe_character(&mut line, character);
            } else {
                line.push(character);
            }
        }

        if color_mode == GameImageColorMode::Color && current_color.is_some() {
            line.push_str("{tc:clear}");
        }
        lines.push(line);
    }

    lines
}

fn crop_and_resize_image(image: &DynamicImage, columns: u32, rows: u32) -> RgbaImage {
    let (source_width, source_height) = image.dimensions();
    let target_ratio = columns as f32 / (rows as f32 * 2.0);
    let source_ratio = source_width as f32 / source_height as f32;

    let cropped_image = if source_ratio > target_ratio {
        let crop_width = (source_height as f32 * target_ratio).round().max(1.0) as u32;
        let start_x = source_width.saturating_sub(crop_width) / 2;
        image.crop_imm(start_x, 0, crop_width, source_height)
    } else {
        let crop_height = (source_width as f32 / target_ratio).round().max(1.0) as u32;
        let start_y = source_height.saturating_sub(crop_height) / 2;
        image.crop_imm(0, start_y, source_width, crop_height)
    };

    cropped_image
        .resize_exact(columns, rows, FilterType::Lanczos3)
        .to_rgba8()
}

fn pixel_luma(red: u8, green: u8, blue: u8) -> u8 {
    ((0.2126 * red as f32) + (0.7152 * green as f32) + (0.0722 * blue as f32))
        .round()
        .clamp(0.0, 255.0) as u8
}

fn brightness_to_character(gray: u8, gradient_chars: &[char]) -> char {
    let max_index = gradient_chars.len().saturating_sub(1);
    let index = (((255 - gray) as f32 / 255.0) * max_index as f32).round() as usize;
    gradient_chars[index.min(max_index)]
}

fn push_rich_text_safe_character(output: &mut String, character: char) {
    if matches!(character, '{' | '}' | '\\') {
        output.push('\\');
    }
    output.push(character);
}
