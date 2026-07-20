use super::{CellStyle, LogoCell, LogoRandom, SELECT_TEMPLATE, cells_to_rich_text};

const FRAME_SECONDS: f64 = 0.06;
const GHOST: (u8, u8, u8) = (80, 80, 90);
const REVEAL: (u8, u8, u8) = (60, 170, 255);
const WHITE: (u8, u8, u8) = (240, 240, 250);
const DOT: (u8, u8, u8) = (110, 110, 125);
const CORNER_TOP_LEFT: (u8, u8, u8) = (255, 55, 55);
const CORNER_BOTTOM_RIGHT: (u8, u8, u8) = (70, 185, 255);

struct SelectionFrame {
  x: usize,
  y: usize,
  width: usize,
  height: usize,
  tick: u32,
}

impl SelectionFrame {
  fn alive(&self) -> bool {
    self.tick < 110
  }
  fn visible_width(&self) -> usize {
    visible_size(self.tick, self.width)
  }
  fn visible_height(&self) -> usize {
    visible_size(self.tick, self.height)
  }
  fn opacity(&self) -> f64 {
    if self.tick < 10 {
      self.tick as f64 / 10.0
    } else if self.tick < 80 {
      1.0
    } else {
      (1.0 - (self.tick - 80) as f64 / 8.0).max(0.0)
    }
  }
  fn inside(&self, x: usize, y: usize) -> bool {
    let width = self.visible_width();
    let height = self.visible_height();
    x > self.x && x + 1 < self.x + width && y > self.y && y + 1 < self.y + height
  }
}

fn visible_size(tick: u32, size: usize) -> usize {
  if tick < 10 {
    let t = tick as f64 / 10.0;
    (1.0 + (1.0 - (1.0 - t).powi(3)) * (size - 1) as f64) as usize
  } else if tick < 80 {
    size
  } else {
    let t = (tick - 80) as f64 / 8.0;
    size
      .saturating_sub((t * t * (size - 1) as f64) as usize)
      .max(1)
  }
}

pub(super) struct SelectLogo {
  frames: Vec<SelectionFrame>,
  tick: u64,
  next_frame: u64,
  steps: u64,
}

