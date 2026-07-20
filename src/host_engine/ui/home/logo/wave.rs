use super::{CellStyle, LogoCell, PADDED_TEMPLATE, cells_to_rich_text};

const FRAME_SECONDS: f64 = 0.10;
const HEIGHT_CHARS: [char; 8] = [' ', '·', '.', '-', '+', '*', '#', '@'];
const COLORS: [(u8, u8, u8); 5] = [
  (10, 25, 55),
  (30, 60, 120),
  (65, 105, 195),
  (120, 190, 235),
  (200, 230, 255),
];

pub(super) struct WaveLogo {
  tick: u64,
}

impl WaveLogo {
  pub fn new() -> Self {
    Self { tick: 1 }
  }

  pub fn advance(&mut self, seconds: f64) {
    self.tick = 1 + (seconds / FRAME_SECONDS).floor() as u64;
  }

  pub fn render(&self) -> String {
    let width = PADDED_TEMPLATE
      .iter()
      .map(|line| line.chars().count())
      .max()
      .unwrap_or(0);
    let text_points = PADDED_TEMPLATE
      .iter()
      .enumerate()
      .flat_map(|(y, line)| {
        line
          .chars()
          .enumerate()
          .filter_map(move |(x, ch)| (ch == '█').then_some((x, y)))
      })
      .collect::<Vec<_>>();
    let rows = PADDED_TEMPLATE
      .iter()
      .enumerate()
      .map(|(y, line)| {
        let chars = line.chars().collect::<Vec<_>>();
        (0..width)
          .map(|x| {
            let distance = text_points
              .iter()
              .map(|(tx, ty)| x.abs_diff(*tx) + y.abs_diff(*ty))
              .min()
              .unwrap_or(width) as f64;
            let time = self.tick as f64 * 0.04;
            let mut height = (distance * 0.30 - time * 1.2).sin() * 0.6
              + (distance * 0.55 - time * 0.8).sin() * 0.25
              + ((x + y) as f64 * 0.08 - time * 0.3).sin() * 0.15;
            height = (height + 1.0) / 2.0;
            if chars.get(x) == Some(&'█') {
              height = (height + 0.45).min(1.0);
            }
            height = height.clamp(0.0, 1.0);
            let ch = HEIGHT_CHARS[(height * (HEIGHT_CHARS.len() - 1) as f64) as usize];
            if ch == ' ' {
              LogoCell::plain(ch)
            } else {
              LogoCell::styled(
                ch,
                CellStyle {
                  fg: Some(interpolate_color(height)),
                  ..Default::default()
                },
              )
            }
          })
          .collect::<Vec<_>>()
      })
      .collect::<Vec<_>>();
    cells_to_rich_text(&rows)
  }
}

fn interpolate_color(value: f64) -> (u8, u8, u8) {
  let position = value.clamp(0.0, 1.0) * (COLORS.len() - 1) as f64;
  let index = position.floor() as usize;
  if index >= COLORS.len() - 1 {
    return COLORS[COLORS.len() - 1];
  }
  let amount = position - index as f64;
  let left = COLORS[index];
  let right = COLORS[index + 1];
  (
    (left.0 as f64 + (right.0 as f64 - left.0 as f64) * amount) as u8,
    (left.1 as f64 + (right.1 as f64 - left.1 as f64) * amount) as u8,
    (left.2 as f64 + (right.2 as f64 - left.2 as f64) * amount) as u8,
  )
}
