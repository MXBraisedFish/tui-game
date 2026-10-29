//! Progress indicators with independent committed and preview values.

mod state;
mod types;

pub(crate) use self::state::ProgressBarObjects;
use self::state::ProgressBarState;
pub use self::types::{
  ProgressBarFillOrigin, ProgressBarId, ProgressBarOptions, ProgressBarSegmentStyle,
};
use crate::SliceId;
use crate::UiObjectPool;
use tg_core_style::TextStyle;
use tg_core_unicode::char_width;
use tg_service_canvas::CanvasService;
use tg_service_layout::Rect;

/// The public entry point for progress bar operations.
#[derive(Default)]
pub struct ProgressBarService;

impl ProgressBarService {
  /// Create a progress bar service with its initial state.
  pub fn new() -> Self {
    Self
  }

  /// Create an owned progress bar object and return its identity.
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

  /// Remove the identified widget object and release its owned state.
  pub fn remove(&self, pool: &mut UiObjectPool, id: ProgressBarId) -> bool {
    pool.progress_bars.bars.remove(&id).is_some()
  }

  /// Report whether the identified widget object is still present.
  pub fn exists(&self, pool: &UiObjectPool, id: ProgressBarId) -> bool {
    pool.progress_bars.bars.contains_key(&id)
  }

  /// Replace the segment styles and fill origin when all characters occupy one cell.
  ///
  /// Return false if the object is missing or a configured character has an invalid width.
  ///
  /// # Arguments
  ///
  /// * `pool` - The owning widget pool.
  /// * `id` - The progress bar identity.
  /// * `options` - The replacement configuration.
  pub fn set_options(
    &self,
    pool: &mut UiObjectPool,
    id: ProgressBarId,
    options: ProgressBarOptions,
  ) -> bool {
    if !valid_options(&options) {
      return false;
    }
    let Some(state) = pool.progress_bars.bars.get_mut(&id) else {
      return false;
    };
    state.options = options;
    true
  }

  /// Render a single-row progress bar at a signed parent-local position.
  ///
  /// Clip the output without changing the logical width used to compute completed cells.
  /// Return false if the object or destination is unavailable, or no cells remain visible.
  ///
  /// # Arguments
  ///
  /// * `pool` - The owning widget pool.
  /// * `id` - The progress bar identity.
  /// * `target` - The prepared parent slice, or None for the base.
  /// * `position` - The original signed column and row.
  /// * `width` - The complete logical width in terminal cells.
  /// * `canvas` - The destination canvas.
  pub fn render_at(
    &self,
    pool: &UiObjectPool,
    id: ProgressBarId,
    target: Option<SliceId>,
    position: (i32, i32),
    width: u16,
    canvas: &mut CanvasService,
  ) -> bool {
    let Some((parent_clip, parent_origin, _)) = canvas.surface_geometry(target) else {
      return false;
    };
    let origin = (
      i64::from(parent_origin.0) + i64::from(position.0),
      i64::from(parent_origin.1) + i64::from(position.1),
    );
    let clip = crate::clip_rect(
      origin,
      tg_service_layout::Size { width, height: 1 },
      parent_clip,
    );
    if clip.width == 0 || clip.height == 0 {
      return false;
    }
    let start = (i64::from(clip.x) - origin.0) as u16;
    let Some(segments) = self.segments_for(pool, id, width, start..start + clip.width) else {
      return false;
    };
    for (column, character, style) in segments {
      let x = (i64::from(position.0) + i64::from(column)) as i32;
      match target {
        None => canvas.styled_text(x, position.1, &character.to_string(), style),
        Some(slice) => {
          canvas.styled_text_on(slice, x, position.1, &character.to_string(), style);
        }
      }
    }
    true
  }

  /// Return the committed progress amount of the identified progress bar.
  pub fn completed(&self, pool: &UiObjectPool, id: ProgressBarId) -> Option<f32> {
    Some(pool.progress_bars.bars.get(&id)?.completed)
  }

  /// Return the preview progress amount of the identified progress bar.
  pub fn preview(&self, pool: &UiObjectPool, id: ProgressBarId) -> Option<f32> {
    Some(pool.progress_bars.bars.get(&id)?.preview)
  }

