use super::{DYNAMIC_TEMPLATE, LogoCell, LogoRandom, cells_to_rich_text, random_rgb};

const FRAME_SECONDS: f64 = 0.16;
const GLITCH_CHARS: &[char] = &[
  '#', '@', '$', '%', '&', '!', '?', '*', '=', '+', '~', 'x', 'X', 'O', '0', 'a', 'b', 'c', 'd',
  'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w',
  'x', 'y', 'z',
];
const BLOCK_FRAGMENTS: &[char] = &[
  '▀', '▄', '▌', '▐', '▖', '▗', '▘', '▙', '▚', '▛', '▜', '▝', '▞', '▟',
];
const NOISE_CHARS: &[char] = &[
  '░', '▒', '▓', '▏', '▎', '▍', '▌', '▋', '▊', '▉', '╶', '╴', '╵', '╷', '╸', '╹', '╺', '╻', '·',
  '∙',
];

struct Corruption {
  row: usize,
  start: usize,
  end: usize,
  color: (u8, u8, u8),
  remaining: u32,
}
struct Cross {
  row: usize,
  column: usize,
  ch: char,
  bg: (u8, u8, u8),
  remaining: u32,
}
struct BlueBlock {
  row: usize,
  column: usize,
  height: usize,
  width: usize,
  gray: (u8, u8, u8),
  remaining: u32,
}
struct Noise {
  row: usize,
  column: usize,
  ch: char,
  remaining: u32,
}
struct Flicker {
  row: usize,
  reverse: bool,
  remaining: u32,
}
struct Fragment {
  row: usize,
  column: usize,
  ch: char,
  remaining: u32,
}

pub(super) struct ErrorLogo {
  rows: Vec<Vec<char>>,
  corruptions: Vec<Corruption>,
  crosses: Vec<Cross>,
  blue_blocks: Vec<BlueBlock>,
  noises: Vec<Noise>,
  flickers: Vec<Flicker>,
  fragments: Vec<Fragment>,
  steps: u64,
}

