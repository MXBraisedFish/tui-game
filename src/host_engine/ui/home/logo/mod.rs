mod character;
mod error;
mod glitch;
mod neon;
mod select;
mod wave;

use std::{sync::Arc, time::Duration};

use crate::host_engine::services::{
  AnimationBinding, AnimationClock, AnimationEasing, AnimationHandle, AnimationInterpolation,
  AnimationOwner, AnimationPlaybackOptions, AnimationProperty, AnimationRepeatCount,
  AnimationRepeatMode, AnimationRepeatOptions, AnimationService, AnimationSource, AnimationTarget,
  AnimationValue, DisplayLogoMode, RandomGeneratorId, RandomSeed, RandomService, RuntimeObjectPool,
  TweenDefinition,
};

use self::{
  character::CharacterLogo, error::ErrorLogo, glitch::GlitchLogo, neon::NeonLogo,
  select::SelectLogo, wave::WaveLogo,
};

pub(super) const DYNAMIC_TEMPLATE: [&str; 5] = [
  "   ████████  ██    ██  ██     ██████    █████   ███    ███  ███████   ",
  "      ██     ██    ██  ██    ██        ██   ██  ████  ████  ██        ",
  "      ██     ██    ██  ██    ██   ███  ███████  ██ ████ ██  █████     ",
  "      ██     ██    ██  ██    ██    ██  ██   ██  ██  ██  ██  ██        ",
  "      ██      ██████   ██     ██████   ██   ██  ██      ██  ███████   ",
];

pub(super) const PADDED_TEMPLATE: [&str; 7] = [
  "                                                                      ",
  DYNAMIC_TEMPLATE[0],
  DYNAMIC_TEMPLATE[1],
  DYNAMIC_TEMPLATE[2],
  DYNAMIC_TEMPLATE[3],
  DYNAMIC_TEMPLATE[4],
  "                                                                      ",
];

pub(super) const SELECT_TEMPLATE: [&str; 7] = [
  "▟                                                                    ▙",
  DYNAMIC_TEMPLATE[0],
  DYNAMIC_TEMPLATE[1],
  DYNAMIC_TEMPLATE[2],
  DYNAMIC_TEMPLATE[3],
  DYNAMIC_TEMPLATE[4],
  "▜                                                                    ▛",
];

pub(super) const GLITCH_TEMPLATE: [&str; 5] = [
  "  ██████████ ████  ████ ████     ████████   ███████  █████  █████ █████████  ",
  "     ████    ████  ████ ████    ████       ████ ████ ████████████ ████       ",
  "     ████    ████  ████ ████    ████ █████ █████████ ████████████ ███████    ",
  "     ████    ████  ████ ████    ████  ████ ████ ████ ████████████ ████       ",
  "     ████     ████████  ████     ████████  ████ ████ ████    ████ █████████  ",
];

const DYNAMIC_MODES: [DisplayLogoMode; 6] = [
  DisplayLogoMode::Neon,
  DisplayLogoMode::Wave,
  DisplayLogoMode::Error,
  DisplayLogoMode::Glitch,
  DisplayLogoMode::Select,
  DisplayLogoMode::Char,
];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct CellStyle {
  pub fg: Option<(u8, u8, u8)>,
  pub bg: Option<(u8, u8, u8)>,
  pub bold: bool,
  pub dim: bool,
  pub reverse: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct LogoCell {
  pub ch: char,
  pub style: CellStyle,
}

impl LogoCell {
  pub fn plain(ch: char) -> Self {
    Self {
      ch,
      style: CellStyle::default(),
    }
  }

  pub fn styled(ch: char, style: CellStyle) -> Self {
    Self { ch, style }
  }
}

pub(super) struct LogoRandom<'a> {
  service: &'a RandomService,
  pool: &'a mut RuntimeObjectPool,
  id: RandomGeneratorId,
}

impl<'a> LogoRandom<'a> {
  fn new(
    service: &'a RandomService,
    pool: &'a mut RuntimeObjectPool,
    id: RandomGeneratorId,
  ) -> Self {
    Self { service, pool, id }
  }

  pub fn usize_inclusive(&mut self, min: usize, max: usize) -> usize {
    self
      .service
      .int_range(self.pool, self.id, min as i64, max as i64 + 1)
      .unwrap_or(min as i64) as usize
  }

  pub fn i32_inclusive(&mut self, min: i32, max: i32) -> i32 {
    self
      .service
      .int_range(self.pool, self.id, min as i64, max as i64 + 1)
      .unwrap_or(min as i64) as i32
  }

  pub fn f64(&mut self) -> f64 {
    self.service.float_01(self.pool, self.id).unwrap_or(0.0)
  }

  pub fn chance(&mut self, probability: f64) -> bool {
    self
      .service
      .bool(self.pool, self.id, probability)
      .unwrap_or(false)
  }

  pub fn choose<T: Copy>(&mut self, values: &[T]) -> T {
    values[self.usize_inclusive(0, values.len() - 1)]
  }

