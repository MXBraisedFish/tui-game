use crate::fixture::{COLS, Profile, ROWS, Run};
use harfrust::{FontRef, ShapeOptions, ShaperData, UnicodeBuffer};
use image::{Rgba, RgbaImage};
use serde_json::{Value, json};

pub struct SoftwareFont {
  font: fontdue::Font,
  bytes: Vec<u8>,
  face: u32,
  pub info: Value,
}
impl SoftwareFont {
  pub fn load(p: &Profile) -> Result<Self, Box<dyn std::error::Error>> {
    let mut db = fontdb::Database::new();
    db.load_system_fonts();
    let id = db
      .query(&fontdb::Query {
        families: &[fontdb::Family::Name(&p.family)],
        weight: fontdb::Weight(p.weight),
        ..Default::default()
      })
      .ok_or("primary font not installed")?;
    let face = db.face(id).ok_or("missing font face")?;
    let info = json!({"families":face.families.iter().map(|(name,_)|name).collect::<Vec<_>>(),"weight":face.weight.0,"postscript_name":face.post_script_name,
            "source":format!("{:?}",face.source),"face_index":face.index,"fallback":"disabled for controlled software experiment"});
    let (bytes, index) = db
      .with_face_data(id, |bytes, index| (bytes.to_vec(), index))
      .ok_or("font data unavailable")?;
    let font = fontdue::Font::from_bytes(
      bytes.clone(),
      fontdue::FontSettings {
        collection_index: index,
        ..Default::default()
      },
    )?;
    Ok(Self {
      font,
      bytes,
      face: index,
      info,
    })
  }
  pub fn render(
    &self,
    p: &Profile,
    runs: &[Run],
    fit: bool,
  ) -> Result<(RgbaImage, Value), Box<dyn std::error::Error>> {
    let mut image = RgbaImage::from_pixel(
      p.x(COLS),
      p.cell_height * ROWS,
      Rgba([p.background[0], p.background[1], p.background[2], 255]),
    );
    let font = FontRef::from_index(&self.bytes, self.face)?;
    let data = ShaperData::new(&font);
    let shaper = data.shaper(&font).build();
    let mut missing = Vec::new();
    for run in runs {
      let (fg, bg) = run.colors(p);
      let left = p.x(run.x);
      let right = p.x(run.x + run.width());
      let top = run.y * p.cell_height;
      for y in top..top + p.cell_height {
        for x in left..right {
          image.put_pixel(x, y, Rgba([bg[0], bg[1], bg[2], 255]));
        }
      }
      let mut buffer = UnicodeBuffer::new();
      buffer.push_str(&run.text);
      buffer.guess_segment_properties();
      let glyphs = shaper.shape(
        buffer,
        ShapeOptions::new().scale(Some((p.font_px * 64.).round() as i32)),
      );
      let total: f32 = glyphs
        .glyph_positions()
        .iter()
        .map(|pos| pos.x_advance as f32 / 64.)
        .sum();
      let ratio = if fit && total > 0.0 {
        (right - left) as f32 / total
      } else {
        1.0
      };
      let mut pen = left as f32;
      for (info, pos) in glyphs.glyph_infos().iter().zip(glyphs.glyph_positions()) {
        if info.glyph_id == 0 {
          missing.push(json!({"row":run.y,"byte_cluster":info.cluster,"text":run.text}));
        }
        let (m, bitmap) = self.font.rasterize_indexed(info.glyph_id as u16, p.font_px);
        let x = (pen + pos.x_offset as f32 / 64. * ratio).round() as i32 + m.xmin;
        let y = (top as f32 + p.baseline - pos.y_offset as f32 / 64.).round() as i32
          - m.height as i32
          - m.ymin;
        for by in 0..m.height {
          for bx in 0..m.width {
            let dx = x + bx as i32;
            let dy = y + by as i32;
            if dx < left as i32
              || dx >= right as i32
              || dy < top as i32
              || dy >= (top + p.cell_height) as i32
            {
              continue;
            }
            let a = u32::from(bitmap[by * m.width + bx]);
            let pixel = image.get_pixel_mut(dx as u32, dy as u32);
            for c in 0..3 {
              pixel[c] =
                ((u32::from(fg[c]) * a + u32::from(pixel[c]) * (255 - a) + 127) / 255) as u8;
            }
          }
        }
        pen += pos.x_advance as f32 / 64. * ratio;
      }
      if run.underline {
        for x in left..right {
          image.put_pixel(
            x,
            top + (p.baseline as u32 + 2).min(p.cell_height - 1),
            Rgba([fg[0], fg[1], fg[2], 255]),
          );
        }
      }
    }
    let metrics = self
      .font
      .horizontal_line_metrics(p.font_px)
      .ok_or("no line metrics")?;
    Ok((
      image,
      json!({"font":self.info,"font_px":p.font_px,"m_advance":self.font.metrics('M',p.font_px).advance_width,
            "ascent":metrics.ascent,"descent":metrics.descent,"line_gap":metrics.line_gap,"missing_glyphs":missing,
            "fit_run_advances":fit,"limitations":["isolated font: no fallback", "bold/italic flags not synthesized", "no paragraph bidi reordering", "fontdue coverage blended in sRGB"]}),
    ))
  }
}