impl SelectLogo {
  pub fn new(rng: &mut LogoRandom<'_>) -> Self {
    let mut logo = Self {
      frames: Vec::new(),
      tick: 0,
      next_frame: rng.usize_inclusive(30, 70) as u64,
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
    let width = SELECT_TEMPLATE
      .iter()
      .map(|line| line.chars().count())
      .max()
      .unwrap_or(0);
    let source = SELECT_TEMPLATE
      .iter()
      .map(|line| line.chars().collect::<Vec<_>>())
      .collect::<Vec<_>>();
    let mut rows = vec![vec![LogoCell::plain(' '); width]; source.len()];
    for y in 0..source.len() {
      for x in 0..source[y].len() {
        if source[y][x] != ' ' {
          rows[y][x] = styled(source[y][x], GHOST, false, true);
        }
      }
    }
    for x in 1..width - 1 {
      rows[0][x] = styled('▪', DOT, false, true);
      rows[source.len() - 1][x] = styled('▪', DOT, false, true);
    }
    for row in rows.iter_mut().take(source.len() - 1).skip(1) {
      row[0] = styled('▪', DOT, false, true);
      row[width - 1] = styled('▪', DOT, false, true);
    }
    let middle = source.len() / 2;
    for x in 1..width - 1 {
      if rows[middle][x].ch == ' ' {
        rows[middle][x] = styled('▪', DOT, false, true);
      }
    }
    for y in (0..source.len()).step_by(5) {
      if rows[y][width / 2].ch == ' ' {
        rows[y][width / 2] = styled('▪', DOT, false, true);
      }
    }
    for x in (5..width - 1).step_by(5) {
      for row in &mut rows {
        if row[x].ch == ' ' {
          row[x] = styled('▪', DOT, false, true);
        }
      }
    }

    for frame in &self.frames {
      if frame.opacity() < 0.3 || frame.visible_width() < 3 || frame.visible_height() < 2 {
        continue;
      }
      for y in 0..source.len() {
        for x in 0..source[y].len() {
          if !frame.inside(x, y) {
            continue;
          }
          if source[y][x] != ' ' {
            rows[y][x] = styled(source[y][x], REVEAL, true, false);
          } else if rows[y][x].ch == '▪' {
            rows[y][x] = LogoCell::plain(' ');
          }
        }
      }
    }
    for frame in &self.frames {
      draw_frame(&mut rows, &source, frame);
    }
    cells_to_rich_text(&rows)
  }

  fn step(&mut self, rng: &mut LogoRandom<'_>) {
    self.steps += 1;
    self.tick += 1;
    if self.tick >= self.next_frame && self.frames.len() < 2 {
      self.spawn(rng);
      self.next_frame = self.tick + rng.usize_inclusive(30, 80) as u64;
    }
    for frame in &mut self.frames {
      frame.tick += 1;
    }
    self.frames.retain(SelectionFrame::alive);
  }

  fn spawn(&mut self, rng: &mut LogoRandom<'_>) {
    let width = SELECT_TEMPLATE[0].chars().count();
    let height = SELECT_TEMPLATE.len();
    let frame_width = rng.usize_inclusive(15, 30);
    let frame_height = rng.usize_inclusive(5, 7);
    for _ in 0..200 {
      let x = rng.usize_inclusive(0, width - frame_width);
      let y = rng.usize_inclusive(0, height - frame_height);
      let overlaps = self.frames.iter().any(|frame| {
        let ax1 = x.saturating_sub(2);
        let ay1 = y.saturating_sub(2);
        let ax2 = x + frame_width + 1;
        let ay2 = y + frame_height + 1;
        ax1 <= frame.x + frame.width - 1
          && ax2 >= frame.x
          && ay1 <= frame.y + frame.height - 1
          && ay2 >= frame.y
      });
      if !overlaps {
        self.frames.push(SelectionFrame {
          x,
          y,
          width: frame_width,
          height: frame_height,
          tick: 0,
        });
        return;
      }
    }
  }
}

fn styled(ch: char, fg: (u8, u8, u8), bold: bool, dim: bool) -> LogoCell {
  LogoCell::styled(
    ch,
    CellStyle {
      fg: Some(fg),
      bold,
      dim,
      ..Default::default()
    },
  )
}

fn draw_frame(rows: &mut [Vec<LogoCell>], source: &[Vec<char>], frame: &SelectionFrame) {
  let width = frame.visible_width();
  let height = frame.visible_height();
  let opacity = frame.opacity();
  if opacity < 0.05 || width < 1 || height < 1 {
    return;
  }
  let x = frame.x;
  let y = frame.y;
  let bottom = y + height - 1;
  let right = x + width - 1;
  rows[y][x] = styled('╋', CORNER_TOP_LEFT, true, false);
  if width >= 2 {
    rows[y][right] = border_cell('┓', WHITE, source[y][right] == '█', true);
  }
  if height >= 2 {
    rows[bottom][x] = border_cell('┗', WHITE, source[bottom][x] == '█', true);
  }
  if width >= 2 && height >= 2 {
    rows[bottom][right] = styled('╋', CORNER_BOTTOM_RIGHT, true, false);
  }
  let color = if opacity > 0.7 { WHITE } else { GHOST };
  for dx in 1..width.saturating_sub(1) {
    rows[y][x + dx] = border_cell('╍', color, source[y][x + dx] == '█', false);
    if height >= 2 {
      rows[bottom][x + dx] = border_cell('╍', color, source[bottom][x + dx] == '█', false);
    }
  }
  for dy in 1..height.saturating_sub(1) {
    rows[y + dy][x] = border_cell('┇', color, source[y + dy][x] == '█', false);
    if width >= 2 {
      rows[y + dy][right] = border_cell('┇', color, source[y + dy][right] == '█', false);
    }
  }
}

fn border_cell(ch: char, fg: (u8, u8, u8), ghost_background: bool, bold: bool) -> LogoCell {
  LogoCell::styled(
    ch,
    CellStyle {
      fg: Some(fg),
      bg: ghost_background.then_some(GHOST),
      bold,
      ..Default::default()
    },
  )
}