  pub fn shuffle<T>(&mut self, values: &mut [T]) {
    for i in (1..values.len()).rev() {
      let j = self.usize_inclusive(0, i);
      values.swap(i, j);
    }
  }
}

enum DynamicLogo {
  Neon(NeonLogo),
  Wave(WaveLogo),
  Error(ErrorLogo),
  Glitch(GlitchLogo),
  Select(SelectLogo),
  Character(CharacterLogo),
}

pub(super) struct HomeLogo {
  mode: DisplayLogoMode,
  animation: Option<AnimationHandle>,
  random: Option<RandomGeneratorId>,
  dynamic: Option<DynamicLogo>,
}

impl HomeLogo {
  pub fn new(
    configured_mode: DisplayLogoMode,
    seed: u64,
    animation: &AnimationService,
    random: &RandomService,
    pool: &mut RuntimeObjectPool,
  ) -> Self {
    let mut random_id = (configured_mode == DisplayLogoMode::Random)
      .then(|| random.create(pool, RandomSeed::U64(seed)));
    let mode = if configured_mode == DisplayLogoMode::Random {
      let mut rng = LogoRandom::new(random, pool, random_id.unwrap());
      DYNAMIC_MODES[rng.usize_inclusive(0, DYNAMIC_MODES.len() - 1)]
    } else {
      configured_mode
    };

    let needs_random = matches!(
      mode,
      DisplayLogoMode::Error
        | DisplayLogoMode::Glitch
        | DisplayLogoMode::Select
        | DisplayLogoMode::Char
    );
    if needs_random && random_id.is_none() {
      random_id = Some(random.create(pool, RandomSeed::U64(seed)));
    }

    let dynamic =
      match mode {
        DisplayLogoMode::Neon => Some(DynamicLogo::Neon(NeonLogo::new())),
        DisplayLogoMode::Wave => Some(DynamicLogo::Wave(WaveLogo::new())),
        DisplayLogoMode::Error => Some(DynamicLogo::Error(ErrorLogo::new(&mut LogoRandom::new(
          random,
          pool,
          random_id.unwrap(),
        )))),
        DisplayLogoMode::Glitch => Some(DynamicLogo::Glitch(GlitchLogo::new(
          &mut LogoRandom::new(random, pool, random_id.unwrap()),
        ))),
        DisplayLogoMode::Select => Some(DynamicLogo::Select(SelectLogo::new(
          &mut LogoRandom::new(random, pool, random_id.unwrap()),
        ))),
        DisplayLogoMode::Char => Some(DynamicLogo::Character(CharacterLogo::new(
          &mut LogoRandom::new(random, pool, random_id.unwrap()),
        ))),
        _ => None,
      };

    if !needs_random {
      if let Some(id) = random_id.take() {
        random.remove(pool, id);
      }
    }

    let animation_handle = dynamic.as_ref().and_then(|_| create_clock(animation, pool));
    Self {
      mode,
      animation: animation_handle,
      random: random_id,
      dynamic,
    }
  }

  pub fn mode(&self) -> DisplayLogoMode {
    self.mode
  }

  pub fn template_text(&self, classic: &[&str]) -> String {
    let lines: &[&str] = match self.dynamic {
      Some(DynamicLogo::Wave(_) | DynamicLogo::Character(_)) => &PADDED_TEMPLATE,
      Some(DynamicLogo::Select(_)) => &SELECT_TEMPLATE,
      Some(DynamicLogo::Glitch(_)) => &GLITCH_TEMPLATE,
      Some(_) => &DYNAMIC_TEMPLATE,
      None => classic,
    };
    lines.join("\n")
  }

  pub fn render_text(&self, classic: impl FnOnce() -> String) -> String {
    match &self.dynamic {
      Some(DynamicLogo::Neon(logo)) => logo.render(),
      Some(DynamicLogo::Wave(logo)) => logo.render(),
      Some(DynamicLogo::Error(logo)) => logo.render(),
      Some(DynamicLogo::Glitch(logo)) => logo.render(),
      Some(DynamicLogo::Select(logo)) => logo.render(),
      Some(DynamicLogo::Character(logo)) => logo.render(),
      None => classic(),
    }
  }

  pub fn render_y(&self, layout_y: u16) -> u16 {
    if matches!(self.dynamic, Some(DynamicLogo::Error(_))) {
      layout_y.saturating_sub(1)
    } else {
      layout_y
    }
  }

  pub fn update(
    &mut self,
    dt: Duration,
    animation: &AnimationService,
    random: &RandomService,
    pool: &mut RuntimeObjectPool,
  ) {
    let Some(handle) = self.animation else {
      return;
    };
    animation.update(pool, AnimationClock::Ui, dt);
    let time = animation.completed_cycles(pool, handle).unwrap_or(0) as f64
      + animation.progress(pool, handle).unwrap_or(0.0);
    let mut rng = self.random.map(|id| LogoRandom::new(random, pool, id));
    match &mut self.dynamic {
      Some(DynamicLogo::Neon(logo)) => logo.advance(time),
      Some(DynamicLogo::Wave(logo)) => logo.advance(time),
      Some(DynamicLogo::Error(logo)) => logo.advance(time, rng.as_mut().unwrap()),
      Some(DynamicLogo::Glitch(logo)) => logo.advance(time, rng.as_mut().unwrap()),
      Some(DynamicLogo::Select(logo)) => logo.advance(time, rng.as_mut().unwrap()),
      Some(DynamicLogo::Character(logo)) => logo.advance(time, rng.as_mut().unwrap()),
      None => {}
    }
  }
}

