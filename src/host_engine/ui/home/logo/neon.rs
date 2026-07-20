use super::{CellStyle, DYNAMIC_TEMPLATE, LogoCell, cells_to_rich_text};

const FRAME_SECONDS: f64 = 0.05;
const PERIOD: f64 = 22.0;
const DIAGONAL: f64 = 1.8;
const SPEED: f64 = -0.5;

pub(super) struct NeonLogo {
  offset: f64,
  rendered_steps: u64,
}

impl NeonLogo {
  pub fn new() -> Self {
    Self {
      offset: 0.0,
      rendered_steps: 0,
    }
  }

  pub fn advance(&mut self, seconds: f64) {
    let target = (seconds / FRAME_SECONDS).floor() as u64;
    if target > self.rendered_steps {
      self.offset = (self.offset + SPEED * FRAME_SECONDS * (target - self.rendered_steps) as f64)
        .rem_euclid(1.0);
      self.rendered_steps = target;
    }
  }

  pub fn render(&self) -> String {
    let rows = DYNAMIC_TEMPLATE
      .iter()
      .enumerate()
      .map(|(y, line)| {
        let mut block = 0usize;
        line
          .chars()
          .map(|ch| {
            if ch != '█' {
              return LogoCell::plain(ch);
            }
            let hue = ((block as f64 + y as f64 * DIAGONAL) / PERIOD + self.offset).rem_euclid(1.0);
            block += 1;
            LogoCell::styled(
              ch,
              CellStyle {
                fg: Some(hsv_to_rgb(hue, 0.5, 1.0)),
                ..Default::default()
              },
            )
          })
          .collect::<Vec<_>>()
      })
      .collect::<Vec<_>>();
    cells_to_rich_text(&rows)
  }
}

fn hsv_to_rgb(hue: f64, saturation: f64, value: f64) -> (u8, u8, u8) {
  let scaled = hue.rem_euclid(1.0) * 6.0;
  let sector = scaled.floor() as u8;
  let fraction = scaled - scaled.floor();
  let p = value * (1.0 - saturation);
  let q = value * (1.0 - saturation * fraction);
  let t = value * (1.0 - saturation * (1.0 - fraction));
  let (r, g, b) = match sector {
    0 => (value, t, p),
    1 => (q, value, p),
    2 => (p, value, t),
    3 => (p, q, value),
    4 => (t, p, value),
    _ => (value, p, q),
  };
  ((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
}
