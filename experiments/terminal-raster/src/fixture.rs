use serde::{Deserialize, Serialize};
use unicode_width::UnicodeWidthStr;

pub const COLS: u32 = 80;
pub const ROWS: u32 = 20;
pub type Rgb = [u8; 3];

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
  pub family: String,
  pub bold_is_bright: bool,
  pub weight: u16,
  pub font_px: f32,
  pub cell_width: f32,
  pub cell_height: u32,
  pub baseline: f32,
  pub foreground: Rgb,
  pub background: Rgb,
  pub palette: [Rgb; 16],
}

impl Profile {
  pub fn validate(&self) -> Result<(), String> {
    if self.family.trim().is_empty() || !(1..=999).contains(&self.weight) {
      return Err("font family or weight is invalid".into());
    }
    if !self.font_px.is_finite()
      || !(1.0..=128.0).contains(&self.font_px)
      || !self.cell_width.is_finite()
      || !(1.0..=128.0).contains(&self.cell_width)
      || !(1..=256).contains(&self.cell_height)
      || !self.baseline.is_finite()
      || !(0.0..=self.cell_height as f32).contains(&self.baseline)
    {
      return Err("invalid or excessive raster geometry".into());
    }
    Ok(())
  }
  pub fn x(&self, col: u32) -> u32 {
    (col as f32 * self.cell_width).round() as u32
  }
}

#[derive(Clone, Copy, Debug, Serialize)]
pub enum Color {
  Default,
  Ansi(u8),
  Rgb(Rgb),
}

impl Color {
  pub fn rgb(self, profile: &Profile, background: bool) -> Rgb {
    match self {
      Self::Default => {
        if background {
          profile.background
        } else {
          profile.foreground
        }
      }
      Self::Rgb(rgb) => rgb,
      Self::Ansi(i) if i < 16 => profile.palette[i as usize],
      Self::Ansi(i) if i >= 232 => [8 + (i - 232) * 10; 3],
      Self::Ansi(i) => {
        let n = i - 16;
        let channel = |x| if x == 0 { 0 } else { 55 + 40 * x };
        [channel(n / 36), channel(n / 6 % 6), channel(n % 6)]
      }
    }
  }
  pub fn sgr(self, bg: bool) -> String {
    let code = if bg { 48 } else { 38 };
    match self {
      Self::Default => if bg { "49" } else { "39" }.into(),
      Self::Ansi(i) => format!("{code};5;{i}"),
      Self::Rgb([r, g, b]) => format!("{code};2;{r};{g};{b}"),
    }
  }
}

#[derive(Clone, Debug, Serialize)]
pub struct Run {
  pub x: u32,
  pub y: u32,
  pub text: String,
  pub fg: Color,
  pub bg: Color,
  pub bold: bool,
  pub italic: bool,
  pub underline: bool,
  pub reverse: bool,
}
impl Run {
  pub fn width(&self) -> u32 {
    UnicodeWidthStr::width(self.text.as_str()) as u32
  }
  pub fn colors(&self, p: &Profile) -> (Rgb, Rgb) {
    let fg = if self.bold && p.bold_is_bright {
      match self.fg {
        Color::Default => p.palette[15],
        Color::Ansi(i) if i < 8 => p.palette[(i + 8) as usize],
        other => other.rgb(p, false),
      }
    } else {
      self.fg.rgb(p, false)
    };
    let pair = (fg, self.bg.rgb(p, true));
    if self.reverse { (pair.1, pair.0) } else { pair }
  }
}

