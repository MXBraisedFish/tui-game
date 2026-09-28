use image::{Rgb, RgbImage};
use serde_json::{Value, json};
use std::{fs, path::Path};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const COLS: u32 = 80;
const ROWS: u32 = 20;

fn calibrate(window: &Path, profile: &Path, out: &Path) -> Result<()> {
  let image = image::open(window)?.to_rgb8();
  let (x, y, cw, ch) = locate(&image)?;
  if x + COLS * cw > image.width() || y + ROWS * ch > image.height() {
    return Err("fixture clipped: enlarge terminal and recapture".into());
  }
  // Confirm the vertical ruler before accepting a crop; reject accidental horizontal color matches.
  for row in 1..ROWS {
    let expected = if row % 2 == 0 {
      [0, 0, 255]
    } else {
      [255, 255, 0]
    };
    if image.get_pixel(x + cw / 2, y + row * ch + ch / 2).0 != expected {
      return Err(format!("vertical marker missing at row {row}; frame clipped or resized").into());
    }
  }
  fs::create_dir_all(out)?;
  image::imageops::crop_imm(&image, x, y, COLS * cw, ROWS * ch)
    .to_image()
    .save(out.join("reference.png"))?;
  let mut p: Value = serde_json::from_slice(&fs::read(profile)?)?;
  p["cell_width"] = json!(cw);
  p["cell_height"] = json!(ch);
  fs::write(
    out.join("measured-profile.json"),
    serde_json::to_vec_pretty(&p)?,
  )?;
  let report = json!({"crop":{"x":x,"y":y,"width":COLS*cw,"height":ROWS*ch},"cell_width":cw,"cell_height":ch,
        "method":"80 exact alternating magenta/cyan background cells + vertical blue/yellow ruler",
        "baseline":"not measured; retained from input profile, calibrate independently",
        "input":window.display().to_string()});
  fs::write(
    out.join("calibration.json"),
    serde_json::to_vec_pretty(&report)?,
  )?;
  println!("{report}");
  Ok(())
}

fn locate(image: &RgbImage) -> Result<(u32, u32, u32, u32)> {
  for y in 0..image.height() {
    let mut x = 0;
    while x < image.width() {
      if image.get_pixel(x, y).0 != [255, 0, 255] {
        x += 1;
        continue;
      }
      let start = x;
      let mut widths = Vec::new();
      for col in 0..COLS {
        let expected = if col % 2 == 0 {
          [255, 0, 255]
        } else {
          [0, 255, 255]
        };
        let edge = x;
        while x < image.width() && image.get_pixel(x, y).0 == expected {
          x += 1;
        }
        if x == edge {
          break;
        }
        widths.push(x - edge);
      }
      if widths.len() == COLS as usize && widths.iter().all(|w| *w == widths[0]) {
        let cw = widths[0];
        let mut bottom = y;
        while bottom < image.height() && image.get_pixel(start + cw / 2, bottom).0 == [255, 0, 255]
        {
          bottom += 1;
        }
        if bottom > y {
          return Ok((start, y, cw, bottom - y));
        }
      }
      x = x.max(start + 1);
    }
  }
  Err(
    "no complete 80-cell calibration ruler found; do not infer cell width from glyph ink bounds"
      .into(),
  )
}

fn compare(reference: &Path, candidate: &Path, out: &Path) -> Result<()> {
  let a = image::open(reference)?.to_rgb8();
  let b = image::open(candidate)?.to_rgb8();
  if a.dimensions() != b.dimensions() {
    return Err(
      format!(
        "geometry mismatch {:?} vs {:?}; no automatic resize or alignment",
        a.dimensions(),
        b.dimensions()
      )
      .into(),
    );
  }
  if a.height() % ROWS != 0 {
    return Err("reference height is not a multiple of fixture rows".into());
  }
  let mut heat = RgbImage::new(a.width(), a.height());
  let mut rows = Vec::new();
  let ch = a.height() / ROWS;
  for row in 0..ROWS {
    let mut sum = 0u64;
    let mut changed = 0u64;
    let mut active_sum = 0u64;
    let mut active = 0u64;
    // Last cell of each text row is blank in this fixture: use its center as the row background.
    let bg = a.get_pixel(a.width() - 2, row * ch + ch / 2).0;
    for y in row * ch..(row + 1) * ch {
      for x in 0..a.width() {
        let pa = a.get_pixel(x, y).0;
        let pb = b.get_pixel(x, y).0;
        let errors: [u8; 3] = std::array::from_fn(|c| pa[c].abs_diff(pb[c]));
        let error: u64 = errors.iter().map(|v| u64::from(*v)).sum();
        sum += error;
        let peak = *errors.iter().max().unwrap();
        if peak > 16 {
          changed += 1;
        }
        if x >= a.width() / COLS * 2 && (pa != bg || pb != bg) {
          active += 1;
          active_sum += error;
        }
        heat.put_pixel(x, y, Rgb([peak.saturating_mul(4), 0, 0]));
      }
    }
    rows.push(json!({"row":row,"rgb_mae":sum as f64/(a.width()*ch*3) as f64,
            "changed_fraction_gt16":changed as f64/(a.width()*ch) as f64,
            "active_pixels":active,"active_rgb_mae":if active>0 {active_sum as f64/(active*3) as f64} else {0.}}));
  }
  fs::create_dir_all(out)?;
  heat.save(out.join("difference.png"))?;
  let report = json!({"reference":reference.display().to_string(),"candidate":candidate.display().to_string(),
        "width":a.width(),"height":a.height(),"alignment":"none; same cell crop required","rows":rows,
        "interpretation":"lower is closer; no pass threshold claims visual equivalence; inspect text rows separately from background/palette"});
  fs::write(
    out.join("comparison.json"),
    serde_json::to_vec_pretty(&report)?,
  )?;
  println!("{}", out.join("comparison.json").display());
  Ok(())
}
fn main() -> Result<()> {
  let args: Vec<_> = std::env::args().skip(1).collect();
  match args.as_slice() {
        [cmd,a,b,out] if cmd=="calibrate"=>calibrate(Path::new(a),Path::new(b),Path::new(out)),
        [cmd,a,b,out] if cmd=="compare"=>compare(Path::new(a),Path::new(b),Path::new(out)),
        _=>Err("usage: image-compare calibrate <window.png> <profile.json> <out-dir> | compare <reference.png> <candidate.png> <out-dir>".into())
    }
}
#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn ruler_finds_origin_and_integer_pitch() {
    let mut image = RgbImage::new(900, 200);
    for c in 0..COLS {
      for x in 7 + c * 10..7 + (c + 1) * 10 {
        for y in 13..33 {
          image.put_pixel(
            x,
            y,
            Rgb(if c % 2 == 0 {
              [255, 0, 255]
            } else {
              [0, 255, 255]
            }),
          );
        }
      }
    }
    assert_eq!(locate(&image).unwrap(), (7, 13, 10, 20));
  }
  #[test]
  fn missing_ruler_is_error() {
    assert!(locate(&RgbImage::new(80, 20)).is_err());
  }
  #[test]
  fn partial_ruler_is_not_a_measurement() {
    let mut image = RgbImage::new(80, 20);
    for x in 0..79 {
      image.put_pixel(
        x,
        0,
        Rgb(if x % 2 == 0 {
          [255, 0, 255]
        } else {
          [0, 255, 255]
        }),
      );
    }
    assert!(locate(&image).is_err());
  }
}