fn create_clock(
  animation: &AnimationService,
  pool: &mut RuntimeObjectPool,
) -> Option<AnimationHandle> {
  let value = animation.create_value(pool, AnimationValue::Float(0.0));
  animation
    .play(
      pool,
      AnimationOwner::Host,
      AnimationSource::Tween(Arc::new(TweenDefinition {
        from: AnimationValue::Float(0.0),
        to: AnimationValue::Float(1.0),
        duration: Duration::from_secs(1),
        easing: AnimationEasing::Linear,
        interpolation: AnimationInterpolation::Linear,
      })),
      vec![AnimationBinding {
        track: 0,
        target: AnimationTarget::Value(value),
        property: AnimationProperty::Value,
        initial_value: AnimationValue::Float(0.0),
      }],
      AnimationPlaybackOptions {
        repeat: AnimationRepeatOptions {
          mode: AnimationRepeatMode::Restart,
          count: AnimationRepeatCount::Infinite,
        },
        ..Default::default()
      },
    )
    .ok()
}

pub(super) fn cells_to_rich_text(rows: &[Vec<LogoCell>]) -> String {
  let mut output = String::from("f%");
  for (row_index, row) in rows.iter().enumerate() {
    if row_index > 0 {
      output.push('\n');
    }
    let mut start = 0;
    while start < row.len() {
      let style = row[start].style;
      let mut end = start + 1;
      while end < row.len() && row[end].style == style {
        end += 1;
      }
      if style != CellStyle::default() {
        push_style_start(&mut output, style);
      }
      for cell in &row[start..end] {
        output.push(cell.ch);
      }
      if style != CellStyle::default() {
        output.push_str("<reset>");
      }
      start = end;
    }
  }
  output
}

fn push_style_start(output: &mut String, style: CellStyle) {
  if let Some((r, g, b)) = style.fg {
    output.push_str(&format!("<fg:rgb({r},{g},{b})>"));
  }
  if let Some((r, g, b)) = style.bg {
    output.push_str(&format!("<bg:rgb({r},{g},{b})>"));
  }
  if style.bold {
    output.push_str("<b>");
  }
  if style.dim {
    output.push_str("<dim>");
  }
  if style.reverse {
    output.push_str("<reverse>");
  }
}

pub(super) fn random_rgb(rng: &mut LogoRandom<'_>) -> (u8, u8, u8) {
  (
    rng.usize_inclusive(0, 255) as u8,
    rng.usize_inclusive(0, 255) as u8,
    rng.usize_inclusive(0, 255) as u8,
  )
}

pub(super) fn dynamic_mode_for_cursor(cursor: u64) -> DisplayLogoMode {
  DYNAMIC_MODES[cursor as usize % DYNAMIC_MODES.len()]
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn sequential_modes_cycle_through_only_dynamic_logos() {
    assert_eq!(dynamic_mode_for_cursor(0), DisplayLogoMode::Neon);
    assert_eq!(dynamic_mode_for_cursor(5), DisplayLogoMode::Char);
    assert_eq!(dynamic_mode_for_cursor(6), DisplayLogoMode::Neon);
  }

  #[test]
  fn glitch_logo_m_fourth_row_stays_continuous() {
    assert!(GLITCH_TEMPLATE[3].contains("████████████"));
  }

  #[test]
  fn only_selected_logo_allocates_runtime_objects() {
    let animation = AnimationService::new();
    let random = RandomService::new();

    let mut classic_pool = RuntimeObjectPool::new();
    let classic = HomeLogo::new(
      DisplayLogoMode::Classic,
      1,
      &animation,
      &random,
      &mut classic_pool,
    );
    assert_eq!(classic.mode(), DisplayLogoMode::Classic);
    assert!(classic_pool.animations.ids().is_empty());
    assert!(classic_pool.random_generators.generators.is_empty());

    let mut neon_pool = RuntimeObjectPool::new();
    let neon = HomeLogo::new(
      DisplayLogoMode::Neon,
      2,
      &animation,
      &random,
      &mut neon_pool,
    );
    assert_eq!(neon.mode(), DisplayLogoMode::Neon);
    assert_eq!(neon_pool.animations.ids().len(), 1);
    assert!(neon_pool.random_generators.generators.is_empty());

    let mut error_pool = RuntimeObjectPool::new();
    let error = HomeLogo::new(
      DisplayLogoMode::Error,
      3,
      &animation,
      &random,
      &mut error_pool,
    );
    assert_eq!(error.mode(), DisplayLogoMode::Error);
    assert_eq!(error_pool.animations.ids().len(), 1);
    assert_eq!(error_pool.random_generators.generators.len(), 1);
  }
}
