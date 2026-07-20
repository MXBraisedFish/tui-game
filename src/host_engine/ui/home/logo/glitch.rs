use super::{CellStyle, GLITCH_TEMPLATE, LogoCell, LogoRandom, cells_to_rich_text};

const CYAN: (u8, u8, u8) = (0, 255, 255);
const WHITE: (u8, u8, u8) = (255, 255, 255);
const RED: (u8, u8, u8) = (255, 0, 0);
const MAGENTA: (u8, u8, u8) = (255, 0, 255);

struct LineState {
  leading: usize,
  offset: f64,
  initial_offset: f64,
  steps: u32,
  total_steps: u32,
}

pub(super) struct GlitchLogo {
  lines: Vec<LineState>,
  next_step: f64,
  cooldown_until: f64,
  has_triggered: bool,
}

impl GlitchLogo {
  pub fn new(rng: &mut LogoRandom<'_>) -> Self {
    let mut logo = Self {
      lines: GLITCH_TEMPLATE
        .iter()
        .map(|line| LineState {
          leading: line.chars().take_while(|ch| *ch == ' ').count(),
          offset: 0.0,
          initial_offset: 0.0,
          steps: 0,
          total_steps: 0,
        })
        .collect(),
      next_step: 0.0,
      cooldown_until: 0.0,
      has_triggered: false,
    };
    logo.step(0.0, rng);
    logo
  }

  pub fn advance(&mut self, seconds: f64, rng: &mut LogoRandom<'_>) {
    while seconds >= self.next_step {
      let now = self.next_step;
      self.step(now, rng);
      self.next_step += 0.08 + rng.f64() * 0.07;
    }
  }

  pub fn render(&self) -> String {
    let rows = GLITCH_TEMPLATE
      .iter()
      .enumerate()
      .map(|(y, source)| {
        let content = source
          .chars()
          .skip(self.lines[y].leading)
          .collect::<Vec<_>>();
        let extra = round_ties_even(self.lines[y].offset);
        let leading = (self.lines[y].leading as i32 + extra).max(0) as usize;
        let mut row = vec![LogoCell::plain(' '); leading];
        let mut index = 0;
        while index < content.len() {
          if content[index] != '█' {
            row.push(LogoCell::plain(content[index]));
            index += 1;
            continue;
          }
          let start = index;
          while index < content.len() && content[index] == '█' {
            index += 1;
          }
          let length = index - start;
          for run_index in 0..length {
            let fg = block_color(y, length, run_index);
            row.push(LogoCell::styled(
              '█',
              CellStyle {
                fg: Some(fg),
                ..Default::default()
              },
            ));
          }
        }
        row
      })
      .collect::<Vec<_>>();
    cells_to_rich_text(&rows)
  }

  fn step(&mut self, now: f64, rng: &mut LogoRandom<'_>) {
    let glitching = self.lines.iter().any(|line| line.steps > 0);
    if self.has_triggered && !glitching {
      self.cooldown_until = now + 1.0 + rng.f64() * 2.0;
      self.has_triggered = false;
    }
    if !self.has_triggered && !glitching && now >= self.cooldown_until && rng.chance(0.4) {
      let mut indices = (0..self.lines.len()).collect::<Vec<_>>();
      rng.shuffle(&mut indices);
      for index in indices.into_iter().take(5) {
        let offset = rng.choose(&[-2.0, -1.0, 1.0, 2.0]);
        let steps = rng.usize_inclusive(1, 8) as u32;
        self.lines[index].initial_offset = offset;
        self.lines[index].offset = offset;
        self.lines[index].steps = steps;
        self.lines[index].total_steps = steps;
      }
      self.has_triggered = true;
    }
    for line in &mut self.lines {
      if line.steps == 0 {
        continue;
      }
      line.offset -= line.initial_offset / line.total_steps as f64;
      line.steps -= 1;
      if line.steps == 0 {
        line.offset = 0.0;
      }
    }
  }
}

fn block_color(row: usize, run_length: usize, index: usize) -> (u8, u8, u8) {
  if index == 0 || row == 3 && run_length == 12 && index == 3 {
    CYAN
  } else if index + 1 == run_length || row == 3 && run_length == 12 && index + 4 == run_length {
    RED
  } else if row == 2 && run_length >= 10 && (index == 3 || index + 4 == run_length) {
    MAGENTA
  } else {
    WHITE
  }
}

fn round_ties_even(value: f64) -> i32 {
  let floor = value.floor();
  let fraction = value - floor;
  if fraction < 0.5 {
    floor as i32
  } else if fraction > 0.5 {
    floor as i32 + 1
  } else if floor as i32 % 2 == 0 {
    floor as i32
  } else {
    floor as i32 + 1
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn m_fourth_row_colors_internal_edges_without_removing_cells() {
    let colors = (0..12)
      .map(|index| block_color(3, 12, index))
      .collect::<Vec<_>>();

    assert_eq!(colors[0], CYAN);
    assert_eq!(colors[3], CYAN);
    assert_eq!(colors[8], RED);
    assert_eq!(colors[11], RED);
    assert!(colors[1..3].iter().all(|color| *color == WHITE));
    assert!(colors[4..8].iter().all(|color| *color == WHITE));
    assert!(colors[9..11].iter().all(|color| *color == WHITE));
  }
}
