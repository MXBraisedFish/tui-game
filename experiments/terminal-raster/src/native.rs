//! Windows-only control: DirectWrite shaping/fallback and Direct2D software rasterization.
//! This is not a reimplementation of Windows Terminal's Atlas renderer.
use crate::fixture::{COLS, Profile, ROWS, Rgb, Run};
use image::RgbaImage;
use serde_json::{Value, json};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;
use windows::{
  Win32::{
    Graphics::{
      Direct2D::{Common::*, *},
      DirectWrite::*,
      Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM,
      Imaging::*,
    },
    System::Com::*,
  },
  core::*,
};

struct Com;
impl Com {
  fn new() -> Result<Self> {
    unsafe {
      CoInitializeEx(None, COINIT_MULTITHREADED).ok()?;
    }
    Ok(Self)
  }
}
impl Drop for Com {
  fn drop(&mut self) {
    unsafe {
      CoUninitialize();
    }
  }
}
fn wide(s: &str) -> Vec<u16> {
  s.encode_utf16().chain(Some(0)).collect()
}
fn color(c: Rgb) -> D2D1_COLOR_F {
  D2D1_COLOR_F {
    r: c[0] as f32 / 255.,
    g: c[1] as f32 / 255.,
    b: c[2] as f32 / 255.,
    a: 1.,
  }
}

