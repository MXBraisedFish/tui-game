mod state;
mod types;

pub(crate) use self::state::ProgressBarObjects;
use self::state::ProgressBarState;
pub use self::types::{
  ProgressBarFillOrigin, ProgressBarId, ProgressBarOptions, ProgressBarSegmentStyle,
};
use crate::host_engine::services::ui::UiObjectPool;
use crate::host_engine::services::unicode::char_width;
use crate::host_engine::services::{CanvasService, Rect, SliceId, TextStyle};

pub struct ProgressBarService;

impl ProgressBarService {
  pub fn new() -> Self {
    Self
  }

  pub fn create(
    &self,
    pool: &mut UiObjectPool,
    options: ProgressBarOptions,
  ) -> Option<ProgressBarId> {
    valid_options(&options).then(|| {
      let id = ProgressBarId(pool.progress_bars.next_id);
      pool.progress_bars.next_id += 1;
      pool.progress_bars.bars.insert(
        id,
        ProgressBarState {
          options,
          completed: 0.0,
          preview: 0.0,
        },
      );
      id
    })
  }

  pub fn remove(&self, pool: &mut UiObjectPool, id: ProgressBarId) -> bool {
    pool.progress_bars.bars.remove(&id).is_some()
  }

  pub fn exists(&self, pool: &UiObjectPool, id: ProgressBarId) -> bool {
    pool.progress_bars.bars.contains_key(&id)
  }

  pub fn completed(&self, pool: &UiObjectPool, id: ProgressBarId) -> Option<f32> {
    Some(pool.progress_bars.bars.get(&id)?.completed)
  }

  pub fn preview(&self, pool: &UiObjectPool, id: ProgressBarId) -> Option<f32> {
    Some(pool.progress_bars.bars.get(&id)?.preview)
  }

  pub fn set_completed(&self, pool: &mut UiObjectPool, id: ProgressBarId, value: f32) -> bool {
    let Some(state) = pool.progress_bars.bars.get_mut(&id) else {
      return false;
    };
    state.completed = percent(value);
    true
  }

  pub fn set_preview(&self, pool: &mut UiObjectPool, id: ProgressBarId, value: f32) -> bool {
    let Some(state) = pool.progress_bars.bars.get_mut(&id) else {
      return false;
    };
    state.preview = percent(value);
    true
  }

  pub fn set_progress(
    &self,
    pool: &mut UiObjectPool,
    id: ProgressBarId,
    completed: f32,
    preview: f32,
  ) -> bool {
    let Some(state) = pool.progress_bars.bars.get_mut(&id) else {
      return false;
    };
    state.completed = percent(completed);
    state.preview = percent(preview);
    true
  }

  pub fn origin(&self, pool: &UiObjectPool, id: ProgressBarId) -> Option<ProgressBarFillOrigin> {
    Some(pool.progress_bars.bars.get(&id)?.options.origin)
  }

  pub fn set_origin(
    &self,
    pool: &mut UiObjectPool,
    id: ProgressBarId,
    origin: ProgressBarFillOrigin,
  ) -> bool {
    let Some(state) = pool.progress_bars.bars.get_mut(&id) else {
      return false;
    };
    state.options.origin = origin;
    true
  }

  pub fn render(
    &self,
    pool: &UiObjectPool,
    id: ProgressBarId,
    rect: Rect,
    canvas: &mut CanvasService,
  ) -> bool {
    let Some(segments) = self.segments(pool, id, rect) else {
      return false;
    };
    for (x, ch, style) in segments {
      canvas.styled_text(rect.x + x, rect.y, &ch.to_string(), style);
    }
    true
  }

  pub fn render_on(
    &self,
    pool: &UiObjectPool,
    id: ProgressBarId,
    slice: SliceId,
    rect: Rect,
    canvas: &mut CanvasService,
  ) -> bool {
    let Some(segments) = self.segments(pool, id, rect) else {
      return false;
    };
    for (x, ch, style) in segments {
      if !canvas.styled_text_on(slice, rect.x + x, rect.y, &ch.to_string(), style) {
        return false;
      }
    }
    true
  }

  pub(crate) fn render_host(
    &self,
    pool: &UiObjectPool,
    id: ProgressBarId,
    rect: Rect,
    canvas: &mut CanvasService,
  ) -> bool {
    let Some(segments) = self.segments(pool, id, rect) else {
      return false;
    };
    for (x, ch, style) in segments {
      canvas.host_styled_text(rect.x + x, rect.y, &ch.to_string(), style);
    }
    true
  }

