//! Export loading overlay state, owned interactions, and clipped terminal presentation.

use std::time::Duration;

use crate::host_engine::services::{CanvasService, Rect};
use crate::host_engine::services::{
  DrawTextParams, I18nService, LayoutService, ProgressBarFillOrigin, ProgressBarId,
  ProgressBarOptions, ProgressBarSegmentStyle, ProgressBarService, RenderService,
  RuntimeObjectPool, RuntimeObjectPoolOwner, TerminalColor, TextColor, TextStyle, TimeService,
  TimerId, UiObjectPool, UiObjectPoolOwner,
};

/// The state and owned widgets of the export loading view.
pub struct ExportLoadingUi {
  objects: UiObjectPool,
  runtime_objects: RuntimeObjectPool,
  bar: ProgressBarId,
  animation_timer: TimerId,
}

impl ExportLoadingUi {
  /// Create the export loading view and allocate its owned UI objects.
  ///
  /// # Panics
  ///
  /// Panic if an internal invariant is violated: `valid export loading progress bar options`.
  pub fn init(progress_bar: &ProgressBarService, time: &TimeService) -> Self {
    let mut objects = UiObjectPool::new();
    let bar = progress_bar
      .create(&mut objects, block_options())
      .expect("valid export loading progress bar options");
    let mut runtime_objects = RuntimeObjectPool::new();
    let animation_timer = time.create_count_up(&mut runtime_objects.time);
    let _ = time.start(&mut runtime_objects.time, animation_timer);
    Self {
      objects,
      runtime_objects,
      bar,
      animation_timer,
    }
  }

  /// Reset and start the loading-indicator animation timer.
  pub fn restart_animation(&mut self, time: &TimeService) {
    let _ = time.reset(&mut self.runtime_objects.time, self.animation_timer);
    let _ = time.start(&mut self.runtime_objects.time, self.animation_timer);
  }

  /// Advance the export loading view's transient state for this host frame.
  pub fn update(&mut self, time: &TimeService, dt: Duration) {
    time.update(&mut self.runtime_objects.time, dt);
  }

  /// Update the progress used by this export loading ui.
  ///
  /// # Arguments
  ///
  /// * `progress_bar` - The progress bar.
  /// * `completed` - The completed.
  /// * `preview` - The preview.
  pub fn set_progress(&mut self, progress_bar: &ProgressBarService, completed: f32, preview: f32) {
    let _ = progress_bar.set_progress(&mut self.objects, self.bar, completed, preview);
  }

  /// Draw the export loading view and register interaction regions in its assigned surfaces.
  ///
  /// # Arguments
  ///
  /// * `render` - The drawing service used to render terminal cells.
  /// * `canvas` - The clipped canvas used for drawing.
  /// * `layout` - The service resolving terminal sizes and positions.
  /// * `i18n` - The service resolving localized text.
  /// * `progress_bar` - The progress bar.
  /// * `time` - The time.
  pub fn render(
    &mut self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    i18n: &I18nService,
    progress_bar: &ProgressBarService,
    time: &TimeService,
  ) {
    let size = layout.physical_size();
    if size.height < 3 {
      return;
    }

    let elapsed = time
      .elapsed(&self.runtime_objects.time, self.animation_timer)
      .unwrap_or(Duration::ZERO);
    let dots = ".".repeat((elapsed.as_millis() / 500 % 3 + 1) as usize);
    let tip = format!(
      "{}{}",
      i18n.get_runtime_text("export_loading", "export_loading.tip"),
      dots
    );

    let start_y = size.height.saturating_sub(3) / 2;
    let tip_width = layout.get_text_width(&tip, None);
    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: layout.resolve_host_x(LayoutService::ALIGN_CENTER, tip_width, 0),
        y: start_y,
        text: tip,
        ..Default::default()
      },
    );

    let bar_width = size.width.saturating_sub(24);
    if bar_width == 0 {
      return;
    }

    let _ = progress_bar.render_host(
      &self.objects,
      self.bar,
      Rect {
        x: 12,
        y: start_y.saturating_add(2),
        width: bar_width,
        height: 1,
      },
      canvas,
    );
  }
}

impl UiObjectPoolOwner for ExportLoadingUi {
  fn objects(&self) -> &UiObjectPool {
    &self.objects
  }

  fn objects_mut(&mut self) -> &mut UiObjectPool {
    &mut self.objects
  }
}

impl RuntimeObjectPoolOwner for ExportLoadingUi {
  fn runtime_objects(&self) -> &RuntimeObjectPool {
    &self.runtime_objects
  }

  fn runtime_objects_mut(&mut self) -> &mut RuntimeObjectPool {
    &mut self.runtime_objects
  }
}

fn block_options() -> ProgressBarOptions {
  ProgressBarOptions {
    completed: segment(TerminalColor::Green),
    preview: segment(TerminalColor::BrightBlue),
    remaining: segment(TerminalColor::White),
    origin: ProgressBarFillOrigin::Left,
  }
}

fn segment(color: TerminalColor) -> ProgressBarSegmentStyle {
  ProgressBarSegmentStyle {
    ch: '█',
    style: TextStyle {
      foreground: Some(TextColor::Terminal(color)),
      background: Some(TextColor::Transparent),
      ..Default::default()
    },
  }
}