impl ErrorLogo {
  pub fn new(rng: &mut LogoRandom<'_>) -> Self {
    let width = DYNAMIC_TEMPLATE
      .iter()
      .map(|line| line.chars().count())
      .max()
      .unwrap_or(0);
    let mut rows = vec![vec![' '; width]];
    rows.extend(DYNAMIC_TEMPLATE.iter().map(|line| {
      let mut row = line.chars().collect::<Vec<_>>();
      row.resize(width, ' ');
      row
    }));
    rows.push(vec![' '; width]);
    let mut logo = Self {
      rows,
      corruptions: Vec::new(),
      crosses: Vec::new(),
      blue_blocks: Vec::new(),
      noises: Vec::new(),
      flickers: Vec::new(),
      fragments: Vec::new(),
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
    let mut canvas = self
      .rows
      .iter()
      .map(|row| {
        row
          .iter()
          .map(|ch| LogoCell::plain(*ch))
          .collect::<Vec<_>>()
      })
      .collect::<Vec<_>>();
    for effect in &self.flickers {
      for cell in &mut canvas[effect.row] {
        if effect.reverse {
          cell.style.reverse = !cell.style.reverse;
        } else {
          cell.style.dim = true;
        }
      }
    }
    for effect in &self.corruptions {
      for cell in &mut canvas[effect.row][effect.start..effect.end] {
        cell.style.fg = Some(effect.color);
      }
    }
    for effect in &self.blue_blocks {
      for y in 0..effect.height {
        for x in 0..effect.width {
          let cell = &mut canvas[effect.row + y][effect.column + x];
          if y == 0 {
            cell.ch = '▅';
            cell.style.fg = Some((0, 102, 255));
            cell.style.bg = Some(if x + 1 == effect.width {
              (255, 68, 68)
            } else {
              effect.gray
            });
          } else {
            cell.ch = '█';
            cell.style.fg = Some((0, 102, 255));
            cell.style.bg = None;
          }
        }
      }
    }
    for effect in &self.noises {
      canvas[effect.row][effect.column].ch = effect.ch;
      canvas[effect.row][effect.column].style.dim = true;
    }
    for effect in &self.crosses {
      let cell = &mut canvas[effect.row][effect.column];
      cell.ch = effect.ch;
      cell.style.bg = Some(effect.bg);
      cell.style.reverse = true;
    }
    for effect in &self.fragments {
      canvas[effect.row][effect.column].ch = effect.ch;
    }
    cells_to_rich_text(&canvas)
  }

  fn step(&mut self, rng: &mut LogoRandom<'_>) {
    self.steps += 1;
    for _ in 0..rng.usize_inclusive(0, 2) {
      self.spawn_corruption(rng);
    }
    for _ in 0..rng.usize_inclusive(0, 2) {
      self.spawn_cross(rng);
    }
    for _ in 0..rng.usize_inclusive(0, 1) {
      self.spawn_blue(rng);
    }
    self.spawn_noise(rng);
    self.spawn_flicker(rng);
    for _ in 0..rng.usize_inclusive(0, 1) {
      self.spawn_fragment(rng);
    }
    tick_and_retain(&mut self.corruptions, |v| &mut v.remaining);
    tick_and_retain(&mut self.crosses, |v| &mut v.remaining);
    tick_and_retain(&mut self.blue_blocks, |v| &mut v.remaining);
    tick_and_retain(&mut self.noises, |v| &mut v.remaining);
    tick_and_retain(&mut self.flickers, |v| &mut v.remaining);
    tick_and_retain(&mut self.fragments, |v| &mut v.remaining);
  }

  fn spawn_corruption(&mut self, rng: &mut LogoRandom<'_>) {
    if self.corruptions.len() >= 5 || !rng.chance(0.4) {
      return;
    }
    let row = rng.usize_inclusive(1, 5);
    let width = self.rows[row].len();
    let start = rng.usize_inclusive(0, width - 2);
    let end = rng.usize_inclusive(start + 1, (start + 15).min(width));
    self.corruptions.push(Corruption {
      row,
      start,
      end,
      color: random_rgb(rng),
      remaining: rng.usize_inclusive(5, 25) as u32,
    });
  }
  fn spawn_cross(&mut self, rng: &mut LogoRandom<'_>) {
    if self.crosses.len() >= 4 || !rng.chance(0.45) {
      return;
    }
    self.crosses.push(Cross {
      row: rng.usize_inclusive(0, 6),
      column: rng.usize_inclusive(0, self.rows[0].len() - 1),
      ch: rng.choose(GLITCH_CHARS),
      bg: random_rgb(rng),
      remaining: rng.usize_inclusive(8, 30) as u32,
    });
  }
  fn spawn_blue(&mut self, rng: &mut LogoRandom<'_>) {
    if self.blue_blocks.len() >= 6 || !rng.chance(0.15) {
      return;
    }
    let height = rng.usize_inclusive(1, 3);
    let width = rng.usize_inclusive(3, 6);
    let gray = rng.usize_inclusive(90, 160) as u8;
    self.blue_blocks.push(BlueBlock {
      row: rng.usize_inclusive(0, 7 - height),
      column: rng.usize_inclusive(0, self.rows[0].len() - width),
      height,
      width,
      gray: (gray, gray, gray),
      remaining: rng.usize_inclusive(20, 55) as u32,
    });
  }
  fn spawn_noise(&mut self, rng: &mut LogoRandom<'_>) {
    if self.noises.len() >= 6 || !rng.chance(0.12) {
      return;
    }
    for _ in 0..rng.usize_inclusive(5, 15) {
      self.noises.push(Noise {
        row: rng.usize_inclusive(0, 6),
        column: rng.usize_inclusive(0, self.rows[0].len() - 1),
        ch: rng.choose(NOISE_CHARS),
        remaining: rng.usize_inclusive(1, 4) as u32,
      });
    }
  }
  fn spawn_flicker(&mut self, rng: &mut LogoRandom<'_>) {
    if !self.flickers.is_empty() || !rng.chance(0.08) {
      return;
    }
    self.flickers.push(Flicker {
      row: rng.usize_inclusive(0, 6),
      reverse: rng.chance(0.5),
      remaining: rng.usize_inclusive(1, 3) as u32,
    });
  }
  fn spawn_fragment(&mut self, rng: &mut LogoRandom<'_>) {
    if self.fragments.len() >= 4 || !rng.chance(0.20) {
      return;
    }
    let row = rng.usize_inclusive(1, 5);
    let mut positions = self.rows[row]
      .iter()
      .enumerate()
      .filter_map(|(x, ch)| (*ch == '█').then_some(x))
      .collect::<Vec<_>>();
    rng.shuffle(&mut positions);
    for column in positions.into_iter().take(rng.usize_inclusive(1, 5)) {
      self.fragments.push(Fragment {
        row,
        column,
        ch: rng.choose(BLOCK_FRAGMENTS),
        remaining: rng.usize_inclusive(2, 10) as u32,
      });
    }
  }
}

fn tick_and_retain<T>(items: &mut Vec<T>, mut remaining: impl FnMut(&mut T) -> &mut u32) {
  for item in items.iter_mut() {
    *remaining(item) = remaining(item).saturating_sub(1);
  }
  items.retain_mut(|item| *remaining(item) > 0);
}
