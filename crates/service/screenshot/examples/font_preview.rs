//! Generate and print the screenshot service's fixed Unicode font preview.

use std::{env, fs, path::PathBuf};

use image::RgbaImage;
use serde_json::json;
use tg_service_screenshot::{ScreenshotService, TerminalFrameRasterizer};
use tg_service_storage::RecordingPixelScale;

const REFERENCE_FONT_SIZE_PX: f32 = 24.0;
const REFERENCE_LINE_HEIGHT_PX: f32 = 31.68;
const REFERENCE_CELL_HEIGHT_PX: f32 = 36.0;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let mut args = env::args_os().skip(1);
  let deployment_root = args
    .next()
    .map(PathBuf::from)
    .ok_or("usage: font_preview <deployment-root> <output-directory>")?;
  let output_dir = args
    .next()
    .map(PathBuf::from)
    .ok_or("usage: font_preview <deployment-root> <output-directory>")?;
  fs::create_dir_all(&output_dir)?;

  let frame = ScreenshotService::font_preview_frame();
  let rect = ScreenshotService::whole_frame_rect(&frame).ok_or("preview frame is empty")?;
  let text = ScreenshotService::plain_text(&frame, rect);
  let font_fixture = "assets/fonts/mmo.ttf";
  let font_bytes = fs::read(deployment_root.join(font_fixture))?;
  let fixture_font = fontdue::Font::from_bytes(font_bytes, fontdue::FontSettings::default())?;
  let reference_font_size_px = REFERENCE_FONT_SIZE_PX;
  let line_metrics = fixture_font
    .horizontal_line_metrics(reference_font_size_px)
    .ok_or("font fixture has no horizontal line metrics")?;
  let normalized_font_size =
    reference_font_size_px * REFERENCE_LINE_HEIGHT_PX / line_metrics.new_line_size;
  let normalized_line_metrics = fixture_font
    .horizontal_line_metrics(normalized_font_size)
    .ok_or("font fixture has no normalized horizontal line metrics")?;
  let glyph_metrics = fixture_font.metrics('M', normalized_font_size);
  let baseline = normalized_line_metrics.ascent
    + (REFERENCE_CELL_HEIGHT_PX - normalized_line_metrics.new_line_size) / 2.0;
  let rasterizer = TerminalFrameRasterizer::load(&[font_fixture.to_string()], &deployment_root)?;
  let image = rasterizer.render(&frame, rect, RecordingPixelScale::Original, |_, _| {});
  let png_path = output_dir.join("font_preview.png");
  let text_path = output_dir.join("font_preview.txt");
  let metadata_path = output_dir.join("font_preview.json");
  image.save(&png_path)?;
  fs::write(&text_path, &text)?;
  let (non_background_pixels, content_bounds) = pixel_summary(&image);
  let metadata = json!({
    "font_fixture": font_fixture,
    "font_face": "Maple Mono NF CN",
    "recording_scale": "Original",
    "font_size_px": normalized_font_size,
    "baseline_ratio": baseline / REFERENCE_CELL_HEIGHT_PX,
    "baseline_offset_px": baseline,
    "cell_advance_px": glyph_metrics.advance_width,
    "cell_height_px": REFERENCE_CELL_HEIGHT_PX,
    "reference_font_metrics": {
      "reference_font_size_px": reference_font_size_px,
      "ascent_px": line_metrics.ascent,
      "descent_px": line_metrics.descent,
      "line_gap_px": line_metrics.line_gap,
      "line_height_px": line_metrics.new_line_size,
      "monospace_advance_px": glyph_metrics.advance_width,
      "capital_m_ink_size_px": {
        "width": glyph_metrics.width,
        "height": glyph_metrics.height,
      },
    },
    "frame_cells": {
      "width": rect.width,
      "height": rect.height,
    },
    "cell_pixels": {
      "width_advance": glyph_metrics.advance_width,
      "height": REFERENCE_CELL_HEIGHT_PX,
    },
    "image_pixels": {
      "width": image.width(),
      "height": image.height(),
    },
    "non_background_pixels": non_background_pixels,
    "content_bounds": content_bounds.map(|(left, top, right, bottom)| {
      json!({"left": left, "top": top, "right": right, "bottom": bottom})
    }),
  });
  fs::write(&metadata_path, serde_json::to_vec_pretty(&metadata)?)?;

  println!(
    "Preview frame: {}x{} cells; raster: {}x{} px; cell: {}x{} px",
    rect.width,
    rect.height,
    image.width(),
    image.height(),
    image.width() / u32::from(rect.width),
    image.height() / u32::from(rect.height),
  );
  println!("Font fixture: {font_fixture} (Maple Mono NF CN)");
  println!(
    "Maple Mono at {normalized_font_size:.2}px: line ascent/descent/gap={:.2}/{:.2}/{:.2}px, advance M={:.2}px, baseline={baseline:.2}px",
    line_metrics.ascent, line_metrics.descent, line_metrics.line_gap, glyph_metrics.advance_width,
  );
  println!("Non-background pixels: {non_background_pixels}; bounds: {content_bounds:?}");
  println!("PNG: {}", png_path.display());
  println!("Metadata: {}", metadata_path.display());
  println!("Terminal text sample:\n{text}");
  Ok(())
}

fn pixel_summary(image: &RgbaImage) -> (u64, Option<(u32, u32, u32, u32)>) {
  let mut count = 0;
  let mut bounds: Option<(u32, u32, u32, u32)> = None;
  for (x, y, pixel) in image.enumerate_pixels() {
    if pixel.0[..3] == [0, 0, 0] {
      continue;
    }
    count += 1;
    bounds = Some(match bounds {
      Some((left, top, right, bottom)) => (left.min(x), top.min(y), right.max(x), bottom.max(y)),
      None => (x, y, x, y),
    });
  }
  (count, bounds)
}