  /// Update the completed used by this progress bar service.
  ///
  /// # Arguments
  ///
  /// * `pool` - The object pool that owns the component.
  /// * `id` - The identifier of the owned object.
  /// * `value` - The value to store or convert.
  pub fn set_completed(&self, pool: &mut UiObjectPool, id: ProgressBarId, value: f32) -> bool {
    let Some(state) = pool.progress_bars.bars.get_mut(&id) else {
      return false;
    };
    state.completed = percent(value);
    true
  }

  /// Update the preview used by this progress bar service.
  ///
  /// # Arguments
  ///
  /// * `pool` - The object pool that owns the component.
  /// * `id` - The identifier of the owned object.
  /// * `value` - The value to store or convert.
  pub fn set_preview(&self, pool: &mut UiObjectPool, id: ProgressBarId, value: f32) -> bool {
    let Some(state) = pool.progress_bars.bars.get_mut(&id) else {
      return false;
    };
    state.preview = percent(value);
    true
  }

  /// Update the progress used by this progress bar service.
  ///
  /// # Arguments
  ///
  /// * `pool` - The object pool that owns the component.
  /// * `id` - The identifier of the owned object.
  /// * `completed` - The completed.
  /// * `preview` - The preview.
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

  /// Return the edge from which the identified progress bar fills.
  pub fn origin(&self, pool: &UiObjectPool, id: ProgressBarId) -> Option<ProgressBarFillOrigin> {
    Some(pool.progress_bars.bars.get(&id)?.options.origin)
  }

  /// Update the origin used by this progress bar service.
  ///
  /// # Arguments
  ///
  /// * `pool` - The object pool that owns the component.
  /// * `id` - The identifier of the owned object.
  /// * `origin` - The origin.
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

  /// Render the progress bar service into its requested terminal-cell surface.
  ///
  /// # Arguments
  ///
  /// * `pool` - The object pool that owns the component.
  /// * `id` - The identifier of the owned object.
  /// * `rect` - The rectangular region in terminal cells.
  /// * `canvas` - The clipped canvas used for drawing.
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

  /// Render the component into the identified clipped slice and update its interaction geometry.
  ///
  /// # Arguments
  ///
  /// * `pool` - The object pool that owns the component.
  /// * `id` - The identifier of the owned object.
  /// * `slice` - The slice.
  /// * `rect` - The rectangular region in terminal cells.
  /// * `canvas` - The clipped canvas used for drawing.
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

  /// Render the component into the physical host surface and update its interaction geometry.
  ///
  /// # Arguments
  ///
  /// * `pool` - The object pool that owns the component.
  /// * `id` - The identifier of the owned object.
  /// * `rect` - The rectangular region in terminal cells.
  /// * `canvas` - The clipped canvas used for drawing.
  pub fn render_host(
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
    self.segments_for(pool, id, rect.width, 0..rect.width)
  }

  fn segments_for(
    &self,
    pool: &UiObjectPool,
    id: ProgressBarId,
    width: u16,
    columns: std::ops::Range<u16>,
  ) -> Option<Vec<(u16, char, TextStyle)>> {
    let state = pool.progress_bars.bars.get(&id)?;
    let completed = filled_range(
      state.options.origin,
      width,
      cells_for(width, state.completed),
    );
    let preview = filled_range(state.options.origin, width, cells_for(width, state.preview));
    Some(
      columns
        .map(|column| {
          let segment = if completed.contains(&column) {
            &state.options.completed
          } else if preview.contains(&column) {
            &state.options.preview
          } else {
            &state.options.remaining
          };
          (column, segment.ch, segment.style.clone())
        })
        .collect(),
    )
  }
}

fn filled_range(origin: ProgressBarFillOrigin, width: u16, count: u16) -> std::ops::Range<u16> {
  let start = match origin {
    ProgressBarFillOrigin::Left => 0,
    ProgressBarFillOrigin::Right => width - count,
    ProgressBarFillOrigin::Center => (width - count) / 2,
  };
  start..start + count
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

#[cfg(test)]
mod tests {
  use super::*;
  use tg_core_style::{TextColor, TextStyle};
  use tg_service_canvas::CanvasService;

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
        tg_core_style::TerminalColor::BrightBlue
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
          == Some(TextColor::Terminal(tg_core_style::TerminalColor::Green)))
        .count(),
      6
    );
    assert_eq!(
      segments
        .iter()
        .filter(|(_, _, style)| style.foreground
          == Some(TextColor::Terminal(
            tg_core_style::TerminalColor::BrightBlue
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