  fn segments(
    &self,
    pool: &UiObjectPool,
    id: ProgressBarId,
    rect: Rect,
  ) -> Option<Vec<(u16, char, TextStyle)>> {
    if rect.width == 0 || rect.height == 0 {
      return None;
    }
    let state = pool.progress_bars.bars.get(&id)?;
    let mut cells = vec![SegmentKind::Remaining; rect.width as usize];
    fill_cells(
      &mut cells,
      state.options.origin,
      cells_for(rect.width, state.preview),
      SegmentKind::Preview,
    );
    fill_cells(
      &mut cells,
      state.options.origin,
      cells_for(rect.width, state.completed),
      SegmentKind::Completed,
    );
    Some(
      cells
        .into_iter()
        .enumerate()
        .map(|(x, kind)| {
          let segment = match kind {
            SegmentKind::Completed => &state.options.completed,
            SegmentKind::Preview => &state.options.preview,
            SegmentKind::Remaining => &state.options.remaining,
          };
          (x as u16, segment.ch, segment.style.clone())
        })
        .collect(),
    )
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SegmentKind {
  Completed,
  Preview,
  Remaining,
}

fn valid_options(options: &ProgressBarOptions) -> bool {
  [
    options.completed.ch,
    options.preview.ch,
    options.remaining.ch,
  ]
  .into_iter()
  .all(|ch| char_width(ch) == 1)
}

fn percent(value: f32) -> f32 {
  if value.is_nan() {
    0.0
  } else {
    value.clamp(0.0, 1.0)
  }
}

fn cells_for(width: u16, value: f32) -> u16 {
  ((width as f32) * percent(value))
    .floor()
    .clamp(0.0, width as f32) as u16
}

fn fill_cells(
  cells: &mut [SegmentKind],
  origin: ProgressBarFillOrigin,
  count: u16,
  kind: SegmentKind,
) {
  let width = cells.len();
  match origin {
    ProgressBarFillOrigin::Left => {
      for cell in cells.iter_mut().take(count as usize) {
        *cell = kind;
      }
    }
    ProgressBarFillOrigin::Right => {
      for cell in cells.iter_mut().rev().take(count as usize) {
        *cell = kind;
      }
    }
    ProgressBarFillOrigin::Center => {
      for index in center_indices(width).into_iter().take(count as usize) {
        cells[index] = kind;
      }
    }
  }
}

fn center_indices(width: usize) -> Vec<usize> {
  let mut indices = Vec::with_capacity(width);
  if width == 0 {
    return indices;
  }
  let left_end = (width - 1) / 2;
  let right_start = width / 2;
  for step in 0..=left_end.max(width.saturating_sub(right_start + 1)) {
    if left_end >= step {
      indices.push(left_end - step);
    }
    let right = right_start + step;
    if right != left_end.saturating_sub(step) && right < width {
      indices.push(right);
    }
  }
  indices
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::host_engine::services::{CanvasService, TextColor, TextStyle};

  fn service_pool_id() -> (ProgressBarService, UiObjectPool, ProgressBarId) {
    let service = ProgressBarService::new();
    let mut pool = UiObjectPool::new();
    let id = service
      .create(&mut pool, ProgressBarOptions::default())
      .unwrap();
    (service, pool, id)
  }

  fn texts(
    service: &ProgressBarService,
    pool: &UiObjectPool,
    id: ProgressBarId,
    width: u16,
  ) -> String {
    service
      .segments(
        pool,
        id,
        Rect {
          x: 0,
          y: 0,
          width,
          height: 1,
        },
      )
      .unwrap()
      .into_iter()
      .map(|(_, ch, _)| ch)
      .collect()
  }

  #[test]
  fn create_returns_unique_ids_and_remove_clears_state() {
    let service = ProgressBarService::new();
    let mut pool = UiObjectPool::new();
    let a = service
      .create(&mut pool, ProgressBarOptions::default())
      .unwrap();
    let b = service
      .create(&mut pool, ProgressBarOptions::default())
      .unwrap();
    assert_ne!(a, b);
    assert!(service.exists(&pool, a));
    assert!(service.remove(&mut pool, a));
    assert!(!service.exists(&pool, a));
  }

  #[test]
  fn default_preview_style_is_bright_blue() {
    assert_eq!(
      ProgressBarOptions::default().preview.style.foreground,
      Some(TextColor::Terminal(
        crate::host_engine::services::TerminalColor::BrightBlue
      ))
    );
  }

  #[test]
  fn rejects_non_single_width_chars() {
    let service = ProgressBarService::new();
    let mut pool = UiObjectPool::new();
    let mut options = ProgressBarOptions::default();
    options.completed.ch = '我';
    assert!(service.create(&mut pool, options).is_none());
  }

  #[test]
  fn progress_values_are_clamped_and_nan_becomes_zero() {
    let (service, mut pool, id) = service_pool_id();
    assert!(service.set_progress(&mut pool, id, f32::NAN, 2.0));
    assert_eq!(service.completed(&pool, id), Some(0.0));
    assert_eq!(service.preview(&pool, id), Some(1.0));
  }

  #[test]
  fn left_and_right_origins_fill_from_edges() {
    let (service, mut pool, id) = service_pool_id();
    service.set_progress(&mut pool, id, 0.4, 0.7);
    assert_eq!(texts(&service, &pool, id, 10), "███████───");
    service.set_origin(&mut pool, id, ProgressBarFillOrigin::Right);
    assert_eq!(texts(&service, &pool, id, 10), "───███████");
  }

  #[test]
  fn completed_overrides_preview() {
    let (service, mut pool, id) = service_pool_id();
    service.set_progress(&mut pool, id, 0.6, 0.8);
    let segments = service
      .segments(
        &pool,
        id,
        Rect {
          x: 0,
          y: 0,
          width: 10,
          height: 1,
        },
      )
      .unwrap();
    assert_eq!(
      segments
        .iter()
        .filter(|(_, _, style)| style.foreground
          == Some(TextColor::Terminal(
            crate::host_engine::services::TerminalColor::Green
          )))
        .count(),
      6
    );
    assert_eq!(
      segments
        .iter()
        .filter(|(_, _, style)| style.foreground
          == Some(TextColor::Terminal(
            crate::host_engine::services::TerminalColor::BrightBlue
          )))
        .count(),
      2
    );
  }

  #[test]
  fn center_origin_matches_odd_and_even_examples() {
    let (service, mut pool, id) = service_pool_id();
    let mut options = ProgressBarOptions::default();
    options.completed.ch = '+';
    options.preview.ch = '+';
    options.remaining.ch = '-';
    options.origin = ProgressBarFillOrigin::Center;
    pool.progress_bars.bars.get_mut(&id).unwrap().options = options;

    service.set_progress(&mut pool, id, 0.0, 0.0);
    assert_eq!(texts(&service, &pool, id, 5), "-----");
    service.set_progress(&mut pool, id, 0.34, 0.34);
    assert_eq!(texts(&service, &pool, id, 5), "--+--");
    service.set_progress(&mut pool, id, 0.67, 0.67);
    assert_eq!(texts(&service, &pool, id, 5), "-+++-");
    service.set_progress(&mut pool, id, 1.0, 1.0);
    assert_eq!(texts(&service, &pool, id, 5), "+++++");

    service.set_progress(&mut pool, id, 0.34, 0.34);
    assert_eq!(texts(&service, &pool, id, 6), "--++--");
    service.set_progress(&mut pool, id, 0.67, 0.67);
    assert_eq!(texts(&service, &pool, id, 6), "-++++-");
  }

  #[test]
  fn empty_rect_does_not_render() {
    let (service, pool, id) = service_pool_id();
    assert!(service.segments(&pool, id, Rect::default()).is_none());
  }

  #[test]
  fn render_writes_base_cells() {
    let service = ProgressBarService::new();
    let mut pool = UiObjectPool::new();
    let options = ProgressBarOptions {
      completed: ProgressBarSegmentStyle {
        ch: 'C',
        style: TextStyle::default(),
      },
      preview: ProgressBarSegmentStyle {
        ch: 'P',
        style: TextStyle::default(),
      },
      remaining: ProgressBarSegmentStyle {
        ch: 'R',
        style: TextStyle::default(),
      },
      origin: ProgressBarFillOrigin::Left,
    };
    let id = service.create(&mut pool, options).unwrap();
    service.set_progress(&mut pool, id, 0.4, 0.7);
    let mut canvas = CanvasService::new();

    assert!(service.render(
      &pool,
      id,
      Rect {
        x: 0,
        y: 0,
        width: 10,
        height: 1,
      },
      &mut canvas,
    ));
    let text = (0..10)
      .map(|x| canvas.cell_at(x, 0).unwrap().text.as_str())
      .collect::<Vec<_>>()
      .join("");
    assert_eq!(text, "CCCCPPPRRR");
  }
}