pub fn render(
  p: &Profile,
  runs: &[Run],
  mode: &str,
  cleartype: bool,
) -> std::result::Result<(RgbaImage, Value), Box<dyn std::error::Error>> {
  let _com = Com::new()?;
  // All COM objects stay on this initialized thread; WIC owns the bitmap throughout drawing.
  unsafe {
    let dw: IDWriteFactory = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED)?;
    let mut collection = None;
    dw.GetSystemFontCollection(&mut collection, false)?;
    let collection = collection.ok_or("system font collection unavailable")?;
    let family = wide(&p.family);
    let locale = wide("en-us");
    let mut index = 0;
    let mut exists = BOOL(0);
    collection.FindFamilyName(PCWSTR(family.as_ptr()), &mut index, &mut exists)?;
    if !exists.as_bool() {
      return Err(format!("DirectWrite family not installed: {}", p.family).into());
    }
    let matched = collection.GetFontFamily(index)?.GetFirstMatchingFont(
      DWRITE_FONT_WEIGHT(p.weight as i32),
      DWRITE_FONT_STRETCH_NORMAL,
      DWRITE_FONT_STYLE_NORMAL,
    )?;
    let face = matched.CreateFontFace()?;
    let mut metrics = DWRITE_FONT_METRICS::default();
    face.GetMetrics(&mut metrics);
    let scale = p.font_px / metrics.designUnitsPerEm as f32;
    let wic: IWICImagingFactory =
      CoCreateInstance(&CLSID_WICImagingFactory, None, CLSCTX_INPROC_SERVER)?;
    let (w, h) = (p.x(COLS), p.cell_height * ROWS);
    let bitmap = wic
      .CreateBitmap(w, h, &GUID_WICPixelFormat32bppBGR, WICBitmapCacheOnLoad)
      .map_err(|e| format!("WIC CreateBitmap: {e}"))?;
    let d2d: ID2D1Factory = D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None)?;
    let target = d2d
      .CreateWicBitmapRenderTarget(
        &bitmap,
        &D2D1_RENDER_TARGET_PROPERTIES {
          r#type: D2D1_RENDER_TARGET_TYPE_SOFTWARE,
          pixelFormat: D2D1_PIXEL_FORMAT {
            format: DXGI_FORMAT_B8G8R8A8_UNORM,
            alphaMode: D2D1_ALPHA_MODE_IGNORE,
          },
          dpiX: 96.,
          dpiY: 96.,
          usage: D2D1_RENDER_TARGET_USAGE_NONE,
          minLevel: D2D1_FEATURE_LEVEL_DEFAULT,
        },
      )
      .map_err(|e| format!("CreateWicBitmapRenderTarget: {e}"))?;
    target.SetTextAntialiasMode(if cleartype {
      D2D1_TEXT_ANTIALIAS_MODE_CLEARTYPE
    } else {
      D2D1_TEXT_ANTIALIAS_MODE_GRAYSCALE
    });
    target.SetAntialiasMode(D2D1_ANTIALIAS_MODE_ALIASED);
    target.BeginDraw();
    target.Clear(Some(&color(p.background)));
    let mut line_metrics = Vec::new();
    for run in runs {
      let (fg, bg) = run.colors(p);
      let left = p.x(run.x) as f32;
      let top = (run.y * p.cell_height) as f32;
      let right = p.x(run.x + run.width()) as f32;
      let rect = D2D_RECT_F {
        left,
        top,
        right,
        bottom: top + p.cell_height as f32,
      };
      let brush = target.CreateSolidColorBrush(&color(bg), None)?;
      target.FillRectangle(&rect, &brush);
      // Profile intenseTextStyle=bright: bold is deliberately not converted to a heavier face.
      let format = dw.CreateTextFormat(
        PCWSTR(family.as_ptr()),
        &collection,
        DWRITE_FONT_WEIGHT(p.weight as i32),
        if run.italic {
          DWRITE_FONT_STYLE_ITALIC
        } else {
          DWRITE_FONT_STYLE_NORMAL
        },
        DWRITE_FONT_STRETCH_NORMAL,
        p.font_px,
        PCWSTR(locale.as_ptr()),
      )?;
      format.SetWordWrapping(DWRITE_WORD_WRAPPING_NO_WRAP)?;
      format.SetLineSpacing(
        DWRITE_LINE_SPACING_METHOD_UNIFORM,
        p.cell_height as f32,
        p.baseline,
      )?;
      let text: Vec<u16> = run.text.encode_utf16().collect();
      let layout = dw.CreateTextLayout(&text, &format, 4096., p.cell_height as f32)?;
      let mut measured = DWRITE_TEXT_METRICS::default();
      layout.GetMetrics(&mut measured)?;
      line_metrics.push(json!({"row":run.y,"text":run.text,"natural_width":measured.widthIncludingTrailingWhitespace,"grid_width":right-left}));
      let brush = target.CreateSolidColorBrush(&color(fg), None)?;
      target.PushAxisAlignedClip(&rect, D2D1_ANTIALIAS_MODE_ALIASED);
      if mode == "cluster" {
        let spaced: IDWriteTextLayout1 = layout.cast()?;
        let mut clusters = vec![DWRITE_CLUSTER_METRICS::default(); text.len()];
        let mut count = 0;
        layout.GetClusterMetrics(Some(&mut clusters), &mut count)?;
        let mut offset = 0;
        for cluster in &clusters[..count as usize] {
          let length = cluster.length as usize;
          let value = String::from_utf16(&text[offset..offset + length])?;
          let expected = UnicodeWidthStr::width(value.as_str()) as f32 * p.cell_width;
          spaced.SetCharacterSpacing(
            0.,
            expected - cluster.width,
            0.,
            DWRITE_TEXT_RANGE {
              startPosition: offset as u32,
              length: length as u32,
            },
          )?;
          offset += length;
        }
      }
      if mode == "cell" {
        let mut col = run.x;
        for grapheme in run.text.graphemes(true) {
          let text: Vec<u16> = grapheme.encode_utf16().collect();
          let layout = dw.CreateTextLayout(&text, &format, 4096., p.cell_height as f32)?;
          target.DrawTextLayout(
            windows_numerics::Vector2 {
              X: p.x(col) as f32,
              Y: top,
            },
            &layout,
            &brush,
            D2D1_DRAW_TEXT_OPTIONS_NONE,
          );
          col += UnicodeWidthStr::width(grapheme) as u32;
        }
      } else {
        target.DrawTextLayout(
          windows_numerics::Vector2 { X: left, Y: top },
          &layout,
          &brush,
          D2D1_DRAW_TEXT_OPTIONS_NONE,
        );
      }
      if run.underline {
        let y = (top + p.baseline + 2.).min(rect.bottom - 1.);
        target.FillRectangle(
          &D2D_RECT_F {
            left,
            top: y,
            right,
            bottom: y + 1.,
          },
          &brush,
        );
      }
      target.PopAxisAlignedClip();
    }
    target
      .EndDraw(None, None)
      .map_err(|e| format!("D2D EndDraw: {e}"))?;
    let mut bytes = vec![0; w as usize * h as usize * 4];
    bitmap.CopyPixels(std::ptr::null(), w * 4, &mut bytes)?;
    for pixel in bytes.chunks_exact_mut(4) {
      pixel.swap(0, 2);
      pixel[3] = 255;
    }
    let image = RgbaImage::from_raw(w, h, bytes).ok_or("invalid WIC pixel buffer")?;
    Ok((
      image,
      json!({"backend":"DirectWrite + Direct2D WIC software target","layout_mode":mode,"cleartype":cleartype,
            "primary_family":p.family,"weight":matched.GetWeight().0,"simulations":face.GetSimulations().0,"ascent":metrics.ascent as f32*scale,
            "descent":metrics.descent as f32*scale,"line_gap":metrics.lineGap as f32*scale,
            "runs":line_metrics,"limitations":["system fallback differs from strict software probe", "cell mode isolates graphemes and loses cross-cluster joining", "run mode preserves shaping but not terminal cell advances", "monochrome emoji; native color layers disabled", "not the terminal Atlas renderer"]}),
    ))
  }
}
