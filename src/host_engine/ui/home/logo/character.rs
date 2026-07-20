use super::{CellStyle, LogoCell, LogoRandom, PADDED_TEMPLATE, cells_to_rich_text};

const FRAME_SECONDS: f64 = 0.06;
const COLOR: (u8, u8, u8) = (81, 209, 107);
const POOL: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

pub(super) struct CharacterLogo {
  grid: Vec<Vec<bool>>,
  chars: Vec<Vec<char>>,
  timers: Vec<Vec<i32>>,
  sweep_diagonal: i32,
  hold: u32,
  steps: u64,
}

impl CharacterLogo {
  pub fn new(rng: &mut LogoRandom<'_>) -> Self {
    let width = PADDED_TEMPLATE
      .iter()
      .map(|line| line.chars().count())
      .max()
      .unwrap_or(0);
    let mut grid = vec![vec![false; width]; PADDED_TEMPLATE.len()];
    let mut chars = vec![vec![' '; width]; PADDED_TEMPLATE.len()];
    let mut timers = vec![vec![0; width]; PADDED_TEMPLATE.len()];
    for (y, line) in PADDED_TEMPLATE.iter().enumerate() {
      let source = line.chars().collect::<Vec<_>>();
      for x in 0..width {
        grid[y][x] = source.get(x).copied().unwrap_or(' ') != '█';
        chars[y][x] = if grid[y][x] { random_char(rng) } else { ' ' };
        timers[y][x] = rng.i32_inclusive(0, 36);
      }
    }
    let mut logo = Self {
      grid,
      chars,
      timers,
      sweep_diagonal: -1,
      hold: 0,
      steps: 0,
    };
    logo.step(rng);
    logo
  }

  pub fn advance(&mut self, seconds: f64, rng: &mut LogoRandom<'_>) {
    let target = 1 + (seconds / FRAME_SECONDS).floor() as u64;
    while self.steps < target {
      self.step(rng);
    }
  }

  pub fn render(&self) -> String {
    let rows = self
      .chars
      .iter()
      .map(|row| {
        row
          .iter()
          .map(|ch| {
            LogoCell::styled(
              *ch,
              CellStyle {
                fg: Some(COLOR),
                ..Default::default()
              },
            )
          })
          .collect::<Vec<_>>()
      })
      .collect::<Vec<_>>();
    cells_to_rich_text(&rows)
  }

  fn step(&mut self, rng: &mut LogoRandom<'_>) {
    self.steps += 1;
    if self.hold > 0 {
      self.refresh(rng);
      self.hold -= 1;
      if self.hold == 0 {
        self.sweep_diagonal = -1;
      }
      return;
    }

    self.sweep_diagonal += 1;
    let max_diagonal = (self.grid.len() + self.grid[0].len() - 2) as i32;
    if self.sweep_diagonal <= max_diagonal {
      for y in 0..self.grid.len() {
        let x = self.sweep_diagonal - y as i32;
        if x >= 0 && (x as usize) < self.grid[y].len() {
          let x = x as usize;
          self.grid[y][x] = !self.grid[y][x];
          self.chars[y][x] = if self.grid[y][x] {
            random_char(rng)
          } else {
            ' '
          };
        }
      }
    } else {
      self.hold = 80;
    }
    self.refresh(rng);
  }

  fn refresh(&mut self, rng: &mut LogoRandom<'_>) {
    for y in 0..self.grid.len() {
      for x in 0..self.grid[y].len() {
        if self.grid[y][x] {
          self.timers[y][x] -= 1;
          if self.timers[y][x] <= 0 {
            self.chars[y][x] = random_char(rng);
            self.timers[y][x] = rng.i32_inclusive(12, 36);
          }
        }
      }
    }
  }
}

fn random_char(rng: &mut LogoRandom<'_>) -> char {
  POOL[rng.usize_inclusive(0, POOL.len() - 1)] as char
}