pub fn runs() -> Vec<Run> {
  let mut rows = Vec::new();
  let normal = |x, y, text: String| Run {
    x,
    y,
    text,
    fg: Color::Default,
    bg: Color::Default,
    bold: false,
    italic: false,
    underline: false,
    reverse: false,
  };
  for x in 0..COLS {
    let mut r = normal(x, 0, " ".into());
    r.bg = Color::Rgb(if x % 2 == 0 {
      [255, 0, 255]
    } else {
      [0, 255, 255]
    });
    rows.push(r);
  }
  for y in 1..ROWS {
    let mut r = normal(0, y, " ".into());
    r.bg = Color::Rgb(if y % 2 == 0 {
      [0, 0, 255]
    } else {
      [255, 255, 0]
    });
    rows.push(r);
  }
  for (y, text) in [
    (
      1,
      "ASCII 0123456789 ABCDEFGHIJKLMNOPQRSTUVWXYZ abcdefghijklmnopqrstuvwxyz",
    ),
    (
      2,
      "WIDTH |iiiiiiiiii|MMMMMMMMMM|..........|WWWWWWWWWW|0000000000|",
    ),
    (3, "CJK   中文字体截图终端宽度测试 日本語 한글 ＡＢＣ１２３"),
    (
      4,
      "MARK  e\u{301} a\u{308} n\u{303} A\u{30a} o\u{302}\u{301} | é ä ñ Å | fi ffi -> => != ===",
    ),
    (5, "RTL   العربية سلام עברית abc 123"),
    (6, "INDIC नमस्ते दुनिया क्षि कि हिन्दी বাংলা தமிழ்"),
    (7, "EMOJI 😀 👩\u{200d}💻 ❤️ ☺️ 🇨🇳 👍🏽"),
    (8, "BOX   ┌──────────┐ ╔══════════╗ ┏━━━━━━━━━━┓"),
    (9, "BOX   │          │ ║          ║ ┃          ┃"),
    (10, "BOX   └──────────┘ ╚══════════╝ ┗━━━━━━━━━━┛"),
    (11, "BLOCK ▀▄█▌▐░▒▓ ▁▂▃▄▅▆▇█ ▖▗▘▙▚▛▜▝▞▟"),
    (12, "STYLE regular: Mm 中文 0123"),
    (13, "STYLE bold:    Mm 中文 0123"),
    (14, "STYLE italic:  Mm 中文 0123"),
    (15, "STYLE underline + inverse"),
    (
      18,
      "DEFAULT foreground / background (profile theme, not hardcoded white)",
    ),
    (19, "END | right edge follows 80 cells; Q/Esc exits |"),
  ] {
    let mut r = normal(2, y, text.into());
    r.bold = y == 13;
    r.italic = y == 14;
    r.underline = y == 15;
    r.reverse = y == 15;
    rows.push(r);
  }
  for i in 0..16u8 {
    let mut r = normal(2 + u32::from(i) * 4, 16, "    ".into());
    r.bg = Color::Ansi(i);
    rows.push(r);
    let mut r = normal(2 + u32::from(i) * 4, 17, "    ".into());
    r.bg = Color::Rgb([i * 17, 255 - i * 17, 128]);
    rows.push(r);
  }
  rows
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn all_runs_stay_within_fixture() {
    for r in runs() {
      assert!(r.x + r.width() <= COLS, "{}", r.text);
      assert!(r.y < ROWS);
    }
  }
  #[test]
  fn invalid_geometry_is_rejected() {
    let mut p: Profile =
      serde_json::from_str(include_str!("../../../profiles/current.json")).unwrap();
    p.cell_width = f32::NAN;
    assert!(p.validate().is_err());
    p.cell_width = 15.;
    p.cell_height = 0;
    assert!(p.validate().is_err());
    p.cell_height = 36;
    p.baseline = 37.;
    assert!(p.validate().is_err());
  }
  #[test]
  fn bright_and_reverse_follow_profile() {
    let p: Profile = serde_json::from_str(include_str!("../../../profiles/current.json")).unwrap();
    let mut r = runs().into_iter().find(|r| r.y == 13 && r.x == 2).unwrap();
    assert_eq!(r.colors(&p).0, p.palette[15]);
    r.reverse = true;
    assert_eq!(r.colors(&p), (p.background, p.palette[15]));
  }
  #[test]
  fn ansi_extremes() {
    let p: Profile = serde_json::from_str(include_str!("../../../profiles/current.json")).unwrap();
    assert_eq!(Color::Ansi(16).rgb(&p, false), [0; 3]);
    assert_eq!(Color::Ansi(231).rgb(&p, false), [255; 3]);
    assert_eq!(Color::Ansi(255).rgb(&p, false), [238; 3]);
  }
}
